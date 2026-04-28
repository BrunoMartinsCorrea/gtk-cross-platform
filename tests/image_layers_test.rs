// SPDX-License-Identifier: GPL-3.0-or-later
mod support;

use gtk_cross_platform::infrastructure::containers::error::ContainerError;
use gtk_cross_platform::ports::use_cases::i_image_use_case::IImageUseCase;

use support::image_uc;

#[test]
fn layers_returns_deterministic_list() {
    let uc = image_uc();
    let layers = uc.layers("sha256:aaaa").expect("layers");
    assert!(!layers.is_empty());
    assert_eq!(layers.len(), 3);
}

#[test]
fn layers_for_known_image_have_populated_fields() {
    let uc = image_uc();
    let layers = uc.layers("sha256:aaaa").expect("layers");
    let first = &layers[0];
    assert!(!first.id.is_empty());
    assert!(!first.cmd.is_empty());
    assert!(first.size > 0);
}

#[test]
fn layers_unknown_image_returns_not_found() {
    let uc = image_uc();
    let result = uc.layers("sha256:doesnotexist");
    assert!(
        matches!(result, Err(ContainerError::NotFound(_))),
        "expected NotFound for unknown image, got: {result:?}"
    );
}

#[test]
fn layers_cumulative_size_positive() {
    let uc = image_uc();
    let layers = uc.layers("sha256:aaaa").expect("layers");
    let total: u64 = layers.iter().map(|l| l.size).sum();
    assert!(total > 0);
}
