// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for container resource stats (Feature 4 — Stats tab).
//!
//! Verifies that the `stats` use case returns valid data for running containers
//! and the appropriate error for stopped/non-existent containers.
mod support;

use gtk_cross_platform::infrastructure::containers::error::ContainerError;
use gtk_cross_platform::ports::use_cases::i_container_use_case::IContainerUseCase;

use support::{
    MOCK_MEMORY_LIMIT_BYTES, MOCK_MEMORY_USAGE_BYTES, MOCK_NET_RX_BYTES, MOCK_NET_TX_BYTES,
    RUNNING_CONTAINER_ID, STOPPED_CONTAINER_ID, container_uc,
};

#[test]
fn stats_for_running_container_returns_values() {
    let uc = container_uc();
    let stats = uc.stats(RUNNING_CONTAINER_ID).expect("stats");
    assert!(stats.memory_usage > 0);
    assert_eq!(stats.net_rx_bytes, MOCK_NET_RX_BYTES);
    assert_eq!(stats.net_tx_bytes, MOCK_NET_TX_BYTES);
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
    assert_eq!(stats.memory_limit, MOCK_MEMORY_LIMIT_BYTES);
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
    let expected_mb = MOCK_MEMORY_USAGE_BYTES as f64 / (1024.0 * 1024.0);
    assert!(
        (stats.memory_usage_mb() - expected_mb).abs() < 1.0,
        "memory_usage_mb() = {}, expected ~{expected_mb}",
        stats.memory_usage_mb()
    );
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
    assert!(
        matches!(result, Err(ContainerError::NotRunning(_))),
        "expected NotRunning after stopping container, got: {result:?}"
    );
}
