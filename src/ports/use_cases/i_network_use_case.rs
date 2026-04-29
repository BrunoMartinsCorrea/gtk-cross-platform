// SPDX-License-Identifier: GPL-3.0-or-later
use crate::core::domain::network::{
    ContainerEvent, CreateNetworkOptions, HostStats, Network, PruneReport, SystemUsage,
};
use crate::infrastructure::containers::error::ContainerError;

pub trait INetworkUseCase: Send + Sync {
    fn list(&self) -> Result<Vec<Network>, ContainerError>;
    fn create(&self, opts: &CreateNetworkOptions) -> Result<Network, ContainerError>;
    fn remove(&self, id: &str) -> Result<(), ContainerError>;
    fn system_df(&self) -> Result<SystemUsage, ContainerError>;
    fn host_stats(&self) -> Result<HostStats, ContainerError>;
    fn prune(&self, volumes: bool) -> Result<PruneReport, ContainerError>;
    fn events(
        &self,
        since: Option<i64>,
        limit: Option<usize>,
    ) -> Result<Vec<ContainerEvent>, ContainerError>;
}
