// SPDX-License-Identifier: GPL-3.0-or-later
use crate::core::domain::container::{Container, ContainerStats, CreateContainerOptions};
use crate::infrastructure::containers::error::ContainerError;

pub trait IContainerUseCase: Send + Sync {
    fn list(&self, all: bool) -> Result<Vec<Container>, ContainerError>;
    fn inspect(&self, id: &str) -> Result<Container, ContainerError>;
    fn start(&self, id: &str) -> Result<(), ContainerError>;
    fn stop(&self, id: &str, timeout_secs: Option<u32>) -> Result<(), ContainerError>;
    fn restart(&self, id: &str, timeout_secs: Option<u32>) -> Result<(), ContainerError>;
    fn pause(&self, id: &str) -> Result<(), ContainerError>;
    fn unpause(&self, id: &str) -> Result<(), ContainerError>;
    fn remove(&self, id: &str, force: bool, remove_volumes: bool) -> Result<(), ContainerError>;
    fn create(&self, opts: &CreateContainerOptions) -> Result<String, ContainerError>;
    fn rename(&self, id: &str, new_name: &str) -> Result<(), ContainerError>;
    fn logs(&self, id: &str, tail: Option<u32>, timestamps: bool)
    -> Result<String, ContainerError>;
    fn stats(&self, id: &str) -> Result<ContainerStats, ContainerError>;
    fn inspect_json(&self, id: &str) -> Result<String, ContainerError>;
    fn exec(&self, id: &str, cmd: &[&str]) -> Result<String, ContainerError>;
    fn start_all(&self, ids: &[&str]) -> Result<Vec<Result<(), ContainerError>>, ContainerError>;
    fn stop_all(
        &self,
        ids: &[&str],
        timeout_secs: Option<u32>,
    ) -> Result<Vec<Result<(), ContainerError>>, ContainerError>;
}
