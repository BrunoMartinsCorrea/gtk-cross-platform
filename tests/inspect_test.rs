// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for container JSON inspection (Feature 5 — Inspect tab).
//!
//! Verifies that `inspect_json` returns valid, pretty-printed JSON containing
//! the container's ID and name, and fails appropriately for unknown IDs.
mod support;

use gtk_cross_platform::infrastructure::containers::error::ContainerError;
use gtk_cross_platform::ports::use_cases::i_container_use_case::IContainerUseCase;

use support::{RUNNING_CONTAINER_ID, UNKNOWN_CONTAINER_ID, container_uc};

#[test]
fn inspect_json_returns_valid_json() {
    let uc = container_uc();
    let json_str = uc.inspect_json(RUNNING_CONTAINER_ID).expect("inspect json");
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("must be valid JSON");
    assert!(parsed.is_object());
}

#[test]
fn inspect_json_contains_container_id() {
    let uc = container_uc();
    let json_str = uc.inspect_json(RUNNING_CONTAINER_ID).expect("inspect json");
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(parsed.get("Id").is_some(), "JSON must contain 'Id' field");
}

#[test]
fn inspect_json_contains_container_name() {
    let uc = container_uc();
    let json_str = uc.inspect_json(RUNNING_CONTAINER_ID).expect("inspect json");
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let name = parsed["Name"].as_str().unwrap_or("");
    assert!(
        name.contains("web-server"),
        "Name field must contain container name, got: {name}"
    );
}

#[test]
fn inspect_json_contains_config_section() {
    let uc = container_uc();
    let json_str = uc.inspect_json(RUNNING_CONTAINER_ID).expect("inspect json");
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(
        parsed.get("Config").is_some(),
        "JSON must contain 'Config' section"
    );
}

#[test]
fn inspect_json_contains_state_section() {
    let uc = container_uc();
    let json_str = uc.inspect_json(RUNNING_CONTAINER_ID).expect("inspect json");
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(
        parsed.get("State").is_some(),
        "JSON must contain 'State' section"
    );
}

#[test]
fn inspect_json_unknown_id_returns_not_found() {
    let uc = container_uc();
    let result = uc.inspect_json(UNKNOWN_CONTAINER_ID);
    assert!(
        matches!(result, Err(ContainerError::NotFound(_))),
        "expected NotFound error, got: {result:?}"
    );
}

#[test]
fn inspect_json_is_pretty_printed() {
    let uc = container_uc();
    let json_str = uc.inspect_json(RUNNING_CONTAINER_ID).expect("inspect json");
    // Pretty-printed JSON contains newlines
    assert!(
        json_str.contains('\n'),
        "inspect JSON should be pretty-printed (contain newlines)"
    );
}
