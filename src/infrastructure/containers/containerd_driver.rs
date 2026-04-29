// SPDX-License-Identifier: GPL-3.0-or-later
//
// Adapter for containerd via nerdctl CLI.
//
// containerd's native API is gRPC (heavy protobuf deps). Instead we shell out to
// `nerdctl`, which mirrors the Docker CLI surface and emits JSON-compatible output.
// This keeps the dependency footprint minimal while covering the same feature set.

use std::collections::HashMap;
use std::process::{Command, Output};

use serde_json::Value;

use crate::core::domain::{
    container::{
        Container, ContainerPort, ContainerStats, ContainerStatus, CreateContainerOptions,
        PullProgress, PullStatus,
    },
    image::{Image, ImageLayer},
    network::{ContainerEvent, CreateNetworkOptions, HostStats, Network, PruneReport, SystemUsage},
    volume::{CreateVolumeOptions, Volume},
};
use crate::infrastructure::containers::error::ContainerError;
use crate::ports::i_container_driver::IContainerDriver;

pub struct ContainerdDriver {
    nerdctl: String,
    namespace: String,
}

impl ContainerdDriver {
    pub fn new(nerdctl_path: impl Into<String>, namespace: impl Into<String>) -> Self {
        Self {
            nerdctl: nerdctl_path.into(),
            namespace: namespace.into(),
        }
    }

    pub fn detect() -> Option<Self> {
        let output = Command::new("nerdctl").arg("version").output().ok()?;
        if output.status.success() {
            return Some(Self::new("nerdctl", "default"));
        }
        None
    }

    fn run(&self, args: &[&str]) -> Result<String, ContainerError> {
        let mut cmd = Command::new(&self.nerdctl);
        cmd.args(["--namespace", &self.namespace]);
        cmd.args(args);
        let out: Output = cmd.output()?;
        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).into_owned())
        } else {
            Err(ContainerError::SubprocessFailed {
                code: out.status.code(),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            })
        }
    }

    fn run_json(&self, args: &[&str]) -> Result<Value, ContainerError> {
        let stdout = self.run(args)?;
        // nerdctl --format json outputs one JSON object per line for lists
        let trimmed = stdout.trim();
        if trimmed.starts_with('[') {
            Ok(serde_json::from_str(trimmed)?)
        } else {
            // Newline-delimited JSON → wrap in array
            let items: Result<Vec<Value>, _> = trimmed
                .lines()
                .filter(|l| !l.is_empty())
                .map(serde_json::from_str)
                .collect();
            Ok(Value::Array(
                items.map_err(|e| ContainerError::ParseError(e.to_string()))?,
            ))
        }
    }
}

fn nerdctl_container(v: &Value) -> Container {
    let id = v["ID"].as_str().unwrap_or_default().to_string();
    let short_id = id.chars().take(12).collect();
    let name = v["Names"].as_str().unwrap_or_default().to_string();
    let image = v["Image"].as_str().unwrap_or_default().to_string();
    let command = v["Command"]
        .as_str()
        .unwrap_or_default()
        .trim_matches('"')
        .to_string();
    let created = 0i64; // nerdctl doesn't expose epoch in list JSON
    let state_str = v["Status"].as_str().unwrap_or("unknown").to_lowercase();
    let status = if state_str.starts_with("up") {
        ContainerStatus::Running
    } else if state_str.starts_with("exited") {
        ContainerStatus::Exited(0)
    } else if state_str.starts_with("paused") {
        ContainerStatus::Paused
    } else {
        ContainerStatus::Unknown(state_str.clone())
    };
    let ports: Vec<ContainerPort> = v["Ports"]
        .as_str()
        .unwrap_or_default()
        .split(',')
        .filter_map(|p| {
            // Format: "0.0.0.0:8080->80/tcp"
            let p = p.trim();
            if p.is_empty() {
                return None;
            }
            let (host_part, ctr_part) = p.split_once("->")?;
            let (ctr_port_str, proto) = ctr_part.split_once('/')?;
            let container_port: u16 = ctr_port_str.trim().parse().ok()?;
            let (host_ip, host_port) = if host_part.contains(':') {
                let (ip, port) = host_part.rsplit_once(':')?;
                (Some(ip.to_string()), port.trim().parse::<u16>().ok())
            } else {
                (None, host_part.trim().parse::<u16>().ok())
            };
            Some(ContainerPort {
                host_ip,
                host_port,
                container_port,
                protocol: proto.trim().to_string(),
            })
        })
        .collect();

    Container {
        id,
        short_id,
        name,
        image,
        command,
        created,
        status,
        status_text: v["Status"].as_str().unwrap_or_default().to_string(),
        ports,
        labels: HashMap::new(),
        mounts: vec![],
        env: vec![],
        compose_project: None,
        networks: vec![],
    }
}

