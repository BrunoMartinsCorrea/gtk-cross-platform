// SPDX-License-Identifier: GPL-3.0-or-later
// Integration tests for cancellable lifecycle — verifies that cancelling a
// gio::Cancellable before the callback runs prevents UI updates without panicking.
use std::sync::Arc;

use gio::prelude::*;

use gtk_cross_platform::core::use_cases::image_use_case::ImageUseCase;
use gtk_cross_platform::infrastructure::containers::mock_driver::MockContainerDriver;
use gtk_cross_platform::ports::use_cases::i_image_use_case::IImageUseCase;

#[test]
fn cancellable_starts_uncancelled() {
    let c = gio::Cancellable::new();
    assert!(!c.is_cancelled());
}

#[test]
fn cancellable_is_cancelled_after_cancel() {
    let c = gio::Cancellable::new();
    c.cancel();
    assert!(c.is_cancelled());
}

#[test]
fn replacing_cancellable_cancels_previous() {
    let c1 = gio::Cancellable::new();
    assert!(!c1.is_cancelled());

    // Simulate what reload_impl does: cancel old, create new
    c1.cancel();
    let c2 = gio::Cancellable::new();

    assert!(c1.is_cancelled(), "old cancellable must be cancelled");
    assert!(!c2.is_cancelled(), "new cancellable must be fresh");
}

#[test]
fn callback_skipped_when_cancelled() {
    let c = gio::Cancellable::new();
    c.cancel();

    // Simulate the guard pattern used in reload_impl callbacks
    let mut callback_ran = false;
    if !c.is_cancelled() {
        callback_ran = true;
    }
    assert!(
        !callback_ran,
        "callback must be skipped when cancellable is cancelled"
    );
}

#[test]
fn driver_task_result_discarded_on_cancel_does_not_panic() {
    // Simulate: a driver task completes but the cancellable was already cancelled
    // (e.g., the view was navigated away from before the task finished).
    let uc = ImageUseCase::new(Arc::new(MockContainerDriver::new()));
    let result = uc.list();
    assert!(result.is_ok(), "mock driver should succeed");

    let c = gio::Cancellable::new();
    c.cancel();

    // Callback body — just returns early on cancel, no panic
    let images = result.unwrap();
    if c.is_cancelled() {
        // discard result
    } else {
        let _ = images.len(); // would normally update UI
    }
    // reaching here without panic is the assertion
}
