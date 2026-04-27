// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for dashboard data (Feature D — Dashboard/Home tab).
//!
//! Verifies system_df and prune_system return valid, non-negative data that the
//! dashboard can display without additional validation.
mod support;

use gtk_cross_platform::ports::i_container_driver::IContainerDriver;

use support::mock_driver as driver;

/// system_df should return a SystemUsage consistent with the mock state.
#[test]
fn system_df_returns_usage() {
    let d = driver();
    let usage = d.system_df().expect("system_df");
    // Mock has 3 containers (web-server Running, db Exited, standalone Stopped)
    assert_eq!(usage.containers_total, 3, "mock has 3 containers");
    // Mock has 3 images (nginx:latest, postgres:15, dangling sha256:cccc)
    assert_eq!(usage.images_total, 3, "mock has 3 images");
    assert!(
        usage.images_size > 0,
        "mock should report non-zero images size"
    );
}

/// When listing all containers, the total count must be ≥ running count.
#[test]
fn system_df_containers_total_includes_stopped() {
    let d = driver();
    let usage = d.system_df().expect("system_df");
    assert!(
        usage.containers_total >= usage.containers_running,
        "total ({}) must be >= running ({})",
        usage.containers_total,
        usage.containers_running
    );
}

/// prune_system should return a PruneReport with the expected deleted resources.
#[test]
fn prune_system_returns_report() {
    let d = driver();
    let report = d.prune_system(false).expect("prune");
    // The mock declares "112233445566" as the deleted container
    assert_eq!(report.containers_deleted.len(), 1);
    assert!(
        report
            .containers_deleted
            .contains(&"112233445566".to_string()),
        "expected 112233445566 in deleted containers, got: {:?}",
        report.containers_deleted
    );
    assert!(
        report.images_deleted.is_empty(),
        "mock prune deletes no images"
    );
    assert!(
        report.volumes_deleted.is_empty(),
        "mock prune deletes no volumes"
    );
    assert_eq!(report.space_reclaimed, 0);
}
