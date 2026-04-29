// SPDX-License-Identifier: GPL-3.0-or-later
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

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

fn make_container(
    id: &str,
    name: &str,
    image: &str,
    status: ContainerStatus,
    compose: Option<&str>,
    env: Vec<&str>,
) -> Container {
    let mut labels = HashMap::new();
    if let Some(project) = compose {
        labels.insert(
            "com.docker.compose.project".to_string(),
            project.to_string(),
        );
    }
    Container {
        id: id.to_string(),
        short_id: id[..12].to_string(),
        name: name.to_string(),
        image: image.to_string(),
        command: "/entrypoint.sh".to_string(),
        created: 1_700_000_000,
        status_text: status.label().to_string(),
        status,
        ports: vec![ContainerPort {
            host_ip: Some("0.0.0.0".to_string()),
            host_port: Some(8080),
            container_port: 80,
            protocol: "tcp".to_string(),
        }],
        compose_project: compose.map(str::to_string),
        env: env.into_iter().map(str::to_string).collect(),
        labels,
        mounts: vec![],
        networks: vec!["bridge".to_string()],
    }
}

pub struct MockContainerDriver {
    running: Mutex<Vec<String>>,
    containers: Mutex<Vec<Container>>,
    available: bool,
    pull_cancelled: Arc<AtomicBool>,
}

