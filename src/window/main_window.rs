// SPDX-License-Identifier: GPL-3.0-or-later
use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::{gettext, ngettext};
use gtk4::CompositeTemplate;
use gtk4::gio;

use gtk_cross_platform::config;
use gtk_cross_platform::infrastructure::containers::background::spawn_driver_task;
use gtk_cross_platform::infrastructure::containers::dynamic_driver::DynamicDriver;
use gtk_cross_platform::infrastructure::containers::error::ContainerError;
use gtk_cross_platform::infrastructure::containers::factory::{
    ContainerDriverFactory, RuntimeKind,
};
use gtk_cross_platform::ports::use_cases::i_container_use_case::IContainerUseCase;
use gtk_cross_platform::ports::use_cases::i_image_use_case::IImageUseCase;
use gtk_cross_platform::ports::use_cases::i_network_use_case::INetworkUseCase;
use gtk_cross_platform::ports::use_cases::i_volume_use_case::IVolumeUseCase;

use crate::window::components::{confirm_dialog, toast_util::ToastUtil};
use crate::window::views::containers_view::ContainersView;
use crate::window::views::dashboard_view::DashboardView;
use crate::window::views::images_view::ImagesView;
use crate::window::views::networks_view::NetworksView;
use crate::window::views::volumes_view::VolumesView;

mod imp {
    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/example/GtkCrossPlatform/window.ui")]
    pub struct MainWindow {
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub spinner: TemplateChild<gtk4::Spinner>,
        #[template_child]
        pub refresh_button: TemplateChild<gtk4::Button>,
        #[template_child]
        pub header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub runtime_banner: TemplateChild<adw::Banner>,
        #[template_child]
        pub split_view: TemplateChild<adw::NavigationSplitView>,
        #[template_child]
        pub content_page: TemplateChild<adw::NavigationPage>,
        #[template_child]
        pub view_stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        pub detail_stack: TemplateChild<gtk4::Stack>,
        #[template_child]
        pub detail_content: TemplateChild<gtk4::Box>,

        pub network_uc: RefCell<Option<Arc<dyn INetworkUseCase>>>,
        pub dynamic_driver: RefCell<Option<Arc<DynamicDriver>>>,
        pub available_runtimes: RefCell<Vec<RuntimeKind>>,
        pub refresh_timer: Cell<Option<glib::SourceId>>,

        pub dashboard_view: OnceCell<DashboardView>,
        pub containers_view: OnceCell<ContainersView>,
        pub images_view: OnceCell<ImagesView>,
        pub volumes_view: OnceCell<VolumesView>,
        pub networks_view: OnceCell<NetworksView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MainWindow {
        const NAME: &'static str = "GtkCrossPlatformMainWindow";
        type Type = super::MainWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MainWindow {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_icon_name(Some(config::APP_ID));
            self.setup_prune_action();
            self.setup_refresh_action();
            self.setup_search_actions();
            self.setup_undo_remove_actions();
            self.setup_runtime_banner();
        }
    }

    impl WidgetImpl for MainWindow {}
    impl WindowImpl for MainWindow {}
    impl ApplicationWindowImpl for MainWindow {}
    impl AdwApplicationWindowImpl for MainWindow {}

    impl MainWindow {
        pub fn show_toast(&self, message: &str) {
            ToastUtil::show(&self.toast_overlay, message);
        }

        pub fn show_toast_error(&self, message: &str) {
            ToastUtil::show_error(&self.toast_overlay, message);
        }

        pub fn set_loading(&self, active: bool) {
            self.spinner.set_visible(active);
            self.spinner.set_spinning(active);
            self.refresh_button.set_sensitive(!active);
        }

        fn setup_prune_action(&self) {
            let win = self.obj().clone();
            let action = gio::SimpleAction::new("prune-system", None);
            action.connect_activate(move |_, _| {
                win.imp().confirm_prune();
            });
            self.obj().add_action(&action);
        }

