// SPDX-License-Identifier: GPL-3.0-or-later
use std::path::Path;

use crate::core::domain::{
    container::{Container, ContainerStats, CreateContainerOptions, PullProgress},
    image::{Image, ImageLayer},
    network::{ContainerEvent, CreateNetworkOptions, Network, PruneReport, SystemUsage},
    volume::{CreateVolumeOptions, Volume},
};
use crate::infrastructure::containers::{docker_driver::DockerDriver, error::ContainerError};
use crate::ports::i_container_driver::IContainerDriver;

pub struct PodmanDriver {
    inner: DockerDriver,
    socket_path: String,
}

impl PodmanDriver {
    pub fn new(socket: impl Into<String>) -> Self {
        let socket = socket.into();
        Self {
            inner: DockerDriver::new(socket.clone()),
            socket_path: socket,
        }
    }

    pub fn detect() -> Option<Self> {
        // Industry-standard override — checked before any path heuristic.
        if let Ok(host) = std::env::var("CONTAINER_HOST") {
            let path = host.trim_start_matches("unix://");
            if Path::new(path).exists() {
                return Some(Self::new(path));
            }
        }
        // Rootless socket (Linux, most common)
        let uid = libc_getuid();
        let rootless = format!("/run/user/{uid}/podman/podman.sock");
        if Path::new(&rootless).exists() {
            return Some(Self::new(rootless));
        }
        // Root socket (Linux)
        let root = "/run/podman/podman.sock";
        if Path::new(root).exists() {
            return Some(Self::new(root));
        }
        let home = std::env::var("HOME").unwrap_or_default();
        // macOS Podman 5.x — default machine (Apple Virtualization or QEMU)
        let mac_v5 = format!("{home}/.local/share/containers/podman/machine/default/podman.sock");
        if Path::new(&mac_v5).exists() {
            return Some(Self::new(mac_v5));
        }
        // macOS Podman 4.x — QEMU machine
        let mac_v4 = format!("{home}/.local/share/containers/podman/machine/qemu/podman.sock");
        if Path::new(&mac_v4).exists() {
            return Some(Self::new(mac_v4));
        }
        None
    }

    pub fn socket_path(&self) -> &str {
        &self.socket_path
    }
}

fn libc_getuid() -> u32 {
    #[cfg(unix)]
    {
        unsafe extern "C" {
            fn getuid() -> u32;
        }
        unsafe { getuid() }
    }
    #[cfg(not(unix))]
    {
        1000
    }
}

// Delegate every method to the inner DockerDriver — the Podman socket speaks
// the same Docker-compatible HTTP API, so no translation is needed.
impl IContainerDriver for PodmanDriver {
    fn runtime_name(&self) -> &str {
        "Podman"
    }

    fn is_available(&self) -> bool {
        Path::new(&self.socket_path).exists() && self.inner.ping().is_ok()
    }

    fn list_containers(&self, all: bool) -> Result<Vec<Container>, ContainerError> {
        self.inner.list_containers(all)
    }
    fn inspect_container(&self, id: &str) -> Result<Container, ContainerError> {
        self.inner.inspect_container(id)
    }
    fn inspect_container_json(&self, id: &str) -> Result<String, ContainerError> {
        self.inner.inspect_container_json(id)
    }
    fn start_container(&self, id: &str) -> Result<(), ContainerError> {
        self.inner.start_container(id)
    }
    fn stop_container(&self, id: &str, t: Option<u32>) -> Result<(), ContainerError> {
        self.inner.stop_container(id, t)
    }
    fn restart_container(&self, id: &str, t: Option<u32>) -> Result<(), ContainerError> {
        self.inner.restart_container(id, t)
    }
    fn pause_container(&self, id: &str) -> Result<(), ContainerError> {
        self.inner.pause_container(id)
    }
    fn unpause_container(&self, id: &str) -> Result<(), ContainerError> {
        self.inner.unpause_container(id)
    }
    fn remove_container(&self, id: &str, force: bool, vols: bool) -> Result<(), ContainerError> {
        self.inner.remove_container(id, force, vols)
    }
    fn create_container(&self, opts: &CreateContainerOptions) -> Result<String, ContainerError> {
        self.inner.create_container(opts)
    }
    fn rename_container(&self, id: &str, name: &str) -> Result<(), ContainerError> {
        self.inner.rename_container(id, name)
    }
    fn container_logs(
        &self,
        id: &str,
        tail: Option<u32>,
        ts: bool,
    ) -> Result<String, ContainerError> {
        self.inner.container_logs(id, tail, ts)
    }
    fn container_stats(&self, id: &str) -> Result<ContainerStats, ContainerError> {
        self.inner.container_stats(id)
    }
    fn exec_in_container(&self, id: &str, cmd: &[&str]) -> Result<String, ContainerError> {
        self.inner.exec_in_container(id, cmd)
    }
    fn list_images(&self) -> Result<Vec<Image>, ContainerError> {
        self.inner.list_images()
    }
    fn pull_image(&self, reference: &str) -> Result<(), ContainerError> {
        self.inner.pull_image(reference)
    }
    fn pull_image_streaming(
        &self,
        reference: &str,
        tx: std::sync::mpsc::SyncSender<PullProgress>,
    ) -> Result<(), ContainerError> {
        self.inner.pull_image_streaming(reference, tx)
    }
    fn cancel_pull(&self) {
        self.inner.cancel_pull()
    }
    fn remove_image(&self, id: &str, force: bool) -> Result<(), ContainerError> {
        self.inner.remove_image(id, force)
    }
    fn tag_image(&self, src: &str, target: &str) -> Result<(), ContainerError> {
        self.inner.tag_image(src, target)
    }
    fn inspect_image(&self, id: &str) -> Result<Image, ContainerError> {
        self.inner.inspect_image(id)
    }
    fn list_volumes(&self) -> Result<Vec<Volume>, ContainerError> {
        self.inner.list_volumes()
    }
    fn create_volume(&self, opts: &CreateVolumeOptions) -> Result<Volume, ContainerError> {
        self.inner.create_volume(opts)
    }
    fn remove_volume(&self, name: &str, force: bool) -> Result<(), ContainerError> {
        self.inner.remove_volume(name, force)
    }
    fn list_networks(&self) -> Result<Vec<Network>, ContainerError> {
        self.inner.list_networks()
    }
    fn create_network(&self, opts: &CreateNetworkOptions) -> Result<Network, ContainerError> {
        self.inner.create_network(opts)
    }
    fn remove_network(&self, id: &str) -> Result<(), ContainerError> {
        self.inner.remove_network(id)
    }
    fn ping(&self) -> Result<(), ContainerError> {
        self.inner.ping()
    }
    fn version(&self) -> Result<String, ContainerError> {
        // Override label so the UI shows "Podman …" instead of "Docker …"
        let raw = self.inner.version()?;
        Ok(raw.replace("Docker", "Podman"))
    }
    fn system_df(&self) -> Result<SystemUsage, ContainerError> {
        self.inner.system_df()
    }
    fn prune_system(&self, volumes: bool) -> Result<PruneReport, ContainerError> {
        self.inner.prune_system(volumes)
    }
    fn inspect_image_layers(&self, id: &str) -> Result<Vec<ImageLayer>, ContainerError> {
        self.inner.inspect_image_layers(id)
    }
    fn system_events(
        &self,
        since: Option<i64>,
        limit: Option<usize>,
    ) -> Result<Vec<ContainerEvent>, ContainerError> {
        self.inner.system_events(since, limit)
    }
}
