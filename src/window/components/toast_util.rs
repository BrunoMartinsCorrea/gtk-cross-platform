// SPDX-License-Identifier: GPL-3.0-or-later
#[allow(unused_imports)]
use adw::prelude::*;
use gettextrs::gettext;

pub struct ToastUtil;

impl ToastUtil {
    /// Transient confirmation (3 s). Use for non-destructive feedback.
    pub fn show(overlay: &adw::ToastOverlay, message: &str) {
        let toast = adw::Toast::new(message);
        toast.set_timeout(3);
        overlay.add_toast(toast);
    }

    /// Persistent error (timeout = 0, stays until dismissed). Use for failures.
    pub fn show_error(overlay: &adw::ToastOverlay, message: &str) {
        let toast = adw::Toast::new(message);
        toast.set_timeout(0);
        overlay.add_toast(toast);
    }

    /// Destructive confirmation with Undo button (timeout = 10 s, HIGH priority).
    ///
    /// `undo_action_name` must be a registered `gio::SimpleAction` on the window.
    pub fn show_destructive(overlay: &adw::ToastOverlay, message: &str, undo_action_name: &str) {
        let toast = adw::Toast::new(message);
        toast.set_priority(adw::ToastPriority::High);
        toast.set_timeout(10);
        toast.set_button_label(Some(&gettext("Undo")));
        toast.set_action_name(Some(undo_action_name));
        overlay.add_toast(toast);
    }
}
