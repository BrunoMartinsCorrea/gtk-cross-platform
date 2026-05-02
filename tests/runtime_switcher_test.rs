// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for the runtime switcher (Feature E).
//!
//! `detect_specific` must reject unrecognised runtime names. Real socket availability
//! is not tested here (CI has no Docker/Podman sockets), but `is_available()` on the
//! mock unavailable driver is verified.
use std::sync::Arc;

use gtk_cross_platform::infrastructure::containers::{
    error::ContainerError, factory::ContainerDriverFactory, mock_driver::MockContainerDriver,
};
use gtk_cross_platform::ports::i_container_driver::IContainerDriver;

/// Requesting an unknown runtime name must return RuntimeNotAvailable.
#[test]
fn detect_specific_unknown_returns_err() {
    let result = ContainerDriverFactory::detect_specific("fakeruntime");
    assert!(
        matches!(result, Err(ContainerError::RuntimeNotAvailable(_))),
        "expected RuntimeNotAvailable for unknown runtime"
    );
}

/// A driver constructed with `MockContainerDriver::unavailable()` reports is_available = false.
#[test]
fn is_available_false_when_socket_missing() {
    let driver: Arc<dyn IContainerDriver> = Arc::new(MockContainerDriver::unavailable());
    assert!(
        !driver.is_available(),
        "unavailable mock driver should return false from is_available()"
    );
}

/// A normal mock driver reports is_available = true.
#[test]
fn is_available_true_for_normal_mock() {
    let driver: Arc<dyn IContainerDriver> = Arc::new(MockContainerDriver::new());
    assert!(
        driver.is_available(),
        "normal mock driver should return true from is_available()"
    );
}
