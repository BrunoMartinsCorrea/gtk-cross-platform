// SPDX-License-Identifier: GPL-3.0-or-later
use std::sync::Arc;

use crate::core::domain::network::{
    ContainerEvent, CreateNetworkOptions, HostStats, Network, PruneReport, SystemUsage,
};
use crate::infrastructure::containers::error::ContainerError;
use crate::ports::i_container_driver::IContainerDriver;
use crate::ports::use_cases::i_network_use_case::INetworkUseCase;

pub struct NetworkUseCase {
    driver: Arc<dyn IContainerDriver>,
}

impl NetworkUseCase {
    pub fn new(driver: Arc<dyn IContainerDriver>) -> Self {
        Self { driver }
    }
}

impl INetworkUseCase for NetworkUseCase {
    fn list(&self) -> Result<Vec<Network>, ContainerError> {
        self.driver.list_networks()
    }

    fn create(&self, opts: &CreateNetworkOptions) -> Result<Network, ContainerError> {
        self.driver.create_network(opts)
    }

    fn remove(&self, id: &str) -> Result<(), ContainerError> {
        self.driver.remove_network(id)
    }

    fn system_df(&self) -> Result<SystemUsage, ContainerError> {
        self.driver.system_df()
    }

    fn host_stats(&self) -> Result<HostStats, ContainerError> {
        self.driver.host_stats()
    }

    fn prune(&self, volumes: bool) -> Result<PruneReport, ContainerError> {
        self.driver.prune_system(volumes)
    }

    fn events(
        &self,
        since: Option<i64>,
        limit: Option<usize>,
    ) -> Result<Vec<ContainerEvent>, ContainerError> {
        self.driver.system_events(since, limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::containers::mock_driver::MockContainerDriver;

    fn use_case() -> NetworkUseCase {
        NetworkUseCase::new(Arc::new(MockContainerDriver::new()))
    }

    #[test]
    fn list_networks_returns_two() {
        let uc = use_case();
        let nets = uc.list().expect("list networks");
        assert_eq!(nets.len(), 2);
        assert!(nets.iter().any(|n| n.name == "bridge"));
    }

    #[test]
    fn prune_system_returns_report() {
        let uc = use_case();
        let report = uc.prune(false).expect("prune");
        assert_eq!(report.space_reclaimed, 0);
    }

    #[test]
    fn create_network_returns_named_network() {
        let uc = use_case();
        let opts = CreateNetworkOptions {
            name: "my-net".to_string(),
            driver: "bridge".to_string(),
            subnet: None,
        };
        let net = uc.create(&opts).expect("create");
        assert_eq!(net.name, "my-net");
    }
}
