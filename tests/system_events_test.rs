// SPDX-License-Identifier: GPL-3.0-or-later
use std::sync::Arc;

use gtk_cross_platform::core::use_cases::network_use_case::NetworkUseCase;
use gtk_cross_platform::infrastructure::containers::mock_driver::MockContainerDriver;
use gtk_cross_platform::ports::use_cases::i_network_use_case::INetworkUseCase;

fn use_case() -> NetworkUseCase {
    NetworkUseCase::new(Arc::new(MockContainerDriver::new()))
}

#[test]
fn events_returns_list() {
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
