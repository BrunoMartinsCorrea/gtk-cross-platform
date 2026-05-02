// SPDX-License-Identifier: GPL-3.0-or-later
use crate::core::domain::volume::{CreateVolumeOptions, Volume};
use crate::infrastructure::containers::error::ContainerError;

pub trait IVolumeUseCase: Send + Sync {
    fn list(&self) -> Result<Vec<Volume>, ContainerError>;
    fn create(&self, opts: &CreateVolumeOptions) -> Result<Volume, ContainerError>;
    fn remove(&self, name: &str, force: bool) -> Result<(), ContainerError>;
}
