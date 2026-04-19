// SPDX-License-Identifier: GPL-3.0-or-later
mod app;
mod window;

use adw::prelude::ApplicationExtManual;
use gettextrs::{bindtextdomain, textdomain, LocaleCategory};
use gtk4::gio;
use gtk_cross_platform::config;

fn main() -> glib::ExitCode {
    gio::resources_register_include!("compiled.gresource")
        .expect("Failed to register resources.");

    gettextrs::setlocale(LocaleCategory::LcAll, "");
    bindtextdomain(config::GETTEXT_PACKAGE, config::LOCALEDIR)
        .expect("Failed to bind text domain");
    textdomain(config::GETTEXT_PACKAGE).expect("Failed to set text domain");

    let app = app::GtkCrossPlatformApp::new();
    app.run()
}