fn nerdctl_image(v: &Value) -> Image {
    let id = v["ID"].as_str().unwrap_or_default().to_string();
    let short_id = id.chars().take(12).collect();
    let tag = format!(
        "{}:{}",
        v["Repository"].as_str().unwrap_or("<none>"),
        v["Tag"].as_str().unwrap_or("<none>")
    );
    let size = v["Size"].as_str().and_then(parse_size_str).unwrap_or(0);
    Image {
        id,
        short_id,
        tags: vec![tag],
        size,
        created: 0,
        digest: None,
        labels: HashMap::new(),
        in_use: false,
    }
}

fn parse_size_str(s: &str) -> Option<u64> {
    // nerdctl reports e.g. "77.8 MiB"
    let s = s.trim();
    let (num, unit) = s.split_once(' ')?;
    let n: f64 = num.parse().ok()?;
    let mult = match unit {
        "B" => 1.0,
        "KiB" | "kB" => 1024.0,
        "MiB" | "MB" => 1_048_576.0,
        "GiB" | "GB" => 1_073_741_824.0,
        _ => 1.0,
    };
    Some((n * mult) as u64)
}

impl IContainerDriver for ContainerdDriver {
    fn runtime_name(&self) -> &str {
        "containerd (nerdctl)"
    }

    fn is_available(&self) -> bool {
        self.ping().is_ok()
    }

    fn list_containers(&self, all: bool) -> Result<Vec<Container>, ContainerError> {
        let mut args = vec!["ps", "--format", "json"];
        if all {
            args.push("-a");
        }
        let json = self.run_json(&args)?;
        Ok(json
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(nerdctl_container)
            .collect())
    }

    fn inspect_container(&self, id: &str) -> Result<Container, ContainerError> {
        let stdout = self.run(&["inspect", "--format", "json", id])?;
        let arr: Value = serde_json::from_str(stdout.trim())?;
        let v = arr
            .as_array()
            .and_then(|a| a.first())
            .ok_or_else(|| ContainerError::NotFound(id.to_string()))?;
        Ok(nerdctl_container(v))
    }

    fn inspect_container_json(&self, id: &str) -> Result<String, ContainerError> {
        let stdout = self.run(&["inspect", "--format", "json", id])?;
        // Validate it parses as JSON, then return pretty-printed
        let value: Value = serde_json::from_str(stdout.trim())?;
        Ok(serde_json::to_string_pretty(&value)?)
    }

    fn start_container(&self, id: &str) -> Result<(), ContainerError> {
        self.run(&["start", id])?;
        Ok(())
    }

    fn stop_container(&self, id: &str, timeout_secs: Option<u32>) -> Result<(), ContainerError> {
        let t = timeout_secs.unwrap_or(10).to_string();
        self.run(&["stop", "-t", &t, id])?;
        Ok(())
    }

    fn restart_container(&self, id: &str, timeout_secs: Option<u32>) -> Result<(), ContainerError> {
        let t = timeout_secs.unwrap_or(10).to_string();
        self.run(&["restart", "-t", &t, id])?;
        Ok(())
    }

    fn pause_container(&self, id: &str) -> Result<(), ContainerError> {
        self.run(&["pause", id])?;
        Ok(())
    }

    fn unpause_container(&self, id: &str) -> Result<(), ContainerError> {
        self.run(&["unpause", id])?;
        Ok(())
    }

    fn remove_container(&self, id: &str, force: bool, vols: bool) -> Result<(), ContainerError> {
        let mut args = vec!["rm"];
        if force {
            args.push("-f");
        }
        if vols {
            args.push("-v");
        }
        args.push(id);
        self.run(&args)?;
        Ok(())
    }

    fn create_container(&self, opts: &CreateContainerOptions) -> Result<String, ContainerError> {
        let mut args = vec!["create".to_string()];
        if let Some(name) = &opts.name {
            args.extend_from_slice(&["--name".to_string(), name.clone()]);
        }
        for env in &opts.env {
            args.extend_from_slice(&["-e".to_string(), env.clone()]);
        }
        for (hp, cp) in &opts.port_bindings {
            args.extend_from_slice(&["-p".to_string(), format!("{hp}:{cp}")]);
        }
        for (h, c) in &opts.volume_bindings {
            args.extend_from_slice(&["-v".to_string(), format!("{h}:{c}")]);
        }
        args.push(opts.image.clone());
        for cmd in &opts.command {
            args.push(cmd.clone());
        }
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let id = self.run(&refs)?.trim().to_string();
        Ok(id)
    }

