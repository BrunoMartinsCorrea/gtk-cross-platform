// SPDX-License-Identifier: GPL-3.0-or-later
use adw::prelude::*;

/// A group of labeled key-value rows with an optional section header.
pub struct PropertyGroup {
    pub title: String,
    pub rows: Vec<(String, String)>,
}

/// Build a vertical property grid from `groups`, wrapped in an `adw::Clamp`.
///
/// The clamp limits maximum width to 720 sp so the grid does not over-stretch
/// on wide monitors. Tab order follows visual top-to-bottom order (GTK default).
/// Each row title is the accessible label; subtitle is the accessible value.
pub fn build(groups: &[PropertyGroup]) -> adw::Clamp {
    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    for group in groups {
        let pref_group = adw::PreferencesGroup::new();
        if !group.title.is_empty() {
            pref_group.set_title(&group.title);
        }
        for (label, value) in &group.rows {
            let row = adw::ActionRow::new();
            row.set_title(label.as_str());
            row.set_subtitle(value.as_str());
            pref_group.add(&row);
        }
        vbox.append(&pref_group);
    }
    let clamp = adw::Clamp::new();
    clamp.set_maximum_size(720);
    clamp.set_child(Some(&vbox));
    clamp
}
