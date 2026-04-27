// SPDX-License-Identifier: GPL-3.0-or-later
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use adw::prelude::*;
use gettextrs::gettext;

use gtk_cross_platform::core::domain::container::{Container, ContainerStatus};
use gtk_cross_platform::core::domain::network::{ContainerEvent, SystemUsage};
use gtk_cross_platform::infrastructure::containers::background::spawn_driver_task;
use gtk_cross_platform::ports::use_cases::i_container_use_case::IContainerUseCase;
use gtk_cross_platform::ports::use_cases::i_network_use_case::INetworkUseCase;

use crate::window::utils::format::fmt_bytes;

/// Both use cases bundled for a single `spawn_driver_task` call so containers
/// and system_df are fetched in the same worker thread.
struct DashboardUseCase {
    container: Arc<dyn IContainerUseCase>,
    network: Arc<dyn INetworkUseCase>,
}

type FastResult = (Vec<Container>, usize, Vec<ContainerEvent>);

fn fetch_fast(
    uc: &DashboardUseCase,
) -> Result<FastResult, gtk_cross_platform::infrastructure::containers::error::ContainerError> {
    let containers = uc.container.list(true)?;
    let networks = uc.network.list()?;
    // Limit events to the last 24 hours so Docker returns a bounded response.
    let since_24h = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
        - 86_400;
    let events = uc
        .network
        .events(Some(since_24h), Some(10))
        .unwrap_or_default();
    Ok((containers, networks.len(), events))
}

// ───────────────────────────── widget builders ────────────────────────────────

fn make_stat_card(
    value_label: &gtk4::Label,
    description: &str,
    css_class: &str,
    on_click: impl Fn() + 'static,
) -> gtk4::Button {
    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    vbox.set_halign(gtk4::Align::Fill);
    vbox.set_margin_top(12);
    vbox.set_margin_bottom(12);
    vbox.set_margin_start(12);
    vbox.set_margin_end(12);

    value_label.add_css_class("title-1");
    value_label.add_css_class(css_class);
    value_label.set_halign(gtk4::Align::Start);

    let desc = gtk4::Label::new(Some(description));
    desc.add_css_class("caption");
    desc.set_halign(gtk4::Align::Start);

    vbox.append(value_label);
    vbox.append(&desc);

    let btn = gtk4::Button::new();
    btn.set_child(Some(&vbox));
    btn.add_css_class("card");
    btn.set_hexpand(true);
    btn.set_tooltip_text(Some(&format!(
        "{} — {}",
        gettext("Navigate to"),
        description
    )));
    btn.update_property(&[gtk4::accessible::Property::Label(description)]);
    btn.connect_clicked(move |_| on_click());
    btn
}

fn make_progress_row(label: &str, bar: &gtk4::ProgressBar, size_label: &gtk4::Label) -> gtk4::Box {
    let row = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    row.set_margin_top(8);
    row.set_margin_bottom(8);
    row.set_margin_start(12);
    row.set_margin_end(12);

    let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    let name_lbl = gtk4::Label::new(Some(label));
    name_lbl.add_css_class("caption-heading");
    name_lbl.set_hexpand(true);
    name_lbl.set_halign(gtk4::Align::Start);

    size_label.add_css_class("caption");
    size_label.set_halign(gtk4::Align::End);

    header.append(&name_lbl);
    header.append(size_label);

    bar.set_hexpand(true);

    row.append(&header);
    row.append(bar);
    row
}

// ───────────────────────────── DashboardView ──────────────────────────────────

struct Inner {
    widget: gtk4::Box,
    spinner: gtk4::Spinner,
    usage_stack: gtk4::Stack,

    running_lbl: gtk4::Label,
    paused_lbl: gtk4::Label,
    stopped_lbl: gtk4::Label,
    errored_lbl: gtk4::Label,

    images_lbl: gtk4::Label,
    volumes_lbl: gtk4::Label,
    networks_lbl: gtk4::Label,

    images_bar: gtk4::ProgressBar,
    images_size_lbl: gtk4::Label,
    volumes_bar: gtk4::ProgressBar,
    volumes_size_lbl: gtk4::Label,

    recent_list: gtk4::ListBox,
    events_list: gtk4::ListBox,
    last_updated_lbl: gtk4::Label,

    use_cases: Arc<DashboardUseCase>,
    on_loading: Rc<dyn Fn(bool)>,
    on_toast: Rc<dyn Fn(&str)>,
    loading: Cell<bool>,
    loaded: Cell<bool>,
    refresh_timer: RefCell<Option<glib::SourceId>>,
}