        fn setup_refresh_action(&self) {
            let win = self.obj().clone();
            let action = gio::SimpleAction::new("refresh", None);
            action.connect_activate(move |_, _| {
                win.refresh_all();
            });
            self.obj().add_action(&action);
        }

        fn setup_search_actions(&self) {
            // focus-search: activate search mode on the visible view's search bar.
            let win_weak = self.obj().downgrade();
            let focus_action = gio::SimpleAction::new("focus-search", None);
            focus_action.connect_activate(move |_, _| {
                if let Some(win) = win_weak.upgrade() {
                    win.imp().activate_search(true);
                }
            });
            self.obj().add_action(&focus_action);

            // clear-search: deactivate search mode.
            let win_weak = self.obj().downgrade();
            let clear_action = gio::SimpleAction::new("clear-search", None);
            clear_action.connect_activate(move |_, _| {
                if let Some(win) = win_weak.upgrade() {
                    win.imp().activate_search(false);
                }
            });
            self.obj().add_action(&clear_action);
        }

        fn activate_search(&self, active: bool) {
            let page = self.view_stack.visible_child_name().unwrap_or_default();
            match page.as_str() {
                "containers" => {
                    if let Some(v) = self.containers_view.get() {
                        v.set_search_active(active);
                    }
                }
                "images" => {
                    if let Some(v) = self.images_view.get() {
                        v.set_search_active(active);
                    }
                }
                "volumes" => {
                    if let Some(v) = self.volumes_view.get() {
                        v.set_search_active(active);
                    }
                }
                "networks" => {
                    if let Some(v) = self.networks_view.get() {
                        v.set_search_active(active);
                    }
                }
                _ => {}
            }
        }

        fn setup_undo_remove_actions(&self) {
            // Container and image removal is irreversible — the undo action shows an
            // explanatory toast rather than attempting to re-create the resource.
            for action_name in &["undo-remove-container", "undo-remove-image"] {
                let win_weak = self.obj().downgrade();
                let name = *action_name;
                let action = gio::SimpleAction::new(name, None);
                action.connect_activate(move |_, _| {
                    if let Some(win) = win_weak.upgrade() {
                        win.imp().show_toast(&gettext("Remove cannot be undone"));
                    }
                });
                self.obj().add_action(&action);
            }
        }

        pub fn setup_runtime_banner(&self) {
            let win_weak = self.obj().downgrade();
            self.runtime_banner.connect_button_clicked(move |banner| {
                if let Some(win) = win_weak.upgrade() {
                    banner.set_revealed(false);
                    win.refresh_all();
                }
            });
        }

