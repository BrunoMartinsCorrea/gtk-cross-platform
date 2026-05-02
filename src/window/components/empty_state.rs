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

    /// Wrap a StatusPage in an AdwClamp to prevent over-stretching on wide monitors.
    pub fn in_clamp(page: adw::StatusPage) -> adw::Clamp {
        let clamp = adw::Clamp::new();
        clamp.set_maximum_size(480);
        clamp.set_child(Some(&page));
        clamp
    }
}
