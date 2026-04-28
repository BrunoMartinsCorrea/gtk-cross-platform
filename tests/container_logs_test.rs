// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for container_logs (Feature B — Logs tab).
//!
//! Verifies that the logs use case returns log content for running and stopped
//! containers, respects the tail limit, and returns NotFound for unknown IDs.
mod support;

use gtk_cross_platform::infrastructure::containers::error::ContainerError;
use gtk_cross_platform::ports::use_cases::i_container_use_case::IContainerUseCase;

use support::{
    RUNNING_CONTAINER_ID, STOPPED_CONTAINER_ID, UNKNOWN_CONTAINER_ID, container_uc,
};

#[test]
fn logs_running_container_returns_string() {
    let uc = container_uc();
    let logs = uc
        .logs(RUNNING_CONTAINER_ID, None, false)
        .expect("logs");
    assert!(
        !logs.is_empty(),
        "expected non-empty log output for running container"
    );
}

#[test]
fn logs_stopped_container_returns_string() {
    let uc = container_uc();
    let logs = uc
        .logs(STOPPED_CONTAINER_ID, None, false)
        .expect("logs for exited container");
    assert!(
        !logs.is_empty(),
        "expected non-empty log output for exited container"
    );
}

#[test]
fn logs_unknown_id_returns_not_found() {
    let uc = container_uc();
    let result = uc.logs(UNKNOWN_CONTAINER_ID, None, false);
    assert!(
        matches!(result, Err(ContainerError::NotFound(_))),
        "expected NotFound for unknown container ID, got: {result:?}"
    );
}

#[test]
fn logs_tail_limits_lines() {
    let uc = container_uc();
    let logs = uc
        .logs(RUNNING_CONTAINER_ID, Some(5), false)
        .expect("tail logs");
    let line_count = logs.lines().count();
    assert!(
        line_count <= 5,
        "expected at most 5 lines with tail=5, got {line_count}"
    );
}
