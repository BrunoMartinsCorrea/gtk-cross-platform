// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for system events (Feature G — Events tab).
//!
//! Verifies that the `events` use case returns a non-empty list with required fields,
//! respects the limit parameter, and includes expected event types from the mock.
mod support;

use gtk_cross_platform::ports::use_cases::i_network_use_case::INetworkUseCase;

use support::network_uc as use_case;

#[test]
fn events_with_no_filter_returns_all_events() {
    let uc = use_case();
    let events = uc.events(None, None).expect("events");
    assert!(!events.is_empty());
}

#[test]
fn events_respect_limit() {
    let uc = use_case();
    let events = uc.events(None, Some(1)).expect("events");
    assert_eq!(events.len(), 1);
}

#[test]
fn events_have_required_fields() {
    let uc = use_case();
    let events = uc.events(None, None).expect("events");
    let e = &events[0];
    assert!(!e.event_type.is_empty());
    assert!(!e.action.is_empty());
    assert!(!e.timestamp.is_empty());
}

#[test]
fn events_contain_container_type() {
    let uc = use_case();
    let events = uc.events(None, None).expect("events");
    assert!(events.iter().any(|e| e.event_type == "container"));
}
