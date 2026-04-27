// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for Compose stack grouping in the containers sidebar (Feature 7).
//!
//! The `group_by_compose` domain function groups containers by their
//! `compose_project` field. Named groups are sorted alphabetically and
//! ungrouped containers (compose_project = None) appear last.
//! Tests also verify that the mock driver correctly populates compose_project
//! from the `com.docker.compose.project` label.
use std::collections::HashMap;
use std::sync::Arc;

use gtk_cross_platform::core::domain::container::{Container, ContainerStatus, group_by_compose};
use gtk_cross_platform::core::use_cases::container_use_case::ContainerUseCase;
use gtk_cross_platform::infrastructure::containers::mock_driver::MockContainerDriver;
use gtk_cross_platform::ports::use_cases::i_container_use_case::IContainerUseCase;

fn make_container(name: &str, compose: Option<&str>) -> Container {
    Container {
        id: format!("{name}aabbccdd1122"),
        short_id: name[..4.min(name.len())].to_string(),
        name: name.to_string(),
        image: "test:latest".to_string(),
        command: "/run".to_string(),
        created: 0,
        status: ContainerStatus::Running,
        status_text: "Running".to_string(),
        ports: vec![],
        labels: HashMap::new(),
        mounts: vec![],
        env: vec![],
        compose_project: compose.map(str::to_string),
    }
}

#[test]
fn group_three_containers_same_project() {
    let containers = vec![
        make_container("web", Some("my-stack")),
        make_container("db", Some("my-stack")),
        make_container("cache", Some("my-stack")),
    ];
    let groups = group_by_compose(&containers);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].0.as_deref(), Some("my-stack"));
    assert_eq!(groups[0].1.len(), 3);
}

#[test]
fn ungrouped_containers_are_last() {
    let containers = vec![
        make_container("web", Some("stack-a")),
        make_container("solo", None),
    ];
    let groups = group_by_compose(&containers);
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].0.as_deref(), Some("stack-a"));
    assert!(groups[1].0.is_none(), "ungrouped should be last");
}

#[test]
fn all_ungrouped_containers_in_single_none_group() {
    let containers = vec![make_container("a", None), make_container("b", None)];
    let groups = group_by_compose(&containers);
    assert_eq!(groups.len(), 1);
    assert!(groups[0].0.is_none());
    assert_eq!(groups[0].1.len(), 2);
}

#[test]
fn multiple_projects_sorted_alphabetically() {
    let containers = vec![
        make_container("svc", Some("z-project")),
        make_container("api", Some("a-project")),
        make_container("db", Some("m-project")),
    ];
    let groups = group_by_compose(&containers);
    assert_eq!(groups.len(), 3);
    assert_eq!(groups[0].0.as_deref(), Some("a-project"));
    assert_eq!(groups[1].0.as_deref(), Some("m-project"));
    assert_eq!(groups[2].0.as_deref(), Some("z-project"));
}

#[test]
fn empty_container_list_produces_no_groups() {
    let groups = group_by_compose(&[]);
    assert!(groups.is_empty());
}

#[test]
fn mixed_grouped_and_ungrouped() {
    let containers = vec![
        make_container("web", Some("stack-a")),
        make_container("db", Some("stack-a")),
        make_container("solo", None),
        make_container("aux", Some("stack-b")),
    ];
    let groups = group_by_compose(&containers);
    // stack-a, stack-b (alphabetical), then ungrouped
    assert_eq!(groups.len(), 3);
    assert_eq!(groups[0].0.as_deref(), Some("stack-a"));
    assert_eq!(groups[0].1.len(), 2);
    assert_eq!(groups[1].0.as_deref(), Some("stack-b"));
    assert_eq!(groups[1].1.len(), 1);
    assert!(groups[2].0.is_none());
    assert_eq!(groups[2].1.len(), 1);
}

// ── Integration: mock driver populates compose_project from labels ─────────────

#[test]
fn mock_containers_have_compose_project_populated() {
    let uc = ContainerUseCase::new(Arc::new(MockContainerDriver::new()));
    let containers = uc.list(true).expect("list");
    let web = containers
        .iter()
        .find(|c| c.name == "web-server")
        .expect("web-server");
    assert_eq!(
        web.compose_project.as_deref(),
        Some("web-stack"),
        "web-server should belong to web-stack compose project"
    );
}

#[test]
fn standalone_container_has_no_compose_project() {
    let uc = ContainerUseCase::new(Arc::new(MockContainerDriver::new()));
    let containers = uc.list(true).expect("list");
    let solo = containers
        .iter()
        .find(|c| c.name == "standalone")
        .expect("standalone");
    assert!(
        solo.compose_project.is_none(),
        "standalone container should have no compose project"
    );
}

#[test]
fn grouping_mock_containers_produces_named_group_and_ungrouped() {
    let uc = ContainerUseCase::new(Arc::new(MockContainerDriver::new()));
    let containers = uc.list(true).expect("list");
    let groups = group_by_compose(&containers);

    // At least one named group (web-stack) and the ungrouped solo container
    let has_named = groups.iter().any(|(k, _)| k.is_some());
    let has_ungrouped = groups.iter().any(|(k, _)| k.is_none());
    assert!(has_named, "should have at least one named compose group");
    assert!(has_ungrouped, "should have ungrouped containers");
}