        fn confirm_prune(&self) {
            let Some(network_uc) = self.network_uc.borrow().clone() else {
                return;
            };
            let win_weak = self.obj().downgrade();
            confirm_dialog::ask(
                self.obj().upcast_ref::<gtk4::Widget>(),
                &gettext("Prune System?"),
                &gettext(
                    "This will remove all stopped containers, dangling images, and unused \
                     networks. This cannot be undone.",
                ),
                &gettext("Prune"),
                move || {
                    let Some(win) = win_weak.upgrade() else {
                        return;
                    };
                    win.imp().set_loading(true);
                    let win_weak2 = win.downgrade();
                    spawn_driver_task(
                        network_uc.clone(),
                        |uc| uc.prune(false),
                        move |result| {
                            let Some(win) = win_weak2.upgrade() else {
                                return;
                            };
                            win.imp().set_loading(false);
                            match result {
                                Ok(report) => {
                                    let nc = report.containers_deleted.len() as u32;
                                    let ni = report.images_deleted.len() as u32;
                                    let containers_str =
                                        ngettext("Pruned 1 container", "Pruned {n} containers", nc)
                                            .replace("{n}", &nc.to_string());
                                    let images_str = ngettext("1 image", "{n} images", ni)
                                        .replace("{n}", &ni.to_string());
                                    win.imp()
                                        .show_toast(&format!("{containers_str}, {images_str}"));
                                    win.refresh_all();
                                }
                                Err(e) => win
                                    .imp()
                                    .show_toast(&format!("{}: {e}", gettext("Prune failed"))),
                            }
                        },
                    );
                },
            );
        }
    }
}

glib::wrapper! {
    pub struct MainWindow(ObjectSubclass<imp::MainWindow>)
        @extends adw::ApplicationWindow, gtk4::ApplicationWindow, gtk4::Window, gtk4::Widget,
        @implements gio::ActionGroup, gio::ActionMap,
                    gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget,
                    gtk4::Native, gtk4::Root, gtk4::ShortcutManager;
}

impl MainWindow {
    pub fn new(
        app: &impl IsA<adw::Application>,
        container_uc: Arc<dyn IContainerUseCase>,
        image_uc: Arc<dyn IImageUseCase>,
        volume_uc: Arc<dyn IVolumeUseCase>,
        network_uc: Arc<dyn INetworkUseCase>,
        dynamic_driver: Option<Arc<DynamicDriver>>,
        available_runtimes: Vec<RuntimeKind>,
    ) -> Self {
        let win: Self = glib::Object::builder().property("application", app).build();
        *win.imp().network_uc.borrow_mut() = Some(network_uc.clone());
        *win.imp().dynamic_driver.borrow_mut() = dynamic_driver;
        *win.imp().available_runtimes.borrow_mut() = available_runtimes;
        win.setup_views(container_uc, image_uc, volume_uc, network_uc);
        win.setup_runtime_switcher();
        win.setup_signals();
        win.setup_auto_refresh();
        // Dashboard is the initial tab; collapse the split view so the sidebar fills 100%.
        win.imp().content_page.set_visible(false);
        win.imp().split_view.set_collapsed(true);
        win.reload_visible_page();
        win
    }

    pub fn set_runtime_banner_visible(&self, visible: bool) {
        self.imp().runtime_banner.set_revealed(visible);
    }

    fn setup_auto_refresh(&self) {
        let schema_src = gio::SettingsSchemaSource::default();
        let interval = if schema_src
            .and_then(|s| s.lookup(config::APP_ID, true))
            .is_some()
        {
            gio::Settings::new(config::APP_ID).int("refresh-interval-seconds") as u32
        } else {
            30
        };

        let win_weak = self.downgrade();
        let source_id =
            glib::timeout_add_seconds_local(interval, move || match win_weak.upgrade() {
                Some(win) => {
                    win.refresh_all();
                    glib::ControlFlow::Continue
                }
                None => glib::ControlFlow::Break,
            });
        self.imp().refresh_timer.set(Some(source_id));
    }

