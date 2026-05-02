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

    pub fn subdomain(&self, suffix: &str) -> Self {
        Self {
            domain: format!("{}.{}", self.domain, suffix),
        }
    }

    /// GLib has no level below DEBUG; trace maps to g_debug!.
    pub fn trace(&self, message: &str) {
        glib::g_debug!(self.domain.as_str(), "{}", message);
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

    pub fn critical(&self, message: &str) {
        glib::g_critical!(self.domain.as_str(), "{}", message);
    }

    /// Log `message` at `level` with optional key-value fields.
    ///
    /// Fields are appended to the message as `key=value` pairs in brackets so
    /// they are visible in both journald (structured) and plain-text handlers.
    /// A future migration to `glib::log_structured` for true structured fields
    /// is straightforward — the public signature is already compatible.
    pub fn log_with_fields(&self, level: glib::LogLevel, message: &str, fields: &[(&str, &str)]) {
        let full_msg = Self::format_with_fields(message, fields);
        match level {
            glib::LogLevel::Debug => glib::g_debug!(self.domain.as_str(), "{}", full_msg),
            glib::LogLevel::Info | glib::LogLevel::Message => {
                glib::g_info!(self.domain.as_str(), "{}", full_msg)
            }
            glib::LogLevel::Warning => glib::g_warning!(self.domain.as_str(), "{}", full_msg),
            _ => glib::g_critical!(self.domain.as_str(), "{}", full_msg),
        }
    }

    /// Builds the formatted message string for log_with_fields without emitting it.
    /// Used internally and in tests.
    fn format_with_fields(message: &str, fields: &[(&str, &str)]) -> String {
        if fields.is_empty() {
            message.to_string()
        } else {
            let pairs = fields
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join(" ");
            format!("{message} [{pairs}]")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppLogger;

    #[test]
    fn subdomain_appends_suffix() {
        let log = AppLogger::new("com.example.App");
        let sub = log.subdomain("containers");
        assert_eq!(sub.domain, "com.example.App.containers");
    }

    #[test]
    fn subdomain_chain() {
        let log = AppLogger::new("com.example.App");
        let sub = log.subdomain("view").subdomain("containers");
        assert_eq!(sub.domain, "com.example.App.view.containers");
    }

    #[test]
    fn new_stores_domain() {
        let log = AppLogger::new("com.example.Test");
        assert_eq!(log.domain, "com.example.Test");
    }

    #[test]
    fn format_with_fields_empty_returns_message() {
        let out = AppLogger::format_with_fields("hello", &[]);
        assert_eq!(out, "hello");
    }

    #[test]
    fn format_with_fields_single() {
        let out = AppLogger::format_with_fields("msg", &[("key", "val")]);
        assert_eq!(out, "msg [key=val]");
    }

    #[test]
    fn format_with_fields_multiple() {
        let out = AppLogger::format_with_fields("op", &[("a", "1"), ("b", "2")]);
        assert_eq!(out, "op [a=1 b=2]");
    }
}
