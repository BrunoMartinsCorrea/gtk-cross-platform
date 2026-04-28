// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for container creation (Feature 3 — Create container wizard).
//!
//! Verifies that the `create` use case correctly delegates spec fields to the driver,
//! validates image existence, and rejects duplicate container names.
//! All tests use MockContainerDriver — no runtime required.
mod support;

use gtk_cross_platform::core::domain::container::{CreateContainerOptions, RestartPolicy};
use gtk_cross_platform::infrastructure::containers::error::ContainerError;
use gtk_cross_platform::ports::use_cases::i_container_use_case::IContainerUseCase;

use support::container_uc;

#[test]
fn create_container_minimal_image_only() {
    let uc = container_uc();
    let opts = CreateContainerOptions {
        image: "nginx:latest".to_string(),
        ..Default::default()
    };
    let id = uc.create(&opts).expect("create");
    assert!(!id.is_empty());
}

#[test]
fn create_container_with_port_mapping() {
    let uc = container_uc();
    let opts = CreateContainerOptions {
        image: "nginx:latest".to_string(),
        port_bindings: vec![(8080, 80)],
        ..Default::default()
    };
    let id = uc.create(&opts).expect("create");
    assert!(!id.is_empty());

    // The newly created container should appear in the full list
    let containers = uc.list(true).expect("list");
    assert!(containers.iter().any(|c| c.id == id));
}

#[test]
fn create_container_with_env_vars() {
    let uc = container_uc();
    let opts = CreateContainerOptions {
        image: "nginx:latest".to_string(),
        env: vec!["MY_VAR=hello".to_string(), "OTHER=world".to_string()],
        ..Default::default()
    };
    let id = uc.create(&opts).expect("create");
    let containers = uc.list(true).expect("list");
    let created = containers
        .iter()
        .find(|c| c.id == id)
        .expect("find created");
    assert!(created.env.contains(&"MY_VAR=hello".to_string()));
    assert!(created.env.contains(&"OTHER=world".to_string()));
}

#[test]
fn create_container_unknown_image_returns_not_found() {
    let uc = container_uc();
    let opts = CreateContainerOptions {
        image: "this-image-does-not-exist:latest".to_string(),
        ..Default::default()
    };
    let result = uc.create(&opts);
    assert!(
        matches!(result, Err(ContainerError::NotFound(_))),
        "expected NotFound error, got: {result:?}"
    );
}

#[test]
fn create_container_name_conflict_returns_already_exists() {
    let uc = container_uc();
    // First creation succeeds
    let opts = CreateContainerOptions {
        image: "nginx:latest".to_string(),
        name: Some("my-unique-name".to_string()),
        ..Default::default()
    };
    uc.create(&opts).expect("first create");

    // Second creation with the same name must fail
    let opts2 = CreateContainerOptions {
        image: "nginx:latest".to_string(),
        name: Some("my-unique-name".to_string()),
        ..Default::default()
    };
    let result = uc.create(&opts2);
    assert!(
        matches!(result, Err(ContainerError::AlreadyExists(_))),
        "expected AlreadyExists error, got: {result:?}"
    );
}

#[test]
fn create_container_restart_policy_no_panics() {
    let uc = container_uc();
    let opts = CreateContainerOptions {
        image: "nginx:latest".to_string(),
        restart_policy: RestartPolicy::Always,
        ..Default::default()
    };
    uc.create(&opts).expect("create with restart policy must succeed");
}

#[test]
fn create_container_appears_in_running_list() {
    let uc = container_uc();
    let before = uc.list(false).expect("list running before").len();
    let opts = CreateContainerOptions {
        image: "nginx:latest".to_string(),
        ..Default::default()
    };
    let id = uc.create(&opts).expect("create");
    let after = uc.list(false).expect("list running after");
    assert!(after.len() > before, "new container should be running");
    assert!(after.iter().any(|c| c.id == id));
}