    fn setup_views(
        &self,
        container_uc: Arc<dyn IContainerUseCase>,
        image_uc: Arc<dyn IImageUseCase>,
        volume_uc: Arc<dyn IVolumeUseCase>,
        network_uc: Arc<dyn INetworkUseCase>,
    ) {
        let imp = self.imp();

        let loading_count: Rc<Cell<u32>> = Rc::new(Cell::new(0));

        let on_loading = {
            let lc = loading_count.clone();
            let win_weak = self.downgrade();
            Rc::new(move |active: bool| {
                let count = lc.get();
                lc.set(if active {
                    count + 1
                } else {
                    count.saturating_sub(1)
                });
                if let Some(win) = win_weak.upgrade() {
                    win.imp().set_loading(lc.get() > 0);
                }
            })
        };

        let on_toast = {
            let win_weak = self.downgrade();
            Rc::new(move |msg: &str| {
                if let Some(win) = win_weak.upgrade() {
                    win.imp().show_toast(msg);
                }
            })
        };

        let on_toast_destructive = {
            let win_weak = self.downgrade();
            Rc::new(move |msg: &str, action: &str| {
                if let Some(win) = win_weak.upgrade() {
                    use crate::window::components::toast_util::ToastUtil;
                    ToastUtil::show_destructive(&win.imp().toast_overlay, msg, action);
                }
            })
        };

        let detail_content: gtk4::Box = (*imp.detail_content).clone();
        let detail_stack: gtk4::Stack = (*imp.detail_stack).clone();

        macro_rules! make_callbacks {
            () => {{
                let ol = on_loading.clone();
                let ot = on_toast.clone();
                let otd = on_toast_destructive.clone();
                (
                    move |msg: &str| ot(msg),
                    move |msg: &str, action: &str| otd(msg, action),
                    move |active: bool| ol(active),
                )
            }};
        }

        // ── Home / Dashboard tab (first position) ────────────────────────────
        // Navigation targets may carry a status fragment: "containers:running",
        // "containers:paused", "containers:stopped", "containers:errors".
        // The fragment is forwarded to ContainersView::set_status_filter so
        // users land on a pre-filtered list when clicking dashboard stat cards.
        let view_stack_weak = imp.view_stack.downgrade();
        let win_weak_nav = self.downgrade();
        let on_navigate = Rc::new(move |target: &str| {
            let (tab, filter) = target.split_once(':').unwrap_or((target, ""));
            if let Some(vs) = view_stack_weak.upgrade() {
                vs.set_visible_child_name(tab);
            }
            if !filter.is_empty()
                && tab == "containers"
                && let Some(win) = win_weak_nav.upgrade()
                && let Some(cv) = win.imp().containers_view.get()
            {
                cv.set_status_filter(filter);
            }
        });

        let win_weak_banner = self.downgrade();
        let on_error = move |e: &ContainerError| {
            if matches!(e, ContainerError::RuntimeNotAvailable(_))
                && let Some(win) = win_weak_banner.upgrade()
            {
                win.imp().runtime_banner.set_revealed(true);
            }
        };

        let (ot, _otd, ol) = make_callbacks!();
        let dv = DashboardView::new(
            container_uc.clone(),
            network_uc.clone(),
            on_navigate,
            move |msg: &str| ot(msg),
            move |active: bool| ol(active),
            on_error,
        );
        let page = imp
            .view_stack
            .add_titled(dv.widget(), Some("home"), &gettext("Home"));
        page.set_icon_name(Some("go-home-symbolic"));
        let _ = imp.dashboard_view.set(dv);

        // ── Containers ───────────────────────────────────────────────────────
        let (ot, otd, ol) = make_callbacks!();
        let cv = ContainersView::new(
            Arc::clone(&container_uc),
            detail_content.clone(),
            detail_stack.clone(),
            ot,
            otd,
            ol,
        );
        let page =
            imp.view_stack
                .add_titled(cv.widget(), Some("containers"), &gettext("Containers"));
        page.set_icon_name(Some("system-run-symbolic"));
        let _ = imp.containers_view.set(cv);

        // ── Images ───────────────────────────────────────────────────────────
        let (ot, otd, ol) = make_callbacks!();
        let view_stack_w = imp.view_stack.downgrade();
        let containers_view_ref = imp.containers_view.clone();
        let iv = ImagesView::new(
            image_uc,
            detail_content.clone(),
            detail_stack.clone(),
            ot,
            otd,
            ol,
            move |image_tag| {
                // Switch to containers tab and trigger create wizard pre-filled with image
                if let Some(vs) = view_stack_w.upgrade() {
                    vs.set_visible_child_name("containers");
                }
                if let Some(cv) = containers_view_ref.get() {
                    cv.show_create_wizard_with_image(image_tag);
                }
            },
        );
        let page = imp
            .view_stack
            .add_titled(iv.widget(), Some("images"), &gettext("Images"));
        page.set_icon_name(Some("image-x-generic-symbolic"));
        let _ = imp.images_view.set(iv);

        // ── Volumes ──────────────────────────────────────────────────────────
        let (ot, _otd, ol) = make_callbacks!();
        let vv = VolumesView::new(
            volume_uc,
            detail_content.clone(),
            detail_stack.clone(),
            ot,
            ol,
        );
        let page = imp
            .view_stack
            .add_titled(vv.widget(), Some("volumes"), &gettext("Volumes"));
        page.set_icon_name(Some("drive-harddisk-symbolic"));
        let _ = imp.volumes_view.set(vv);

        // ── Networks ─────────────────────────────────────────────────────────
        let (ot, _otd, ol) = make_callbacks!();
        let nv = NetworksView::new(
            network_uc,
            Arc::clone(&container_uc),
            detail_content.clone(),
            detail_stack.clone(),
            ot,
            ol,
        );
        let page = imp
            .view_stack
            .add_titled(nv.widget(), Some("networks"), &gettext("Networks"));
        page.set_icon_name(Some("network-wired-symbolic"));
        let _ = imp.networks_view.set(nv);
    }

