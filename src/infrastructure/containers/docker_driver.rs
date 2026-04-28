// SPDX-License-Identifier: GPL-3.0-or-later
use std::collections::HashMap;
use std::path::Path;

use serde_json::Value;

use crate::core::domain::{
    container::{
        Container, ContainerPort, ContainerStats, ContainerStatus, CreateContainerOptions,
        PullProgress, PullStatus,
    },
    image::{Image, ImageLayer},
    network::{ContainerEvent, CreateNetworkOptions, Network, PruneReport, SystemUsage},
    volume::{CreateVolumeOptions, Volume},
};
use crate::infrastructure::containers::error::ContainerError;
use crate::infrastructure::containers::http_over_unix::{self as http, strip_log_frames};
use crate::ports::i_container_driver::IContainerDriver;

const API: &str = "/v1.41";

pub struct DockerDriver {
    socket: String,
}

impl DockerDriver {
    pub fn new(socket: impl Into<String>) -> Self {
        Self {
            socket: socket.into(),
        }
    }

    pub fn default_socket() -> Self {
        Self::new("/var/run/docker.sock")
    }

    fn get(&self, path: &str) -> Result<Value, ContainerError> {
        let resp = http::request(&self.socket, "GET", path, None)?;
        check_status(&resp.status, &resp.body)?;
        Ok(serde_json::from_str(&resp.body)?)
    }

    fn post(&self, path: &str, body: Option<&str>) -> Result<http::HttpResponse, ContainerError> {
        let resp = http::request(&self.socket, "POST", path, body)?;
        check_status(&resp.status, &resp.body)?;
        Ok(resp)
    }

    fn delete(&self, path: &str) -> Result<(), ContainerError> {
        let resp = http::request(&self.socket, "DELETE", path, None)?;
        check_status(&resp.status, &resp.body)?;
        Ok(())
    }
}

fn check_status(status: &u16, body: &str) -> Result<(), ContainerError> {
    match status {
        200..=299 | 304 => Ok(()),
        404 => {
            let msg = extract_message(body);
            Err(ContainerError::NotFound(msg))
        }
        _ => {
            let msg = extract_message(body);
            Err(ContainerError::ApiError {
                status: *status,
                message: msg,
            })
        }
    }
}

fn extract_message(body: &str) -> String {
    serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|v| v["message"].as_str().map(str::to_string))
        .unwrap_or_else(|| body.chars().take(200).collect())
}