    fn rename_container(&self, id: &str, new_name: &str) -> Result<(), ContainerError> {
        self.run(&["rename", id, new_name])?;
        Ok(())
    }

    fn container_logs(
        &self,
        id: &str,
        tail: Option<u32>,
        timestamps: bool,
    ) -> Result<String, ContainerError> {
        let mut args = vec!["logs".to_string()];
        if timestamps {
            args.push("-t".to_string());
        }
        if let Some(n) = tail {
            args.extend_from_slice(&["--tail".to_string(), n.to_string()]);
        }
        args.push(id.to_string());
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.run(&refs)
    }

    fn container_stats(&self, id: &str) -> Result<ContainerStats, ContainerError> {
        let stdout = self.run(&["stats", "--no-stream", "--format", "json", id])?;
        let v: Value = serde_json::from_str(stdout.trim()).unwrap_or_default();
        let cpu_str = v["CPUPerc"].as_str().unwrap_or("0%");
        let cpu = cpu_str.trim_end_matches('%').parse::<f64>().unwrap_or(0.0);
        Ok(ContainerStats {
            id: id.to_string(),
            name: v["Name"].as_str().unwrap_or_default().to_string(),
            cpu_percent: cpu,
            ..Default::default()
        })
    }

    fn exec_in_container(&self, id: &str, cmd: &[&str]) -> Result<String, ContainerError> {
        let mut args = vec!["exec", id];
        args.extend_from_slice(cmd);
        self.run(&args)
    }

    fn list_images(&self) -> Result<Vec<Image>, ContainerError> {
        let json = self.run_json(&["images", "--format", "json"])?;
        Ok(json
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(nerdctl_image)
            .collect())
    }

    fn pull_image(&self, reference: &str) -> Result<(), ContainerError> {
        self.run(&["pull", reference])?;
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
        self.run(&["pull", reference])?;
        // nerdctl doesn't emit per-layer JSON events; report a single Done.
        let _ = tx.try_send(PullProgress {
            layer_id: "complete".to_string(),
            status: PullStatus::Done,
            percent: Some(100),
        });
        Ok(())
    }

    fn cancel_pull(&self) {
        // Best-effort no-op for subprocess-based drivers
    }

    fn remove_image(&self, id: &str, force: bool) -> Result<(), ContainerError> {
        let mut args = vec!["rmi"];
        if force {
            args.push("-f");
        }
        args.push(id);
        self.run(&args)?;
        Ok(())
    }

    fn tag_image(&self, source: &str, target: &str) -> Result<(), ContainerError> {
        self.run(&["tag", source, target])?;
        Ok(())
    }

    fn inspect_image(&self, id: &str) -> Result<Image, ContainerError> {
        let stdout = self.run(&["image", "inspect", "--format", "json", id])?;
        let arr: Value = serde_json::from_str(stdout.trim())?;
        let v = arr
            .as_array()
            .and_then(|a| a.first())
            .ok_or_else(|| ContainerError::NotFound(id.to_string()))?;
        Ok(nerdctl_image(v))
    }

