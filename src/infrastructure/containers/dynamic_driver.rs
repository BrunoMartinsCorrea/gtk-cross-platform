// SPDX-License-Identifier: GPL-3.0-or-later
use std::sync::{Arc, RwLock};

use crate::core::domain::{
    container::{Container, ContainerStats, CreateContainerOptions, PullProgress},
    image::{Image, ImageLayer},
    network::{ContainerEvent, CreateNetworkOptions, Network, PruneReport, SystemUsage},
    volume::{CreateVolumeOptions, Volume},
};
use crate::infrastructure::containers::error::ContainerError;
use crate::ports::i_container_driver::IContainerDriver;

/// Hot-swappable driver wrapper.
///
/// All use cases hold `Arc<DynamicDriver>` upcast to `Arc<dyn IContainerDriver>`.
/// Calling `swap()` atomically replaces the inner driver so every use case
/// transparently picks up the new runtime without re-wiring.
pub struct DynamicDriver {
    inner: RwLock<Arc<dyn IContainerDriver>>,
}

impl DynamicDriver {
    pub fn new(driver: Arc<dyn IContainerDriver>) -> Self {
        Self {
            inner: RwLock::new(driver),
        }
    }

    /// Replace the inner driver atomically. All subsequent use-case calls will
    /// use the new driver.
    pub fn swap(&self, new_driver: Arc<dyn IContainerDriver>) {
        *self.inner.write().expect("DynamicDriver write lock") = new_driver;
    }

    fn driver(&self) -> Arc<dyn IContainerDriver> {
        self.inner.read().expect("DynamicDriver read lock").clone()
    }
}

impl IContainerDriver for DynamicDriver {
    // runtime_name() returns a &str tied to &self. We cannot borrow from a
    // RwLockReadGuard that is dropped at end of the call. Return a fixed sentinel;
    // callers that need the real name should use version() or track it externally.
    fn runtime_name(&self) -> &str {
        "dynamic"
    }

    fn is_available(&self) -> bool {
        self.driver().is_available()
    }

    fn list_containers(&self, all: bool) -> Result<Vec<Container>, ContainerError> {
        self.driver().list_containers(all)
    }

    fn inspect_container(&self, id: &str) -> Result<Container, ContainerError> {
        self.driver().inspect_container(id)
    }

    fn start_container(&self, id: &str) -> Result<(), ContainerError> {
        self.driver().start_container(id)
    }

    fn stop_container(&self, id: &str, timeout_secs: Option<u32>) -> Result<(), ContainerError> {
        self.driver().stop_container(id, timeout_secs)
    }

    fn restart_container(&self, id: &str, timeout_secs: Option<u32>) -> Result<(), ContainerError> {
        self.driver().restart_container(id, timeout_secs)
    }

    fn pause_container(&self, id: &str) -> Result<(), ContainerError> {
        self.driver().pause_container(id)
    }

    fn unpause_container(&self, id: &str) -> Result<(), ContainerError> {
        self.driver().unpause_container(id)
    }

    fn remove_container(
        &self,
        id: &str,
        force: bool,
        remove_volumes: bool,
    ) -> Result<(), ContainerError> {
        self.driver().remove_container(id, force, remove_volumes)
    }

    fn create_container(&self, opts: &CreateContainerOptions) -> Result<String, ContainerError> {
        self.driver().create_container(opts)
    }

    fn rename_container(&self, id: &str, new_name: &str) -> Result<(), ContainerError> {
        self.driver().rename_container(id, new_name)
    }

    fn container_logs(
        &self,
        id: &str,
        tail: Option<u32>,
        timestamps: bool,
    ) -> Result<String, ContainerError> {
        self.driver().container_logs(id, tail, timestamps)
    }

    fn container_stats(&self, id: &str) -> Result<ContainerStats, ContainerError> {
        self.driver().container_stats(id)
    }

    fn inspect_container_json(&self, id: &str) -> Result<String, ContainerError> {
        self.driver().inspect_container_json(id)
    }

    fn exec_in_container(&self, id: &str, cmd: &[&str]) -> Result<String, ContainerError> {
        self.driver().exec_in_container(id, cmd)
    }

    fn list_images(&self) -> Result<Vec<Image>, ContainerError> {
        self.driver().list_images()
    }

    fn pull_image(&self, reference: &str) -> Result<(), ContainerError> {
        self.driver().pull_image(reference)
    }

    fn pull_image_streaming(
        &self,
        reference: &str,
        tx: std::sync::mpsc::SyncSender<PullProgress>,
    ) -> Result<(), ContainerError> {
        self.driver().pull_image_streaming(reference, tx)
    }

    fn cancel_pull(&self) {
        self.driver().cancel_pull();
    }

    fn remove_image(&self, id: &str, force: bool) -> Result<(), ContainerError> {
        self.driver().remove_image(id, force)
    }

    fn tag_image(&self, source: &str, target: &str) -> Result<(), ContainerError> {
        self.driver().tag_image(source, target)
    }

    fn inspect_image(&self, id: &str) -> Result<Image, ContainerError> {
        self.driver().inspect_image(id)
    }

    fn inspect_image_layers(&self, id: &str) -> Result<Vec<ImageLayer>, ContainerError> {
        self.driver().inspect_image_layers(id)
    }

    fn list_volumes(&self) -> Result<Vec<Volume>, ContainerError> {
        self.driver().list_volumes()
    }

    fn create_volume(&self, opts: &CreateVolumeOptions) -> Result<Volume, ContainerError> {
        self.driver().create_volume(opts)
    }

    fn remove_volume(&self, name: &str, force: bool) -> Result<(), ContainerError> {
        self.driver().remove_volume(name, force)
    }

    fn list_networks(&self) -> Result<Vec<Network>, ContainerError> {
        self.driver().list_networks()
    }

    fn create_network(&self, opts: &CreateNetworkOptions) -> Result<Network, ContainerError> {
        self.driver().create_network(opts)
    }

    fn remove_network(&self, id: &str) -> Result<(), ContainerError> {
        self.driver().remove_network(id)
    }

    fn ping(&self) -> Result<(), ContainerError> {
        self.driver().ping()
    }

    fn version(&self) -> Result<String, ContainerError> {
        self.driver().version()
    }

    fn system_df(&self) -> Result<SystemUsage, ContainerError> {
        self.driver().system_df()
    }

    fn prune_system(&self, volumes: bool) -> Result<PruneReport, ContainerError> {
        self.driver().prune_system(volumes)
    }

    fn system_events(
        &self,
        since: Option<i64>,
        limit: Option<usize>,
    ) -> Result<Vec<ContainerEvent>, ContainerError> {
        self.driver().system_events(since, limit)
    }
}
