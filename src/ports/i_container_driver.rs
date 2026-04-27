// SPDX-License-Identifier: GPL-3.0-or-later
use crate::core::domain::{
    container::{Container, ContainerStats, CreateContainerOptions, PullProgress},
    image::{Image, ImageLayer},
    network::{ContainerEvent, CreateNetworkOptions, Network, PruneReport, SystemUsage},
    volume::{CreateVolumeOptions, Volume},
};
use crate::infrastructure::containers::error::ContainerError;

pub trait IContainerDriver: Send + Sync {
    fn runtime_name(&self) -> &str;

    fn is_available(&self) -> bool;

    fn list_containers(&self, all: bool) -> Result<Vec<Container>, ContainerError>;

    fn inspect_container(&self, id: &str) -> Result<Container, ContainerError>;

    fn start_container(&self, id: &str) -> Result<(), ContainerError>;

    fn stop_container(&self, id: &str, timeout_secs: Option<u32>) -> Result<(), ContainerError>;

    fn restart_container(&self, id: &str, timeout_secs: Option<u32>) -> Result<(), ContainerError>;

    fn pause_container(&self, id: &str) -> Result<(), ContainerError>;

    fn unpause_container(&self, id: &str) -> Result<(), ContainerError>;

    fn remove_container(
        &self,
        id: &str,
        force: bool,
        remove_volumes: bool,
    ) -> Result<(), ContainerError>;

    fn create_container(&self, opts: &CreateContainerOptions) -> Result<String, ContainerError>;

    fn rename_container(&self, id: &str, new_name: &str) -> Result<(), ContainerError>;

    fn container_logs(
        &self,
        id: &str,
        tail: Option<u32>,
        timestamps: bool,
    ) -> Result<String, ContainerError>;

    fn container_stats(&self, id: &str) -> Result<ContainerStats, ContainerError>;

    fn inspect_container_json(&self, id: &str) -> Result<String, ContainerError>;

    fn exec_in_container(&self, id: &str, cmd: &[&str]) -> Result<String, ContainerError>;

    fn list_images(&self) -> Result<Vec<Image>, ContainerError>;

    fn pull_image(&self, reference: &str) -> Result<(), ContainerError>;

    /// Stream per-layer pull progress. Sends events via `tx` as each layer progresses.
    /// Returns `Ok(())` after the final layer reaches `Done` or when cancelled.
    /// Must be called through `spawn_driver_task` — never on the GTK main thread.
    fn pull_image_streaming(
        &self,
        reference: &str,
        tx: std::sync::mpsc::SyncSender<PullProgress>,
    ) -> Result<(), ContainerError>;

    /// Cancel an in-flight `pull_image_streaming` call. Best-effort; may be a no-op.
    fn cancel_pull(&self);

    fn remove_image(&self, id: &str, force: bool) -> Result<(), ContainerError>;

    fn tag_image(&self, source: &str, target: &str) -> Result<(), ContainerError>;

    fn inspect_image(&self, id: &str) -> Result<Image, ContainerError>;

    fn inspect_image_layers(&self, id: &str) -> Result<Vec<ImageLayer>, ContainerError>;

    fn list_volumes(&self) -> Result<Vec<Volume>, ContainerError>;

    fn create_volume(&self, opts: &CreateVolumeOptions) -> Result<Volume, ContainerError>;

    fn remove_volume(&self, name: &str, force: bool) -> Result<(), ContainerError>;

    fn list_networks(&self) -> Result<Vec<Network>, ContainerError>;

    fn create_network(&self, opts: &CreateNetworkOptions) -> Result<Network, ContainerError>;

    fn remove_network(&self, id: &str) -> Result<(), ContainerError>;

    fn ping(&self) -> Result<(), ContainerError>;

    fn version(&self) -> Result<String, ContainerError>;

    fn system_df(&self) -> Result<SystemUsage, ContainerError>;

    fn prune_system(&self, volumes: bool) -> Result<PruneReport, ContainerError>;

    fn system_events(
        &self,
        since: Option<i64>,
        limit: Option<usize>,
    ) -> Result<Vec<ContainerEvent>, ContainerError>;
}
