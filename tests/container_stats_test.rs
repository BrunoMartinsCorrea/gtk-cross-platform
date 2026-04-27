// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for container resource stats (Feature 4 — Stats tab).
//!
//! Verifies that the `stats` use case returns valid data for running containers
//! and the appropriate error for stopped/non-existent containers.
mod support;

use gtk_cross_platform::infrastructure::containers::error::ContainerError;
use gtk_cross_platform::ports::use_cases::i_container_use_case::IContainerUseCase;

use support::{RUNNING_CONTAINER_ID, STOPPED_CONTAINER_ID, container_uc};

#[test]
fn stats_for_running_container_returns_values() {
    let uc = container_uc();
    let stats = uc.stats(RUNNING_CONTAINER_ID).expect("stats");
    assert!(stats.cpu_percent >= 0.0);
    assert!(stats.memory_usage > 0);
    // Verify exact mock values (net_rx=1024, net_tx=512)
    assert_eq!(stats.net_rx_bytes, 1024, "mock net_rx_bytes must be 1024");
    assert_eq!(stats.net_tx_bytes, 512, "mock net_tx_bytes must be 512");
}

#[test]
fn stats_cpu_percent_within_valid_range() {
    let uc = container_uc();
    let stats = uc.stats(RUNNING_CONTAINER_ID).expect("stats");
    assert!(stats.cpu_percent >= 0.0, "cpu_percent must be non-negative");
    assert!(
        stats.cpu_percent <= 100.0,
        "cpu_percent must not exceed 100%"
    );
}

#[test]
fn stats_memory_usage_does_not_exceed_limit() {
    let uc = container_uc();
    let stats = uc.stats(RUNNING_CONTAINER_ID).expect("stats");
    assert!(
        stats.memory_usage <= stats.memory_limit,
        "memory_usage ({}) must not exceed memory_limit ({})",
        stats.memory_usage,
        stats.memory_limit
    );
}

#[test]
fn stats_memory_mb_conversion_is_accurate() {
    let uc = container_uc();
    let stats = uc.stats(RUNNING_CONTAINER_ID).expect("stats");
    // 52_428_800 bytes = 50 MiB
    assert!((stats.memory_usage_mb() - 50.0).abs() < 1.0);
}

#[test]
fn stats_for_stopped_container_returns_not_running_error() {
    let uc = container_uc();
    let result = uc.stats(STOPPED_CONTAINER_ID);
    assert!(
        matches!(result, Err(ContainerError::NotRunning(_))),
        "expected NotRunning error, got: {result:?}"
    );
}

#[test]
fn stats_for_container_stopped_at_runtime() {
    let uc = container_uc();
    uc.stop(RUNNING_CONTAINER_ID, None).expect("stop");
    let result = uc.stats(RUNNING_CONTAINER_ID);
    assert!(result.is_err(), "stopped container should return error");
}
