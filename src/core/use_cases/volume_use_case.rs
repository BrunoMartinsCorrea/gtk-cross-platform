// SPDX-License-Identifier: GPL-3.0-or-later
use std::sync::Arc;

use crate::core::domain::volume::{CreateVolumeOptions, Volume};
use crate::infrastructure::containers::error::ContainerError;
use crate::ports::i_container_driver::IContainerDriver;
use crate::ports::use_cases::i_volume_use_case::IVolumeUseCase;

pub struct VolumeUseCase {
    driver: Arc<dyn IContainerDriver>,
}

impl VolumeUseCase {
    pub fn new(driver: Arc<dyn IContainerDriver>) -> Self {
        Self { driver }
    }
}

impl IVolumeUseCase for VolumeUseCase {
    fn list(&self) -> Result<Vec<Volume>, ContainerError> {
        self.driver.list_volumes()
    }

    fn create(&self, opts: &CreateVolumeOptions) -> Result<Volume, ContainerError> {
        self.driver.create_volume(opts)
    }

    fn remove(&self, name: &str, force: bool) -> Result<(), ContainerError> {
        self.driver.remove_volume(name, force)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::containers::mock_driver::MockContainerDriver;

    fn use_case() -> VolumeUseCase {
        VolumeUseCase::new(Arc::new(MockContainerDriver::new()))
    }

    #[test]
    fn list_returns_volumes() {
        let uc = use_case();
        let volumes = uc.list().expect("list volumes");
        assert!(!volumes.is_empty());
        assert!(volumes.iter().any(|v| v.name == "postgres-data"));
    }

    #[test]
    fn create_volume_returns_named_volume() {
        let uc = use_case();
        let opts = CreateVolumeOptions {
            name: "new-vol".to_string(),
            driver: "local".to_string(),
            labels: Default::default(),
        };
        let vol = uc.create(&opts).expect("create");
        assert_eq!(vol.name, "new-vol");
    }

    #[test]
    fn remove_volume_succeeds() {
        let uc = use_case();
        assert!(uc.remove("postgres-data", false).is_ok());
    }
}
