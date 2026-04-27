// SPDX-License-Identifier: GPL-3.0-or-later
use crate::core::domain::image::{Image, ImageLayer};
use crate::infrastructure::containers::error::ContainerError;

pub trait IImageUseCase: Send + Sync {
    fn list(&self) -> Result<Vec<Image>, ContainerError>;
    fn pull(&self, reference: &str) -> Result<(), ContainerError>;
    fn remove(&self, id: &str, force: bool) -> Result<(), ContainerError>;
    fn tag(&self, source: &str, target: &str) -> Result<(), ContainerError>;
    fn inspect(&self, id: &str) -> Result<Image, ContainerError>;
    fn layers(&self, id: &str) -> Result<Vec<ImageLayer>, ContainerError>;
}
