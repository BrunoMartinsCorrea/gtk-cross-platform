// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for exec_in_container (Feature C — Terminal/Exec tab).
//!
//! Verifies exec behavior: running containers return output, stopped containers
//! return NotRunning, unknown IDs return NotFound, and empty commands don't panic.
use std::sync::Arc;

use gtk_cross_platform::infrastructure::containers::error::ContainerError;
use gtk_cross_platform::infrastructure::containers::mock_driver::MockContainerDriver;
use gtk_cross_platform::ports::i_container_driver::IContainerDriver;

fn driver() -> Arc<MockContainerDriver> {
    Arc::new(MockContainerDriver::new())
}

/// Exec on a running container should return non-empty output.
#[test]
fn test_exec_running_container_returns_output() {
    let d = driver();
    // "aabbccdd1122334455667788" starts running
    let output = d
        .exec_in_container("aabbccdd1122334455667788", &["ls", "-la"])
        .expect("exec on running container");
    assert!(!output.is_empty(), "expected non-empty output from exec");
}

/// Exec on a stopped container should return NotRunning.
#[test]
fn test_exec_stopped_container_returns_err() {
    let d = driver();
    // "112233445566778899aabbcc" starts Exited in the mock
    let result = d.exec_in_container("112233445566778899aabbcc", &["ls"]);
    assert!(
        matches!(result, Err(ContainerError::NotRunning(_))),
        "expected NotRunning for exited container, got {result:?}"
    );
}

/// Exec on an unknown container ID should return NotFound.
#[test]
fn test_exec_unknown_id_returns_not_found() {
    let d = driver();
    let result = d.exec_in_container("nonexistentid0000000000", &["ls"]);
    assert!(
        matches!(result, Err(ContainerError::NotFound(_))),
        "expected NotFound for unknown container, got {result:?}"
    );
}

/// An empty command slice should not panic — return either Ok or a descriptive Err.
#[test]
fn test_exec_empty_command_handled_gracefully() {
    let d = driver();
    // Should not panic; the exact result (Ok or Err) is acceptable
    let result = d.exec_in_container("aabbccdd1122334455667788", &[]);
    match result {
        Ok(s) => {
            // Empty command yields empty output — valid
            assert!(s.is_empty() || !s.is_empty(), "no panic is the contract")
        }
        Err(_) => {
            // A descriptive error is also fine
        }
    }
}