#[derive(Clone)]
pub struct DashboardView(Rc<Inner>);

impl DashboardView {
    pub fn new(
        container_uc: Arc<dyn IContainerUseCase>,
        network_uc: Arc<dyn INetworkUseCase>,
        on_navigate: Rc<dyn Fn(&str)>,
        on_toast: impl Fn(&str) + 'static,
        on_loading: impl Fn(bool) + 'static,
    ) -> Self {
        // ── value labels (updated on reload) ──────────────────────────────────
        let running_lbl = gtk4::Label::new(Some("—"));
        let paused_lbl = gtk4::Label::new(Some("—"));
        let stopped_lbl = gtk4::Label::new(Some("—"));
        let errored_lbl = gtk4::Label::new(Some("—"));

        let images_lbl = gtk4::Label::new(Some("—"));
        let volumes_lbl = gtk4::Label::new(Some("—"));
        let networks_lbl = gtk4::Label::new(Some("—"));

        let images_bar = gtk4::ProgressBar::new();
        let images_size_lbl = gtk4::Label::new(Some("—"));
        let volumes_bar = gtk4::ProgressBar::new();
        let volumes_size_lbl = gtk4::Label::new(Some("—"));

        let recent_list = gtk4::ListBox::new();
        recent_list.set_selection_mode(gtk4::SelectionMode::None);
        recent_list.add_css_class("boxed-list");

        let events_list = gtk4::ListBox::new();
        events_list.set_selection_mode(gtk4::SelectionMode::None);
        events_list.add_css_class("boxed-list");

        // ── container stat cards row ─────────────────────────────────────────
        let nav_containers = on_navigate.clone();
        let nav_containers2 = on_navigate.clone();
        let nav_containers3 = on_navigate.clone();
        let nav_containers4 = on_navigate.clone();

        let running_card =
            make_stat_card(&running_lbl, &gettext("Running"), "success", move || {
                nav_containers("containers")
            });
        let paused_card = make_stat_card(&paused_lbl, &gettext("Paused"), "warning", move || {
            nav_containers2("containers")
        });
        let stopped_card =
            make_stat_card(&stopped_lbl, &gettext("Stopped"), "dim-label", move || {
                nav_containers3("containers")
            });
        let errored_card = make_stat_card(&errored_lbl, &gettext("Errors"), "error", move || {
            nav_containers4("containers")
        });

        let container_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        container_row.append(&running_card);
        container_row.append(&paused_card);
        container_row.append(&stopped_card);
        container_row.append(&errored_card);

        // ── resource cards row ───────────────────────────────────────────────
        let nav_images = on_navigate.clone();
        let nav_volumes = on_navigate.clone();
        let nav_networks = on_navigate.clone();

        let images_card = make_stat_card(&images_lbl, &gettext("Images"), "accent", move || {
            nav_images("images")
        });
        let volumes_card = make_stat_card(&volumes_lbl, &gettext("Volumes"), "accent", move || {
            nav_volumes("volumes")
        });
        let networks_card =
            make_stat_card(&networks_lbl, &gettext("Networks"), "accent", move || {
                nav_networks("networks")
            });

        let resource_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        resource_row.append(&images_card);
        resource_row.append(&volumes_card);
        resource_row.append(&networks_card);

        // ── storage section ──────────────────────────────────────────────────
        let storage_group = adw::PreferencesGroup::new();
        storage_group.set_title(&gettext("Storage"));
        storage_group.add(&make_progress_row(
            &gettext("Images"),
            &images_bar,
            &images_size_lbl,
        ));
        storage_group.add(&make_progress_row(
            &gettext("Volumes"),
            &volumes_bar,
            &volumes_size_lbl,
        ));

        let usage_spinner = gtk4::Spinner::new();
        usage_spinner.set_spinning(true);
        usage_spinner.set_halign(gtk4::Align::Center);
        usage_spinner.set_valign(gtk4::Align::Center);
        usage_spinner.set_height_request(48);

        let last_updated_lbl = gtk4::Label::new(None);
        last_updated_lbl.add_css_class("caption");
        last_updated_lbl.add_css_class("dim-label");
        last_updated_lbl.set_halign(gtk4::Align::End);
        last_updated_lbl.set_margin_top(4);

        let storage_with_footer = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        storage_with_footer.append(&storage_group);
        storage_with_footer.append(&last_updated_lbl);

        let usage_stack = gtk4::Stack::new();
        usage_stack.add_named(&usage_spinner, Some("loading"));
        usage_stack.add_named(&storage_with_footer, Some("content"));
        usage_stack.set_visible_child_name("content");

        // ── recent containers ────────────────────────────────────────────────
        let recent_title = gtk4::Label::new(Some(&gettext("Recent Containers")));
        recent_title.add_css_class("heading");
        recent_title.set_halign(gtk4::Align::Start);

        // ── recent events ────────────────────────────────────────────────────
        let events_title = gtk4::Label::new(Some(&gettext("Recent Events")));
        events_title.add_css_class("heading");
        events_title.set_halign(gtk4::Align::Start);

        // ── spinner (shown while loading) ─────────────────────────────────────
        let spinner = gtk4::Spinner::new();
        spinner.set_halign(gtk4::Align::Center);
        spinner.set_visible(false);

        // ── outer scrollable box ──────────────────────────────────────────────
        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);
        content.append(&spinner);
        content.append(&container_row);
        content.append(&resource_row);
        content.append(&usage_stack);
        content.append(&recent_title);
        content.append(&recent_list);
        content.append(&events_title);
        content.append(&events_list);

        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_hscrollbar_policy(gtk4::PolicyType::Never);
        scroll.set_vexpand(true);
        scroll.set_child(Some(&content));

