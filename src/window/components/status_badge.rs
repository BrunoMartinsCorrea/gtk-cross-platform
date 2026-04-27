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

#[allow(dead_code)]
pub fn update(badge: &gtk4::Label, status: &ContainerStatus) {
    badge.set_text(status.label());
    for cls in &["success", "warning", "dim-label", "error", "accent"] {
        badge.remove_css_class(cls);
    }
    badge.add_css_class(status.css_class());
    badge.set_tooltip_text(Some(status.label()));
}

#[cfg(test)]
mod tests {
    use gtk_cross_platform::core::domain::container::ContainerStatus;

    #[test]
    fn css_class_matches_domain() {
        assert_eq!(ContainerStatus::Running.css_class(), "success");
        assert_eq!(ContainerStatus::Stopped.css_class(), "dim-label");
        assert_eq!(ContainerStatus::Exited(1).css_class(), "dim-label");
        assert_eq!(ContainerStatus::Dead.css_class(), "error");
        assert_eq!(ContainerStatus::Restarting.css_class(), "accent");
        assert_eq!(ContainerStatus::Paused.css_class(), "warning");
    }

    #[test]
    fn label_is_non_empty_for_all_variants() {
        let statuses = [
            ContainerStatus::Running,
            ContainerStatus::Paused,
            ContainerStatus::Stopped,
            ContainerStatus::Exited(0),
            ContainerStatus::Restarting,
            ContainerStatus::Dead,
            ContainerStatus::Unknown("custom".into()),
        ];
        for s in &statuses {
            assert!(!s.label().is_empty(), "empty label for {s:?}");
        }
    }
}
