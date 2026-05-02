// SPDX-License-Identifier: GPL-3.0-or-later
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;

use adw::prelude::*;
use gettextrs::{gettext, pgettext};
use glib;
use gtk4::gio;

use gtk_cross_platform::core::domain::container::{
    Container, CreateContainerOptions, RestartPolicy, is_secret_env_key,
};
use gtk_cross_platform::infrastructure::containers::background::spawn_driver_task;
use gtk_cross_platform::infrastructure::containers::error::log_container_error;
use gtk_cross_platform::infrastructure::logging::app_logger::AppLogger;
use gtk_cross_platform::ports::use_cases::i_container_use_case::IContainerUseCase;

use crate::window::components::{
    clear_box, confirm_dialog, detail_pane, empty_state::EmptyState, resource_row, status_badge,
};
use crate::window::objects::ContainerObject;
use crate::window::utils::store::find_store_position;

const LOG_DOMAIN: &str = concat!(env!("APP_ID"), ".view.containers");
const MAX_SAMPLES: usize = 60;

type OnToastDestructive = Rc<dyn Fn(&str, &str)>;

// ── Sparkline data ────────────────────────────────────────────────────────────

struct SparklineData {
    cpu: VecDeque<f64>,
    mem: VecDeque<f64>,
    net_rx: VecDeque<f64>,
    net_tx: VecDeque<f64>,
    prev_rx: u64,
    prev_tx: u64,
}

impl SparklineData {
    fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            cpu: VecDeque::with_capacity(MAX_SAMPLES),
            mem: VecDeque::with_capacity(MAX_SAMPLES),
            net_rx: VecDeque::with_capacity(MAX_SAMPLES),
            net_tx: VecDeque::with_capacity(MAX_SAMPLES),
            prev_rx: 0,
            prev_tx: 0,
        }))
    }

    fn push(&mut self, cpu: f64, mem: f64, rx: u64, tx: u64) {
        let rx_rate = rx.saturating_sub(self.prev_rx) as f64 / 1024.0;
        let tx_rate = tx.saturating_sub(self.prev_tx) as f64 / 1024.0;
        self.prev_rx = rx;
        self.prev_tx = tx;
        push_bounded(&mut self.cpu, cpu);
        push_bounded(&mut self.mem, mem);
        push_bounded(&mut self.net_rx, rx_rate);
        push_bounded(&mut self.net_tx, tx_rate);
    }

    fn last_values(&self) -> (f64, f64, f64, f64) {
        (
            self.cpu.back().copied().unwrap_or(0.0),
            self.mem.back().copied().unwrap_or(0.0),
            self.net_rx.back().copied().unwrap_or(0.0),
            self.net_tx.back().copied().unwrap_or(0.0),
        )
    }
}

fn push_bounded(q: &mut VecDeque<f64>, val: f64) {
    if q.len() == MAX_SAMPLES {
        q.pop_front();
    }
    q.push_back(val);
}

// ── Inner ─────────────────────────────────────────────────────────────────────

struct Inner {
    store: gio::ListStore,
    filter: gtk4::CustomFilter,
    filter_model: gtk4::FilterListModel,
    selection: gtk4::SingleSelection,
    list_view: gtk4::ListView,
    list_stack: gtk4::Stack,
    empty_status: adw::StatusPage,
    sidebar_box: gtk4::Box,
    search_bar: gtk4::SearchBar,
    search_entry: gtk4::SearchEntry,
    detail_content: gtk4::Box,
    detail_stack: gtk4::Stack,
    use_case: Arc<dyn IContainerUseCase>,
    /// Full domain containers kept for detail pane lookups
    containers: RefCell<Vec<Container>>,
    stats_source: RefCell<Option<glib::SourceId>>,
    on_toast: Rc<dyn Fn(&str)>,
    on_toast_destructive: OnToastDestructive,
    on_loading: Rc<dyn Fn(bool)>,
    loading: Cell<bool>,
    loaded: Cell<bool>,
    selection_handler: RefCell<Option<glib::SignalHandlerId>>,
    list_cancellable: RefCell<Option<gio::Cancellable>>,
    detail_cancellable: RefCell<Option<gio::Cancellable>>,
    /// Status filter pre-applied when navigating from the dashboard cards.
    /// Values: "running" | "paused" | "stopped" | "errors" | "" (no filter).
    status_filter: Rc<RefCell<Option<String>>>,
}

// ── ContainersView ────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ContainersView(Rc<Inner>);

impl ContainersView {
    pub fn new(
        use_case: Arc<dyn IContainerUseCase>,
        detail_content: gtk4::Box,
        detail_stack: gtk4::Stack,
        on_toast: impl Fn(&str) + 'static,
        on_toast_destructive: impl Fn(&str, &str) + 'static,
        on_loading: impl Fn(bool) + 'static,
    ) -> Self {
        let store = gio::ListStore::new::<ContainerObject>();

        let search_entry = gtk4::SearchEntry::new();
        search_entry.set_hexpand(true);
        search_entry.set_placeholder_text(Some(&gettext("Search containers…")));

        let search_bar = gtk4::SearchBar::new();
        search_bar.set_search_mode(false);
        search_bar.set_show_close_button(true);
        search_bar.set_child(Some(&search_entry));

        // Filter: text search (name, image, short_id, compose_project) + optional status filter.
        let status_filter: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
        let se_weak = search_entry.downgrade();
        let sf_clone = status_filter.clone();
        let filter = gtk4::CustomFilter::new(move |obj| {
            let c = obj.downcast_ref::<ContainerObject>().unwrap();

            // Status filter applied by dashboard card navigation.
            if let Some(ref sf) = *sf_clone.borrow() {
                let css = c.status_css();
                let matches = match sf.as_str() {
                    "running" => css == "success",
                    "paused" => css == "warning",
                    "stopped" => css == "dim-label",
                    "errors" => css == "error",
                    _ => true,
                };
                if !matches {
                    return false;
                }
            }

            let Some(entry) = se_weak.upgrade() else {
                return true;
            };
            let query = entry.text();
            if query.is_empty() {
                return true;
            }
            let q = query.to_ascii_lowercase();
            c.name().to_ascii_lowercase().contains(&q)
                || c.image().to_ascii_lowercase().contains(&q)
                || c.short_id().to_ascii_lowercase().contains(&q)
                || c.compose_project().to_ascii_lowercase().contains(&q)
        });

        let filter_model = gtk4::FilterListModel::new(Some(store.clone()), Some(filter.clone()));

        // Phase 5: sort by compose_project (non-empty first) → name
        let item_sorter = gtk4::CustomSorter::new(|a, b| {
            let a = a.downcast_ref::<ContainerObject>().unwrap();
            let b = b.downcast_ref::<ContainerObject>().unwrap();
            let pa = a.compose_project();
            let pb = b.compose_project();
            match (pa.is_empty(), pb.is_empty()) {
                (false, true) => gtk4::Ordering::Smaller,
                (true, false) => gtk4::Ordering::Larger,
                _ => pa.cmp(&pb).then(a.name().cmp(&b.name())).into(),
            }
        });

        let sort_model = gtk4::SortListModel::new(Some(filter_model.clone()), Some(item_sorter));

        // Section sorter: same compose_project → same section
        let section_sorter = gtk4::CustomSorter::new(|a, b| {
            let a = a.downcast_ref::<ContainerObject>().unwrap();
            let b = b.downcast_ref::<ContainerObject>().unwrap();
            a.compose_project().cmp(&b.compose_project()).into()
        });
        sort_model.set_section_sorter(Some(&section_sorter));

        let selection = gtk4::SingleSelection::new(Some(sort_model.clone()));
        selection.set_autoselect(false);

        let list_view =
            gtk4::ListView::new(Some(selection.clone()), None::<gtk4::SignalListItemFactory>);
        list_view.add_css_class("boxed-list");
        list_view.set_hexpand(true);
        list_view.set_show_separators(true);

        let add_btn = gtk4::Button::new();
        add_btn.set_icon_name("list-add-symbolic");
        add_btn.set_label(&gettext("New Container"));
        add_btn.add_css_class("pill");
        add_btn.set_halign(gtk4::Align::Center);
        add_btn.set_margin_top(4);
        add_btn.set_margin_bottom(4);
        add_btn.set_tooltip_text(Some(&gettext("Create a new container")));
        add_btn.update_property(&[gtk4::accessible::Property::Label(&gettext(
            "Create a new container",
        ))]);

        let list_wrapper = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        list_wrapper.set_margin_top(4);
        list_wrapper.set_margin_bottom(12);
        list_wrapper.set_margin_start(12);
        list_wrapper.set_margin_end(12);
        list_wrapper.append(&list_view);

        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_hscrollbar_policy(gtk4::PolicyType::Never);
        scroll.set_vexpand(true);
        scroll.set_child(Some(&list_wrapper));

        let empty_status = EmptyState::no_items(
            "application-x-executable-symbolic",
            "No Containers",
            "No containers found.",
        );
        empty_status.set_vexpand(true);

        let list_stack = gtk4::Stack::new();
        list_stack.set_vexpand(true);
        list_stack.add_named(&scroll, Some("list"));
        list_stack.add_named(&empty_status, Some("empty"));

        let sidebar_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        sidebar_box.append(&search_bar);
        sidebar_box.append(&add_btn);
        sidebar_box.append(&list_stack);

        let inner = Rc::new(Inner {
            store,
            filter,
            filter_model,
            selection,
            list_view,
            list_stack,
            empty_status,
            sidebar_box,
            search_bar: search_bar.clone(),
            search_entry,
            detail_content,
            detail_stack,
            use_case,
            containers: RefCell::new(Vec::new()),
            stats_source: RefCell::new(None),
            on_toast: Rc::new(on_toast),
            on_toast_destructive: Rc::new(on_toast_destructive),
            on_loading: Rc::new(on_loading),
            loading: Cell::new(false),
            loaded: Cell::new(false),
            selection_handler: RefCell::new(None),
            list_cancellable: RefCell::new(None),
            detail_cancellable: RefCell::new(None),
            status_filter,
        });

        let view = Self(inner);
        view.wire_signals(&search_bar, &add_btn);
        view
    }

    pub fn widget(&self) -> &gtk4::Box {
        &self.0.sidebar_box
    }

    pub fn set_search_active(&self, active: bool) {
        self.0.search_bar.set_search_mode(active);
        if active {
            self.0.search_entry.grab_focus();
        }
    }

    /// Pre-apply a status filter from dashboard card navigation.
    /// Pass an empty string to clear the filter.
    pub fn set_status_filter(&self, status: &str) {
        let new_filter = if status.is_empty() {
            None
        } else {
            Some(status.to_string())
        };
        *self.0.status_filter.borrow_mut() = new_filter;
        self.0.filter.changed(gtk4::FilterChange::Different);
    }

