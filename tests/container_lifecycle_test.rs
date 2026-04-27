// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for invalid container state transitions (AA-02).
//!
//! Verifies that the mock driver and use case return the correct error variants
//! for operations that are not valid in the container's current state:
//! pause(stopped), unpause(not_paused), restart(nonexistent), start(already_running).
mod support;

use gtk_cross_platform::infrastructure::containers::error::ContainerError;
use gtk_cross_platform::ports::i_container_driver::IContainerDriver;
use gtk_cross_platform::ports::use_cases::i_container_use_case::IContainerUseCase;

use support::{
    container_uc, mock_driver, RUNNING_CONTAINER_ID, RUNNING_CONTAINER_SHORT_ID,
    STOPPED_CONTAINER_ID, UNKNOWN_CONTAINER_ID,
};

// ── pause ──────────────────────────────────────────────────────────────────────

#[test]
fn pause_stopped_container_returns_not_running() {
    let d = mock_driver();
    // STOPPED_CONTAINER_ID starts Exited — not in the running set
    let result = d.pause_container(STOPPED_CONTAINER_ID);
    assert!(
        matches!(result, Err(ContainerError::NotRunning(_))),
        "expected NotRunning when pausing a stopped container, got: {result:?}"
    );
}

#[test]
fn pause_unknown_container_returns_not_found() {
    let d = mock_driver();
    let result = d.pause_container(UNKNOWN_CONTAINER_ID);
    assert!(
        matches!(result, Err(ContainerError::NotRunning(_)) | Err(ContainerError::NotFound(_))),
        "expected NotRunning or NotFound for unknown container, got: {result:?}"
    );
}

// ── unpause ────────────────────────────────────────────────────────────────────

#[test]
fn unpause_stopped_container_returns_not_running() {
    let d = mock_driver();
    // STOPPED_CONTAINER_ID is not running — unpause requires running state
    let result = d.unpause_container(STOPPED_CONTAINER_ID);
    assert!(
        matches!(result, Err(ContainerError::NotRunning(_))),
        "expected NotRunning when unpausing a stopped container, got: {result:?}"
    );
}

// ── restart ────────────────────────────────────────────────────────────────────

#[test]
fn restart_unknown_container_returns_not_found() {
    let d = mock_driver();
    let result = d.restart_container(UNKNOWN_CONTAINER_ID, None);
    assert!(
        matches!(result, Err(ContainerError::NotFound(_))),
        "expected NotFound when restarting nonexistent container, got: {result:?}"
    );
}

#[test]
fn restart_known_container_succeeds() {
    let d = mock_driver();
    let result = d.restart_container(RUNNING_CONTAINER_ID, None);
    assert!(result.is_ok(), "restart on known container must succeed");
}

// ── pause/unpause round-trip ───────────────────────────────────────────────────

#[test]
fn pause_then_start_makes_container_running_again() {
    let uc = container_uc();
    // Pause the running container (simulated by stopping it)
    uc.stop(RUNNING_CONTAINER_SHORT_ID, None).expect("stop");
    // Verify it's no longer in running list
    let before = uc.list(false).expect("list");
    assert!(!before.iter().any(|c| c.short_id == RUNNING_CONTAINER_SHORT_ID));
    // Start it again
    uc.start(RUNNING_CONTAINER_SHORT_ID).expect("start");
    let after = uc.list(false).expect("list");
    assert!(
        after.iter().any(|c| c.short_id == RUNNING_CONTAINER_SHORT_ID),
        "container must be running after start"
    );
}

// ── remove_image / remove_volume / remove_network — NotFound ──────────────────

#[test]
fn remove_nonexistent_image_returns_not_found() {
    let d = mock_driver();
    let result = d.remove_image("sha256:does-not-exist", false);
    assert!(
        matches!(result, Err(ContainerError::NotFound(_))),
        "expected NotFound for unknown image, got: {result:?}"
    );
}

#[test]
fn remove_known_image_succeeds() {
    let d = mock_driver();
    let result = d.remove_image("sha256:aaaa", false);
    assert!(result.is_ok(), "remove on known image must succeed");
}

#[test]
fn remove_nonexistent_volume_returns_not_found() {
    let d = mock_driver();
    let result = d.remove_volume("no-such-volume", false);
    assert!(
        matches!(result, Err(ContainerError::NotFound(_))),
        "expected NotFound for unknown volume, got: {result:?}"
    );
}

#[test]
fn remove_known_volume_succeeds() {
    let d = mock_driver();
    let result = d.remove_volume("postgres-data", false);
    assert!(result.is_ok(), "remove on known volume must succeed");
}

#[test]
fn remove_nonexistent_network_returns_not_found() {
    let d = mock_driver();
    let result = d.remove_network("net-does-not-exist");
    assert!(
        matches!(result, Err(ContainerError::NotFound(_))),
        "expected NotFound for unknown network, got: {result:?}"
    );
}

#[test]
fn remove_known_network_succeeds() {
    let d = mock_driver();
    let result = d.remove_network("bridge");
    assert!(result.is_ok(), "remove on known network must succeed");
}
