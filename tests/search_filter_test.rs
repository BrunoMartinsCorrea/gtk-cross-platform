// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for client-side container list filtering (Feature 1 — Global search/filter).
//!
//! The `filter_containers` domain function must match containers by name, image,
//! short ID, or compose project using a case-insensitive substring match.
//! All tests run without GTK or a container runtime.
use std::collections::HashMap;

use gtk_cross_platform::core::domain::container::{Container, ContainerStatus, filter_containers};

fn make_container(name: &str, image: &str, short_id: &str, compose: Option<&str>) -> Container {
    Container {
        id: format!("{short_id}aabbccdd1122"),
        short_id: short_id.to_string(),
        name: name.to_string(),
        image: image.to_string(),
        command: "/start.sh".to_string(),
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

fn sample_containers() -> Vec<Container> {
    vec![
        make_container(
            "nginx-proxy",
            "nginx:latest",
            "aabbccdd1122",
            Some("web-stack"),
        ),
        make_container(
            "postgres-db",
            "postgres:15",
            "bbccdd334455",
            Some("web-stack"),
        ),
        make_container("redis-cache", "redis:alpine", "223344556677", None),
    ]
}

#[test]
fn filter_by_name_matches() {
    let containers = sample_containers();
    let result = filter_containers(&containers, "nginx");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "nginx-proxy");
}

#[test]
fn filter_by_image_matches() {
    let containers = sample_containers();
    let result = filter_containers(&containers, "postgres");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "postgres-db");
}

#[test]
fn filter_by_short_id_matches() {
    let containers = sample_containers();
    let result = filter_containers(&containers, "223344");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "redis-cache");
}

#[test]
fn filter_by_compose_project_matches() {
    let containers = sample_containers();
    let result = filter_containers(&containers, "web-stack");
    assert_eq!(result.len(), 2);
}

#[test]
fn filter_empty_query_returns_all() {
    let containers = sample_containers();
    let result = filter_containers(&containers, "");
    assert_eq!(result.len(), 3);
}

#[test]
fn filter_no_match_returns_empty() {
    let containers = sample_containers();
    let result = filter_containers(&containers, "xyzzy");
    assert!(result.is_empty());
}

#[test]
fn filter_case_insensitive_name() {
    let containers = sample_containers();
    let result = filter_containers(&containers, "NGINX");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "nginx-proxy");
}

#[test]
fn filter_case_insensitive_image() {
    let containers = sample_containers();
    let result = filter_containers(&containers, "POSTGRES");
    assert_eq!(result.len(), 1);
}

#[test]
fn filter_cleared_returns_all() {
    let containers = sample_containers();
    // Simulate: filter applied then cleared
    let _filtered = filter_containers(&containers, "nginx");
    let restored = filter_containers(&containers, "");
    assert_eq!(restored.len(), 3);
}

#[test]
fn filter_partial_image_tag_matches() {
    let containers = sample_containers();
    let result = filter_containers(&containers, "alpine");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "redis-cache");
}
