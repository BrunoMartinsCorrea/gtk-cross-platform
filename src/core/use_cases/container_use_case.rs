// SPDX-License-Identifier: GPL-3.0-or-later
use std::sync::Arc;

use crate::core::domain::container::{Container, ContainerStats, CreateContainerOptions};
use crate::infrastructure::containers::error::ContainerError;
use crate::ports::i_container_driver::IContainerDriver;
use crate::ports::use_cases::i_container_use_case::IContainerUseCase;

pub struct ContainerUseCase {
    driver: Arc<dyn IContainerDriver>,
}

impl ContainerUseCase {
    pub fn new(driver: Arc<dyn IContainerDriver>) -> Self {
        Self { driver }
    }
}

impl IContainerUseCase for ContainerUseCase {
    fn list(&self, all: bool) -> Result<Vec<Container>, ContainerError> {
        self.driver.list_containers(all)
    }

    fn inspect(&self, id: &str) -> Result<Container, ContainerError> {
        self.driver.inspect_container(id)
    }

    fn start(&self, id: &str) -> Result<(), ContainerError> {
        self.driver.start_container(id)
    }

    fn stop(&self, id: &str, timeout_secs: Option<u32>) -> Result<(), ContainerError> {
        self.driver.stop_container(id, timeout_secs)
    }

    fn restart(&self, id: &str, timeout_secs: Option<u32>) -> Result<(), ContainerError> {
        self.driver.restart_container(id, timeout_secs)
    }

    fn pause(&self, id: &str) -> Result<(), ContainerError> {
        self.driver.pause_container(id)
    }

    fn unpause(&self, id: &str) -> Result<(), ContainerError> {
        self.driver.unpause_container(id)
    }

    fn remove(&self, id: &str, force: bool, remove_volumes: bool) -> Result<(), ContainerError> {
        self.driver.remove_container(id, force, remove_volumes)
    }

    fn create(&self, opts: &CreateContainerOptions) -> Result<String, ContainerError> {
        self.driver.create_container(opts)
    }

    fn rename(&self, id: &str, new_name: &str) -> Result<(), ContainerError> {
        self.driver.rename_container(id, new_name)
    }

    fn logs(
        &self,
        id: &str,
        tail: Option<u32>,
        timestamps: bool,
    ) -> Result<String, ContainerError> {
        self.driver.container_logs(id, tail, timestamps)
    }

    fn stats(&self, id: &str) -> Result<ContainerStats, ContainerError> {
        self.driver.container_stats(id)
    }

    fn inspect_json(&self, id: &str) -> Result<String, ContainerError> {
        self.driver.inspect_container_json(id)
    }

    fn exec(&self, id: &str, cmd: &[&str]) -> Result<String, ContainerError> {
        self.driver.exec_in_container(id, cmd)
    }

    fn start_all(&self, ids: &[&str]) -> Result<Vec<Result<(), ContainerError>>, ContainerError> {
        Ok(ids
            .iter()
            .map(|id| self.driver.start_container(id))
            .collect())
    }

    fn stop_all(
        &self,
        ids: &[&str],
        timeout_secs: Option<u32>,
    ) -> Result<Vec<Result<(), ContainerError>>, ContainerError> {
        Ok(ids
            .iter()
            .map(|id| self.driver.stop_container(id, timeout_secs))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::containers::mock_driver::MockContainerDriver;

    fn use_case() -> ContainerUseCase {
        ContainerUseCase::new(Arc::new(MockContainerDriver::new()))
    }

    #[test]
    fn list_all_returns_all() {
        let uc = use_case();
        let containers = uc.list(true).expect("list all");
        assert!(!containers.is_empty());
    }

    #[test]
    fn list_running_only() {
        let uc = use_case();
        let running = uc.list(false).expect("list running");
        assert!(!running.is_empty());
        assert!(running.iter().all(|c| c.status.is_running()));
    }

    #[test]
    fn start_stopped_container_makes_it_running() {
        let uc = use_case();
        uc.stop("aabbccdd1122", None).expect("stop");
        uc.start("aabbccdd1122").expect("start");
        let running = uc.list(false).expect("list");
        assert_eq!(running.len(), 1);
    }

    #[test]
    fn remove_container_succeeds() {
        let uc = use_case();
        assert!(uc.remove("aabbccdd1122", true, false).is_ok());
    }
}
