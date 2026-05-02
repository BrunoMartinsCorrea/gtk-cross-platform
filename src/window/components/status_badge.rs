// SPDX-License-Identifier: GPL-3.0-or-later
use gtk4::prelude::*;

use gtk_cross_platform::core::domain::container::ContainerStatus;

/// Build a colored pill label for container status.
///
/// The label text conveys status without relying solely on color (WCAG 1.4.1).
/// `AccessibleRole::Status` marks this as a live-region so assistive technologies
/// announce state changes.
pub fn new(status: &ContainerStatus) -> gtk4::Label {
    let label = gtk4::Label::builder()
        .label(status.label())
        .accessible_role(gtk4::AccessibleRole::Status)
        .valign(gtk4::Align::Center)
        .tooltip_text(status.label())
        .build();
    label.add_css_class("status-badge");
    label.add_css_class(status.css_class());
    label
}
