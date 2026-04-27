// SPDX-License-Identifier: GPL-3.0-or-later
use std::sync::Arc;

use gtk_cross_platform::core::use_cases::image_use_case::ImageUseCase;
use gtk_cross_platform::infrastructure::containers::mock_driver::MockContainerDriver;
use gtk_cross_platform::ports::use_cases::i_image_use_case::IImageUseCase;

fn use_case() -> ImageUseCase {
    ImageUseCase::new(Arc::new(MockContainerDriver::new()))
}

#[test]
fn layers_returns_deterministic_list() {
    let uc = use_case();
    let layers = uc.layers("sha256:aaaa").expect("layers");
    assert!(!layers.is_empty());
    assert_eq!(layers.len(), 3);
}

#[test]
fn layers_have_id_cmd_and_size() {
    let uc = use_case();
    let layers = uc.layers("sha256:aaaa").expect("layers");
    let first = &layers[0];
    assert!(!first.id.is_empty());
    assert!(!first.cmd.is_empty());
    assert!(first.size > 0);
}

#[test]
fn layers_unknown_image_returns_not_found() {
    let uc = use_case();
    let result = uc.layers("sha256:doesnotexist");
    assert!(result.is_err());
}

#[test]
fn layers_cumulative_size_positive() {
    let uc = use_case();
    let layers = uc.layers("sha256:aaaa").expect("layers");
    let total: u64 = layers.iter().map(|l| l.size).sum();
    assert!(total > 0);
}
