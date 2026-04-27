// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for container_logs (Feature B — Logs tab).
//!
//! Verifies that the driver returns logs for running and stopped containers,
//! respects the tail limit, and returns NotFound for unknown IDs.
mod support;

use gtk_cross_platform::infrastructure::containers::error::ContainerError;
use gtk_cross_platform::ports::i_container_driver::IContainerDriver;

use support::{
    RUNNING_CONTAINER_ID, STOPPED_CONTAINER_ID, UNKNOWN_CONTAINER_ID, mock_driver as driver,
};

/// Running container (RUNNING_CONTAINER_ID) should return non-empty logs.
#[test]
fn logs_running_container_returns_string() {
    let d = driver();
    let logs = d
        .container_logs(RUNNING_CONTAINER_ID, None, false)
        .expect("logs");
    assert!(
        !logs.is_empty(),
        "expected non-empty log output for running container"
    );
}

/// Stopped/exited container (STOPPED_CONTAINER_ID) also has logs.
#[test]
fn logs_stopped_container_returns_string() {
    let d = driver();
    let logs = d
        .container_logs(STOPPED_CONTAINER_ID, None, false)
        .expect("logs for exited container");
    assert!(
        !logs.is_empty(),
        "expected non-empty log output for exited container"
    );
}

/// Unknown ID should return NotFound.
#[test]
fn logs_unknown_id_returns_not_found() {
    let d = driver();
    let result = d.container_logs(UNKNOWN_CONTAINER_ID, None, false);
    assert!(
        matches!(result, Err(ContainerError::NotFound(_))),
        "expected NotFound for unknown container ID"
    );
}

/// Tail parameter limits the number of log lines returned.
#[test]
fn logs_tail_limits_lines() {
    let d = driver();
    let logs = d
        .container_logs(RUNNING_CONTAINER_ID, Some(5), false)
        .expect("tail logs");
    let line_count = logs.lines().count();
    assert!(
        line_count <= 5,
        "expected at most 5 lines with tail=5, got {line_count}"
    );
}
