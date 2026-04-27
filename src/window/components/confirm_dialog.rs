// SPDX-License-Identifier: GPL-3.0-or-later
use adw::prelude::*;
use gettextrs::gettext;

/// Show a destructive-action confirmation dialog.
///
/// After the dialog closes (confirm or cancel), focus is restored to `parent` so
/// keyboard users are not left with undefined focus state (GNOME HIG focus rule).
pub fn ask(
    parent: &impl IsA<gtk4::Widget>,
    heading: &str,
    body: &str,
    confirm_label: &str,
    on_confirm: impl Fn() + 'static,
) {
    let window = parent.root().and_downcast::<gtk4::Window>();
    let dialog = adw::MessageDialog::new(window.as_ref(), Some(heading), Some(body));
    dialog.add_response("cancel", &gettext("Cancel"));
    dialog.add_response("confirm", confirm_label);
    dialog.set_response_appearance("confirm", adw::ResponseAppearance::Destructive);
    dialog.set_default_response(Some("cancel"));
    dialog.set_close_response("cancel");

    let parent_weak = parent.upcast_ref::<gtk4::Widget>().downgrade();
    dialog.connect_response(None, move |_, response| {
        if response == "confirm" {
            on_confirm();
        }
        if let Some(w) = parent_weak.upgrade() {
            w.grab_focus();
        }
    });
    dialog.present();
}