    pub fn reload(&self) {
        if self.0.loading.get() {
            return;
        }
        reload_impl(self.0.clone(), None);
    }

    pub fn is_loaded(&self) -> bool {
        self.0.loaded.get()
    }

    pub fn clear_detail(&self) {
        stop_stats_poller(&self.0);
        self.0.detail_stack.set_visible_child_name("empty");
    }

    pub fn show_create_wizard_with_image(&self, image_tag: &str) {
        show_create_dialog_prefilled(&self.0.sidebar_box, self.0.clone(), image_tag);
    }

    fn wire_signals(&self, search_bar: &gtk4::SearchBar, add_btn: &gtk4::Button) {
        let inner_weak = Rc::downgrade(&self.0);

        // ── Factory ──────────────────────────────────────────────────────────
        let factory = gtk4::SignalListItemFactory::new();

        {
            let iw = inner_weak.clone();
            factory.connect_setup(move |_, obj| {
                let item = obj.downcast_ref::<gtk4::ListItem>().unwrap();

                let row = adw::ActionRow::new();
                row.set_activatable(true);
                row.set_focusable(true);

                let badge = gtk4::Label::builder()
                    .accessible_role(gtk4::AccessibleRole::Status)
                    .valign(gtk4::Align::Center)
                    .build();
                badge.add_css_class("status-badge");
                badge.add_css_class("dim-label");
                row.add_prefix(&badge);

                // Start/stop toggle button
                let toggle_btn = gtk4::Button::new();
                toggle_btn.set_icon_name("media-playback-start-symbolic");
                toggle_btn.set_direction(gtk4::TextDirection::Ltr);
                toggle_btn.add_css_class("flat");
                toggle_btn.set_valign(gtk4::Align::Center);
                toggle_btn.set_tooltip_text(Some(&pgettext("container action", "Start container")));
                toggle_btn.update_property(&[gtk4::accessible::Property::Label(&pgettext(
                    "container action",
                    "Start container",
                ))]);
                row.add_suffix(&toggle_btn);

                // Pause/unpause button (visibility driven by Phase 4 binding)
                let pause_btn = gtk4::Button::new();
                pause_btn.set_icon_name("media-playback-pause-symbolic");
                pause_btn.add_css_class("flat");
                pause_btn.set_valign(gtk4::Align::Center);
                pause_btn.set_tooltip_text(Some(&pgettext("container action", "Pause container")));
                pause_btn.update_property(&[gtk4::accessible::Property::Label(&pgettext(
                    "container action",
                    "Pause container",
                ))]);
                pause_btn.set_visible(false);
                row.add_suffix(&pause_btn);

                // Remove button
                let remove_btn = resource_row::icon_button(
                    "user-trash-symbolic",
                    &pgettext("container action", "Remove container"),
                );
                row.add_suffix(&remove_btn);

                // ── Wire toggle click ─────────────────────────────────────────
                {
                    let item_weak = item.downgrade();
                    let iw_t = iw.clone();
                    toggle_btn.connect_clicked(move |_| {
                        let Some(item) = item_weak.upgrade() else {
                            return;
                        };
                        let Some(inner) = iw_t.upgrade() else { return };
                        let Some(c_obj) = item.item().and_downcast::<ContainerObject>() else {
                            return;
                        };
                        let id = c_obj.id();
                        let is_running = c_obj.status() == "Running";
                        let uc = inner.use_case.clone();
                        let cb = inner.clone();
                        if is_running {
                            AppLogger::new(LOG_DOMAIN).debug(&format!("Stopping {id}"));
                            spawn_driver_task(
                                uc,
                                move |uc| uc.stop(&id, Some(10)),
                                move |r| match r {
                                    Ok(()) => {
                                        (cb.on_toast)(&gettext("Container stopped"));
                                        reload_impl(cb.clone(), None);
                                    }
                                    Err(ref e) => {
                                        log_container_error(&AppLogger::new(LOG_DOMAIN), e);
                                        (cb.on_toast)(&format!("{}: {e}", gettext("Stop failed")));
                                    }
                                },
                            );
                        } else {
                            AppLogger::new(LOG_DOMAIN).debug(&format!("Starting {id}"));
                            spawn_driver_task(
                                uc,
                                move |uc| uc.start(&id),
                                move |r| match r {
                                    Ok(()) => {
                                        (cb.on_toast)(&gettext("Container started"));
                                        reload_impl(cb.clone(), None);
                                    }
                                    Err(ref e) => {
                                        log_container_error(&AppLogger::new(LOG_DOMAIN), e);
                                        (cb.on_toast)(&format!("{}: {e}", gettext("Start failed")));
                                    }
                                },
                            );
                        }
                    });
                }

                // ── Wire pause/unpause click ───────────────────────────────────
                {
                    let item_weak = item.downgrade();
                    let iw_p = iw.clone();
                    pause_btn.connect_clicked(move |_| {
                        let Some(item) = item_weak.upgrade() else {
                            return;
                        };
                        let Some(inner) = iw_p.upgrade() else { return };
                        let Some(c_obj) = item.item().and_downcast::<ContainerObject>() else {
                            return;
                        };
                        let id = c_obj.id();
                        let is_paused = c_obj.status() == "Paused";
                        let uc = inner.use_case.clone();
                        let cb = inner.clone();
                        if is_paused {
                            spawn_driver_task(
                                uc,
                                move |uc| uc.unpause(&id),
                                move |r| match r {
                                    Ok(()) => {
                                        (cb.on_toast)(&gettext("Container unpaused"));
                                        reload_impl(cb.clone(), None);
                                    }
                                    Err(ref e) => {
                                        log_container_error(&AppLogger::new(LOG_DOMAIN), e);
                                        (cb.on_toast)(&format!(
                                            "{}: {e}",
                                            gettext("Unpause failed")
                                        ));
                                    }
                                },
                            );
                        } else {
                            spawn_driver_task(
                                uc,
                                move |uc| uc.pause(&id),
                                move |r| match r {
                                    Ok(()) => {
                                        (cb.on_toast)(&gettext("Container paused"));
                                        reload_impl(cb.clone(), None);
                                    }
                                    Err(ref e) => {
                                        log_container_error(&AppLogger::new(LOG_DOMAIN), e);
                                        (cb.on_toast)(&format!("{}: {e}", gettext("Pause failed")));
                                    }
                                },
                            );
                        }
                    });
                }

                // ── Wire remove click ─────────────────────────────────────────
                {
                    let item_weak = item.downgrade();
                    let iw_r = iw.clone();
                    remove_btn.connect_clicked(move |btn| {
                        let Some(item) = item_weak.upgrade() else {
                            return;
                        };
                        let Some(inner) = iw_r.upgrade() else { return };
                        let Some(c_obj) = item.item().and_downcast::<ContainerObject>() else {
                            return;
                        };
                        let id = c_obj.id();
                        let name = c_obj.name();
                        let idx = find_store_position::<ContainerObject, _>(&inner.store, |o| {
                            o.id() == id
                        });
                        let body = gettext("Remove container \"{name}\"? This cannot be undone.")
                            .replace("{name}", &name);
                        let inner2 = inner.clone();
                        confirm_dialog::ask(
                            btn,
                            &gettext("Remove Container?"),
                            &body,
                            &pgettext("container action", "Remove"),
                            move || {
                                let uc = inner2.use_case.clone();
                                let id2 = id.clone();
                                let cb = inner2.clone();
                                spawn_driver_task(
                                    uc,
                                    move |uc| uc.remove(&id2, false, false),
                                    move |r| match r {
                                        Ok(()) => {
                                            (cb.on_toast_destructive)(
                                                &gettext("Container removed"),
                                                "win.undo-remove-container",
                                            );
                                            reload_impl(cb.clone(), idx);
                                        }
                                        Err(ref e) => {
                                            log_container_error(&AppLogger::new(LOG_DOMAIN), e);
                                            (cb.on_toast)(&format!(
                                                "{}: {e}",
                                                gettext("Remove failed")
                                            ));
                                        }
                                    },
                                );
                            },
                        );
                    });
                }

                // GObject carries no typed fields; set_data is the GTK4/Rust idiom for
                // passing widget refs from connect_setup into connect_bind closures.
                unsafe {
                    item.set_data("badge", badge);
                    item.set_data("toggle_btn", toggle_btn);
                    item.set_data("pause_btn", pause_btn);
                }

                item.set_child(Some(&row));
            });
        }

        // Phase 4: update badge + bind button states, store bindings for cleanup
        factory.connect_bind(|_, obj| {
            let item = obj.downcast_ref::<gtk4::ListItem>().unwrap();
            let row = item.child().and_downcast::<adw::ActionRow>().unwrap();
            let c_obj = item.item().and_downcast::<ContainerObject>().unwrap();

            row.set_title(&c_obj.name());
            row.set_subtitle(&c_obj.image());

            // Update badge imperatively (two properties: text + css class)
            let badge = unsafe {
                item.data::<gtk4::Label>("badge")
                    .map(|p| p.as_ref().clone())
            };
            if let Some(badge) = badge {
                badge.set_text(&c_obj.status());
                for cls in ["success", "warning", "dim-label", "error", "accent"] {
                    badge.remove_css_class(cls);
                }
                badge.add_css_class(&c_obj.status_css());
                badge.set_tooltip_text(Some(&c_obj.status()));
            }

            let toggle_btn = unsafe {
                item.data::<gtk4::Button>("toggle_btn")
                    .map(|p| p.as_ref().clone())
            };
            let pause_btn = unsafe {
                item.data::<gtk4::Button>("pause_btn")
                    .map(|p| p.as_ref().clone())
            };

            let mut bindings: Vec<glib::Binding> = Vec::new();

            if let Some(btn) = toggle_btn {
                // Bind toggle button icon-name to status
                let b = c_obj
                    .bind_property("status", &btn, "icon-name")
                    .transform_to(|_, v: &glib::Value| {
                        let status: String = v.get().unwrap_or_default();
                        let icon = if status == "Running" {
                            "media-playback-stop-symbolic"
                        } else {
                            "media-playback-start-symbolic"
                        };
                        Some(icon.to_value())
                    })
                    .sync_create()
                    .build();
                bindings.push(b);
            }

            if let Some(btn) = pause_btn {
                // Bind pause button visibility to status (visible when Running or Paused)
                let b1 = c_obj
                    .bind_property("status", &btn, "visible")
                    .transform_to(|_, v: &glib::Value| {
                        let status: String = v.get().unwrap_or_default();
                        Some((status == "Running" || status == "Paused").to_value())
                    })
                    .sync_create()
                    .build();
                // Bind pause button icon (pause when running, play when paused)
                let b2 = c_obj
                    .bind_property("status", &btn, "icon-name")
                    .transform_to(|_, v: &glib::Value| {
                        let status: String = v.get().unwrap_or_default();
                        let icon = if status == "Paused" {
                            "media-playback-start-symbolic"
                        } else {
                            "media-playback-pause-symbolic"
                        };
                        Some(icon.to_value())
                    })
                    .sync_create()
                    .build();
                bindings.push(b1);
                bindings.push(b2);
            }

            if !bindings.is_empty() {
                unsafe { item.set_data("bindings", bindings) };
            }
        });

        factory.connect_unbind(|_, obj| {
            let item = obj.downcast_ref::<gtk4::ListItem>().unwrap();
            unsafe { item.steal_data::<Vec<glib::Binding>>("bindings") };
        });

        self.0.list_view.set_factory(Some(&factory));

        // Phase 5: section header factory for compose-project grouping.
        // Each compose group header shows "project — X/Y Running" and exposes
        // Start All / Stop All buttons for bulk lifecycle management.
        let header_factory = gtk4::SignalListItemFactory::new();

        {
            let iw = inner_weak.clone();
            header_factory.connect_setup(move |_, obj| {
                let header = obj.downcast_ref::<gtk4::ListHeader>().unwrap();

                let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
                hbox.set_margin_top(4);
                hbox.set_margin_bottom(4);
                hbox.set_margin_start(4);
                hbox.set_margin_end(4);

                let label = gtk4::Label::new(None);
                label.add_css_class("caption-heading");
                label.set_halign(gtk4::Align::Start);
                label.set_hexpand(true);
                hbox.append(&label);

                let start_btn = gtk4::Button::new();
                start_btn.set_icon_name("media-playback-start-symbolic");
                start_btn.add_css_class("flat");
                start_btn.set_valign(gtk4::Align::Center);
                start_btn.set_tooltip_text(Some(&pgettext(
                    "compose action",
                    "Start all containers in group",
                )));
                start_btn.update_property(&[gtk4::accessible::Property::Label(&pgettext(
                    "compose action",
                    "Start all containers in group",
                ))]);
                hbox.append(&start_btn);

                let stop_btn = gtk4::Button::new();
                stop_btn.set_icon_name("media-playback-stop-symbolic");
                stop_btn.add_css_class("flat");
                stop_btn.set_valign(gtk4::Align::Center);
                stop_btn.set_tooltip_text(Some(&pgettext(
                    "compose action",
                    "Stop all containers in group",
                )));
                stop_btn.update_property(&[gtk4::accessible::Property::Label(&pgettext(
                    "compose action",
                    "Stop all containers in group",
                ))]);
                hbox.append(&stop_btn);

                header.set_child(Some(&hbox));

                // Start All — collect IDs of non-running containers and call start_all.
                {
                    let hdr_weak = header.downgrade();
                    let iw_start = iw.clone();
                    start_btn.connect_clicked(move |_| {
                        let Some(header) = hdr_weak.upgrade() else {
                            return;
                        };
                        let Some(inner) = iw_start.upgrade() else {
                            return;
                        };
                        let n = header.n_items();
                        let pos = header.start();
                        let ids: Vec<String> = (pos..pos + n)
                            .filter_map(|i| {
                                inner
                                    .selection
                                    .item(i)
                                    .and_downcast::<ContainerObject>()
                                    .filter(|c| c.status() != "Running")
                                    .map(|c| c.id())
                            })
                            .collect();
                        if ids.is_empty() {
                            return;
                        }
                        let uc = inner.use_case.clone();
                        let cb = inner.clone();
                        spawn_driver_task(
                            uc,
                            move |uc| {
                                let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
                                uc.start_all(&refs)
                            },
                            move |r| match r {
                                Ok(_) => {
                                    (cb.on_toast)(&gettext("Compose group started"));
                                    reload_impl(cb.clone(), None);
                                }
                                Err(ref e) => {
                                    log_container_error(&AppLogger::new(LOG_DOMAIN), e);
                                    (cb.on_toast)(&format!("{}: {e}", gettext("Start all failed")));
                                }
                            },
                        );
                    });
                }

                // Stop All — collect IDs of running containers and call stop_all.
                {
                    let hdr_weak = header.downgrade();
                    let iw_stop = iw.clone();
                    stop_btn.connect_clicked(move |_| {
                        let Some(header) = hdr_weak.upgrade() else {
                            return;
                        };
                        let Some(inner) = iw_stop.upgrade() else {
                            return;
                        };
                        let n = header.n_items();
                        let pos = header.start();
                        let ids: Vec<String> = (pos..pos + n)
                            .filter_map(|i| {
                                inner
                                    .selection
                                    .item(i)
                                    .and_downcast::<ContainerObject>()
                                    .filter(|c| c.status() == "Running")
                                    .map(|c| c.id())
                            })
                            .collect();
                        if ids.is_empty() {
                            return;
                        }
                        let uc = inner.use_case.clone();
                        let cb = inner.clone();
                        spawn_driver_task(
                            uc,
                            move |uc| {
                                let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
                                uc.stop_all(&refs, Some(10))
                            },
                            move |r| match r {
                                Ok(_) => {
                                    (cb.on_toast)(&gettext("Compose group stopped"));
                                    reload_impl(cb.clone(), None);
                                }
                                Err(ref e) => {
                                    log_container_error(&AppLogger::new(LOG_DOMAIN), e);
                                    (cb.on_toast)(&format!("{}: {e}", gettext("Stop all failed")));
                                }
                            },
                        );
                    });
                }
            });
        }

        {
            let iw = inner_weak.clone();
            header_factory.connect_bind(move |_, obj| {
                let header = obj.downcast_ref::<gtk4::ListHeader>().unwrap();
                let Some(hbox) = header.child().and_downcast::<gtk4::Box>() else {
                    return;
                };
                let Some(label) = hbox.first_child().and_downcast::<gtk4::Label>() else {
                    return;
                };
                if let Some(c_obj) = header.item().and_downcast::<ContainerObject>() {
                    let project = c_obj.compose_project();
                    if project.is_empty() {
                        hbox.set_visible(false);
                    } else {
                        hbox.set_visible(true);
                        let n_items = header.n_items();
                        let pos = header.start();
                        let running = if let Some(inner) = iw.upgrade() {
                            (pos..pos + n_items)
                                .filter(|&i| {
                                    inner
                                        .selection
                                        .item(i)
                                        .and_downcast::<ContainerObject>()
                                        .map(|c| c.status() == "Running")
                                        .unwrap_or(false)
                                })
                                .count()
                        } else {
                            0
                        };
                        label.set_text(&format!(
                            "{project} — {running}/{n_items} {}",
                            gettext("Running")
                        ));
                    }
                }
            });
        }

        self.0.list_view.set_header_factory(Some(&header_factory));

        // ── Selection → detail pane ──────────────────────────────────────────
        {
            let iw = inner_weak.clone();
            let handler_id = self
                .0
                .selection
                .connect_selection_changed(move |sel, _, _| {
                    let Some(inner) = iw.upgrade() else { return };
                    if let Some(c_obj) = sel.selected_item().and_downcast::<ContainerObject>() {
                        let id = c_obj.id();
                        let container = inner
                            .containers
                            .borrow()
                            .iter()
                            .find(|c| c.id == id)
                            .cloned();
                        if let Some(c) = container {
                            show_detail(&inner, &c);
                        }
                    } else {
                        stop_stats_poller(&inner);
                        inner.detail_stack.set_visible_child_name("empty");
                    }
                });
            *self.0.selection_handler.borrow_mut() = Some(handler_id);
        }

        // ── Empty state watcher ───────────────────────────────────────────────
        {
            let iw = inner_weak.clone();
            self.0
                .filter_model
                .connect_items_changed(move |model, _, _, _| {
                    let Some(inner) = iw.upgrade() else { return };
                    update_empty_state(&inner, model.n_items());
                });
        }

        // ── Search filter ─────────────────────────────────────────────────────
        {
            let filter_weak = self.0.filter.downgrade();
            self.0.search_entry.connect_search_changed(move |_| {
                if let Some(f) = filter_weak.upgrade() {
                    f.changed(gtk4::FilterChange::Different);
                }
            });
        }

        // Clear on search bar close
        {
            let iw = inner_weak.clone();
            search_bar.connect_notify_local(Some("search-mode-enabled"), move |sb, _| {
                if !sb.is_search_mode() {
                    let Some(inner) = iw.upgrade() else { return };
                    inner.search_entry.set_text("");
                    inner.filter.changed(gtk4::FilterChange::LessStrict);
                }
            });
        }

        // Ctrl+F → open search
        {
            let sb_weak = search_bar.downgrade();
            let key_ctrl = gtk4::EventControllerKey::new();
            key_ctrl.set_propagation_phase(gtk4::PropagationPhase::Capture);
            key_ctrl.connect_key_pressed(move |_, key, _, mods| {
                let primary = mods.contains(gtk4::gdk::ModifierType::CONTROL_MASK)
                    || mods.contains(gtk4::gdk::ModifierType::META_MASK);
                if key == gtk4::gdk::Key::f && primary {
                    if let Some(sb) = sb_weak.upgrade() {
                        sb.set_search_mode(true);
                    }
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            });
            self.0.sidebar_box.add_controller(key_ctrl);
        }

        // Create dialog
        {
            let iw = inner_weak.clone();
            add_btn.connect_clicked(move |btn| {
                let Some(inner) = iw.upgrade() else { return };
                show_create_dialog(btn, inner);
            });
        }
    }
}

// ── Reload ────────────────────────────────────────────────────────────────────

fn reload_impl(inner: Rc<Inner>, focus_after_remove: Option<u32>) {
    let log = AppLogger::new(LOG_DOMAIN);
    log.info("Loading containers list");
    inner.loading.set(true);
    (inner.on_loading)(true);

    if let Some(c) = inner.list_cancellable.borrow_mut().take() {
        c.cancel();
    }
    let cancellable = gio::Cancellable::new();
    *inner.list_cancellable.borrow_mut() = Some(cancellable.clone());

    let use_case = inner.use_case.clone();
    let cb = inner.clone();
    spawn_driver_task(
        use_case,
        |uc| uc.list(true),
        move |result| {
            if cancellable.is_cancelled() {
                return;
            }
            let log = AppLogger::new(LOG_DOMAIN);
            cb.loading.set(false);
            cb.loaded.set(true);
            (cb.on_loading)(false);
            match result {
                Ok(containers) => {
                    log.info(&format!("Loaded {} containers", containers.len()));
                    refresh_store(&cb, containers);
                    if let Some(idx) = focus_after_remove {
                        focus_after_store_update(&cb, idx);
                    }
                }
                Err(ref e) => {
                    log_container_error(&log, e);
                    (cb.on_toast)(&format!("{}: {e}", gettext("Failed to load containers")));
                }
            }
        },
    );
}

// ── Store helpers ─────────────────────────────────────────────────────────────

fn refresh_store(inner: &Rc<Inner>, containers: Vec<Container>) {
    if let Some(id) = inner.selection_handler.borrow().as_ref() {
        inner.selection.block_signal(id);
    }
    inner.store.remove_all();
    for c in &containers {
        inner.store.append(&ContainerObject::from_domain(c));
    }
    if let Some(id) = inner.selection_handler.borrow().as_ref() {
        inner.selection.unblock_signal(id);
    }
    *inner.containers.borrow_mut() = containers;
}

fn focus_after_store_update(inner: &Rc<Inner>, idx: u32) {
    let count = inner.selection.n_items();
    if count == 0 {
        inner.detail_stack.set_visible_child_name("empty");
        return;
    }
    let target = if idx < count { idx } else { count - 1 };
    if let Some(id) = inner.selection_handler.borrow().as_ref() {
        inner.selection.block_signal(id);
    }
    inner.selection.set_selected(target);
    if let Some(id) = inner.selection_handler.borrow().as_ref() {
        inner.selection.unblock_signal(id);
    }
    inner.list_view.grab_focus();
    if let Some(c_obj) = inner
        .selection
        .selected_item()
        .and_downcast::<ContainerObject>()
    {
        let id = c_obj.id();
        let container = inner
            .containers
            .borrow()
            .iter()
            .find(|c| c.id == id)
            .cloned();
        if let Some(c) = container {
            show_detail(inner, &c);
        }
    }
}

fn update_empty_state(inner: &Rc<Inner>, n_items: u32) {
    if n_items == 0 {
        let is_searching = !inner.search_entry.text().is_empty();
        if is_searching {
            inner.empty_status.set_icon_name(Some("edit-find-symbolic"));
            inner.empty_status.set_title(&gettext("No Results"));
            inner.empty_status.set_description(Some(&format!(
                "{} \"{}\"",
                gettext("No containers matched"),
                inner.search_entry.text()
            )));
        } else {
            inner
                .empty_status
                .set_icon_name(Some("application-x-executable-symbolic"));
            inner.empty_status.set_title(&gettext("No Containers"));
            inner
                .empty_status
                .set_description(Some(&gettext("No containers found.")));
        }
        inner.list_stack.set_visible_child_name("empty");
    } else {
        inner.list_stack.set_visible_child_name("list");
    }
}

// ── Detail pane ───────────────────────────────────────────────────────────────

fn show_detail(inner: &Rc<Inner>, c: &Container) {
    stop_stats_poller(inner);
    clear_box(&inner.detail_content);

    let notebook = gtk4::Notebook::new();
    notebook.set_hexpand(true);
    notebook.set_vexpand(true);

    let info_tab = build_info_tab(inner, c);
    notebook.append_page(&info_tab, Some(&gtk4::Label::new(Some(&gettext("Info")))));

    let (stats_tab, stats_handle) = build_stats_tab(c.status.is_running());
    notebook.append_page(&stats_tab, Some(&gtk4::Label::new(Some(&gettext("Stats")))));

    let inspect_tab = build_inspect_tab(inner, c);
    notebook.append_page(
        &inspect_tab,
        Some(&gtk4::Label::new(Some(&gettext("Inspect")))),
    );

    let logs_tab = build_logs_tab(inner, c);
    notebook.append_page(&logs_tab, Some(&gtk4::Label::new(Some(&gettext("Logs")))));

    let terminal_tab = build_terminal_tab(inner, c);
    notebook.append_page(
        &terminal_tab,
        Some(&gtk4::Label::new(Some(&gettext("Terminal")))),
    );

    inner.detail_content.append(&notebook);
    inner.detail_stack.set_visible_child_name("detail");

    if let Some((data, redraw)) = stats_handle {
        start_stats_poller(inner, c.id.clone(), data, redraw);
    }
}

// ── Info tab ──────────────────────────────────────────────────────────────────

fn build_info_tab(inner: &Rc<Inner>, c: &Container) -> gtk4::ScrolledWindow {
    let scroll = gtk4::ScrolledWindow::new();
    scroll.set_hscrollbar_policy(gtk4::PolicyType::Never);

    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    vbox.set_margin_top(12);
    vbox.set_margin_bottom(12);
    vbox.set_margin_start(12);
    vbox.set_margin_end(12);

    let badge = status_badge::new(&c.status);
    badge.set_halign(gtk4::Align::Start);
    vbox.append(&badge);

    let name_group = adw::PreferencesGroup::new();
    let name_entry = adw::EntryRow::new();
    name_entry.set_title(&gettext("Name"));
    name_entry.set_text(&c.name);
    name_group.add(&name_entry);
    vbox.append(&name_group);

    {
        let container_id = c.id.clone();
        let original_name = c.name.clone();
        let entry_w = name_entry.downgrade();
        let cb = inner.clone();

        let do_rename = Rc::new(move || {
            let Some(entry) = entry_w.upgrade() else {
                return;
            };
            let new_name = entry.text().trim().to_string();
            if new_name == original_name || new_name.is_empty() {
                entry.set_text(&original_name);
                return;
            }
            let uc = cb.use_case.clone();
            let id2 = container_id.clone();
            let orig2 = original_name.clone();
            let cb2 = cb.clone();
            let entry_w2 = entry.downgrade();
            spawn_driver_task(
                uc,
                move |uc| uc.rename(&id2, &new_name),
                move |result| match result {
                    Ok(()) => {
                        (cb2.on_toast)(&gettext("Container renamed"));
                        reload_impl(cb2.clone(), None);
                    }
                    Err(ref e) => {
                        if let Some(entry) = entry_w2.upgrade() {
                            entry.set_text(&orig2);
                        }
                        log_container_error(&AppLogger::new(LOG_DOMAIN), e);
                        (cb2.on_toast)(&format!("{}: {e}", gettext("Rename failed")));
                    }
                },
            );
        });

        let dr1 = do_rename.clone();
        name_entry.connect_entry_activated(move |_| dr1());

        let focus_ctrl = gtk4::EventControllerFocus::new();
        let dr2 = do_rename.clone();
        focus_ctrl.connect_leave(move |_| dr2());
        name_entry.add_controller(focus_ctrl);
    }

    let ports_str = if c.ports.is_empty() {
        "—".to_string()
    } else {
        c.ports
            .iter()
            .map(|p| p.display())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let mounts_str = if c.mounts.is_empty() {
        "—".to_string()
    } else {
        c.mounts.join(", ")
    };
    let info_pane = detail_pane::build(&[detail_pane::PropertyGroup {
        title: String::new(),
        rows: vec![
            (gettext("ID"), c.id.clone()),
            (gettext("Image"), c.image.clone()),
            (gettext("Command"), c.command.clone()),
            (gettext("Status"), c.status_text.clone()),
            (gettext("Ports"), ports_str),
            (gettext("Mounts"), mounts_str),
        ],
    }]);
    vbox.append(&info_pane);

    if !c.env.is_empty() {
        let env_group = adw::PreferencesGroup::new();
        env_group.set_title(&gettext("Environment"));
        for env_line in &c.env {
            let (key, value) = env_line.split_once('=').unwrap_or((env_line, ""));
            let is_secret = is_secret_env_key(key);
            let row = adw::ActionRow::new();
            row.set_title(key);
            if is_secret {
                row.set_subtitle("••••••••");
                let actual = value.to_string();
                let visible = Rc::new(Cell::new(false));
                let toggle_btn = gtk4::Button::new();
                toggle_btn.set_icon_name("view-conceal-symbolic");
                toggle_btn.set_tooltip_text(Some(&gettext("Show value")));
                toggle_btn.update_property(&[gtk4::accessible::Property::Label(&gettext(
                    "Toggle secret visibility",
                ))]);
                toggle_btn.add_css_class("flat");
                toggle_btn.set_valign(gtk4::Align::Center);
                let row_weak = row.downgrade();
                let vis = visible.clone();
                toggle_btn.connect_clicked(move |btn| {
                    let now_visible = !vis.get();
                    vis.set(now_visible);
                    if let Some(r) = row_weak.upgrade() {
                        if now_visible {
                            r.set_subtitle(&actual);
                            btn.set_icon_name("view-reveal-symbolic");
                            btn.set_tooltip_text(Some(&gettext("Hide value")));
                            btn.update_property(&[gtk4::accessible::Property::Label(&gettext(
                                "Hide value",
                            ))]);
                        } else {
                            r.set_subtitle("••••••••");
                            btn.set_icon_name("view-conceal-symbolic");
                            btn.set_tooltip_text(Some(&gettext("Show value")));
                            btn.update_property(&[gtk4::accessible::Property::Label(&gettext(
                                "Show value",
                            ))]);
                        }
                    }
                });
                row.add_suffix(&toggle_btn);
            } else {
                row.set_subtitle(value);
            }
            env_group.add(&row);
        }
        vbox.append(&env_group);
    }

    let actions_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    actions_box.set_halign(gtk4::Align::Start);
    let restart_btn = gtk4::Button::with_label(&gettext("Restart"));
    restart_btn.add_css_class("pill");
    restart_btn.set_tooltip_text(Some(&gettext("Restart this container")));
    restart_btn.update_property(&[gtk4::accessible::Property::Label(&gettext(
        "Restart this container",
    ))]);
    let id = c.id.clone();
    let cb = inner.clone();
    restart_btn.connect_clicked(move |_| {
        let uc = cb.use_case.clone();
        let id2 = id.clone();
        let cb2 = cb.clone();
        spawn_driver_task(
            uc,
            move |uc| uc.restart(&id2, Some(10)),
            move |r| match r {
                Ok(()) => {
                    (cb2.on_toast)(&gettext("Container restarted"));
                    reload_impl(cb2.clone(), None);
                }
                Err(ref e) => {
                    log_container_error(&AppLogger::new(LOG_DOMAIN), e);
                    (cb2.on_toast)(&format!("{}: {e}", gettext("Restart failed")));
                }
            },
        );
    });
    actions_box.append(&restart_btn);
    vbox.append(&actions_box);

    scroll.set_child(Some(&vbox));
    scroll
}

// ── Stats tab ─────────────────────────────────────────────────────────────────

type StatsHandle = (Rc<RefCell<SparklineData>>, Rc<dyn Fn(f64, f64, f64, f64)>);
type MappingRows = Rc<RefCell<Vec<(gtk4::Box, gtk4::Entry, gtk4::Entry)>>>;

fn build_stats_tab(is_running: bool) -> (gtk4::Box, Option<StatsHandle>) {
    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    vbox.set_margin_top(12);
    vbox.set_margin_bottom(12);
    vbox.set_margin_start(12);
    vbox.set_margin_end(12);

    if !is_running {
        let status = EmptyState::in_clamp(EmptyState::no_items(
            "utilities-system-monitor-symbolic",
            "Stats unavailable",
            "Stats are only available for running containers.",
        ));
        vbox.append(&status);
        return (vbox, None);
    }

    let data = SparklineData::new();

    struct Sparkline {
        area: gtk4::DrawingArea,
        val_lbl: gtk4::Label,
    }

    let sparklines: Vec<(String, f64, Sparkline)> = [
        (gettext("CPU %"), 100.0_f64),
        (gettext("Memory %"), 100.0_f64),
        (gettext("Net In KB/s"), 0.0_f64),
        (gettext("Net Out KB/s"), 0.0_f64),
    ]
    .into_iter()
    .map(|(title, max)| {
        let section = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let title_lbl = gtk4::Label::new(Some(&title));
        title_lbl.set_hexpand(true);
        title_lbl.set_halign(gtk4::Align::Start);
        title_lbl.add_css_class("caption-heading");
        let val_lbl = gtk4::Label::new(Some("—"));
        val_lbl.add_css_class("caption");
        hbox.append(&title_lbl);
        hbox.append(&val_lbl);
        section.append(&hbox);

        let area = gtk4::DrawingArea::new();
        area.set_hexpand(true);
        area.set_content_height(60);
        section.append(&area);
        vbox.append(&section);

        (title, max, Sparkline { area, val_lbl })
    })
    .collect();

    let colors = [
        (0.33_f64, 0.62_f64, 0.95_f64),
        (0.2_f64, 0.72_f64, 0.42_f64),
        (0.93_f64, 0.67_f64, 0.13_f64),
        (0.85_f64, 0.3_f64, 0.3_f64),
    ];
    let selectors: [fn(&SparklineData) -> &VecDeque<f64>; 4] =
        [|d| &d.cpu, |d| &d.mem, |d| &d.net_rx, |d| &d.net_tx];

    for (i, ((_title, max), sparkline)) in
        sparklines.iter().map(|(t, m, s)| ((t, m), s)).enumerate()
    {
        let data_clone = data.clone();
        let color = colors[i];
        let max_val = *max;
        let sel = selectors[i];
        sparkline.area.set_draw_func(move |_, ctx, w, h| {
            let borrowed = data_clone.borrow();
            let samples = sel(&borrowed);
            draw_sparkline(ctx, samples, max_val, w, h, color);
        });
    }

    let area_weaks: Vec<glib::WeakRef<gtk4::DrawingArea>> = sparklines
        .iter()
        .map(|(_, _, s)| s.area.downgrade())
        .collect();
    let lbl_weaks: Vec<glib::WeakRef<gtk4::Label>> = sparklines
        .iter()
        .map(|(_, _, s)| s.val_lbl.downgrade())
        .collect();

    let on_sample: Rc<dyn Fn(f64, f64, f64, f64)> = Rc::new(move |cpu, mem, rx, tx| {
        for w in &area_weaks {
            if let Some(a) = w.upgrade() {
                a.queue_draw();
            }
        }
        let values = [cpu, mem, rx, tx];
        let units = ["%", "%", " KB/s", " KB/s"];
        for (w, (val, unit)) in lbl_weaks.iter().zip(values.iter().zip(units.iter())) {
            if let Some(l) = w.upgrade() {
                l.set_text(&format!("{val:.1}{unit}"));
            }
        }
    });

    (vbox, Some((data, on_sample)))
}

fn draw_sparkline(
    ctx: &gtk4::cairo::Context,
    data: &VecDeque<f64>,
    max_val: f64,
    width: i32,
    height: i32,
    color: (f64, f64, f64),
) {
    let w = width as f64;
    let h = height as f64;
    let _ = ctx.save();

    ctx.set_source_rgb(0.1, 0.1, 0.12);
    ctx.rectangle(0.0, 0.0, w, h);
    let _ = ctx.fill();

    if data.is_empty() {
        let _ = ctx.restore();
        return;
    }

    let scale = {
        let data_max = data.iter().cloned().fold(0.0_f64, f64::max);
        let m = if max_val > 0.0 { max_val } else { data_max };
        if m <= 0.0 { 1.0 } else { m }
    };

    let n = MAX_SAMPLES as f64;
    let step = w / n;
    let offset = (MAX_SAMPLES - data.len()) as f64 * step;

    ctx.new_path();
    let first_y = h - (data[0] / scale).clamp(0.0, 1.0) * (h - 2.0);
    ctx.move_to(offset, h);
    ctx.line_to(offset, first_y);
    for (i, &val) in data.iter().enumerate() {
        let x = offset + i as f64 * step;
        let y = h - (val / scale).clamp(0.0, 1.0) * (h - 2.0);
        ctx.line_to(x, y);
    }
    ctx.line_to(offset + (data.len() - 1) as f64 * step, h);
    ctx.close_path();
    ctx.set_source_rgba(color.0, color.1, color.2, 0.15);
    let _ = ctx.fill();

    ctx.new_path();
    ctx.move_to(offset, first_y);
    for (i, &val) in data.iter().enumerate() {
        let x = offset + i as f64 * step;
        let y = h - (val / scale).clamp(0.0, 1.0) * (h - 2.0);
        ctx.line_to(x, y);
    }
    ctx.set_source_rgb(color.0, color.1, color.2);
    ctx.set_line_width(1.5);
    let _ = ctx.stroke();

    let _ = ctx.restore();
}

fn start_stats_poller(
    inner: &Rc<Inner>,
    container_id: String,
    data: Rc<RefCell<SparklineData>>,
    on_sample: Rc<dyn Fn(f64, f64, f64, f64)>,
) {
    let uc = inner.use_case.clone();
    let source_id = glib::timeout_add_seconds_local(1, move || {
        let uc2 = uc.clone();
        let id2 = container_id.clone();
        let data2 = data.clone();
        let cb2 = on_sample.clone();
        spawn_driver_task(
            uc2,
            move |uc| uc.stats(&id2),
            move |result| {
                if let Ok(stats) = result {
                    let cpu = stats.cpu_percent.clamp(0.0, 100.0);
                    let mem = stats.memory_percent.clamp(0.0, 100.0);
                    data2
                        .borrow_mut()
                        .push(cpu, mem, stats.net_rx_bytes, stats.net_tx_bytes);
                    let (_, _, rx, tx) = data2.borrow().last_values();
                    cb2(cpu, mem, rx, tx);
                }
            },
        );
        glib::ControlFlow::Continue
    });
    *inner.stats_source.borrow_mut() = Some(source_id);
}

fn stop_stats_poller(inner: &Rc<Inner>) {
    if let Some(id) = inner.stats_source.borrow_mut().take() {
        id.remove();
    }
}

// ── Inspect tab ───────────────────────────────────────────────────────────────

fn build_inspect_tab(inner: &Rc<Inner>, c: &Container) -> gtk4::Box {
    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    vbox.set_margin_top(8);
    vbox.set_margin_bottom(8);
    vbox.set_margin_start(8);
    vbox.set_margin_end(8);

    let toolbar = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    toolbar.set_halign(gtk4::Align::End);
    let copy_btn = gtk4::Button::new();
    copy_btn.set_icon_name("edit-copy-symbolic");
    copy_btn.set_tooltip_text(Some(&gettext("Copy JSON")));
    copy_btn.update_property(&[gtk4::accessible::Property::Label(&gettext("Copy JSON"))]);
    copy_btn.add_css_class("flat");
    toolbar.append(&copy_btn);
    vbox.append(&toolbar);

    let scroll = gtk4::ScrolledWindow::new();
    scroll.set_hexpand(true);
    scroll.set_vexpand(true);
    let text_view = gtk4::TextView::new();
    text_view.set_editable(false);
    text_view.set_cursor_visible(false);
    text_view.set_monospace(true);
    text_view.set_wrap_mode(gtk4::WrapMode::None);
    text_view.add_css_class("card");
    scroll.set_child(Some(&text_view));
    vbox.append(&scroll);

    if let Some(c) = inner.detail_cancellable.borrow_mut().take() {
        c.cancel();
    }
    let detail_c = gio::Cancellable::new();
    *inner.detail_cancellable.borrow_mut() = Some(detail_c.clone());

    let uc = inner.use_case.clone();
    let container_id = c.id.clone();
    let tv_weak = text_view.downgrade();
    let copy_weak = copy_btn.downgrade();
    let on_toast = inner.on_toast.clone();
    spawn_driver_task(
        uc,
        move |uc| uc.inspect_json(&container_id),
        move |result| {
            if detail_c.is_cancelled() {
                return;
            }
            match result {
                Ok(json) => {
                    if let Some(tv) = tv_weak.upgrade() {
                        tv.buffer().set_text(&json);
                        apply_json_syntax_tags(&tv.buffer());
                    }
                    if let Some(btn) = copy_weak.upgrade() {
                        let json2 = json.clone();
                        btn.connect_clicked(move |b| {
                            b.clipboard().set_text(&json2);
                        });
                    }
                }
                Err(e) => {
                    on_toast(&format!("{}: {e}", gettext("Failed to inspect container")));
                }
            }
        },
    );

    vbox
}

fn apply_log_level_tags(buffer: &gtk4::TextBuffer) {
    let table = buffer.tag_table();
    let add_tag = |name: &str, color: &str| {
        if table.lookup(name).is_none() {
            let tag = gtk4::TextTag::new(Some(name));
            tag.set_foreground(Some(color));
            table.add(&tag);
        }
    };
    add_tag("log-error", "#f28b82");
    add_tag("log-warn", "#f4c430");
    add_tag("log-info", "#7ec8e3");
    add_tag("log-debug", "#888888");

    let text = buffer
        .text(&buffer.start_iter(), &buffer.end_iter(), false)
        .to_string();
    let mut char_offset: i32 = 0;
    for line in text.lines() {
        let n_chars = line.chars().count() as i32;
        let u = line.to_ascii_uppercase();
        let tag = if u.contains("[ERROR]")
            || u.contains("LEVEL=ERROR")
            || u.contains("\"LEVEL\":\"ERROR\"")
            || u.contains(" ERROR ")
            || u.contains("[FATAL]")
            || u.contains("[CRIT")
        {
            Some("log-error")
        } else if u.contains("[WARN")
            || u.contains("LEVEL=WARN")
            || u.contains("WARNING")
            || u.contains(" WARN ")
        {
            Some("log-warn")
        } else if u.contains("[INFO")
            || u.contains("LEVEL=INFO")
            || u.contains(" INFO ")
            || u.contains(": INFO")
        {
            Some("log-info")
        } else if u.contains("[DEBUG")
            || u.contains("LEVEL=DEBUG")
            || u.contains("[TRACE]")
            || u.contains(" DEBUG ")
        {
            Some("log-debug")
        } else {
            None
        };
        if let Some(t) = tag {
            let si = buffer.iter_at_offset(char_offset);
            let ei = buffer.iter_at_offset(char_offset + n_chars);
            buffer.apply_tag_by_name(t, &si, &ei);
        }
        char_offset += n_chars + 1; // +1 for '\n'
    }
}

fn apply_json_syntax_tags(buffer: &gtk4::TextBuffer) {
    let table = buffer.tag_table();
    let add = |name: &str, color: &str| {
        let tag = gtk4::TextTag::new(Some(name));
        tag.set_foreground(Some(color));
        table.add(&tag);
    };
    add("json-key", "#7ec8e3");
    add("json-string", "#98d282");
    add("json-number", "#c8a9f0");
    add("json-bool", "#f28b82");

    let text = buffer
        .text(&buffer.start_iter(), &buffer.end_iter(), false)
        .to_string();
    let chars: Vec<char> = text.chars().collect();
    let mut pos = 0usize;

    while pos < chars.len() {
        match chars[pos] {
            '"' => {
                let start = pos;
                pos += 1;
                while pos < chars.len() {
                    if chars[pos] == '\\' {
                        pos += 1;
                    } else if chars[pos] == '"' {
                        break;
                    }
                    pos += 1;
                }
                pos += 1;
                let mut la = pos;
                while la < chars.len() && chars[la] == ' ' {
                    la += 1;
                }
                let tag = if la < chars.len() && chars[la] == ':' {
                    "json-key"
                } else {
                    "json-string"
                };
                apply_char_tag(buffer, &text, start, pos, tag);
            }
            't' | 'f' | 'n'
                if {
                    let end = pos
                        + chars[pos..]
                            .iter()
                            .take(5)
                            .take_while(|c| c.is_alphabetic())
                            .count();
                    let word: String = chars[pos..end].iter().collect();
                    matches!(word.as_str(), "true" | "false" | "null")
                } =>
            {
                let start = pos;
                while pos < chars.len() && chars[pos].is_alphabetic() {
                    pos += 1;
                }
                apply_char_tag(buffer, &text, start, pos, "json-bool");
            }
            c if c.is_ascii_digit()
                || (c == '-' && pos + 1 < chars.len() && chars[pos + 1].is_ascii_digit()) =>
            {
                let start = pos;
                if chars[pos] == '-' {
                    pos += 1;
                }
                while pos < chars.len()
                    && (chars[pos].is_ascii_digit() || ".eE+-".contains(chars[pos]))
                {
                    pos += 1;
                }
                apply_char_tag(buffer, &text, start, pos, "json-number");
            }
            _ => {
                pos += 1;
            }
        }
    }
}

fn apply_char_tag(buffer: &gtk4::TextBuffer, text: &str, start_c: usize, end_c: usize, tag: &str) {
    let byte = |c: usize| -> i32 {
        text.char_indices()
            .nth(c)
            .map(|(b, _)| b as i32)
            .unwrap_or(text.len() as i32)
    };
    let si = buffer.iter_at_offset(byte(start_c));
    let ei = buffer.iter_at_offset(byte(end_c));
    buffer.apply_tag_by_name(tag, &si, &ei);
}

// ── Logs tab ──────────────────────────────────────────────────────────────────

fn build_logs_tab(inner: &Rc<Inner>, c: &Container) -> gtk4::Box {
    let is_running = c.status.is_running();

    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    vbox.set_margin_top(8);
    vbox.set_margin_bottom(8);
    vbox.set_margin_start(8);
    vbox.set_margin_end(8);

    let toolbar = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    toolbar.set_halign(gtk4::Align::End);

    let timestamps_btn = gtk4::ToggleButton::new();
    timestamps_btn.set_icon_name("x-office-spreadsheet-symbolic");
    timestamps_btn.set_tooltip_text(Some(&gettext("Show timestamps")));
    timestamps_btn.update_property(&[gtk4::accessible::Property::Label(&gettext(
        "Toggle timestamps",
    ))]);
    timestamps_btn.add_css_class("flat");

    let follow_btn = gtk4::ToggleButton::new();
    follow_btn.set_icon_name("go-bottom-symbolic");
    follow_btn.set_tooltip_text(Some(&gettext("Auto-scroll to newest log line")));
    follow_btn.update_property(&[gtk4::accessible::Property::Label(&gettext(
        "Follow log output",
    ))]);
    follow_btn.add_css_class("flat");
    follow_btn.set_active(is_running);

    let copy_btn = gtk4::Button::new();
    copy_btn.set_icon_name("edit-copy-symbolic");
    copy_btn.set_tooltip_text(Some(&gettext("Copy logs")));
    copy_btn.update_property(&[gtk4::accessible::Property::Label(&gettext("Copy logs"))]);
    copy_btn.add_css_class("flat");

    let clear_btn = gtk4::Button::new();
    clear_btn.set_icon_name("edit-clear-all-symbolic");
    clear_btn.set_tooltip_text(Some(&gettext("Clear log output")));
    clear_btn.update_property(&[gtk4::accessible::Property::Label(&gettext(
        "Clear log output",
    ))]);
    clear_btn.add_css_class("flat");

    let refresh_btn = gtk4::Button::new();
    refresh_btn.set_icon_name("view-refresh-symbolic");
    refresh_btn.set_tooltip_text(Some(&gettext("Refresh logs")));
    refresh_btn.update_property(&[gtk4::accessible::Property::Label(&gettext("Refresh logs"))]);
    refresh_btn.add_css_class("flat");

    toolbar.append(&timestamps_btn);
    toolbar.append(&follow_btn);
    toolbar.append(&copy_btn);
    toolbar.append(&clear_btn);
    toolbar.append(&refresh_btn);
    vbox.append(&toolbar);

    let spinner = gtk4::Spinner::new();
    spinner.set_spinning(true);
    spinner.set_halign(gtk4::Align::Center);
    spinner.set_valign(gtk4::Align::Center);

    let scroll = gtk4::ScrolledWindow::new();
    scroll.set_hexpand(true);
    scroll.set_vexpand(true);
    let text_view = gtk4::TextView::new();
    text_view.set_editable(false);
    text_view.set_cursor_visible(false);
    text_view.set_monospace(true);
    text_view.set_wrap_mode(gtk4::WrapMode::Word);
    text_view.add_css_class("card");
    scroll.set_child(Some(&text_view));

    let content_stack = gtk4::Stack::new();
    content_stack.set_vexpand(true);
    content_stack.add_named(&spinner, Some("loading"));
    content_stack.add_named(&scroll, Some("content"));
    content_stack.set_visible_child_name("loading");
    vbox.append(&content_stack);

    {
        let tv_weak = text_view.downgrade();
        clear_btn.connect_clicked(move |_| {
            if let Some(tv) = tv_weak.upgrade() {
                tv.buffer().set_text("");
            }
        });
    }

    let fetch_logs = {
        let uc = inner.use_case.clone();
        let container_id = c.id.clone();
        let tv_weak = text_view.downgrade();
        let copy_weak = copy_btn.downgrade();
        let stack_weak = content_stack.downgrade();
        let scroll_weak = scroll.downgrade();
        let follow_weak = follow_btn.downgrade();
        let on_toast = inner.on_toast.clone();
        let ts_weak = timestamps_btn.downgrade();
        Rc::new(move || {
            let timestamps = ts_weak.upgrade().map(|b| b.is_active()).unwrap_or(false);
            if let Some(s) = stack_weak.upgrade() {
                s.set_visible_child_name("loading");
            }
            let uc2 = uc.clone();
            let id2 = container_id.clone();
            let tv_w2 = tv_weak.clone();
            let copy_w2 = copy_weak.clone();
            let stack_w2 = stack_weak.clone();
            let scroll_w2 = scroll_weak.clone();
            let follow_w2 = follow_weak.clone();
            let toast2 = on_toast.clone();
            spawn_driver_task(
                uc2,
                move |uc| uc.logs(&id2, Some(200), timestamps),
                move |result| match result {
                    Ok(text) => {
                        if let Some(tv) = tv_w2.upgrade() {
                            tv.buffer().set_text(&text);
                            apply_log_level_tags(&tv.buffer());
                        }
                        if let Some(s) = stack_w2.upgrade() {
                            s.set_visible_child_name("content");
                        }
                        let following = follow_w2.upgrade().map(|b| b.is_active()).unwrap_or(false);
                        if following && let Some(s) = scroll_w2.upgrade() {
                            let adj = s.vadjustment();
                            adj.set_value(adj.upper() - adj.page_size());
                        }
                        if let Some(btn) = copy_w2.upgrade() {
                            let t2 = text.clone();
                            btn.connect_clicked(move |b| {
                                b.clipboard().set_text(&t2);
                            });
                        }
                    }
                    Err(e) => {
                        if let Some(s) = stack_w2.upgrade() {
                            s.set_visible_child_name("content");
                        }
                        toast2(&format!("{}: {e}", gettext("Failed to load logs")));
                    }
                },
            );
        })
    };

    fetch_logs();

    {
        let f = fetch_logs.clone();
        timestamps_btn.connect_toggled(move |_| f());
    }
    {
        let f = fetch_logs.clone();
        refresh_btn.connect_clicked(move |_| f());
    }

    vbox
}

// ── Terminal tab ──────────────────────────────────────────────────────────────

fn build_terminal_tab(inner: &Rc<Inner>, c: &Container) -> gtk4::Box {
    let is_running = c.status.is_running();

    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    vbox.set_margin_top(8);
    vbox.set_margin_bottom(8);
    vbox.set_margin_start(8);
    vbox.set_margin_end(8);

    let scroll = gtk4::ScrolledWindow::new();
    scroll.set_hexpand(true);
    scroll.set_vexpand(true);
    let text_view = gtk4::TextView::new();
    text_view.set_editable(false);
    text_view.set_cursor_visible(false);
    text_view.set_monospace(true);
    text_view.set_wrap_mode(gtk4::WrapMode::Word);
    text_view.add_css_class("card");
    scroll.set_child(Some(&text_view));
    vbox.append(&scroll);

    let input_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    input_row.set_margin_top(4);

    let shells = gtk4::StringList::new(&["sh", "bash"]);
    let shell_drop = gtk4::DropDown::new(Some(shells), gtk4::Expression::NONE);
    shell_drop.set_valign(gtk4::Align::Center);

    let cmd_entry = gtk4::Entry::new();
    cmd_entry.set_hexpand(true);
    cmd_entry.set_placeholder_text(Some(&gettext("Command…")));
    cmd_entry.set_sensitive(is_running);

    let run_btn = gtk4::Button::with_label(&gettext("Run"));
    run_btn.add_css_class("suggested-action");
    run_btn.set_valign(gtk4::Align::Center);
    run_btn.set_tooltip_text(Some(&pgettext(
        "terminal action",
        "Run command in container",
    )));
    run_btn.update_property(&[gtk4::accessible::Property::Label(&pgettext(
        "terminal action",
        "Run command in container",
    ))]);
    run_btn.set_sensitive(is_running);

    input_row.append(&shell_drop);
    input_row.append(&cmd_entry);
    input_row.append(&run_btn);
    vbox.append(&input_row);

    if !is_running {
        let hint = gtk4::Label::new(Some(&gettext(
            "Terminal is only available for running containers.",
        )));
        hint.add_css_class("dim-label");
        hint.add_css_class("caption");
        hint.set_halign(gtk4::Align::Start);
        vbox.append(&hint);
    }

    let history: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let history_cursor: Rc<Cell<usize>> = Rc::new(Cell::new(0));

    let run_command = {
        let uc = inner.use_case.clone();
        let container_id = c.id.clone();
        let tv_weak = text_view.downgrade();
        let scroll_weak = scroll.downgrade();
        let entry_weak = cmd_entry.downgrade();
        let drop_weak = shell_drop.downgrade();
        let run_btn_weak = run_btn.downgrade();
        let on_toast = inner.on_toast.clone();
        let history = history.clone();
        let history_cursor = history_cursor.clone();
        Rc::new(move || {
            let entry = match entry_weak.upgrade() {
                Some(e) => e,
                None => return,
            };
            let cmd_text = entry.text().to_string();
            if cmd_text.trim().is_empty() {
                return;
            }
            if cmd_text.trim() == "exit" {
                if let Some(tv) = tv_weak.upgrade() {
                    let buf = tv.buffer();
                    let mut end = buf.end_iter();
                    buf.insert(&mut end, &format!("{}\n", gettext("Session closed.")));
                }
                entry.set_sensitive(false);
                if let Some(btn) = run_btn_weak.upgrade() {
                    btn.set_sensitive(false);
                }
                entry.set_text("");
                return;
            }
            {
                let mut h = history.borrow_mut();
                h.push(cmd_text.clone());
                if h.len() > 50 {
                    h.remove(0);
                }
                history_cursor.set(h.len());
            }
            let shell = drop_weak
                .upgrade()
                .and_then(|d| {
                    d.selected_item()
                        .and_downcast::<gtk4::StringObject>()
                        .map(|s| s.string().to_string())
                })
                .unwrap_or_else(|| "sh".to_string());
            let uc2 = uc.clone();
            let id2 = container_id.clone();
            let tv_w2 = tv_weak.clone();
            let scroll_w2 = scroll_weak.clone();
            let toast2 = on_toast.clone();
            let prompt_line = format!("{shell} -c {cmd_text}\n");
            spawn_driver_task(
                uc2,
                move |uc| {
                    let cmd_parts: Vec<&str> = vec![&shell, "-c", &cmd_text];
                    uc.exec(&id2, &cmd_parts)
                },
                move |result| {
                    let output = match result {
                        Ok(out) => out,
                        Err(e) => {
                            toast2(&format!("{}: {e}", gettext("Exec failed")));
                            format!("error: {e}\n")
                        }
                    };
                    if let Some(tv) = tv_w2.upgrade() {
                        let buf = tv.buffer();
                        let mut end = buf.end_iter();
                        buf.insert(&mut end, &format!("$ {prompt_line}{output}\n"));
                        if let Some(s) = scroll_w2.upgrade() {
                            let adj = s.vadjustment();
                            adj.set_value(adj.upper() - adj.page_size());
                        }
                    }
                },
            );
            entry.set_text("");
        })
    };

    {
        let rc = run_command.clone();
        run_btn.connect_clicked(move |_| rc());
    }
    {
        let rc = run_command.clone();
        cmd_entry.connect_activate(move |_| rc());
    }
    {
        let key_ctrl = gtk4::EventControllerKey::new();
        let entry_weak = cmd_entry.downgrade();
        let history = history.clone();
        let history_cursor = history_cursor.clone();
        key_ctrl.connect_key_pressed(move |_, key, _, _| {
            let Some(entry) = entry_weak.upgrade() else {
                return glib::Propagation::Proceed;
            };
            let h = history.borrow();
            if h.is_empty() {
                return glib::Propagation::Proceed;
            }
            match key {
                gtk4::gdk::Key::Up => {
                    let cursor = history_cursor.get();
                    if cursor > 0 {
                        let new_cursor = cursor - 1;
                        history_cursor.set(new_cursor);
                        entry.set_text(&h[new_cursor]);
                        entry.set_position(-1);
                    }
                    glib::Propagation::Stop
                }
                gtk4::gdk::Key::Down => {
                    let cursor = history_cursor.get();
                    if cursor + 1 < h.len() {
                        let new_cursor = cursor + 1;
                        history_cursor.set(new_cursor);
                        entry.set_text(&h[new_cursor]);
                        entry.set_position(-1);
                    } else {
                        history_cursor.set(h.len());
                        entry.set_text("");
                    }
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });
        cmd_entry.add_controller(key_ctrl);
    }

    vbox
}

// ── Create container dialog ───────────────────────────────────────────────────

fn show_create_dialog_prefilled(
    widget: &impl gtk4::prelude::IsA<gtk4::Widget>,
    inner: Rc<Inner>,
    prefill_image: &str,
) {
    let root = widget
        .upcast_ref::<gtk4::Widget>()
        .root()
        .and_downcast::<gtk4::Window>();
    show_create_dialog_impl(root.as_ref(), inner, prefill_image);
}

fn show_create_dialog(parent: &gtk4::Button, inner: Rc<Inner>) {
    let root = parent.root().and_downcast::<gtk4::Window>();
    show_create_dialog_impl(root.as_ref(), inner, "");
}

fn show_create_dialog_impl(root: Option<&gtk4::Window>, inner: Rc<Inner>, prefill_image: &str) {
    let dialog = gtk4::Window::new();
    dialog.set_title(Some(&gettext("New Container")));
    dialog.set_transient_for(root);
    dialog.set_modal(true);
    dialog.set_default_size(520, 460);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    let header_bar = adw::HeaderBar::new();
    let cancel_btn = gtk4::Button::with_label(&gettext("Cancel"));
    cancel_btn.add_css_class("flat");
    header_bar.pack_start(&cancel_btn);
    let create_btn = gtk4::Button::with_label(&gettext("Create"));
    create_btn.add_css_class("suggested-action");
    create_btn.set_sensitive(false);
    header_bar.pack_end(&create_btn);
    content.append(&header_bar);

    let steps_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    steps_box.set_margin_top(8);
    steps_box.set_margin_bottom(8);
    steps_box.set_margin_start(12);
    steps_box.set_margin_end(12);
    let step_names = [
        gettext("Image"),
        gettext("Config"),
        gettext("Ports & Volumes"),
        gettext("Environment"),
    ];
    let step_labels: Vec<gtk4::Label> = step_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let l = gtk4::Label::new(Some(name));
            l.set_hexpand(true);
            if i == 0 {
                l.add_css_class("accent");
            } else {
                l.add_css_class("dim-label");
            }
            steps_box.append(&l);
            l
        })
        .collect();
    content.append(&steps_box);

    let stack = gtk4::Stack::new();
    stack.set_vexpand(true);
    stack.set_hexpand(true);
    stack.set_transition_type(gtk4::StackTransitionType::SlideLeftRight);

    let img_group = adw::PreferencesGroup::new();
    img_group.set_title(&gettext("Image"));
    img_group.set_margin_top(12);
    img_group.set_margin_bottom(12);
    img_group.set_margin_start(12);
    img_group.set_margin_end(12);
    let img_row = adw::EntryRow::new();
    img_row.set_title(&gettext("Image reference (e.g. nginx:latest)"));
    if !prefill_image.is_empty() {
        img_row.set_text(prefill_image);
    }
    img_group.add(&img_row);
    stack.add_named(&img_group, Some("step1"));

    // ── Step 2: Configuration ────────────────────────────────────────────────
    let cfg_group = adw::PreferencesGroup::new();
    cfg_group.set_title(&gettext("Configuration"));
    cfg_group.set_margin_top(12);
    cfg_group.set_margin_bottom(12);
    cfg_group.set_margin_start(12);
    cfg_group.set_margin_end(12);

    let name_row = adw::EntryRow::new();
    name_row.set_title(&gettext("Container name (optional)"));
    cfg_group.add(&name_row);

    // Restart policy selector (No / Always / Unless-Stopped / On-Failure)
    let policy_labels = gtk4::StringList::new(&[
        &gettext("No"),
        &gettext("Always"),
        &gettext("Unless-Stopped"),
        &gettext("On-Failure"),
    ]);
    let restart_drop = adw::ComboRow::new();
    restart_drop.set_title(&gettext("Restart Policy"));
    restart_drop.set_model(Some(&policy_labels));
    restart_drop.set_selected(0);
    cfg_group.add(&restart_drop);

    // Network selector (text entry — user can type any network name)
    let network_row = adw::EntryRow::new();
    network_row.set_title(&gettext("Network (optional, e.g. bridge)"));
    cfg_group.add(&network_row);

    stack.add_named(&cfg_group, Some("step2"));

    // ── Step 3: Ports & Volumes ──────────────────────────────────────────────
    type MappingRows = Rc<RefCell<Vec<(gtk4::Box, gtk4::Entry, gtk4::Entry)>>>;
    let port_rows: MappingRows = Rc::new(RefCell::new(Vec::new()));
    let vol_rows: MappingRows = Rc::new(RefCell::new(Vec::new()));

    let step3_scroll = gtk4::ScrolledWindow::new();
    step3_scroll.set_vexpand(true);
    step3_scroll.set_hscrollbar_policy(gtk4::PolicyType::Never);

    let step3_vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    step3_vbox.set_margin_top(12);
    step3_vbox.set_margin_bottom(12);
    step3_vbox.set_margin_start(12);
    step3_vbox.set_margin_end(12);

    // Port mappings header + add button
    let ports_header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    let ports_title_lbl = gtk4::Label::new(Some(&gettext("Port Mappings")));
    ports_title_lbl.add_css_class("heading");
    ports_title_lbl.set_halign(gtk4::Align::Start);
    ports_title_lbl.set_hexpand(true);
    let ports_hint = gtk4::Label::new(Some(&gettext("Host : Container")));
    ports_hint.add_css_class("caption");
    ports_hint.add_css_class("dim-label");
    ports_hint.set_margin_end(8);
    let add_port_btn = gtk4::Button::new();
    add_port_btn.set_icon_name("list-add-symbolic");
    add_port_btn.set_tooltip_text(Some(&gettext("Add port mapping")));
    add_port_btn.add_css_class("flat");
    ports_header_box.append(&ports_title_lbl);
    ports_header_box.append(&ports_hint);
    ports_header_box.append(&add_port_btn);
    step3_vbox.append(&ports_header_box);

    let ports_rows_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    step3_vbox.append(&ports_rows_box);

    // Volume mounts header + add button
    let vols_header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    let vols_title_lbl = gtk4::Label::new(Some(&gettext("Volume Mounts")));
    vols_title_lbl.add_css_class("heading");
    vols_title_lbl.set_halign(gtk4::Align::Start);
    vols_title_lbl.set_hexpand(true);
    let vols_hint = gtk4::Label::new(Some(&gettext("Host path : Container path")));
    vols_hint.add_css_class("caption");
    vols_hint.add_css_class("dim-label");
    vols_hint.set_margin_end(8);
    let add_vol_btn = gtk4::Button::new();
    add_vol_btn.set_icon_name("list-add-symbolic");
    add_vol_btn.set_tooltip_text(Some(&gettext("Add volume mount")));
    add_vol_btn.add_css_class("flat");
    vols_header_box.append(&vols_title_lbl);
    vols_header_box.append(&vols_hint);
    vols_header_box.append(&add_vol_btn);
    step3_vbox.append(&vols_header_box);

    let vols_rows_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    step3_vbox.append(&vols_rows_box);

    step3_scroll.set_child(Some(&step3_vbox));
    stack.add_named(&step3_scroll, Some("step3"));

    // Wire Add Port button
    {
        let rows = port_rows.clone();
        let parent = ports_rows_box.clone();
        add_port_btn.connect_clicked(move |_| {
            add_mapping_row(
                &rows,
                &parent,
                &gettext("Host port"),
                &gettext("Container port"),
                false,
            );
        });
    }

    // Wire Add Volume button
    {
        let rows = vol_rows.clone();
        let parent = vols_rows_box.clone();
        add_vol_btn.connect_clicked(move |_| {
            add_mapping_row(
                &rows,
                &parent,
                &gettext("Host path"),
                &gettext("Container path"),
                true,
            );
        });
    }

    let env_group = adw::PreferencesGroup::new();
    env_group.set_title(&gettext("Environment Variables"));
    env_group.set_description(Some(&gettext("One KEY=VALUE per line")));
    env_group.set_margin_top(12);
    env_group.set_margin_bottom(12);
    env_group.set_margin_start(12);
    env_group.set_margin_end(12);
    let env_scroll = gtk4::ScrolledWindow::new();
    env_scroll.set_vexpand(true);
    let env_text = gtk4::TextView::new();
    env_text.set_monospace(true);
    env_text.set_wrap_mode(gtk4::WrapMode::None);
    env_scroll.set_child(Some(&env_text));
    let env_frame_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    env_frame_box.set_margin_top(12);
    env_frame_box.set_margin_bottom(12);
    env_frame_box.set_margin_start(12);
    env_frame_box.set_margin_end(12);
    env_frame_box.append(&env_group);
    env_frame_box.append(&env_scroll);
    stack.add_named(&env_frame_box, Some("step4"));

    content.append(&stack);

    let nav_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    nav_box.set_margin_top(8);
    nav_box.set_margin_bottom(12);
    nav_box.set_margin_start(12);
    nav_box.set_margin_end(12);
    nav_box.set_halign(gtk4::Align::End);
    let back_btn = gtk4::Button::with_label(&gettext("Back"));
    back_btn.set_sensitive(false);
    let next_btn = gtk4::Button::with_label(&gettext("Next"));
    next_btn.add_css_class("suggested-action");
    nav_box.append(&back_btn);
    nav_box.append(&next_btn);
    content.append(&nav_box);

    dialog.set_child(Some(&content));

    let current_step = Rc::new(Cell::new(1u32));

    let update_indicators = {
        let labels = step_labels.clone();
        let cs = current_step.clone();
        Rc::new(move || {
            let step = cs.get();
            for (i, l) in labels.iter().enumerate() {
                if i + 1 == step as usize {
                    l.remove_css_class("dim-label");
                    l.add_css_class("accent");
                } else {
                    l.remove_css_class("accent");
                    l.add_css_class("dim-label");
                }
            }
        })
    };

    {
        let cs = current_step.clone();
        let stack_w = stack.downgrade();
        let back_w = back_btn.downgrade();
        let next_w = next_btn.downgrade();
        let create_w = create_btn.downgrade();
        let upd = update_indicators.clone();
        next_btn.connect_clicked(move |_| {
            let step = cs.get();
            if step < 4 {
                cs.set(step + 1);
                if let Some(s) = stack_w.upgrade() {
                    s.set_visible_child_name(&format!("step{}", step + 1));
                }
                if let Some(b) = back_w.upgrade() {
                    b.set_sensitive(true);
                }
                if step + 1 == 4 {
                    if let Some(n) = next_w.upgrade() {
                        n.set_visible(false);
                    }
                    if let Some(c) = create_w.upgrade() {
                        c.set_sensitive(true);
                    }
                }
                upd();
            }
        });
    }

    {
        let cs = current_step.clone();
        let stack_w = stack.downgrade();
        let back_w = back_btn.downgrade();
        let next_w = next_btn.downgrade();
        let create_w = create_btn.downgrade();
        let upd = update_indicators.clone();
        back_btn.connect_clicked(move |_| {
            let step = cs.get();
            if step > 1 {
                cs.set(step - 1);
                if let Some(s) = stack_w.upgrade() {
                    s.set_visible_child_name(&format!("step{}", step - 1));
                }
                if step - 1 == 1
                    && let Some(b) = back_w.upgrade()
                {
                    b.set_sensitive(false);
                }
                if step - 1 < 4 {
                    if let Some(n) = next_w.upgrade() {
                        n.set_visible(true);
                    }
                    if let Some(c) = create_w.upgrade() {
                        c.set_sensitive(false);
                    }
                }
                upd();
            }
        });
    }

    {
        let d = dialog.downgrade();
        cancel_btn.connect_clicked(move |_| {
            if let Some(dlg) = d.upgrade() {
                dlg.close();
            }
        });
    }

    {
        let img_w = img_row.downgrade();
        let name_w = name_row.downgrade();
        let restart_w = restart_drop.downgrade();
        let network_w = network_row.downgrade();
        let env_w = env_text.downgrade();
        let dialog_w = dialog.downgrade();
        let port_rows_ref = port_rows.clone();
        let vol_rows_ref = vol_rows.clone();
        let cb = inner.clone();
        create_btn.connect_clicked(move |_| {
            let image = img_w
                .upgrade()
                .map(|r| r.text().to_string())
                .unwrap_or_default();
            if image.is_empty() {
                return;
            }
            let name = name_w.upgrade().and_then(|r| {
                let t = r.text().to_string();
                if t.is_empty() { None } else { Some(t) }
            });
            let restart_policy = restart_w
                .upgrade()
                .map(|d| match d.selected() {
                    1 => RestartPolicy::Always,
                    2 => RestartPolicy::UnlessStopped,
                    3 => RestartPolicy::OnFailure(0),
                    _ => RestartPolicy::No,
                })
                .unwrap_or(RestartPolicy::No);
            let network = network_w.upgrade().and_then(|r| {
                let t = r.text().trim().to_string();
                if t.is_empty() { None } else { Some(t) }
            });
            let port_bindings: Vec<(u16, u16)> = port_rows_ref
                .borrow()
                .iter()
                .filter_map(|(_, host_e, cont_e)| {
                    let h: u16 = host_e.text().trim().parse().ok()?;
                    let c: u16 = cont_e.text().trim().parse().ok()?;
                    Some((h, c))
                })
                .collect();
            let volume_bindings: Vec<(String, String)> = vol_rows_ref
                .borrow()
                .iter()
                .filter_map(|(_, host_e, cont_e)| {
                    let h = host_e.text().trim().to_string();
                    let c = cont_e.text().trim().to_string();
                    if h.is_empty() || c.is_empty() {
                        None
                    } else {
                        Some((h, c))
                    }
                })
                .collect();
            let env = env_w
                .upgrade()
                .map(|tv| {
                    let buf = tv.buffer();
                    buf.text(&buf.start_iter(), &buf.end_iter(), false)
                        .lines()
                        .filter(|l| l.contains('='))
                        .map(str::to_string)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            let opts = CreateContainerOptions {
                image,
                name,
                port_bindings,
                volume_bindings,
                restart_policy,
                network,
                env,
                ..Default::default()
            };
            let uc = cb.use_case.clone();
            let cb2 = cb.clone();
            let dw = dialog_w.clone();
            spawn_driver_task(
                uc,
                move |uc| uc.create(&opts),
                move |result| match result {
                    Ok(_) => {
                        (cb2.on_toast)(&gettext("Container created"));
                        reload_impl(cb2.clone(), None);
                        if let Some(d) = dw.upgrade() {
                            d.close();
                        }
                    }
                    Err(ref e) => {
                        log_container_error(&AppLogger::new(LOG_DOMAIN), e);
                        (cb2.on_toast)(&format!("{}: {e}", gettext("Create failed")));
                    }
                },
            );
        });
    }

    let key_ctrl = gtk4::EventControllerKey::new();
    let dw = dialog.downgrade();
    key_ctrl.connect_key_pressed(move |_, key, _, _| {
        if key == gtk4::gdk::Key::Escape {
            if let Some(d) = dw.upgrade() {
                d.close();
            }
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    dialog.add_controller(key_ctrl);

    dialog.present();
}

/// Append a dynamic host:container mapping row to `parent_box`.
///
/// Each row contains two `gtk4::Entry` widgets separated by ":" and a remove button.
/// The row is pushed into `rows` and removed from both `rows` and `parent_box` when
/// the remove button is clicked. Set `path_mode = true` for path entries (wider input).
fn add_mapping_row(
    rows: &MappingRows,
    parent_box: &gtk4::Box,
    left_placeholder: &str,
    right_placeholder: &str,
    path_mode: bool,
) {
    let row_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    row_box.set_margin_bottom(2);

    let left_e = gtk4::Entry::new();
    left_e.set_placeholder_text(Some(left_placeholder));
    left_e.set_hexpand(true);
    if path_mode {
        left_e.set_width_chars(16);
    }

    let sep = gtk4::Label::new(Some(":"));
    sep.set_margin_start(4);
    sep.set_margin_end(4);

    let right_e = gtk4::Entry::new();
    right_e.set_placeholder_text(Some(right_placeholder));
    right_e.set_hexpand(true);
    if path_mode {
        right_e.set_width_chars(16);
    }

    let rem_btn = gtk4::Button::new();
    rem_btn.set_icon_name("user-trash-symbolic");
    rem_btn.add_css_class("flat");
    rem_btn.set_valign(gtk4::Align::Center);
    rem_btn.set_tooltip_text(Some(&gettext("Remove")));

    row_box.append(&left_e);
    row_box.append(&sep);
    row_box.append(&right_e);
    row_box.append(&rem_btn);

    let rows_rc = Rc::downgrade(rows);
    let parent_w = parent_box.downgrade();
    let row_w = row_box.downgrade();
    rem_btn.connect_clicked(move |_| {
        let Some(rows) = rows_rc.upgrade() else {
            return;
        };
        let Some(parent) = parent_w.upgrade() else {
            return;
        };
        let Some(rbox) = row_w.upgrade() else { return };
        parent.remove(&rbox);
        rows.borrow_mut().retain(|(b, _, _)| b != &rbox);
    });

    parent_box.append(&row_box.clone());
    rows.borrow_mut().push((row_box, left_e, right_e));
}