    fn list_volumes(&self) -> Result<Vec<Volume>, ContainerError> {
        let json = self.run_json(&["volume", "ls", "--format", "json"])?;
        Ok(json
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|v| Volume {
                name: v["Name"].as_str().unwrap_or_default().to_string(),
                driver: v["Driver"].as_str().unwrap_or("local").to_string(),
                mountpoint: v["Mountpoint"].as_str().unwrap_or_default().to_string(),
                created: String::new(),
                labels: HashMap::new(),
                scope: "local".to_string(),
                size_bytes: None,
                in_use: false,
            })
            .collect())
    }

    fn create_volume(&self, opts: &CreateVolumeOptions) -> Result<Volume, ContainerError> {
        let args = ["volume", "create", "--driver", &opts.driver];
        let label_args: Vec<String> = opts
            .labels
            .iter()
            .flat_map(|(k, v)| vec!["--label".to_string(), format!("{k}={v}")])
            .collect();
        let label_refs: Vec<&str> = label_args.iter().map(String::as_str).collect();
        let mut all: Vec<&str> = args
            .iter()
            .copied()
            .chain(label_refs.iter().copied())
            .collect();
        all.push(&opts.name);
        self.run(&all)?;
        Ok(Volume {
            name: opts.name.clone(),
            driver: opts.driver.clone(),
            mountpoint: String::new(),
            created: String::new(),
            labels: opts.labels.clone(),
            scope: "local".to_string(),
            size_bytes: None,
            in_use: false,
        })
    }

    fn remove_volume(&self, name: &str, force: bool) -> Result<(), ContainerError> {
        let mut args = vec!["volume", "rm"];
        if force {
            args.push("-f");
        }
        args.push(name);
        self.run(&args)?;
        Ok(())
    }

    fn list_networks(&self) -> Result<Vec<Network>, ContainerError> {
        let json = self.run_json(&["network", "ls", "--format", "json"])?;
        Ok(json
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|v| Network {
                id: v["ID"].as_str().unwrap_or_default().to_string(),
                name: v["Name"].as_str().unwrap_or_default().to_string(),
                driver: v["Driver"].as_str().unwrap_or_default().to_string(),
                scope: v["Scope"].as_str().unwrap_or_default().to_string(),
                internal: false,
                created: String::new(),
                subnet: None,
                gateway: None,
                containers_count: 0,
            })
            .collect())
    }

    fn create_network(&self, opts: &CreateNetworkOptions) -> Result<Network, ContainerError> {
        let mut args = vec!["network", "create", "--driver", &opts.driver];
        let subnet_str;
        if let Some(ref subnet) = opts.subnet {
            args.push("--subnet");
            subnet_str = subnet.clone();
            args.push(&subnet_str);
        }
        args.push(&opts.name);
        self.run(&args)?;
        Ok(Network {
            id: opts.name.clone(),
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
        self.run(&["network", "rm", id])?;
        Ok(())
    }

    fn ping(&self) -> Result<(), ContainerError> {
        let out = Command::new(&self.nerdctl)
            .args(["--namespace", &self.namespace, "version"])
            .output()?;
        if out.status.success() {
            Ok(())
        } else {
            Err(ContainerError::RuntimeNotAvailable(
                "nerdctl not available".into(),
            ))
        }
    }

    fn version(&self) -> Result<String, ContainerError> {
        let out = self.run(&["version", "--format", "json"])?;
        let v: Value = serde_json::from_str(out.trim()).unwrap_or_default();
        Ok(format!(
            "containerd/nerdctl {} (containerd {})",
            v["Client"]["Version"].as_str().unwrap_or("?"),
            v["Server"]["Components"]
                .as_array()
                .and_then(|a| a.first())
                .and_then(|c| c["Version"].as_str())
                .unwrap_or("?")
        ))
    }

    fn system_df(&self) -> Result<SystemUsage, ContainerError> {
        // nerdctl system df does not emit JSON yet — provide basic counts
        let containers = self.list_containers(true).unwrap_or_default();
        let images = self.list_images().unwrap_or_default();
        let volumes = self.list_volumes().unwrap_or_default();
        let running = containers.iter().filter(|c| c.status.is_running()).count() as u64;
        Ok(SystemUsage {
            containers_total: containers.len() as u64,
            containers_running: running,
            containers_stopped: containers.len() as u64 - running,
            images_total: images.len() as u64,
            images_size: images.iter().map(|i| i.size).sum(),
            volumes_total: volumes.len() as u64,
            volumes_size: 0,
        })
    }

    fn host_stats(&self) -> Result<HostStats, ContainerError> {
        crate::infrastructure::containers::host_stats::read_host_stats()
    }

    fn prune_system(&self, volumes: bool) -> Result<PruneReport, ContainerError> {
        let mut args = vec!["system", "prune", "-f"];
        if volumes {
            args.push("--volumes");
        }
        self.run(&args)?;
        Ok(PruneReport::default())
    }

    fn inspect_image_layers(&self, id: &str) -> Result<Vec<ImageLayer>, ContainerError> {
        let stdout = self.run(&["image", "history", "--format", "json", id])?;
        let arr: Value = serde_json::from_str(stdout.trim()).unwrap_or(Value::Array(vec![]));
        Ok(arr
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|v| ImageLayer {
                id: v["ID"]
                    .as_str()
                    .unwrap_or("<missing>")
                    .chars()
                    .take(12)
                    .collect(),
                cmd: v["CreatedBy"].as_str().unwrap_or("").to_string(),
                size: v["Size"].as_str().and_then(parse_size_str).unwrap_or(0),
            })
            .collect())
    }

    fn system_events(
        &self,
        _since: Option<i64>,
        _limit: Option<usize>,
    ) -> Result<Vec<ContainerEvent>, ContainerError> {
        Ok(vec![])
    }
}
