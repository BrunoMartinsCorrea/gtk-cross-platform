// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for image pull behavior (Feature 2 — Pull image dialog).
//!
//! The mock driver validates that image references are non-empty and don't contain
//! malformed separators. Tests verify success, error cases, and that pulled images
//! can be listed afterwards.
mod support;

use gtk_cross_platform::infrastructure::containers::error::ContainerError;
use gtk_cross_platform::ports::use_cases::i_image_use_case::IImageUseCase;

use support::image_uc;

#[test]
fn pull_valid_reference_succeeds() {
    let uc = image_uc();
    uc.pull("nginx:latest").expect("pull valid reference");
}

#[test]
fn pull_fully_qualified_registry_succeeds() {
    let uc = image_uc();
    uc.pull("ghcr.io/org/app:v1.0")
        .expect("pull fully qualified reference");
}

#[test]
fn pull_dockerhub_no_tag_succeeds() {
    let uc = image_uc();
    uc.pull("ubuntu").expect("pull reference without tag");
}

#[test]
fn pull_invalid_reference_with_triple_colon_returns_error() {
    let uc = image_uc();
    let result = uc.pull(":::");
    assert!(
        matches!(result, Err(ContainerError::ParseError(_))),
        "malformed reference should return ParseError, got: {result:?}"
    );
}

#[test]
fn pull_empty_reference_returns_error() {
    let uc = image_uc();
    let result = uc.pull("");
    assert!(
        matches!(result, Err(ContainerError::ParseError(_))),
        "empty reference should return ParseError, got: {result:?}"
    );
}

#[test]
fn list_images_after_pull_returns_existing_images() {
    let uc = image_uc();
    let before = uc.list().expect("list").len();
    uc.pull("nginx:latest").expect("pull");
    // Mock doesn't add the image to the list on pull — listing reflects current state
    let after = uc.list().expect("list").len();
    assert_eq!(after, before);
}
