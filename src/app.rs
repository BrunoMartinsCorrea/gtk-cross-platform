// SPDX-License-Identifier: GPL-3.0-or-later
use std::cell::{OnceCell, RefCell};
use std::sync::Arc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk4::gio;

use gtk_cross_platform::config;
use gtk_cross_platform::core::use_cases::container_use_case::ContainerUseCase;
use gtk_cross_platform::core::use_cases::image_use_case::ImageUseCase;
use gtk_cross_platform::core::use_cases::network_use_case::NetworkUseCase;
use gtk_cross_platform::core::use_cases::volume_use_case::VolumeUseCase;
use gtk_cross_platform::infrastructure::containers::background::spawn_driver_task;
use gtk_cross_platform::infrastructure::containers::dynamic_driver::DynamicDriver;
use gtk_cross_platform::infrastructure::containers::factory::ContainerDriverFactory;
use gtk_cross_platform::infrastructure::logging::app_logger::AppLogger;
use gtk_cross_platform::ports::i_container_driver::IContainerDriver;
use gtk_cross_platform::ports::use_cases::i_container_use_case::IContainerUseCase;
use gtk_cross_platform::ports::use_cases::i_image_use_case::IImageUseCase;
use gtk_cross_platform::ports::use_cases::i_network_use_case::INetworkUseCase;
use gtk_cross_platform::ports::use_cases::i_volume_use_case::IVolumeUseCase;

