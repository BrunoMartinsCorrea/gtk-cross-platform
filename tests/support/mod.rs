// SPDX-License-Identifier: GPL-3.0-or-later
//! Shared test infrastructure: constants, factories, and assertion macros.
// Each integration test binary imports only the subset it needs;
// dead_code warnings here are false positives for shared fixtures.
#![allow(dead_code)]
use std::sync::Arc;

use gtk_cross_platform::core::use_cases::container_use_case::ContainerUseCase;
use gtk_cross_platform::core::use_cases::image_use_case::ImageUseCase;
use gtk_cross_platform::core::use_cases::network_use_case::NetworkUseCase;
use gtk_cross_platform::core::use_cases::volume_use_case::VolumeUseCase;
use gtk_cross_platform::infrastructure::containers::mock_driver::MockContainerDriver;

// ── Container ID constants ─────────────────────────────────────────────────────

/// Full ID of the "web-server" (nginx:latest) container — starts Running.
pub const RUNNING_CONTAINER_ID: &str = "aabbccdd1122334455667788";
/// Short ID of the "web-server" container.
pub const RUNNING_CONTAINER_SHORT_ID: &str = "aabbccdd1122";

/// Full ID of the "db" (postgres:15) container — starts Exited(0).
pub const STOPPED_CONTAINER_ID: &str = "112233445566778899aabbcc";
/// Short ID of the "db" container — also appears in mock prune report.
pub const STOPPED_CONTAINER_SHORT_ID: &str = "112233445566";

/// An ID that does not exist in the mock.
pub const UNKNOWN_CONTAINER_ID: &str = "nonexistentid0000000000";

// ── Mock state constants ───────────────────────────────────────────────────────

/// Total number of containers pre-loaded in the mock.
pub const MOCK_CONTAINERS_TOTAL: usize = 3;
/// Number of initially running containers in the mock.
pub const MOCK_CONTAINERS_RUNNING: usize = 1;
/// Total number of images pre-loaded in the mock (including dangling).
pub const MOCK_IMAGES_TOTAL: usize = 3;

/// Stats — network bytes received by a running container in the mock.
pub const MOCK_NET_RX_BYTES: u64 = 1024;
/// Stats — network bytes transmitted by a running container in the mock.
pub const MOCK_NET_TX_BYTES: u64 = 512;
/// Stats — memory usage in bytes for a running container in the mock (50 MiB).
pub const MOCK_MEMORY_USAGE_BYTES: u64 = 52_428_800;
/// Stats — memory limit in bytes for a running container in the mock (2 GiB).
pub const MOCK_MEMORY_LIMIT_BYTES: u64 = 2_147_483_648;

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

// ── Assertion macro ────────────────────────────────────────────────────────────

/// Assert that a `Result` is `Err` matching a given error variant pattern.
///
/// ```ignore
/// assert_error_variant!(result, ContainerError::NotFound(_));
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