    /// Build runtime toggle buttons in the header bar.
    ///
    /// Hidden entirely when ≤ 1 runtime is available; the user has no choice
    /// to make. When a runtime is selected, the `DynamicDriver` is swapped and
    /// all views are refreshed.
    fn setup_runtime_switcher(&self) {
        let imp = self.imp();
        let available = imp.available_runtimes.borrow().clone();

        // Switcher is only useful when the user has at least two runtimes.
        if available.len() < 2 {
            return;
        }

        let Some(driver) = imp.dynamic_driver.borrow().clone() else {
            return;
        };

        let switcher = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        switcher.add_css_class("linked");
        switcher.set_tooltip_text(Some(&gettext("Switch container runtime")));

        let mut first_btn: Option<gtk4::ToggleButton> = None;
        let mut first_available = true;

        for kind in &available {
            let (label, name) = match kind {
                RuntimeKind::Docker => ("Docker", "docker"),
                RuntimeKind::Podman => ("Podman", "podman"),
                RuntimeKind::Containerd => ("containerd", "containerd"),
            };

            let btn = gtk4::ToggleButton::new();
            btn.set_label(label);
            btn.set_tooltip_text(Some(&format!("{}: {label}", gettext("Switch to runtime"))));
            btn.update_property(&[gtk4::accessible::Property::Label(label)]);

            // Form a radio group: each button after the first joins the first's group.
            if let Some(ref first) = first_btn {
                btn.set_group(Some(first));
            } else {
                first_btn = Some(btn.clone());
            }

            // Activate the first button by default.
            if first_available {
                btn.set_active(true);
                first_available = false;
            }

            // Wire switch: swap driver and reload all views.
            let driver_ref = driver.clone();
            let win_weak = self.downgrade();
            let runtime_name = name.to_owned();
            btn.connect_toggled(move |b| {
                if !b.is_active() {
                    return;
                }
                match ContainerDriverFactory::detect_specific(&runtime_name) {
                    Ok(new_driver) => {
                        driver_ref.swap(new_driver);
                        save_runtime_pref(&runtime_name);
                        if let Some(win) = win_weak.upgrade() {
                            win.refresh_all();
                        }
                    }
                    Err(e) => {
                        if let Some(win) = win_weak.upgrade() {
                            win.imp().show_toast(&format!(
                                "{}: {e}",
                                gettext("Failed to switch runtime")
                            ));
                        }
                    }
                }
            });

            switcher.append(&btn);
        }

        imp.header_bar.pack_end(&switcher);
    }

