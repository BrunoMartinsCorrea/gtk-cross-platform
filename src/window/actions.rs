// SPDX-License-Identifier: GPL-3.0-or-later

pub mod app {
    pub const QUIT: &str = "app.quit";
    pub const ABOUT: &str = "app.about";
    pub const PREFERENCES: &str = "app.preferences";
}

pub mod win {
    pub const REFRESH: &str = "win.refresh";
    pub const PRUNE_SYSTEM: &str = "win.prune-system";
    #[allow(dead_code)]
    pub const FOCUS_SEARCH: &str = "win.focus-search";
    #[allow(dead_code)]
    pub const CLEAR_SEARCH: &str = "win.clear-search";
    pub const CLOSE: &str = "window.close";
}
