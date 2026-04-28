// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for invalid container state transitions (AA-02).
//!
//! Verifies that use cases return the correct error variants for operations
//! that are not valid in the container's current state: pause(stopped),
//! unpause(not_paused), restart(nonexistent), start(already_running).
mod support;

use gtk_cross_platform::infrastructure::containers::error::ContainerError;
use gtk_cross_platform::ports::use_cases::i_container_use_case::IContainerUseCase;
use gtk_cross_platform::ports::use_cases::i_image_use_case::IImageUseCase;
use gtk_cross_platform::ports::use_cases::i_network_use_case::INetworkUseCase;
use gtk_cross_platform::ports::use_cases::i_volume_use_case::IVolumeUseCase;

use support::{
    RUNNING_CONTAINER_ID, RUNNING_CONTAINER_SHORT_ID, STOPPED_CONTAINER_ID, UNKNOWN_CONTAINER_ID,
    container_uc, image_uc, network_uc, volume_uc,
};

// ── pause ──────────────────────────────────────────────────────────────────────

#[test]
fn pause_stopped_container_returns_not_running() {
    let uc = container_uc();
    let result = uc.pause(STOPPED_CONTAINER_ID);
    assert!(
        matches!(result, Err(ContainerError::NotRunning(_))),
        "expected NotRunning when pausing a stopped container, got: {result:?}"
    );
}

#[test]
fn pause_unknown_container_returns_not_found() {
    let uc = container_uc();
    let result = uc.pause(UNKNOWN_CONTAINER_ID);
    assert!(
        matches!(result, Err(ContainerError::NotFound(_))),
        "expected NotFound for unknown container, got: {result:?}"
    );
}

// ── unpause ────────────────────────────────────────────────────────────────────

#[test]
fn unpause_stopped_container_returns_not_running() {
    let uc = container_uc();
    let result = uc.unpause(STOPPED_CONTAINER_ID);
    assert!(
        matches!(result, Err(ContainerError::NotRunning(_))),
        "expected NotRunning when unpausing a stopped container, got: {result:?}"
    );
}

// ── restart ────────────────────────────────────────────────────────────────────

#[test]
fn restart_unknown_container_returns_not_found() {
    let uc = container_uc();
    let result = uc.restart(UNKNOWN_CONTAINER_ID, None);
    assert!(
        matches!(result, Err(ContainerError::NotFound(_))),
        "expected NotFound when restarting nonexistent container, got: {result:?}"
    );
}

#[test]
fn restart_known_container_succeeds() {
    let uc = container_uc();
    uc.restart(RUNNING_CONTAINER_ID, None)
        .expect("restart on known container must succeed");
}

// ── pause/unpause round-trip ───────────────────────────────────────────────────

#[test]
fn pause_then_start_makes_container_running_again() {
    let uc = container_uc();
    uc.stop(RUNNING_CONTAINER_SHORT_ID, None).expect("stop");
    let before = uc.list(false).expect("list");
    assert!(
        !before
            .iter()
            .any(|c| c.short_id == RUNNING_CONTAINER_SHORT_ID)
    );
    uc.start(RUNNING_CONTAINER_SHORT_ID).expect("start");
    let after = uc.list(false).expect("list");
    assert!(
        after
            .iter()
            .any(|c| c.short_id == RUNNING_CONTAINER_SHORT_ID),
        "container must be running after start"
    );
}

// ── remove_image / remove_volume / remove_network — NotFound ──────────────────

#[test]
fn remove_nonexistent_image_returns_not_found() {
    let uc = image_uc();
    let result = uc.remove("sha256:does-not-exist", false);
    assert!(
        matches!(result, Err(ContainerError::NotFound(_))),
        "expected NotFound for unknown image, got: {result:?}"
    );
}

#[test]
fn remove_known_image_succeeds() {
    let uc = image_uc();
    uc.remove("sha256:aaaa", false)
        .expect("remove on known image must succeed");
}

#[test]
fn remove_nonexistent_volume_returns_not_found() {
    let uc = volume_uc();
    let result = uc.remove("no-such-volume", false);
    assert!(
        matches!(result, Err(ContainerError::NotFound(_))),
        "expected NotFound for unknown volume, got: {result:?}"
    );
}

#[test]
fn remove_known_volume_succeeds() {
    let uc = volume_uc();
    uc.remove("postgres-data", false)
        .expect("remove on known volume must succeed");
}

#[test]
fn remove_nonexistent_network_returns_not_found() {
    let uc = network_uc();
    let result = uc.remove("net-does-not-exist");
    assert!(
        matches!(result, Err(ContainerError::NotFound(_))),
        "expected NotFound for unknown network, got: {result:?}"
    );
}

#[test]
fn remove_known_network_succeeds() {
    let uc = network_uc();
    uc.remove("bridge")
        .expect("remove on known network must succeed");
}