use crate::window::main_window::{MainWindow, load_runtime_pref};

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct GtkCrossPlatformApp {
        logger: OnceCell<AppLogger>,
        driver: RefCell<Option<Arc<dyn IContainerDriver>>>,
        settings: OnceCell<gio::Settings>,
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
                // Icons bundled in the GResource (hicolor/symbolic/*) are always available.
                icon_theme.add_resource_path("/com/example/GtkCrossPlatform");
                // Filesystem fallback for development: icons in data/icons/ are found here.
                icon_theme.add_search_path(format!("{}/icons", config::SOURCE_DATADIR));
            }

            self.load_css();
            self.setup_app_accels();
            self.init_settings();
            self.setup_dark_mode_tracking();
        }

        fn activate(&self) {
            let logger = self.logger.get_or_init(|| AppLogger::new(config::APP_ID));
            logger.info("Application activating");

            self.setup_app_actions();

            // Detect the initial driver, honouring the saved runtime preference.
            let initial_driver = self.resolve_initial_driver(logger);

            match initial_driver {
                Ok(driver) => {
                    // Wrap in DynamicDriver so the runtime switcher can hot-swap it.
                    let dynamic = Arc::new(DynamicDriver::new(driver));
                    let driver_arc: Arc<dyn IContainerDriver> = dynamic.clone();

                    *self.driver.borrow_mut() = Some(driver_arc.clone());

                    let container_uc: Arc<dyn IContainerUseCase> =
                        Arc::new(ContainerUseCase::new(driver_arc.clone()));
                    let image_uc: Arc<dyn IImageUseCase> =
                        Arc::new(ImageUseCase::new(driver_arc.clone()));
                    let volume_uc: Arc<dyn IVolumeUseCase> =
                        Arc::new(VolumeUseCase::new(driver_arc.clone()));
                    let network_uc: Arc<dyn INetworkUseCase> =
                        Arc::new(NetworkUseCase::new(driver_arc));

                    // Detect all available runtimes for the runtime switcher.
                    let available: Vec<_> = ContainerDriverFactory::available_runtimes()
                        .into_iter()
                        .map(|(kind, _version)| kind)
                        .collect();

                    let win = MainWindow::new(
                        &*self.obj(),
                        container_uc,
                        image_uc,
                        volume_uc,
                        network_uc,
                        Some(dynamic),
                        available,
                    );
                    self.bind_window_settings(&win);
                    win.present();
                }
                Err(e) => {
                    logger.warning(&format!("No container runtime: {e}"));
                    self.show_no_runtime_window(&e.to_string());
                }
            }
        }
    }

    impl GtkApplicationImpl for GtkCrossPlatformApp {}
    impl AdwApplicationImpl for GtkCrossPlatformApp {}

    impl GtkCrossPlatformApp {
        /// Try to build the initial driver.
        ///
        /// If a saved runtime preference exists, attempt to use that first and
        /// fall back to auto-detect on failure. If no preference is saved, use
        /// auto-detect directly.
        fn resolve_initial_driver(
            &self,
            logger: &AppLogger,
        ) -> Result<
            std::sync::Arc<dyn IContainerDriver>,
            gtk_cross_platform::infrastructure::containers::error::ContainerError,
        > {
            let saved_pref = self
                .settings
                .get()
                .map(|s| s.string("preferred-runtime").to_string())
                .filter(|s| !s.is_empty())
                .or_else(load_runtime_pref);
            if let Some(saved) = saved_pref {
                match ContainerDriverFactory::detect_specific(saved.trim()) {
                    Ok(driver) => {
                        logger.info(&format!("Using saved runtime: {saved}"));
                        return Ok(driver);
                    }
                    Err(e) => {
                        logger.warning(&format!(
                            "Saved runtime '{saved}' unavailable ({e}), falling back to auto-detect"
                        ));
                    }
                }
            }
            ContainerDriverFactory::detect()
        }

        fn setup_app_accels(&self) {
            use crate::window::actions;
            let app = self.obj();

            // Register quit action here (in startup) so <Primary>q works before
            // activate() runs — on macOS, Cmd+Q can arrive before the window appears.
            let quit_action = gio::SimpleAction::new("quit", None);
            let app_quit = app.clone();
            quit_action.connect_activate(move |_, _| app_quit.quit());
            app.add_action(&quit_action);

            // <Primary> = Ctrl on Linux/Windows; on macOS the Quartz/Homebrew GTK4
            // build maps Command → META_MASK, so <Primary> alone misses Cmd+Q.
            // <Meta>q is registered on macOS as an explicit Cmd+Q fallback.
            #[cfg(target_os = "macos")]
            app.set_accels_for_action(actions::app::QUIT, &["<Primary>q", "<Meta>q"]);
            #[cfg(not(target_os = "macos"))]
            app.set_accels_for_action(actions::app::QUIT, &["<Primary>q"]);

            app.set_accels_for_action(actions::app::ABOUT, &["F1"]);

            #[cfg(target_os = "macos")]
            app.set_accels_for_action(
                actions::app::PREFERENCES,
                &["<Primary>comma", "<Meta>comma"],
            );
            #[cfg(not(target_os = "macos"))]
            app.set_accels_for_action(actions::app::PREFERENCES, &["<Primary>comma"]);

            #[cfg(target_os = "macos")]
            app.set_accels_for_action(actions::win::REFRESH, &["<Primary>r", "<Meta>r", "F5"]);
            #[cfg(not(target_os = "macos"))]
            app.set_accels_for_action(actions::win::REFRESH, &["<Primary>r", "F5"]);

            #[cfg(target_os = "macos")]
            app.set_accels_for_action(
                actions::win::PRUNE_SYSTEM,
                &["<Primary><Shift>p", "<Meta><Shift>p"],
            );
            #[cfg(not(target_os = "macos"))]
            app.set_accels_for_action(actions::win::PRUNE_SYSTEM, &["<Primary><Shift>p"]);

            #[cfg(target_os = "macos")]
            app.set_accels_for_action(actions::win::CLOSE, &["<Primary>w", "<Meta>w"]);
            #[cfg(not(target_os = "macos"))]
            app.set_accels_for_action(actions::win::CLOSE, &["<Primary>w"]);
        }

        fn setup_app_actions(&self) {
            let app = self.obj().clone();

            let about_action = gio::SimpleAction::new("about", None);
            let app2 = app.clone();
            let driver_cell = self.driver.clone();
            about_action.connect_activate(move |_, _| {
                let driver = driver_cell.borrow().clone();
                app2.imp().show_about_window(driver);
            });
            app.add_action(&about_action);

            let prefs_action = gio::SimpleAction::new("preferences", None);
            let app3 = app.clone();
            prefs_action.connect_activate(move |_, _| {
                if let Some(win) = app3.active_window() {
                    let toast_overlay = win
                        .first_child()
                        .and_then(|c| c.downcast::<adw::ToastOverlay>().ok());
                    if let Some(overlay) = toast_overlay {
                        let toast = adw::Toast::new(&gettext("Preferences not yet implemented"));
                        toast.set_timeout(3);
                        overlay.add_toast(toast);
                    }
                }
            });
            app.add_action(&prefs_action);
        }

        fn show_about_window(&self, driver: Option<Arc<dyn IContainerDriver>>) {
            let parent = self.obj().active_window();
            let about = adw::AboutWindow::builder()
                .application_name(gettext("Container Manager"))
                .application_icon(config::APP_ID)
                .version(config::VERSION)
                .comments(gettext(
                    "A native GNOME application for managing Docker, Podman, and containerd \
                     containers.",
                ))
                .developer_name("Container Manager Contributors")
                .website("https://github.com/your-org/gtk-cross-platform")
                .issue_url("https://github.com/your-org/gtk-cross-platform/issues")
                .license_type(gtk4::License::Gpl30)
                .copyright("© 2026 Container Manager Contributors")
                .translator_credits(gettext("translator-credits"))
                .build();

            if let Some(ref p) = parent {
                about.set_transient_for(Some(p));
            }
            about.set_modal(true);

            about.add_link(
                &gettext("Source Code on GitHub"),
                "https://github.com/your-org/gtk-cross-platform",
            );
            about.add_link(
                &gettext("Report an Issue"),
                "https://github.com/your-org/gtk-cross-platform/issues",
            );

            let toolkit_info = format!("GTK {}.{}", gtk4::major_version(), gtk4::minor_version());
            about.add_credit_section(Some(&gettext("Toolkit")), &[&toolkit_info]);

            if let Some(d) = driver {
                let about_weak = about.downgrade();
                spawn_driver_task(
                    d,
                    |driver| driver.version(),
                    move |result| {
                        let version_str = result.unwrap_or_else(|_| gettext("Unknown"));
                        if let Some(about_ref) = about_weak.upgrade() {
                            about_ref
                                .add_credit_section(Some(&gettext("Runtime")), &[&version_str]);
                        }
                    },
                );
            }

            about.present();
        }

        fn load_css(&self) {
            let Some(display) = gtk4::gdk::Display::default() else {
                return;
            };
            let provider = gtk4::CssProvider::new();
            provider.load_from_resource("/com/example/GtkCrossPlatform/style.css");
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );

            let dark_provider = gtk4::CssProvider::new();
            dark_provider.load_from_resource("/com/example/GtkCrossPlatform/style-dark.css");

            let dp = dark_provider.clone();
            adw::StyleManager::default().connect_dark_notify(move |sm| {
                if sm.is_dark() {
                    gtk4::style_context_add_provider_for_display(
                        &gtk4::gdk::Display::default().unwrap(),
                        &dp,
                        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
                    );
                } else {
                    gtk4::style_context_remove_provider_for_display(
                        &gtk4::gdk::Display::default().unwrap(),
                        &dp,
                    );
                }
            });

            if adw::StyleManager::default().is_dark() {
                gtk4::style_context_add_provider_for_display(
                    &display,
                    &dark_provider,
                    gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
                );
            }
        }

        fn init_settings(&self) {
            // Only initialise if the schema is compiled and available.
            let schema_source = gio::SettingsSchemaSource::default();
            if schema_source
                .and_then(|s| s.lookup(config::APP_ID, true))
                .is_some()
            {
                let _ = self.settings.set(gio::Settings::new(config::APP_ID));
            }
        }

        fn bind_window_settings(&self, win: &MainWindow) {
            let Some(settings) = self.settings.get() else {
                return;
            };
            let width = settings.int("window-width");
            let height = settings.int("window-height");
            let maximized = settings.boolean("window-maximized");
            win.set_default_size(width, height);
            if maximized {
                win.maximize();
            }
            settings.bind("window-width", win, "default-width").build();
            settings
                .bind("window-height", win, "default-height")
                .build();
            settings.bind("window-maximized", win, "maximized").build();
            settings
                .bind(
                    "sidebar-width-fraction",
                    &*win.imp().split_view,
                    "sidebar-width-fraction",
                )
                .build();
        }

        fn setup_dark_mode_tracking(&self) {
            let Some(settings) = self.settings.get() else {
                return;
            };
            let style_manager = adw::StyleManager::default();

            // Apply saved color scheme on startup.
            let scheme = color_scheme_from_str(&settings.string("color-scheme"));
            style_manager.set_color_scheme(scheme);

            // Persist to GSettings when the color scheme changes.
            let settings_clone = settings.clone();
            style_manager.connect_color_scheme_notify(move |sm| {
                let key = match sm.color_scheme() {
                    adw::ColorScheme::ForceDark => "force-dark",
                    adw::ColorScheme::ForceLight => "force-light",
                    _ => "default",
                };
                let _ = settings_clone.set_string("color-scheme", key);
            });

            // Respond to GSettings changes (e.g. from dconf-editor or another process).
            let sm = style_manager.clone();
            settings.connect_changed(Some("color-scheme"), move |s, _| {
                let scheme = color_scheme_from_str(&s.string("color-scheme"));
                sm.set_color_scheme(scheme);
            });
        }

        fn show_no_runtime_window(&self, error_msg: &str) {
            let win = adw::ApplicationWindow::new(&*self.obj());
            win.set_title(Some("Container Manager"));
            win.set_default_size(480, 360);

            let status_page = adw::StatusPage::new();
            status_page.set_title("No Container Runtime");
            status_page.set_description(Some(&format!(
                "{}\n\nInstall Docker, Podman, or nerdctl to use this app.",
                error_msg
            )));
            status_page.set_icon_name(Some("dialog-error-symbolic"));

            let toolbar = adw::ToolbarView::new();
            toolbar.add_top_bar(&adw::HeaderBar::new());
            toolbar.set_content(Some(&status_page));

            win.set_content(Some(&toolbar));
            win.present();
        }
    }
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

fn color_scheme_from_str(s: &str) -> adw::ColorScheme {
    match s {
        "force-dark" => adw::ColorScheme::ForceDark,
        "force-light" => adw::ColorScheme::ForceLight,
        _ => adw::ColorScheme::Default,
    }
}