fn parse_container(v: &Value) -> Container {
    let id = v["Id"].as_str().unwrap_or_default().to_string();
    let short_id = id.chars().take(12).collect();

    let name = v["Names"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|n| n.as_str())
        .unwrap_or_default()
        .trim_start_matches('/')
        .to_string();

    let image = v["Image"].as_str().unwrap_or_default().to_string();
    let command = v["Command"].as_str().unwrap_or_default().to_string();
    let created = v["Created"].as_i64().unwrap_or(0);
    let state = v["State"].as_str().unwrap_or("unknown").to_string();
    let status_text = v["Status"].as_str().unwrap_or_default().to_string();

    let status = ContainerStatus::from_state(&state, None);

    let ports = v["Ports"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|p| {
                    let container_port = p["PrivatePort"].as_u64()? as u16;
                    Some(ContainerPort {
                        host_ip: p["IP"].as_str().map(str::to_string),
                        host_port: p["PublicPort"].as_u64().map(|p| p as u16),
                        container_port,
                        protocol: p["Type"].as_str().unwrap_or("tcp").to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let labels: HashMap<String, String> = v["Labels"]
        .as_object()
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
                .collect()
        })
        .unwrap_or_default();

    let mounts = v["Mounts"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m["Destination"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let compose_project = labels.get("com.docker.compose.project").cloned();

    Container {
        id,
        short_id,
        name,
        image,
        command,
        created,
        status,
        status_text,
        ports,
        labels,
        mounts,
        env: vec![],
        compose_project,
    }
}

fn parse_image(v: &Value) -> Image {
    let id = v["Id"].as_str().unwrap_or_default().to_string();
    let short_id = id
        .strip_prefix("sha256:")
        .unwrap_or(&id)
        .chars()
        .take(12)
        .collect();

    let tags = v["RepoTags"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|t| t.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let digest = v["RepoDigests"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|d| d.as_str())
        .map(str::to_string);

    let size = v["Size"].as_u64().unwrap_or(0);
    let created = v["Created"].as_i64().unwrap_or(0);
    let labels = v["Labels"]
        .as_object()
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
                .collect()
        })
        .unwrap_or_default();

    Image {
        id,
        short_id,
        tags,
        size,
        created,
        digest,
        labels,
        in_use: false,
    }
}

fn parse_volume(v: &Value) -> Volume {
    let size_bytes = v["UsageData"]["Size"].as_u64().filter(|&s| s != u64::MAX);
    let in_use = v["UsageData"]["RefCount"].as_u64().unwrap_or(0) > 0;
    Volume {
        name: v["Name"].as_str().unwrap_or_default().to_string(),
        driver: v["Driver"].as_str().unwrap_or_default().to_string(),
        mountpoint: v["Mountpoint"].as_str().unwrap_or_default().to_string(),
        created: v["CreatedAt"].as_str().unwrap_or_default().to_string(),
        scope: v["Scope"].as_str().unwrap_or_default().to_string(),
        labels: v["Labels"]
            .as_object()
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
                    .collect()
            })
            .unwrap_or_default(),
        size_bytes,
        in_use,
    }
}

fn parse_network(v: &Value) -> Network {
    let subnet = v["IPAM"]["Config"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|c| c["Subnet"].as_str())
        .map(str::to_string);
    let gateway = v["IPAM"]["Config"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|c| c["Gateway"].as_str())
        .map(str::to_string);

    let containers_count = v["Containers"]
        .as_object()
        .map(|c| c.len() as u64)
        .unwrap_or(0);
    Network {
        id: v["Id"].as_str().unwrap_or_default().to_string(),
        name: v["Name"].as_str().unwrap_or_default().to_string(),
        driver: v["Driver"].as_str().unwrap_or_default().to_string(),
        scope: v["Scope"].as_str().unwrap_or_default().to_string(),
        internal: v["Internal"].as_bool().unwrap_or(false),
        created: v["Created"].as_str().unwrap_or_default().to_string(),
        subnet,
        gateway,
        containers_count,
    }
}

impl IContainerDriver for DockerDriver {
    fn runtime_name(&self) -> &str {
        "Docker"
    }

    fn is_available(&self) -> bool {
        Path::new(&self.socket).exists() && self.ping().is_ok()
    }

    fn list_containers(&self, all: bool) -> Result<Vec<Container>, ContainerError> {
        let path = format!("{API}/containers/json?all={}", if all { 1 } else { 0 });
        let json = self.get(&path)?;
        Ok(json
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(parse_container)
            .collect())
    }

    fn inspect_container(&self, id: &str) -> Result<Container, ContainerError> {
        let path = format!("{API}/containers/{id}/json");
        let json = self.get(&path)?;

        let raw_id = json["Id"].as_str().unwrap_or_default().to_string();
        let short_id = raw_id.chars().take(12).collect();
        let name = json["Name"]
            .as_str()
            .unwrap_or_default()
            .trim_start_matches('/')
            .to_string();
        let image = json["Config"]["Image"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        let command = json["Config"]["Cmd"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();
        let created_str = json["Created"].as_str().unwrap_or("0");
        let created = chrono_parse_or_zero(created_str);
        let state_str = json["State"]["Status"].as_str().unwrap_or("unknown");
        let exit_code = json["State"]["ExitCode"].as_i64().map(|c| c as i32);
        let status = ContainerStatus::from_state(state_str, exit_code);
        let status_text = format!("{} (exit {})", state_str, exit_code.unwrap_or(0));
        let labels: HashMap<String, String> = json["Config"]["Labels"]
            .as_object()
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
                    .collect()
            })
            .unwrap_or_default();
        let mounts = json["Mounts"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| m["Destination"].as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();

        let env: Vec<String> = json["Config"]["Env"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        let compose_project = labels.get("com.docker.compose.project").cloned();

        Ok(Container {
            id: raw_id,
            short_id,
            name,
            image,
            command,
            created,
            status,
            status_text,
            ports: vec![],
            labels,
            mounts,
            env,
            compose_project,
        })
    }

    fn inspect_container_json(&self, id: &str) -> Result<String, ContainerError> {
        let path = format!("{API}/containers/{id}/json");
        let json = self.get(&path)?;
        Ok(serde_json::to_string_pretty(&json)?)
    }

    fn start_container(&self, id: &str) -> Result<(), ContainerError> {
        self.post(&format!("{API}/containers/{id}/start"), None)?;
        Ok(())
    }

    fn stop_container(&self, id: &str, timeout_secs: Option<u32>) -> Result<(), ContainerError> {
        let t = timeout_secs.unwrap_or(10);
        self.post(&format!("{API}/containers/{id}/stop?t={t}"), None)?;
        Ok(())
    }

    fn restart_container(&self, id: &str, timeout_secs: Option<u32>) -> Result<(), ContainerError> {
        let t = timeout_secs.unwrap_or(10);
        self.post(&format!("{API}/containers/{id}/restart?t={t}"), None)?;
        Ok(())
    }

    fn pause_container(&self, id: &str) -> Result<(), ContainerError> {
        self.post(&format!("{API}/containers/{id}/pause"), None)?;
        Ok(())
    }

    fn unpause_container(&self, id: &str) -> Result<(), ContainerError> {
        self.post(&format!("{API}/containers/{id}/unpause"), None)?;
        Ok(())
    }

    fn remove_container(
        &self,
        id: &str,
        force: bool,
        remove_volumes: bool,
    ) -> Result<(), ContainerError> {
        self.delete(&format!(
            "{API}/containers/{id}?force={}&v={}",
            force as u8, remove_volumes as u8
        ))
    }

    fn create_container(&self, opts: &CreateContainerOptions) -> Result<String, ContainerError> {
        use serde_json::json;

        let mut port_bindings = serde_json::Map::new();
        let mut exposed_ports = serde_json::Map::new();
        for (host_port, ctr_port) in &opts.port_bindings {
            let key = format!("{ctr_port}/tcp");
            exposed_ports.insert(key.clone(), json!({}));
            port_bindings.insert(key, json!([{"HostPort": host_port.to_string()}]));
        }

        let binds: Vec<String> = opts
            .volume_bindings
            .iter()
            .map(|(h, c)| format!("{h}:{c}"))
            .collect();

        let mut body = json!({
            "Image": opts.image,
            "Cmd": opts.command,
            "Env": opts.env,
            "ExposedPorts": exposed_ports,
            "HostConfig": {
                "PortBindings": port_bindings,
                "Binds": binds,
                "RestartPolicy": {
                    "Name": opts.restart_policy.as_str(),
                    "MaximumRetryCount": 0,
                },
                "AutoRemove": opts.auto_remove,
            },
        });
        if let Some(net) = &opts.network {
            body["NetworkingConfig"] = json!({
                "EndpointsConfig": { net: {} }
            });
        }

        let path = if let Some(name) = &opts.name {
            format!("{API}/containers/create?name={name}")
        } else {
            format!("{API}/containers/create")
        };

        let resp = self.post(&path, Some(&body.to_string()))?;
        let json: Value = serde_json::from_str(&resp.body)?;
        Ok(json["Id"].as_str().unwrap_or_default().to_string())
    }

    fn rename_container(&self, id: &str, new_name: &str) -> Result<(), ContainerError> {
        self.post(
            &format!("{API}/containers/{id}/rename?name={new_name}"),
            None,
        )?;
        Ok(())
    }

    fn container_logs(
        &self,
        id: &str,
        tail: Option<u32>,
        timestamps: bool,
    ) -> Result<String, ContainerError> {
        let tail_param = tail
            .map(|n| n.to_string())
            .unwrap_or_else(|| "all".to_string());
        let ts = if timestamps { 1 } else { 0 };
        let path = format!(
            "{API}/containers/{id}/logs?stdout=1&stderr=1&tail={tail_param}&timestamps={ts}"
        );
        let resp = http::request(&self.socket, "GET", &path, None)?;
        check_status(&resp.status, &resp.body)?;
        Ok(strip_log_frames(&resp.body))
    }

    fn container_stats(&self, id: &str) -> Result<ContainerStats, ContainerError> {
        let path = format!("{API}/containers/{id}/stats?stream=false");
        let json = self.get(&path)?;

        let cpu_delta = json["cpu_stats"]["cpu_usage"]["total_usage"]
            .as_u64()
            .unwrap_or(0)
            .saturating_sub(
                json["precpu_stats"]["cpu_usage"]["total_usage"]
                    .as_u64()
                    .unwrap_or(0),
            );
        let system_delta = json["cpu_stats"]["system_cpu_usage"]
            .as_u64()
            .unwrap_or(0)
            .saturating_sub(
                json["precpu_stats"]["system_cpu_usage"]
                    .as_u64()
                    .unwrap_or(0),
            );
        let num_cpus = json["cpu_stats"]["online_cpus"].as_u64().unwrap_or(1);
        let cpu_percent = if system_delta > 0 {
            (cpu_delta as f64 / system_delta as f64) * num_cpus as f64 * 100.0
        } else {
            0.0
        };

        let mem_usage = json["memory_stats"]["usage"].as_u64().unwrap_or(0);
        let mem_limit = json["memory_stats"]["limit"].as_u64().unwrap_or(1);
        let mem_percent = (mem_usage as f64 / mem_limit as f64) * 100.0;

        let (net_rx, net_tx) = json["networks"]
            .as_object()
            .map(|nets| {
                nets.values().fold((0u64, 0u64), |(rx, tx), iface| {
                    (
                        rx + iface["rx_bytes"].as_u64().unwrap_or(0),
                        tx + iface["tx_bytes"].as_u64().unwrap_or(0),
                    )
                })
            })
            .unwrap_or((0, 0));

        let (blk_read, blk_write) = json["blkio_stats"]["io_service_bytes_recursive"]
            .as_array()
            .map(|entries| {
                entries
                    .iter()
                    .fold((0u64, 0u64), |(r, w), e| match e["op"].as_str() {
                        Some("Read") => (r + e["value"].as_u64().unwrap_or(0), w),
                        Some("Write") => (r, w + e["value"].as_u64().unwrap_or(0)),
                        _ => (r, w),
                    })
            })
            .unwrap_or((0, 0));

        let pids = json["pids_stats"]["current"].as_u64().unwrap_or(0);
        let name = json["name"]
            .as_str()
            .unwrap_or_default()
            .trim_start_matches('/')
            .to_string();

        Ok(ContainerStats {
            id: id.to_string(),
            name,
            cpu_percent,
            memory_usage: mem_usage,
            memory_limit: mem_limit,
            memory_percent: mem_percent,
            net_rx_bytes: net_rx,
            net_tx_bytes: net_tx,
            block_read: blk_read,
            block_write: blk_write,
            pids,
        })
    }

    fn exec_in_container(&self, id: &str, cmd: &[&str]) -> Result<String, ContainerError> {
        use serde_json::json;
        let body = json!({
            "AttachStdout": true,
            "AttachStderr": true,
            "Cmd": cmd,
        });
        let create_resp = self.post(
            &format!("{API}/containers/{id}/exec"),
            Some(&body.to_string()),
        )?;
        let exec_id = serde_json::from_str::<Value>(&create_resp.body)?["Id"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        let start_body = json!({"Detach": false, "Tty": false});
        let start_resp = http::request(
            &self.socket,
            "POST",
            &format!("{API}/exec/{exec_id}/start"),
            Some(&start_body.to_string()),
        )?;
        Ok(strip_log_frames(&start_resp.body))
    }

    fn list_images(&self) -> Result<Vec<Image>, ContainerError> {
        let json = self.get(&format!("{API}/images/json"))?;
        Ok(json
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(parse_image)
            .collect())
    }

    fn pull_image(&self, reference: &str) -> Result<(), ContainerError> {
        let encoded = url_encode(reference);
        http::request(
            &self.socket,
            "POST",
            &format!("{API}/images/create?fromImage={encoded}"),
            None,
        )?;
        Ok(())
    }

    fn pull_image_streaming(
        &self,
        reference: &str,
        tx: async_channel::Sender<PullProgress>,
    ) -> Result<(), ContainerError> {
        if reference.contains(":::") || reference.is_empty() {
            return Err(ContainerError::ParseError(format!(
                "invalid image reference: {reference}"
            )));
        }
        let encoded = url_encode(reference);
        let resp = http::request(
            &self.socket,
            "POST",
            &format!("{API}/images/create?fromImage={encoded}"),
            None,
        )?;
        // Docker returns newline-delimited JSON progress events
        for line in resp.body.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(v) = serde_json::from_str::<Value>(line) {
                let layer_id = v["id"].as_str().unwrap_or("").to_string();
                if layer_id.is_empty() {
                    continue;
                }
                let status_str = v["status"].as_str().unwrap_or("Waiting");
                let current = v["progressDetail"]["current"].as_u64().unwrap_or(0);
                let total = v["progressDetail"]["total"].as_u64().unwrap_or(0);
                let (status, percent) = map_docker_pull_status(status_str, current, total);
                if tx
                    .try_send(PullProgress {
                        layer_id,
                        status,
                        percent,
                    })
                    .is_err()
                {
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    fn cancel_pull(&self) {
        // Best-effort no-op for HTTP-based drivers
    }

    fn remove_image(&self, id: &str, force: bool) -> Result<(), ContainerError> {
        self.delete(&format!("{API}/images/{id}?force={}", force as u8))
    }

    fn tag_image(&self, source: &str, target: &str) -> Result<(), ContainerError> {
        let (repo, tag) = target.split_once(':').unwrap_or((target, "latest"));
        self.post(
            &format!("{API}/images/{source}/tag?repo={repo}&tag={tag}"),
            None,
        )?;
        Ok(())
    }

    fn inspect_image(&self, id: &str) -> Result<Image, ContainerError> {
        let json = self.get(&format!("{API}/images/{id}/json"))?;
        Ok(parse_image(&json))
    }

    fn list_volumes(&self) -> Result<Vec<Volume>, ContainerError> {
        let json = self.get(&format!("{API}/volumes"))?;
        Ok(json["Volumes"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(parse_volume)
            .collect())
    }

    fn create_volume(&self, opts: &CreateVolumeOptions) -> Result<Volume, ContainerError> {
        use serde_json::json;
        let body = json!({
            "Name": opts.name,
            "Driver": opts.driver,
            "Labels": opts.labels,
        });
        let resp = self.post(&format!("{API}/volumes/create"), Some(&body.to_string()))?;
        let json: Value = serde_json::from_str(&resp.body)?;
        Ok(parse_volume(&json))
    }

    fn remove_volume(&self, name: &str, force: bool) -> Result<(), ContainerError> {
        self.delete(&format!("{API}/volumes/{name}?force={}", force as u8))
    }

    fn list_networks(&self) -> Result<Vec<Network>, ContainerError> {
        let json = self.get(&format!("{API}/networks"))?;
        Ok(json
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(parse_network)
            .collect())
    }

    fn create_network(&self, opts: &CreateNetworkOptions) -> Result<Network, ContainerError> {
        let mut body_obj = serde_json::json!({
            "Name": opts.name,
            "Driver": opts.driver,
        });
        if let Some(ref subnet) = opts.subnet {
            body_obj["IPAM"] = serde_json::json!({
                "Config": [{"Subnet": subnet}]
            });
        }
        let resp = self.post(
            &format!("{API}/networks/create"),
            Some(&body_obj.to_string()),
        )?;
        let json: Value = serde_json::from_str(&resp.body)?;
        let id = json["Id"].as_str().unwrap_or(&opts.name).to_string();
        Ok(Network {
            id,
            name: opts.name.clone(),
            driver: opts.driver.clone(),
            scope: "local".to_string(),
            internal: false,
            created: String::new(),
            subnet: opts.subnet.clone(),
            gateway: None,
            containers_count: 0,
        })
    }

    fn remove_network(&self, id: &str) -> Result<(), ContainerError> {
        self.delete(&format!("{API}/networks/{id}"))
    }

    fn ping(&self) -> Result<(), ContainerError> {
        let resp = http::request(&self.socket, "GET", "/_ping", None)?;
        if resp.status == 200 {
            Ok(())
        } else {
            Err(ContainerError::ConnectionFailed(format!(
                "ping returned {}",
                resp.status
            )))
        }
    }

    fn version(&self) -> Result<String, ContainerError> {
        let json = self.get(&format!("{API}/version"))?;
        Ok(format!(
            "Docker {} (API {})",
            json["Version"].as_str().unwrap_or("?"),
            json["ApiVersion"].as_str().unwrap_or("?")
        ))
    }

    fn system_df(&self) -> Result<SystemUsage, ContainerError> {
        let json = self.get(&format!("{API}/system/df"))?;

        let containers_total = json["Containers"]
            .as_array()
            .map(|a| a.len() as u64)
            .unwrap_or(0);
        let containers_running = json["Containers"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter(|c| c["State"].as_str() == Some("running"))
                    .count() as u64
            })
            .unwrap_or(0);

        let images_total = json["Images"]
            .as_array()
            .map(|a| a.len() as u64)
            .unwrap_or(0);
        let images_size = json["Images"]
            .as_array()
            .map(|a| a.iter().map(|i| i["Size"].as_u64().unwrap_or(0)).sum())
            .unwrap_or(0);

        let volumes_total = json["Volumes"]
            .as_array()
            .map(|a| a.len() as u64)
            .unwrap_or(0);
        let volumes_size = json["Volumes"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|v| v["UsageData"]["Size"].as_u64().unwrap_or(0))
                    .sum()
            })
            .unwrap_or(0);

        Ok(SystemUsage {
            containers_total,
            containers_running,
            containers_stopped: containers_total.saturating_sub(containers_running),
            images_total,
            images_size,
            volumes_total,
            volumes_size,
        })
    }

    fn inspect_image_layers(&self, id: &str) -> Result<Vec<ImageLayer>, ContainerError> {
        let json = self.get(&format!("{API}/images/{id}/history"))?;
        let layers = json
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|v| {
                let raw_id = v["Id"].as_str().unwrap_or("<missing>");
                let short_id = if raw_id == "<missing>" {
                    raw_id.to_string()
                } else {
                    raw_id
                        .strip_prefix("sha256:")
                        .unwrap_or(raw_id)
                        .chars()
                        .take(12)
                        .collect()
                };
                ImageLayer {
                    id: short_id,
                    cmd: v["CreatedBy"].as_str().unwrap_or("").to_string(),
                    size: v["Size"].as_u64().unwrap_or(0),
                }
            })
            .collect();
        Ok(layers)
    }

    fn prune_system(&self, volumes: bool) -> Result<PruneReport, ContainerError> {
        let mut report = PruneReport::default();

        let resp = self.post(&format!("{API}/containers/prune"), None)?;
        let json: Value = serde_json::from_str(&resp.body).unwrap_or_default();
        report.containers_deleted = json["ContainersDeleted"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        report.space_reclaimed += json["SpaceReclaimed"].as_u64().unwrap_or(0);

        let resp = self.post(&format!("{API}/images/prune"), None)?;
        let json: Value = serde_json::from_str(&resp.body).unwrap_or_default();
        report.images_deleted = json["ImagesDeleted"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v["Deleted"].as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        report.space_reclaimed += json["SpaceReclaimed"].as_u64().unwrap_or(0);

        if volumes {
            let resp = self.post(&format!("{API}/volumes/prune"), None)?;
            let json: Value = serde_json::from_str(&resp.body).unwrap_or_default();
            report.volumes_deleted = json["VolumesDeleted"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            report.space_reclaimed += json["SpaceReclaimed"].as_u64().unwrap_or(0);
        }

        Ok(report)
    }

    fn system_events(
        &self,
        since: Option<i64>,
        limit: Option<usize>,
    ) -> Result<Vec<ContainerEvent>, ContainerError> {
        // `until` is mandatory: without it /events is a streaming endpoint that
        // never closes, causing read_to_string() to block for the full READ_TIMEOUT.
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let since_param = since.map(|s| format!("&since={s}")).unwrap_or_default();
        let path = format!("{API}/events?until={now}{since_param}");
        let resp = http::request(&self.socket, "GET", &path, None)?;
        let mut events: Vec<ContainerEvent> = resp
            .body
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| serde_json::from_str::<Value>(l).ok())
            .map(|v| {
                let ts = v["time"].as_i64().unwrap_or(0);
                ContainerEvent {
                    timestamp: ts.to_string(),
                    event_type: v["Type"].as_str().unwrap_or("").to_string(),
                    action: v["Action"].as_str().unwrap_or("").to_string(),
                    actor: v["Actor"]["Attributes"]["name"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    actor_id: v["Actor"]["ID"]
                        .as_str()
                        .unwrap_or("")
                        .chars()
                        .take(12)
                        .collect(),
                }
            })
            .collect();
        if let Some(n) = limit {
            events.truncate(n);
        }
        Ok(events)
    }
}

fn url_encode(s: &str) -> String {
    s.replace(':', "%3A")
        .replace('/', "%2F")
        .replace('@', "%40")
}

fn chrono_parse_or_zero(s: &str) -> i64 {
    // Best-effort: just count from 0 if we can't parse
    if let Ok(n) = s.parse::<i64>() {
        return n;
    }
    0
}

fn map_docker_pull_status(status: &str, current: u64, total: u64) -> (PullStatus, Option<u8>) {
    match status {
        "Pull complete" | "Already exists" => (PullStatus::Done, Some(100)),
        "Verifying Checksum" | "Download complete" => (PullStatus::Verifying, Some(100)),
        "Pulling fs layer" | "Waiting" => (PullStatus::Waiting, None),
        "Downloading" => {
            let pct = (current * 100).checked_div(total).unwrap_or(0).min(100) as u8;
            (PullStatus::Downloading(pct), Some(pct))
        }
        _ => (PullStatus::Pulling, None),
    }
}
