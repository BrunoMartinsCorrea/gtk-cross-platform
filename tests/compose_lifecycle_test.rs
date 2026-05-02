// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for compose project lifecycle operations (Feature H — Compose groups).
//!
//! Verifies that `start_all` and `stop_all` on groups of containers produce
//! per-container results and correctly affect the running set.
mod support;

use gtk_cross_platform::ports::use_cases::i_container_use_case::IContainerUseCase;

use support::container_uc as use_case;

#[test]
fn start_all_returns_results_for_each_id() {
    let uc = use_case();
    let ids = ["aabbccdd1122", "112233445566"];
    let results = uc.start_all(&ids).expect("start_all");
    assert_eq!(results.len(), 2);
    for (id, result) in ids.iter().zip(&results) {
        assert!(result.is_ok(), "start_all failed for {id}: {result:?}");
    }
}

#[test]
fn stop_all_returns_results_for_each_id() {
    let uc = use_case();
    let ids = ["aabbccdd1122", "112233445566"];
    let results = uc.stop_all(&ids, None).expect("stop_all");
    assert_eq!(results.len(), 2);
    for (id, result) in ids.iter().zip(&results) {
        assert!(result.is_ok(), "stop_all failed for {id}: {result:?}");
    }
}

#[test]
fn stop_all_with_timeout_succeeds() {
    let uc = use_case();
    let ids = ["aabbccdd1122"];
    let results = uc.stop_all(&ids, Some(30)).expect("stop_all");
    assert_eq!(results.len(), 1);
    assert!(
        results[0].is_ok(),
        "stop_all with timeout failed: {:?}",
        results[0]
    );
}

#[test]
fn start_all_empty_ids_returns_empty() {
    let uc = use_case();
    let results = uc.start_all(&[]).expect("start_all empty");
    assert!(results.is_empty());
}

#[test]
fn stop_all_after_start_all_leaves_containers_stopped() {
    let uc = use_case();
    let ids = ["aabbccdd1122"];
    uc.start_all(&ids).expect("start_all");
    uc.stop_all(&ids, None).expect("stop_all");
    let running = uc.list(false).expect("list");
    assert!(running.is_empty());
}
