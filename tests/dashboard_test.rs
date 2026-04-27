// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for dashboard data (Feature D — Dashboard/Home tab).
//!
//! Verifies system_df and prune_system return valid, non-negative data that the
//! dashboard can display without additional validation.
use std::sync::Arc;

use gtk_cross_platform::infrastructure::containers::mock_driver::MockContainerDriver;
use gtk_cross_platform::ports::i_container_driver::IContainerDriver;

fn driver() -> Arc<MockContainerDriver> {
    Arc::new(MockContainerDriver::new())
}

/// system_df should return a SystemUsage with sensible counts (images_total >= 1 for the mock).
#[test]
fn test_system_df_returns_usage() {
    let d = driver();
    let usage = d.system_df().expect("system_df");
    // Mock has 2 images; ensure the count is non-zero
    assert!(
        usage.images_total > 0,
        "mock should report at least one image"
    );
    // images_size should be populated
    assert!(
        usage.images_size > 0,
        "mock should report non-zero images size"
    );
}

/// When listing all containers, the total count must be ≥ running count.
#[test]
fn test_system_df_containers_total_includes_stopped() {
    let d = driver();
    let usage = d.system_df().expect("system_df");
    assert!(
        usage.containers_total >= usage.containers_running,
        "total ({}) must be >= running ({})",
        usage.containers_total,
        usage.containers_running
    );
}

/// prune_system should return a PruneReport (the call itself must not error).
#[test]
fn test_prune_system_returns_report() {
    let d = driver();
    let report = d.prune_system(false).expect("prune");
    // The mock lists deleted containers; the field must be a Vec (may be empty or not)
    let _ = report.containers_deleted.len();
    let _ = report.images_deleted.len();
    let _ = report.volumes_deleted.len();
    // space_reclaimed is a u64 — verify the call returned successfully
    let _ = report.space_reclaimed;
}
