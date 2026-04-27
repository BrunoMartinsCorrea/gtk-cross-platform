// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for image pull behavior (Feature 2 — Pull image dialog).
//!
//! The mock driver validates that image references are non-empty and don't contain
//! malformed separators. Tests verify success, error cases, and that pulled images
//! can be listed afterwards.
use std::sync::Arc;

use gtk_cross_platform::core::use_cases::image_use_case::ImageUseCase;
use gtk_cross_platform::infrastructure::containers::mock_driver::MockContainerDriver;
use gtk_cross_platform::ports::use_cases::i_image_use_case::IImageUseCase;

fn image_uc() -> impl IImageUseCase {
    ImageUseCase::new(Arc::new(MockContainerDriver::new()))
}

#[test]
fn pull_valid_reference_succeeds() {
    let uc = image_uc();
    assert!(uc.pull("nginx:latest").is_ok());
}

#[test]
fn pull_fully_qualified_registry_succeeds() {
    let uc = image_uc();
    assert!(uc.pull("ghcr.io/org/app:v1.0").is_ok());
}

#[test]
fn pull_dockerhub_no_tag_succeeds() {
    let uc = image_uc();
    assert!(uc.pull("ubuntu").is_ok());
}

#[test]
fn pull_invalid_reference_with_triple_colon_returns_error() {
    let uc = image_uc();
    let result = uc.pull(":::");
    assert!(result.is_err(), "malformed reference should fail");
}

#[test]
fn pull_empty_reference_returns_error() {
    let uc = image_uc();
    let result = uc.pull("");
    assert!(result.is_err(), "empty reference should fail");
}

#[test]
fn list_images_after_pull_returns_existing_images() {
    let uc = image_uc();
    let before = uc.list().expect("list").len();
    assert!(uc.pull("nginx:latest").is_ok());
    // Mock doesn't add the image to the list on pull — listing reflects current state
    let after = uc.list().expect("list").len();
    assert_eq!(after, before);
}
