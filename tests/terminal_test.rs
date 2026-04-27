// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for exec_in_container (Feature C — Terminal/Exec tab).
//!
//! Verifies exec behavior: running containers return output, stopped containers
//! return NotRunning, unknown IDs return NotFound, and empty commands don't panic.
mod support;

use gtk_cross_platform::infrastructure::containers::error::ContainerError;
use gtk_cross_platform::ports::i_container_driver::IContainerDriver;

use support::{
    RUNNING_CONTAINER_ID, STOPPED_CONTAINER_ID, UNKNOWN_CONTAINER_ID, mock_driver as driver,
};

/// Exec on a running container should return non-empty output.
#[test]
fn exec_running_container_returns_output() {
    let d = driver();
    // RUNNING_CONTAINER_ID starts running
    let output = d
        .exec_in_container(RUNNING_CONTAINER_ID, &["ls", "-la"])
        .expect("exec on running container");
    assert!(!output.is_empty(), "expected non-empty output from exec");
}

/// Exec on a stopped container should return NotRunning.
#[test]
fn exec_stopped_container_returns_err() {
    let d = driver();
    // STOPPED_CONTAINER_ID starts Exited in the mock
    let result = d.exec_in_container(STOPPED_CONTAINER_ID, &["ls"]);
    assert!(
        matches!(result, Err(ContainerError::NotRunning(_))),
        "expected NotRunning for exited container, got {result:?}"
    );
}

/// Exec on an unknown container ID should return NotFound.
#[test]
fn exec_unknown_id_returns_not_found() {
    let d = driver();
    let result = d.exec_in_container(UNKNOWN_CONTAINER_ID, &["ls"]);
    assert!(
        matches!(result, Err(ContainerError::NotFound(_))),
        "expected NotFound for unknown container, got {result:?}"
    );
}

/// An empty command slice should not panic — return either Ok or a descriptive Err.
#[test]
fn exec_empty_command_handled_gracefully() {
    let d = driver();
    // Should not panic; the exact result (Ok or Err) is acceptable
    let result = d.exec_in_container(RUNNING_CONTAINER_ID, &[]);
    match result {
        Ok(s) => {
            assert!(
                s.is_empty(),
                "empty command should yield empty output, got: {s:?}"
            )
        }
        Err(_) => {
            // A descriptive error is also fine
        }
    }
}
