// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for pull_image_streaming (Feature A — layer progress streaming).
//!
//! Uses MockContainerDriver directly (not through a use case) because streaming is a
//! driver-level concern that does not go through a use-case port.
mod support;

use gtk_cross_platform::core::domain::container::PullStatus;
use gtk_cross_platform::infrastructure::containers::error::ContainerError;
use gtk_cross_platform::ports::i_container_driver::IContainerDriver;

use support::mock_driver as driver;

fn collect_events(
    d: &impl IContainerDriver,
    reference: &str,
) -> Vec<gtk_cross_platform::core::domain::container::PullProgress> {
    let (tx, rx) = async_channel::bounded(64);
    d.pull_image_streaming(reference, tx).expect("pull");
    let mut events = vec![];
    while let Ok(e) = rx.try_recv() {
        events.push(e);
    }
    events
}

#[test]
fn streaming_pull_emits_layer_events() {
    let d = driver();
    let events = collect_events(&*d, "nginx:latest");
    assert!(
        events.len() >= 3,
        "expected 3+ events, got {}",
        events.len()
    );
}

#[test]
fn streaming_pull_all_layers_reach_done() {
    let d = driver();
    let events = collect_events(&*d, "nginx:latest");
    let mut final_status: std::collections::HashMap<String, PullStatus> =
        std::collections::HashMap::new();
    for e in &events {
        final_status.insert(e.layer_id.clone(), e.status.clone());
    }
    assert!(
        !final_status.is_empty(),
        "expected at least one layer in events"
    );
    for (layer_id, status) in &final_status {
        assert_eq!(
            status,
            &PullStatus::Done,
            "layer {layer_id} did not reach Done"
        );
    }
}

#[test]
fn streaming_pull_invalid_ref_errors() {
    let d = driver();
    let (tx, _rx) = async_channel::bounded(64);
    let result = d.pull_image_streaming(":::", tx);
    assert!(
        matches!(result, Err(ContainerError::ParseError(_))),
        "malformed reference should return ParseError, got: {result:?}"
    );
}

#[test]
fn streaming_pull_cancel_stops_stream() {
    let d = driver();
    d.cancel_pull();
    let (tx, rx) = async_channel::bounded(64);
    d.pull_image_streaming("nginx:latest", tx)
        .expect("pull after cancel should not error");
    let mut events = vec![];
    while let Ok(e) = rx.try_recv() {
        events.push(e);
    }
    assert!(
        events.is_empty(),
        "expected no events after pre-cancel, got {}",
        events.len()
    );
}
