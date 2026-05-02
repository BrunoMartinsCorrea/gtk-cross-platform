// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for exec_in_container (Feature C — Terminal/Exec tab).
//!
//! Verifies exec behavior: running containers return output, stopped containers
//! return NotRunning, unknown IDs return NotFound, and empty commands don't panic.
mod support;

use gtk_cross_platform::infrastructure::containers::error::ContainerError;
use gtk_cross_platform::ports::use_cases::i_container_use_case::IContainerUseCase;

use support::{RUNNING_CONTAINER_ID, STOPPED_CONTAINER_ID, UNKNOWN_CONTAINER_ID, container_uc};

#[test]
fn exec_running_container_returns_output() {
    let uc = container_uc();
    let output = uc
        .exec(RUNNING_CONTAINER_ID, &["ls", "-la"])
        .expect("exec on running container");
    assert!(!output.is_empty(), "expected non-empty output from exec");
}

#[test]
fn exec_stopped_container_returns_err() {
    let uc = container_uc();
    let result = uc.exec(STOPPED_CONTAINER_ID, &["ls"]);
    assert!(
        matches!(result, Err(ContainerError::NotRunning(_))),
        "expected NotRunning for exited container, got {result:?}"
    );
}

#[test]
fn exec_unknown_id_returns_not_found() {
    let uc = container_uc();
    let result = uc.exec(UNKNOWN_CONTAINER_ID, &["ls"]);
    assert!(
        matches!(result, Err(ContainerError::NotFound(_))),
        "expected NotFound for unknown container, got {result:?}"
    );
}

#[test]
fn exec_empty_command_handled_gracefully() {
    let uc = container_uc();
    let result = uc.exec(RUNNING_CONTAINER_ID, &[]);
    match result {
        Ok(s) => assert!(
            s.is_empty(),
            "empty command should yield empty output, got: {s:?}"
        ),
        Err(_) => {
            // A descriptive error is also acceptable
        }
    }
}
