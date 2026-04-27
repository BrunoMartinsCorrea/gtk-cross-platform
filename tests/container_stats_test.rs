// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for container resource stats (Feature 4 — Stats tab).
//!
//! Verifies that the `stats` use case returns valid data for running containers
//! and the appropriate error for stopped/non-existent containers.
use std::sync::Arc;

use gtk_cross_platform::core::use_cases::container_use_case::ContainerUseCase;
use gtk_cross_platform::infrastructure::containers::mock_driver::MockContainerDriver;
use gtk_cross_platform::ports::use_cases::i_container_use_case::IContainerUseCase;

fn container_uc() -> ContainerUseCase {
    ContainerUseCase::new(Arc::new(MockContainerDriver::new()))
}

#[test]
fn stats_for_running_container_returns_values() {
    let uc = container_uc();
    // "aabbccdd1122334455667788" is the web-server container, starts running
    let stats = uc.stats("aabbccdd1122334455667788").expect("stats");
    assert!(stats.cpu_percent >= 0.0);
    assert!(stats.memory_usage > 0);
    assert!(stats.net_rx_bytes >= 0);
    assert!(stats.net_tx_bytes >= 0);
}

#[test]
fn stats_cpu_percent_within_valid_range() {
    let uc = container_uc();
    let stats = uc.stats("aabbccdd1122334455667788").expect("stats");
    assert!(stats.cpu_percent >= 0.0, "cpu_percent must be non-negative");
    assert!(
        stats.cpu_percent <= 100.0,
        "cpu_percent must not exceed 100%"
    );
}

#[test]
fn stats_memory_usage_does_not_exceed_limit() {
    let uc = container_uc();
    let stats = uc.stats("aabbccdd1122334455667788").expect("stats");
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
    let stats = uc.stats("aabbccdd1122334455667788").expect("stats");
    // 52_428_800 bytes = 50 MiB
    assert!((stats.memory_usage_mb() - 50.0).abs() < 1.0);
}

#[test]
fn stats_for_stopped_container_returns_not_running_error() {
    let uc = container_uc();
    // "112233445566778899aabbcc" is the db container, starts in Exited state
    let result = uc.stats("112233445566778899aabbcc");
    assert!(result.is_err(), "stopped container should return error");
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("not running") || msg.contains("Not running"),
        "expected NotRunning error, got: {msg}"
    );
}

#[test]
fn stats_for_container_stopped_at_runtime() {
    let uc = container_uc();
    // Stop the running container first
    uc.stop("aabbccdd1122334455667788", None).expect("stop");
    // Now stats should fail
    let result = uc.stats("aabbccdd1122334455667788");
    assert!(result.is_err(), "stopped container should return error");
}
