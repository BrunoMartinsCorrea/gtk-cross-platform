// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for container_logs (Feature B — Logs tab).
//!
//! Verifies that the driver returns logs for running and stopped containers,
//! respects the tail limit, and returns NotFound for unknown IDs.
use std::sync::Arc;

use gtk_cross_platform::infrastructure::containers::error::ContainerError;
use gtk_cross_platform::infrastructure::containers::mock_driver::MockContainerDriver;
use gtk_cross_platform::ports::i_container_driver::IContainerDriver;

fn driver() -> Arc<MockContainerDriver> {
    Arc::new(MockContainerDriver::new())
}

/// Running container ("aabbccdd1122334455667788") should return non-empty logs.
#[test]
fn test_logs_running_container_returns_string() {
    let d = driver();
    let logs = d
        .container_logs("aabbccdd1122334455667788", None, false)
        .expect("logs");
    assert!(
        !logs.is_empty(),
        "expected non-empty log output for running container"
    );
}

/// Stopped/exited container ("112233445566778899aabbcc") also has logs.
#[test]
fn test_logs_stopped_container_returns_string() {
    let d = driver();
    let logs = d
        .container_logs("112233445566778899aabbcc", None, false)
        .expect("logs for exited container");
    assert!(
        !logs.is_empty(),
        "expected non-empty log output for exited container"
    );
}

/// Unknown ID should return NotFound.
#[test]
fn test_logs_unknown_id_returns_not_found() {
    let d = driver();
    let result = d.container_logs("nonexistentid0000000000", None, false);
    assert!(
        matches!(result, Err(ContainerError::NotFound(_))),
        "expected NotFound for unknown container ID"
    );
}

/// Tail parameter limits the number of log lines returned.
#[test]
fn test_logs_tail_limits_lines() {
    let d = driver();
    let logs = d
        .container_logs("aabbccdd1122334455667788", Some(5), false)
        .expect("tail logs");
    let line_count = logs.lines().count();
    assert!(
        line_count <= 5,
        "expected at most 5 lines with tail=5, got {line_count}"
    );
}
