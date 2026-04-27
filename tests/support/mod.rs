// SPDX-License-Identifier: GPL-3.0-or-later
//! Shared test infrastructure: constants, factories, builders, and assertion macros.
#![allow(dead_code)]
use std::collections::HashMap;
use std::sync::Arc;

use gtk_cross_platform::core::domain::container::{Container, ContainerPort, ContainerStatus};
use gtk_cross_platform::core::use_cases::container_use_case::ContainerUseCase;
use gtk_cross_platform::core::use_cases::image_use_case::ImageUseCase;
use gtk_cross_platform::core::use_cases::network_use_case::NetworkUseCase;
use gtk_cross_platform::core::use_cases::volume_use_case::VolumeUseCase;
use gtk_cross_platform::infrastructure::containers::mock_driver::MockContainerDriver;

// ── Object Mother constants ────────────────────────────────────────────────────

/// Full ID of the "web-server" (nginx:latest) container — starts Running.
pub const RUNNING_CONTAINER_ID: &str = "aabbccdd1122334455667788";
/// Short ID of the "web-server" container.
pub const RUNNING_CONTAINER_SHORT_ID: &str = "aabbccdd1122";

/// Full ID of the "db" (postgres:15) container — starts Exited(0).
pub const STOPPED_CONTAINER_ID: &str = "112233445566778899aabbcc";
/// Short ID of the "db" container.
pub const STOPPED_CONTAINER_SHORT_ID: &str = "112233445566";

/// Full ID of the "standalone" (redis:alpine) container — starts Stopped.
pub const STANDALONE_CONTAINER_ID: &str = "223344556677889900aabbcc";

/// An ID that does not exist in the mock.
pub const UNKNOWN_CONTAINER_ID: &str = "nonexistentid0000000000";

/// Total number of containers pre-loaded in the mock.
pub const MOCK_CONTAINERS_TOTAL: usize = 3;
/// Number of initially running containers in the mock.
pub const MOCK_CONTAINERS_RUNNING: usize = 1;

// ── Factories ──────────────────────────────────────────────────────────────────

pub fn mock_driver() -> Arc<MockContainerDriver> {
    Arc::new(MockContainerDriver::new())
}

pub fn container_uc() -> ContainerUseCase {
    ContainerUseCase::new(mock_driver())
}

pub fn image_uc() -> ImageUseCase {
    ImageUseCase::new(mock_driver())
}

pub fn volume_uc() -> VolumeUseCase {
    VolumeUseCase::new(mock_driver())
}

pub fn network_uc() -> NetworkUseCase {
    NetworkUseCase::new(mock_driver())
}

// ── ContainerBuilder ───────────────────────────────────────────────────────────

/// Fluent builder for `Container` in tests.
#[derive(Default)]
pub struct ContainerBuilder {
    id: Option<String>,
    short_id: Option<String>,
    name: Option<String>,
    image: Option<String>,
    status: Option<ContainerStatus>,
    compose_project: Option<String>,
    env: Vec<String>,
    ports: Vec<ContainerPort>,
}

impl ContainerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        let s: String = id.into();
        self.short_id = Some(s.chars().take(12).collect());
        self.id = Some(s);
        self
    }

    pub fn short_id(mut self, short_id: impl Into<String>) -> Self {
        let s: String = short_id.into();
        self.id.get_or_insert_with(|| format!("{s}aabbccdd1122"));
        self.short_id = Some(s);
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn image(mut self, image: impl Into<String>) -> Self {
        self.image = Some(image.into());
        self
    }

    pub fn status(mut self, status: ContainerStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn compose_project(mut self, project: impl Into<String>) -> Self {
        self.compose_project = Some(project.into());
        self
    }

    pub fn env(mut self, env_vars: Vec<String>) -> Self {
        self.env = env_vars;
        self
    }

    pub fn build(self) -> Container {
        let short_id = self
            .short_id
            .unwrap_or_else(|| "aabbccdd1122".to_string());
        let id = self
            .id
            .unwrap_or_else(|| format!("{short_id}aabbccdd1122"));
        let mut labels = HashMap::new();
        if let Some(ref project) = self.compose_project {
            labels.insert(
                "com.docker.compose.project".to_string(),
                project.clone(),
            );
        }
        let status = self.status.unwrap_or(ContainerStatus::Running);
        Container {
            id,
            short_id,
            name: self.name.unwrap_or_else(|| "test-container".to_string()),
            image: self.image.unwrap_or_else(|| "alpine:latest".to_string()),
            command: "/start.sh".to_string(),
            created: 0,
            status_text: status.label().to_string(),
            status,
            ports: self.ports,
            labels,
            mounts: vec![],
            env: self.env,
            compose_project: self.compose_project,
        }
    }
}

// ── Assertion macro ────────────────────────────────────────────────────────────

/// Assert that a `Result` is `Err` matching a given error variant pattern.
///
/// ```ignore
/// assert_error_variant!(result, ContainerError::NotFound(_));
/// assert_error_variant!(result, ContainerError::NotRunning(_));
/// ```
#[macro_export]
macro_rules! assert_error_variant {
    ($result:expr, $variant:pat) => {
        assert!(
            matches!($result, Err($variant)),
            "expected {}, got {:?}",
            stringify!($variant),
            $result
        )
    };
}
