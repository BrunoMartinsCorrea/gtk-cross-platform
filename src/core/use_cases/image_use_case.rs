// SPDX-License-Identifier: GPL-3.0-or-later
use std::sync::Arc;

use crate::core::domain::image::{Image, ImageLayer};
use crate::infrastructure::containers::error::ContainerError;
use crate::ports::i_container_driver::IContainerDriver;
use crate::ports::use_cases::i_image_use_case::IImageUseCase;

pub struct ImageUseCase {
    driver: Arc<dyn IContainerDriver>,
}

impl ImageUseCase {
    pub fn new(driver: Arc<dyn IContainerDriver>) -> Self {
        Self { driver }
    }
}

impl IImageUseCase for ImageUseCase {
    fn list(&self) -> Result<Vec<Image>, ContainerError> {
        self.driver.list_images()
    }

    fn pull(&self, reference: &str) -> Result<(), ContainerError> {
        self.driver.pull_image(reference)
    }

    fn remove(&self, id: &str, force: bool) -> Result<(), ContainerError> {
        self.driver.remove_image(id, force)
    }

    fn tag(&self, source: &str, target: &str) -> Result<(), ContainerError> {
        self.driver.tag_image(source, target)
    }

    fn inspect(&self, id: &str) -> Result<Image, ContainerError> {
        self.driver.inspect_image(id)
    }

    fn layers(&self, id: &str) -> Result<Vec<ImageLayer>, ContainerError> {
        self.driver.inspect_image_layers(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::containers::mock_driver::MockContainerDriver;

    fn use_case() -> ImageUseCase {
        ImageUseCase::new(Arc::new(MockContainerDriver::new()))
    }

    #[test]
    fn list_returns_images() {
        let uc = use_case();
        let images = uc.list().expect("list images");
        assert!(images.len() >= 2);
        assert!(images.iter().any(|i| i.primary_tag() == "nginx:latest"));
        assert!(images.iter().any(|i| i.primary_tag() == "postgres:15"));
    }

    #[test]
    fn remove_image_succeeds() {
        let uc = use_case();
        let images = uc.list().expect("list");
        let first_id = images[0].id.clone();
        assert!(uc.remove(&first_id, false).is_ok());
    }
}
