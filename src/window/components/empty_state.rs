// SPDX-License-Identifier: GPL-3.0-or-later
use gettextrs::gettext;

pub struct EmptyState;

impl EmptyState {
    /// Empty list — no resources exist yet.
    pub fn no_items(icon: &str, title: &str, body: &str) -> adw::StatusPage {
        let page = adw::StatusPage::new();
        page.set_icon_name(Some(icon));
        page.set_title(&gettext(title));
        page.set_description(Some(&gettext(body)));
        page
    }

    /// No-results — list is non-empty but search filter yields nothing.
    pub fn no_results(query: &str) -> adw::StatusPage {
        let page = adw::StatusPage::new();
        page.set_icon_name(Some("edit-find-symbolic"));
        page.set_title(&gettext("No Results"));
        page.set_description(Some(&format!(
            "{} \u{201c}{}\u{201d}",
            gettext("No matches for"),
            query
        )));
        page
    }

    /// Empty selection — detail pane when nothing is selected.
    #[allow(dead_code)]
    pub fn no_selection(icon: &str, title: &str, body: &str) -> adw::StatusPage {
        Self::no_items(icon, title, body)
    }

    /// Wrap a StatusPage in an AdwClamp to prevent over-stretching on wide monitors.
    pub fn in_clamp(page: adw::StatusPage) -> adw::Clamp {
        let clamp = adw::Clamp::new();
        clamp.set_maximum_size(480);
        clamp.set_child(Some(&page));
        clamp
    }
}