    fn setup_signals(&self) {
        let imp = self.imp();

        let win_weak = self.downgrade();
        imp.refresh_button.connect_clicked(move |_| {
            if let Some(win) = win_weak.upgrade() {
                win.refresh_all();
            }
        });

        let detail_stack_weak = imp.detail_stack.downgrade();
        let content_page_weak = imp.content_page.downgrade();
        let split_view_weak = imp.split_view.downgrade();
        let win_weak_nav = self.downgrade();
        imp.view_stack
            .connect_notify_local(Some("visible-child"), move |vs, _| {
                let is_home = vs.visible_child_name().as_deref() == Some("home");
                // Hide the detail pane on the dashboard so it fills the full window width.
                if let Some(cp) = content_page_weak.upgrade() {
                    cp.set_visible(!is_home);
                }
                // Collapse the split view on home so the dashboard fills the full window.
                // sidebar-width-fraction alone is not enough because the content pane still
                // gets a minimum allocation even when hidden.
                if let Some(sv) = split_view_weak.upgrade() {
                    sv.set_collapsed(is_home);
                }
                if let Some(ds) = detail_stack_weak.upgrade() {
                    ds.set_visible_child_name("empty");
                }
                if let Some(win) = win_weak_nav.upgrade() {
                    if !is_home && let Some(dv) = win.imp().dashboard_view.get() {
                        dv.stop_auto_refresh();
                    }
                    win.reload_visible_page();
                }
            });

        // Guard: the AdwBreakpoint at ≤ 900sp reverts `collapsed` to false when the
        // window widens. Re-apply collapsed=true whenever that happens on the home tab.
        let view_stack_weak = imp.view_stack.downgrade();
        imp.split_view
            .connect_notify_local(Some("collapsed"), move |sv, _| {
                if !sv.is_collapsed()
                    && view_stack_weak
                        .upgrade()
                        .and_then(|vs| vs.visible_child_name())
                        .as_deref()
                        == Some("home")
                {
                    sv.set_collapsed(true);
                }
            });
    }

    /// Load only the currently visible page if it has not yet been loaded.
    /// Called on startup and on every tab switch.
    fn reload_visible_page(&self) {
        let imp = self.imp();
        let page = imp.view_stack.visible_child_name().unwrap_or_default();
        match page.as_str() {
            "home" => {
                if let Some(v) = imp.dashboard_view.get()
                    && !v.is_loaded()
                {
                    v.reload();
                }
            }
            "containers" => {
                if let Some(v) = imp.containers_view.get()
                    && !v.is_loaded()
                {
                    v.reload();
                }
            }
            "images" => {
                if let Some(v) = imp.images_view.get()
                    && !v.is_loaded()
                {
                    v.reload();
                }
            }
            "volumes" => {
                if let Some(v) = imp.volumes_view.get()
                    && !v.is_loaded()
                {
                    v.reload();
                }
            }
            "networks" => {
                if let Some(v) = imp.networks_view.get()
                    && !v.is_loaded()
                {
                    v.reload();
                }
            }
            _ => {}
        }
    }

    /// Reload all views. Dashboard always reloads; other views only if
    /// they have been visited at least once (to avoid wasting driver calls
    /// on tabs the user has never opened).
    pub fn refresh_all(&self) {
        let imp = self.imp();
        if let Some(v) = imp.dashboard_view.get() {
            v.reload();
        }
        if let Some(v) = imp.containers_view.get()
            && v.is_loaded()
        {
            v.reload();
        }
        if let Some(v) = imp.images_view.get()
            && v.is_loaded()
        {
            v.reload();
        }
        if let Some(v) = imp.volumes_view.get()
            && v.is_loaded()
        {
            v.reload();
        }
        if let Some(v) = imp.networks_view.get()
            && v.is_loaded()
        {
            v.reload();
        }
    }
}

// ── Runtime preference persistence ───────────────────────────────────────────

fn save_runtime_pref(name: &str) {
    let source = gio::SettingsSchemaSource::default();
    if source
        .and_then(|s| s.lookup(config::APP_ID, true))
        .is_some()
    {
        let settings = gio::Settings::new(config::APP_ID);
        let _ = settings.set_string("preferred-runtime", name);
    }
}

/// Fallback runtime preference reader used by app.rs when GSettings is not available.
pub fn load_runtime_pref() -> Option<String> {
    let path = glib::user_config_dir().join(config::APP_ID).join("runtime");
    std::fs::read_to_string(path).ok()
}
