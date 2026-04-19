// SPDX-License-Identifier: GPL-3.0-or-later
pub struct AppLogger {
    domain: String,
}

impl AppLogger {
    pub fn new(domain: &str) -> Self {
        Self {
            domain: domain.to_string(),
        }
    }

    pub fn debug(&self, message: &str) {
        glib::g_debug!(self.domain.as_str(), "{}", message);
    }

    pub fn info(&self, message: &str) {
        glib::g_info!(self.domain.as_str(), "{}", message);
    }

    pub fn warning(&self, message: &str) {
        glib::g_warning!(self.domain.as_str(), "{}", message);
    }

    pub fn error(&self, message: &str) {
        glib::g_critical!(self.domain.as_str(), "{}", message);
    }
}