impl MockContainerDriver {
    pub fn new() -> Self {
        let initial_running = vec!["aabbccdd1122".to_string()];
        let containers = vec![
            make_container(
                "aabbccdd1122334455667788",
                "web-server",
                "nginx:latest",
                ContainerStatus::Running,
                Some("web-stack"),
                vec!["NGINX_HOST=example.com", "NGINX_PORT=80", "TZ=UTC"],
            ),
            make_container(
                "112233445566778899aabbcc",
                "db",
                "postgres:15",
                ContainerStatus::Exited(0),
                Some("web-stack"),
                vec![
                    "POSTGRES_DB=appdb",
                    "POSTGRES_USER=app",
                    "POSTGRES_PASSWORD=secret123",
                ],
            ),
            make_container(
                "223344556677889900aabbcc",
                "standalone",
                "redis:alpine",
                ContainerStatus::Stopped,
                None,
                vec!["REDIS_MAXMEMORY=256mb"],
            ),
        ];
        Self {
            running: Mutex::new(initial_running),
            containers: Mutex::new(containers),
            available: true,
            pull_cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn unavailable() -> Self {
        Self {
            running: Mutex::new(vec![]),
            containers: Mutex::new(vec![]),
            available: false,
            pull_cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Default for MockContainerDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl IContainerDriver for MockContainerDriver {
    fn runtime_name(&self) -> &str {
        "Mock"
    }

    fn is_available(&self) -> bool {
        self.available
    }

    fn list_containers(&self, all: bool) -> Result<Vec<Container>, ContainerError> {
        let running = self.running.lock().unwrap();
        let mut containers = self.containers.lock().unwrap().clone();
        for c in &mut containers {
            if running.contains(&c.short_id) {
                c.status = ContainerStatus::Running;
                c.status_text = "Running".to_string();
            } else if matches!(
                c.status,
                ContainerStatus::Running | ContainerStatus::Restarting
            ) {
                c.status = ContainerStatus::Stopped;
                c.status_text = "Stopped".to_string();
            }
        }
        if all {
            Ok(containers)
        } else {
            Ok(containers
                .into_iter()
                .filter(|c| c.status.is_running())
                .collect())
        }
    }

    fn inspect_container(&self, id: &str) -> Result<Container, ContainerError> {
        self.list_containers(true)?
            .into_iter()
            .find(|c| c.id.starts_with(id) || c.short_id.starts_with(id))
            .ok_or_else(|| ContainerError::NotFound(id.to_string()))
    }

    fn inspect_container_json(&self, id: &str) -> Result<String, ContainerError> {
        let c = self.inspect_container(id)?;
        let json = serde_json::json!({
            "Id": c.id,
            "Name": format!("/{}", c.name),
            "State": {
                "Status": c.status_text.to_lowercase(),
                "Running": matches!(c.status, crate::core::domain::container::ContainerStatus::Running),
            },
            "Config": {
                "Image": c.image,
                "Env": c.env,
                "Labels": c.labels,
            },
        });
        Ok(serde_json::to_string_pretty(&json)?)
    }

    fn start_container(&self, id: &str) -> Result<(), ContainerError> {
        let short = id.chars().take(12).collect::<String>();
        self.running.lock().unwrap().push(short);
        Ok(())
    }

    fn stop_container(&self, id: &str, _timeout_secs: Option<u32>) -> Result<(), ContainerError> {
        let short = id.chars().take(12).collect::<String>();
        self.running.lock().unwrap().retain(|r| r != &short);
        Ok(())
    }

    fn restart_container(
        &self,
        id: &str,
        _timeout_secs: Option<u32>,
    ) -> Result<(), ContainerError> {
        self.inspect_container(id)?;
        Ok(())
    }

    fn pause_container(&self, id: &str) -> Result<(), ContainerError> {
        self.inspect_container(id)?;
        let short = id.chars().take(12).collect::<String>();
        if !self.running.lock().unwrap().contains(&short) {
            return Err(ContainerError::NotRunning(id.to_string()));
        }
        Ok(())
    }

    fn unpause_container(&self, id: &str) -> Result<(), ContainerError> {
        self.inspect_container(id)?;
        let short = id.chars().take(12).collect::<String>();
        if !self.running.lock().unwrap().contains(&short) {
            return Err(ContainerError::NotRunning(id.to_string()));
        }
        Ok(())
    }

    fn remove_container(
        &self,
        id: &str,
        _force: bool,
        _remove_volumes: bool,
    ) -> Result<(), ContainerError> {
        let short = id.chars().take(12).collect::<String>();
        self.running.lock().unwrap().retain(|r| r != &short);
        self.containers
            .lock()
            .unwrap()
            .retain(|c| c.short_id != short && !c.id.starts_with(id));
        Ok(())
    }

    fn create_container(&self, opts: &CreateContainerOptions) -> Result<String, ContainerError> {
        // Validate: image must exist
        let known: Vec<String> = self
            .list_images()?
            .into_iter()
            .flat_map(|i| i.tags)
            .collect();
        if !opts.image.is_empty() && !known.contains(&opts.image) {
            return Err(ContainerError::NotFound(format!(
                "image not found: {}",
                opts.image
            )));
        }

        // Validate: name conflict
        if let Some(ref name) = opts.name {
            let exists = self
                .containers
                .lock()
                .unwrap()
                .iter()
                .any(|c| &c.name == name);
            if exists {
                return Err(ContainerError::AlreadyExists(name.clone()));
            }
        }

        let new_id = "fakecontainerid0000000000".to_string();
        let name = opts
            .name
            .clone()
            .unwrap_or_else(|| format!("container-{}", &new_id[..8]));
        let c = make_container(
            &new_id,
            &name,
            &opts.image,
            ContainerStatus::Running,
            None,
            opts.env.iter().map(String::as_str).collect(),
        );
        self.containers.lock().unwrap().push(c);
        self.running.lock().unwrap().push(new_id[..12].to_string());
        Ok(new_id)
    }

    fn rename_container(&self, _id: &str, _new_name: &str) -> Result<(), ContainerError> {
        Ok(())
    }

    fn container_logs(
        &self,
        id: &str,
        tail: Option<u32>,
        _timestamps: bool,
    ) -> Result<String, ContainerError> {
        // Validate container exists (logs available for both running and stopped)
        self.inspect_container(id)?;
        let all_lines = [
            "2024-01-01T00:00:00Z server started",
            "2024-01-01T00:00:01Z initializing database",
            "2024-01-01T00:00:02Z listening on :8080",
            "2024-01-01T00:00:03Z ready",
            "2024-01-01T00:00:04Z accepted connection",
            "2024-01-01T00:00:05Z request GET /",
            "2024-01-01T00:00:06Z response 200",
        ];
        let lines: Vec<&str> = match tail {
            Some(n) => {
                let skip = all_lines.len().saturating_sub(n as usize);
                all_lines[skip..].to_vec()
            }
            None => all_lines.to_vec(),
        };
        Ok(lines.join("\n") + "\n")
    }

    fn container_stats(&self, id: &str) -> Result<ContainerStats, ContainerError> {
        // Return NotRunning for containers not in the running set
        let short = id.chars().take(12).collect::<String>();
        let running = self.running.lock().unwrap();
        if !running.contains(&short) {
            return Err(ContainerError::NotRunning(id.to_string()));
        }
        Ok(ContainerStats {
            id: id.to_string(),
            name: "mock-container".to_string(),
            cpu_percent: 2.5,
            memory_usage: 52_428_800, // 50 MiB
            memory_limit: 2_147_483_648,
            memory_percent: 2.44,
            net_rx_bytes: 1024,
            net_tx_bytes: 512,
            block_read: 0,
            block_write: 0,
            pids: 3,
        })
    }

    fn exec_in_container(&self, id: &str, cmd: &[&str]) -> Result<String, ContainerError> {
        // Validate container exists
        self.inspect_container(id)?;
        // Must be running to exec
        let short = id.chars().take(12).collect::<String>();
        if !self.running.lock().unwrap().contains(&short) {
            return Err(ContainerError::NotRunning(id.to_string()));
        }
        if cmd.is_empty() {
            return Ok(String::new());
        }
        Ok(format!("mock output for: {}\n", cmd.join(" ")))
    }

    fn list_images(&self) -> Result<Vec<Image>, ContainerError> {
        Ok(vec![
            Image {
                id: "sha256:aaaa".to_string(),
                short_id: "aaaa".to_string(),
                tags: vec!["nginx:latest".to_string()],
                size: 187_000_000,
                created: 1_700_000_000,
                digest: None,
                labels: HashMap::new(),
                in_use: true,
            },
            Image {
                id: "sha256:bbbb".to_string(),
                short_id: "bbbb".to_string(),
                tags: vec!["postgres:15".to_string()],
                size: 379_000_000,
                created: 1_690_000_000,
                digest: None,
                labels: HashMap::new(),
                in_use: false,
            },
            Image {
                id: "sha256:cccc".to_string(),
                short_id: "cccc".to_string(),
                tags: vec![],
                size: 45_000_000,
                created: 1_680_000_000,
                digest: None,
                labels: HashMap::new(),
                in_use: false,
            },
        ])
    }

    fn pull_image(&self, reference: &str) -> Result<(), ContainerError> {
        // Validate: reject obviously malformed references (e.g., contains ":::")
        if reference.contains(":::") || reference.is_empty() {
            return Err(ContainerError::ParseError(format!(
                "invalid image reference: {reference}"
            )));
        }
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
        let layers = ["sha256:aaa111", "sha256:bbb222", "sha256:ccc333"];
        for layer_id in &layers {
            if self.pull_cancelled.load(Ordering::Relaxed) {
                return Ok(());
            }
            let events: &[PullProgress] = &[
                PullProgress {
                    layer_id: layer_id.to_string(),
                    status: PullStatus::Waiting,
                    percent: None,
                },
                PullProgress {
                    layer_id: layer_id.to_string(),
                    status: PullStatus::Downloading(50),
                    percent: Some(50),
                },
                PullProgress {
                    layer_id: layer_id.to_string(),
                    status: PullStatus::Downloading(100),
                    percent: Some(100),
                },
                PullProgress {
                    layer_id: layer_id.to_string(),
                    status: PullStatus::Done,
                    percent: Some(100),
                },
            ];
            for event in events {
                if tx.try_send(event.clone()).is_err() {
                    return Ok(());
                }
                if self.pull_cancelled.load(Ordering::Relaxed) {
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    fn cancel_pull(&self) {
        self.pull_cancelled.store(true, Ordering::Relaxed);
    }

    fn remove_image(&self, id: &str, _force: bool) -> Result<(), ContainerError> {
        self.inspect_image(id)?;
        Ok(())
    }

    fn tag_image(&self, _source: &str, _target: &str) -> Result<(), ContainerError> {
        Ok(())
    }

    fn inspect_image(&self, id: &str) -> Result<Image, ContainerError> {
        self.list_images()?
            .into_iter()
            .find(|i| i.id.contains(id) || i.short_id.starts_with(id))
            .ok_or_else(|| ContainerError::NotFound(id.to_string()))
    }

    fn inspect_image_layers(&self, id: &str) -> Result<Vec<ImageLayer>, ContainerError> {
        self.inspect_image(id)?;
        Ok(vec![
            ImageLayer {
                id: "a1b2c3d4e5f6".to_string(),
                cmd: "FROM debian:bookworm-slim".to_string(),
                size: 30_000_000,
            },
            ImageLayer {
                id: "b2c3d4e5f6a1".to_string(),
                cmd: "RUN apt-get update && apt-get install -y curl ca-certificates".to_string(),
                size: 25_000_000,
            },
            ImageLayer {
                id: "c3d4e5f6a1b2".to_string(),
                cmd: "COPY . /app".to_string(),
                size: 1_000_000,
            },
        ])
    }

    fn list_volumes(&self) -> Result<Vec<Volume>, ContainerError> {
        Ok(vec![
            Volume {
                name: "postgres-data".to_string(),
                driver: "local".to_string(),
                mountpoint: "/var/lib/docker/volumes/postgres-data/_data".to_string(),
                created: "2024-01-01T00:00:00Z".to_string(),
                labels: HashMap::new(),
                scope: "local".to_string(),
                size_bytes: Some(2_415_919_104),
                in_use: true,
            },
            Volume {
                name: "orphan-data".to_string(),
                driver: "local".to_string(),
                mountpoint: "/var/lib/docker/volumes/orphan-data/_data".to_string(),
                created: "2024-02-01T00:00:00Z".to_string(),
                labels: HashMap::new(),
                scope: "local".to_string(),
                size_bytes: Some(150_000_000),
                in_use: false,
            },
        ])
    }

    fn create_volume(&self, opts: &CreateVolumeOptions) -> Result<Volume, ContainerError> {
        Ok(Volume {
            name: opts.name.clone(),
            driver: opts.driver.clone(),
            mountpoint: format!("/var/lib/docker/volumes/{}/_data", opts.name),
            created: "2024-01-01T00:00:00Z".to_string(),
            labels: opts.labels.clone(),
            scope: "local".to_string(),
            size_bytes: None,
            in_use: false,
        })
    }

    fn remove_volume(&self, name: &str, _force: bool) -> Result<(), ContainerError> {
        let exists = self.list_volumes()?.into_iter().any(|v| v.name == name);
        if !exists {
            return Err(ContainerError::NotFound(format!(
                "volume not found: {name}"
            )));
        }
        Ok(())
    }

    fn list_networks(&self) -> Result<Vec<Network>, ContainerError> {
        Ok(vec![
            Network {
                id: "net-bridge-id".to_string(),
                name: "bridge".to_string(),
                driver: "bridge".to_string(),
                scope: "local".to_string(),
                internal: false,
                created: "2024-01-01T00:00:00Z".to_string(),
                subnet: Some("172.17.0.0/16".to_string()),
                gateway: Some("172.17.0.1".to_string()),
                containers_count: 2,
            },
            Network {
                id: "net-host-id".to_string(),
                name: "host".to_string(),
                driver: "host".to_string(),
                scope: "host".to_string(),
                internal: false,
                created: "2024-01-01T00:00:00Z".to_string(),
                subnet: None,
                gateway: None,
                containers_count: 0,
            },
        ])
    }

    fn create_network(&self, opts: &CreateNetworkOptions) -> Result<Network, ContainerError> {
        Ok(Network {
            id: format!("net-{}-id", opts.name),
            name: opts.name.clone(),
            driver: opts.driver.clone(),
            scope: "local".to_string(),
            internal: false,
            created: "2024-01-01T00:00:00Z".to_string(),
            subnet: opts.subnet.clone(),
            gateway: None,
            containers_count: 0,
        })
    }

    fn remove_network(&self, id: &str) -> Result<(), ContainerError> {
        let exists = self
            .list_networks()?
            .into_iter()
            .any(|n| n.id == id || n.name == id);
        if !exists {
            return Err(ContainerError::NotFound(format!("network not found: {id}")));
        }
        Ok(())
    }

    fn ping(&self) -> Result<(), ContainerError> {
        if self.available {
            Ok(())
        } else {
            Err(ContainerError::ConnectionFailed(
                "mock not available".into(),
            ))
        }
    }

    fn version(&self) -> Result<String, ContainerError> {
        Ok("Mock 1.0.0".to_string())
    }

    fn system_df(&self) -> Result<SystemUsage, ContainerError> {
        let containers = self.list_containers(true)?;
        let running = containers.iter().filter(|c| c.status.is_running()).count() as u64;
        let total = containers.len() as u64;
        let images = self.list_images()?;
        Ok(SystemUsage {
            containers_total: total,
            containers_running: running,
            containers_stopped: total - running,
            images_total: images.len() as u64,
            images_size: images.iter().map(|i| i.size).sum(),
            volumes_total: 1,
            volumes_size: 0,
        })
    }

    fn host_stats(&self) -> Result<HostStats, ContainerError> {
        Ok(HostStats {
            cpu_percent: 25.0,
            mem_percent: 60.0,
            disk_percent: 45.0,
            disk_total_bytes: 100_000_000_000, // 100 GB sentinel for tests
        })
    }

    fn prune_system(&self, _volumes: bool) -> Result<PruneReport, ContainerError> {
        Ok(PruneReport {
            containers_deleted: vec!["112233445566".to_string()],
            images_deleted: vec![],
            volumes_deleted: vec![],
            space_reclaimed: 0,
        })
    }

    fn system_events(
        &self,
        _since: Option<i64>,
        limit: Option<usize>,
    ) -> Result<Vec<ContainerEvent>, ContainerError> {
        let events = vec![
            ContainerEvent {
                timestamp: "12:00:00".to_string(),
                event_type: "container".to_string(),
                action: "start".to_string(),
                actor: "web-server".to_string(),
                actor_id: "aabbccdd1122".to_string(),
            },
            ContainerEvent {
                timestamp: "11:59:00".to_string(),
                event_type: "container".to_string(),
                action: "stop".to_string(),
                actor: "db".to_string(),
                actor_id: "112233445566".to_string(),
            },
            ContainerEvent {
                timestamp: "11:58:00".to_string(),
                event_type: "image".to_string(),
                action: "pull".to_string(),
                actor: "nginx:latest".to_string(),
                actor_id: "sha256:aaaa".to_string(),
            },
        ];
        if let Some(n) = limit {
            Ok(events.into_iter().take(n).collect())
        } else {
            Ok(events)
        }
    }
}
