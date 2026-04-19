// SPDX-License-Identifier: GPL-3.0-or-later
use std::cell::OnceCell;
use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk4::gio;

use gtk_cross_platform::config;
use gtk_cross_platform::infrastructure::greeting::greeting_service::GreetingService;
use gtk_cross_platform::infrastructure::logging::app_logger::AppLogger;
use gtk_cross_platform::ports::i_greeting_service::IGreetingService;

use crate::window::main_window::MainWindow;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct GtkCrossPlatformApp {
        logger: OnceCell<AppLogger>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GtkCrossPlatformApp {
        const NAME: &'static str = "GtkCrossPlatformApp";
        type Type = super::GtkCrossPlatformApp;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for GtkCrossPlatformApp {}

    impl ApplicationImpl for GtkCrossPlatformApp {
        fn startup(&self) {
            self.parent_startup();

            let logger = self.logger.get_or_init(|| AppLogger::new(config::APP_ID));

            if config::PROFILE == "development" {
                // Safety: only sets an env var for debug output, not security-sensitive.
                unsafe { std::env::set_var("G_MESSAGES_DEBUG", "all") };
                logger.debug("Development profile active — G_MESSAGES_DEBUG=all");
            }

            if let Some(display) = gtk4::gdk::Display::default() {
                let icon_theme = gtk4::IconTheme::for_display(&display);
                icon_theme.add_search_path(format!("{}/icons", config::SOURCE_DATADIR));
            }
        }

        fn activate(&self) {
            let logger = self.logger.get_or_init(|| AppLogger::new(config::APP_ID));
            logger.info("Application activating");

            // Composition root: wire up dependencies here, never inside widgets.
            let greeting_service: Rc<dyn IGreetingService> = Rc::new(GreetingService::new());
            let win = MainWindow::new(&*self.obj(), greeting_service);
            win.present();
        }
    }

    impl GtkApplicationImpl for GtkCrossPlatformApp {}
    impl AdwApplicationImpl for GtkCrossPlatformApp {}
}

glib::wrapper! {
    pub struct GtkCrossPlatformApp(ObjectSubclass<imp::GtkCrossPlatformApp>)
        @extends adw::Application, gtk4::Application, gio::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl GtkCrossPlatformApp {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("application-id", config::APP_ID)
            .property("flags", gio::ApplicationFlags::empty())
            .build()
    }
}