        // The sidebar_box is the actual widget added to the ViewStack.
        let sidebar_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        sidebar_box.append(&scroll);

        Self(Rc::new(Inner {
            widget: sidebar_box,
            spinner,
            usage_stack,
            running_lbl,
            paused_lbl,
            stopped_lbl,
            errored_lbl,
            images_lbl,
            volumes_lbl,
            networks_lbl,
            images_bar,
            images_size_lbl,
            volumes_bar,
            volumes_size_lbl,
            recent_list,
            events_list,
            last_updated_lbl,
            use_cases: Arc::new(DashboardUseCase {
                container: container_uc,
                network: network_uc,
            }),
            on_loading: Rc::new(on_loading),
            on_toast: Rc::new(on_toast),
            loading: Cell::new(false),
            loaded: Cell::new(false),
            refresh_timer: RefCell::new(None),
        }))
    }

    pub fn widget(&self) -> &gtk4::Box {
        &self.0.widget
    }

    pub fn reload(&self) {
        if self.0.loading.get() {
            return;
        }
        self.0.loading.set(true);
        self.0.spinner.set_visible(true);
        self.0.spinner.set_spinning(true);
        (self.0.on_loading)(true);

        let inner_weak = Rc::downgrade(&self.0);
        spawn_driver_task(self.0.use_cases.clone(), fetch_fast, move |result| {
            let Some(inner) = inner_weak.upgrade() else {
                return;
            };
            inner.spinner.set_visible(false);
            inner.spinner.set_spinning(false);
            inner.loading.set(false);
            inner.loaded.set(true);
            (inner.on_loading)(false);

            match result {
                Ok((containers, networks_count, events)) => {
                    Self::apply_fast(&inner, &containers, networks_count, &events);
                    inner.usage_stack.set_visible_child_name("loading");
                    Self::load_usage_stats(inner);
                }
                Err(e) => {
                    (inner.on_toast)(&format!("{}: {e}", gettext("Dashboard load failed")));
                }
            }
        });
    }

    pub fn is_loaded(&self) -> bool {
        self.0.loaded.get()
    }

    pub fn stop_auto_refresh(&self) {
        if let Some(id) = self.0.refresh_timer.borrow_mut().take() {
            id.remove();
        }
    }

    fn load_usage_stats(inner: Rc<Inner>) {
        let use_cases = inner.use_cases.clone();
        let inner_weak = Rc::downgrade(&inner);
        spawn_driver_task(
            use_cases,
            |uc| uc.network.system_df(),
            move |result| {
                let Some(inner) = inner_weak.upgrade() else {
                    return;
                };
                match result {
                    Ok(usage) => {
                        Self::apply_usage(&inner, &usage);
                        Self::update_last_updated(&inner);
                        Self::schedule_refresh(inner);
                    }
                    Err(e) => {
                        (inner.on_toast)(&format!("{}: {e}", gettext("Usage stats failed")));
                        inner.usage_stack.set_visible_child_name("content");
                    }
                }
            },
        );
    }

    fn update_last_updated(inner: &Inner) {
        let time_str = glib::DateTime::now_local()
            .ok()
            .and_then(|dt| dt.format("%H:%M:%S").ok())
            .map(|s| s.to_string())
            .unwrap_or_default();
        if !time_str.is_empty() {
            inner
                .last_updated_lbl
                .set_text(&format!("{} {time_str}", gettext("Last updated")));
        }
    }

    fn schedule_refresh(inner: Rc<Inner>) {
        if let Some(id) = inner.refresh_timer.borrow_mut().take() {
            id.remove();
        }
        let inner_weak = Rc::downgrade(&inner);
        let new_id = glib::timeout_add_seconds_local(30, move || {
            if let Some(inner) = inner_weak.upgrade() {
                Self::load_usage_stats(inner);
            }
            glib::ControlFlow::Break
        });
        *inner.refresh_timer.borrow_mut() = Some(new_id);
    }

    fn apply_fast(
        inner: &Inner,
        containers: &[Container],
        networks_count: usize,
        events: &[ContainerEvent],
    ) {
        let running = containers
            .iter()
            .filter(|c| matches!(c.status, ContainerStatus::Running))
            .count();
        let paused = containers
            .iter()
            .filter(|c| matches!(c.status, ContainerStatus::Paused))
            .count();
        let stopped = containers
            .iter()
            .filter(|c| {
                matches!(
                    c.status,
                    ContainerStatus::Stopped | ContainerStatus::Exited(_)
                )
            })
            .count();
        let errored = containers
            .iter()
            .filter(|c| {
                matches!(
                    c.status,
                    ContainerStatus::Dead
                        | ContainerStatus::Unknown(_)
                        | ContainerStatus::Restarting
                )
            })
            .count();

        inner.running_lbl.set_text(&running.to_string());
        inner.paused_lbl.set_text(&paused.to_string());
        inner.stopped_lbl.set_text(&stopped.to_string());
        inner.errored_lbl.set_text(&errored.to_string());
        inner.networks_lbl.set_text(&networks_count.to_string());

        // Recent containers — clear and repopulate
        while let Some(child) = inner.recent_list.first_child() {
            inner.recent_list.remove(&child);
        }
        for c in containers.iter().take(5) {
            let row = adw::ActionRow::new();
            row.set_title(&glib::markup_escape_text(&c.name));
            row.set_subtitle(&glib::markup_escape_text(&c.image));
            let badge = gtk4::Label::new(Some(c.status.label()));
            badge.add_css_class("caption");
            badge.add_css_class(&format!("status-{}", c.status.css_class()));
            row.add_suffix(&badge);
            inner.recent_list.append(&row);
        }
        if containers.is_empty() {
            let row = adw::ActionRow::new();
            row.set_title(&gettext("No containers"));
            inner.recent_list.append(&row);
        }

        // Recent events — clear and repopulate
        while let Some(child) = inner.events_list.first_child() {
            inner.events_list.remove(&child);
        }
        for ev in events.iter().take(10) {
            let row = adw::ActionRow::new();
            row.set_title(&glib::markup_escape_text(&format!(
                "{} {}",
                ev.event_type, ev.action
            )));
            row.set_subtitle(&glib::markup_escape_text(&format!(
                "{} · {}",
                ev.actor, ev.timestamp
            )));
            inner.events_list.append(&row);
        }
        if events.is_empty() {
            let row = adw::ActionRow::new();
            row.set_title(&gettext("No recent events"));
            inner.events_list.append(&row);
        }
    }

    fn apply_usage(inner: &Inner, usage: &SystemUsage) {
        inner.images_lbl.set_text(&usage.images_total.to_string());
        inner.volumes_lbl.set_text(&usage.volumes_total.to_string());

        // Storage bars: fraction of a 50 GiB ceiling (visual guide, not exact)
        const CEIL: u64 = 50 * 1_073_741_824;
        let img_frac = (usage.images_size as f64 / CEIL as f64).min(1.0);
        let vol_frac = (usage.volumes_size as f64 / CEIL as f64).min(1.0);
        inner.images_bar.set_fraction(img_frac);
        inner.volumes_bar.set_fraction(vol_frac);
        inner
            .images_size_lbl
            .set_text(&fmt_bytes(usage.images_size));
        inner
            .volumes_size_lbl
            .set_text(&fmt_bytes(usage.volumes_size));

        inner.usage_stack.set_visible_child_name("content");
    }
}
