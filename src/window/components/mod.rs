// SPDX-License-Identifier: GPL-3.0-or-later
use gtk4::prelude::*;

pub mod confirm_dialog;
pub mod detail_pane;
pub mod empty_state;
pub mod resource_row;
pub mod status_badge;
pub mod toast_util;

pub fn clear_box(b: &gtk4::Box) {
    while let Some(child) = b.first_child() {
        b.remove(&child);
    }
}
