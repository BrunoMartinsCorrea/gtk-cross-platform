// SPDX-License-Identifier: GPL-3.0-or-later
//
// Widget tests require a display and GTK main-thread init.
// Run explicitly on Linux with:
//   xvfb-run cargo test --test widget_test -- --test-threads=1 --ignored
// macOS: must run with: cargo test --test widget_test -- --test-threads=1 --ignored

use adw::prelude::*;
use std::sync::OnceLock;

fn init_gtk() {
    static INIT: OnceLock<()> = OnceLock::new();
    INIT.get_or_init(|| {
        adw::init().expect("libadwaita init failed");
    });
}

#[test]
#[ignore = "requires display and main-thread; run with --test-threads=1 --ignored"]
fn adw_status_page_title_property() {
    init_gtk();
    let page = adw::StatusPage::new();
    page.set_title("No containers");
    assert_eq!(page.title().as_str(), "No containers");
}

#[test]
#[ignore = "requires display and main-thread; run with --test-threads=1 --ignored"]
fn adw_action_row_title_and_subtitle() {
    init_gtk();
    let row = adw::ActionRow::new();
    row.set_title("my-nginx");
    row.set_subtitle("nginx:latest · abc123def456");
    assert_eq!(row.title().as_str(), "my-nginx");
    assert_eq!(
        row.subtitle().as_deref(),
        Some("nginx:latest · abc123def456")
    );
}

#[test]
#[ignore = "requires display and main-thread; run with --test-threads=1 --ignored"]
fn status_badge_css_class_applied() {
    use gtk_cross_platform::core::domain::container::ContainerStatus;
    init_gtk();

    let label = gtk4::Label::new(Some("●"));
    label.add_css_class("status-badge");

    let status = ContainerStatus::Running;
    label.add_css_class(status.css_class());

    assert!(label.has_css_class("status-badge"));
    assert!(label.has_css_class("running"));
}

#[test]
#[ignore = "requires display and main-thread; run with --test-threads=1 --ignored"]
fn status_badge_stopped_css_class() {
    use gtk_cross_platform::core::domain::container::ContainerStatus;
    init_gtk();

    let label = gtk4::Label::new(Some("●"));
    label.add_css_class("status-badge");
    label.add_css_class(ContainerStatus::Stopped.css_class());

    assert!(label.has_css_class("stopped"));
    assert!(!label.has_css_class("running"));
}

#[test]
#[ignore = "requires display and main-thread; run with --test-threads=1 --ignored"]
fn gtk_list_box_accepts_action_rows() {
    init_gtk();
    let list_box = gtk4::ListBox::new();
    list_box.add_css_class("boxed-list-separate");

    let row = adw::ActionRow::new();
    row.set_title("container-name");
    list_box.append(&row);

    assert!(list_box.has_css_class("boxed-list-separate"));
}
