// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for dashboard data (Feature D — Dashboard/Home tab).
//!
//! Verifies system_df and prune return valid, non-negative data that the
//! dashboard can display without additional validation.
mod support;

use gtk_cross_platform::ports::use_cases::i_network_use_case::INetworkUseCase;

use support::{
    MOCK_CONTAINERS_RUNNING, MOCK_CONTAINERS_TOTAL, MOCK_IMAGES_TOTAL, STOPPED_CONTAINER_SHORT_ID,
    network_uc,
};

#[test]
fn system_df_returns_usage() {
    let uc = network_uc();
    let usage = uc.system_df().expect("system_df");
    assert_eq!(
        usage.containers_total, MOCK_CONTAINERS_TOTAL as u64,
        "mock has {MOCK_CONTAINERS_TOTAL} containers"
    );
    assert_eq!(
        usage.images_total, MOCK_IMAGES_TOTAL as u64,
        "mock has {MOCK_IMAGES_TOTAL} images (including dangling)"
    );
    assert_eq!(
        usage.containers_running, MOCK_CONTAINERS_RUNNING as u64,
        "mock has {MOCK_CONTAINERS_RUNNING} running container(s)"
    );
    assert!(
        usage.images_size > 0,
        "mock should report non-zero images size"
    );
}

#[test]
fn system_df_containers_total_includes_stopped() {
    let uc = network_uc();
    let usage = uc.system_df().expect("system_df");
    assert!(
        usage.containers_total >= usage.containers_running,
        "total ({}) must be >= running ({})",
        usage.containers_total,
        usage.containers_running
    );
}

#[test]
fn prune_system_returns_report() {
    let uc = network_uc();
    let report = uc.prune(false).expect("prune");
    assert_eq!(report.containers_deleted.len(), 1);
    assert!(
        report
            .containers_deleted
            .contains(&STOPPED_CONTAINER_SHORT_ID.to_string()),
        "expected {STOPPED_CONTAINER_SHORT_ID} in deleted containers, got: {:?}",
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
