// SPDX-License-Identifier: GPL-3.0-or-later
use adw::prelude::*;

/// Build an icon-only action button for use as a row suffix.
///
/// Both `set_tooltip_text` and `update_property(Property::Label)` are set from `tooltip`,
/// satisfying WCAG 2.4.6 and the GNOME HIG requirement that icon-only buttons have
/// both a visible tooltip and a screen-reader-accessible label.
pub fn icon_button(icon_name: &str, tooltip: &str) -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.set_icon_name(icon_name);
    btn.set_tooltip_text(Some(tooltip));
    btn.set_valign(gtk4::Align::Center);
    btn.add_css_class("flat");
    btn.add_css_class("action-button");
    btn.update_property(&[gtk4::accessible::Property::Label(tooltip)]);
    btn
}
