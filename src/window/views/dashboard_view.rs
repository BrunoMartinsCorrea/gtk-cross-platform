// SPDX-License-Identifier: GPL-3.0-or-later
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use adw::prelude::*;
use gettextrs::gettext;

use gtk_cross_platform::core::domain::container::{Container, ContainerStatus};
use gtk_cross_platform::core::domain::network::{ContainerEvent, HostStats, SystemUsage};
use gtk_cross_platform::infrastructure::containers::background::spawn_driver_task;
use gtk_cross_platform::ports::use_cases::i_container_use_case::IContainerUseCase;
use gtk_cross_platform::ports::use_cases::i_network_use_case::INetworkUseCase;

use crate::window::components::status_badge;
use crate::window::utils::format::fmt_bytes;

/// Both use cases bundled for a single `spawn_driver_task` call so containers
/// and system_df are fetched in the same worker thread.
struct DashboardUseCase {
    container: Arc<dyn IContainerUseCase>,
    network: Arc<dyn INetworkUseCase>,
}

struct DashboardSnapshot {
    containers: Vec<Container>,
    networks_count: usize,
    events: Vec<ContainerEvent>,
}

fn fetch_fast(
    uc: &DashboardUseCase,
) -> Result<DashboardSnapshot, gtk_cross_platform::infrastructure::containers::error::ContainerError>
{
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
    Ok(DashboardSnapshot {
        containers,
        networks_count: networks.len(),
        events,
    })
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

/// Apply a CSS level class to a progress bar based on usage fraction.
fn set_bar_level(bar: &gtk4::ProgressBar, fraction: f64) {
    bar.remove_css_class("level-warning");
    bar.remove_css_class("level-error");
    if fraction > 0.8 {
        bar.add_css_class("level-error");
    } else if fraction > 0.6 {
        bar.add_css_class("level-warning");
    }
}

/// Format a Unix timestamp string as relative human-readable time.
fn format_relative_time(timestamp_str: &str) -> String {
    let ts = match timestamp_str.parse::<i64>() {
        Ok(t) => t,
        Err(_) => return timestamp_str.to_string(),
    };
    let event_dt = match glib::DateTime::from_unix_local(ts) {
        Ok(dt) => dt,
        Err(_) => return timestamp_str.to_string(),
    };
    let now = match glib::DateTime::now_local() {
        Ok(dt) => dt,
        Err(_) => return timestamp_str.to_string(),
    };
    // TimeSpan is microseconds; convert to seconds.
    let diff_secs = (now.difference(&event_dt).0.abs()) / 1_000_000;
    if diff_secs < 60 {
        gettext("just now")
    } else if diff_secs < 3_600 {
        format!("{} min ago", diff_secs / 60)
    } else if diff_secs < 86_400 {
        format!("{} h ago", diff_secs / 3_600)
    } else {
        format!("{} d ago", diff_secs / 86_400)
    }
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

    /// Held to update accessible labels after each data refresh.
    running_btn: gtk4::Button,
    paused_btn: gtk4::Button,
    stopped_btn: gtk4::Button,
    errored_btn: gtk4::Button,

    images_lbl: gtk4::Label,
    volumes_lbl: gtk4::Label,
    networks_lbl: gtk4::Label,

    images_bar: gtk4::ProgressBar,
    images_size_lbl: gtk4::Label,
    volumes_bar: gtk4::ProgressBar,
    volumes_size_lbl: gtk4::Label,

    cpu_bar: gtk4::ProgressBar,
    cpu_lbl: gtk4::Label,
    mem_bar: gtk4::ProgressBar,
    mem_lbl: gtk4::Label,
    disk_bar: gtk4::ProgressBar,
    disk_lbl: gtk4::Label,

    recent_list: gtk4::ListBox,
    events_list: gtk4::ListBox,
    last_updated_lbl: gtk4::Label,
    refresh_spinner: gtk4::Spinner,

    use_cases: Arc<DashboardUseCase>,
    on_loading: Rc<dyn Fn(bool)>,
    on_toast: Rc<dyn Fn(&str)>,
    on_navigate: Rc<dyn Fn(&str)>,
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

        let cpu_bar = gtk4::ProgressBar::new();
        let cpu_lbl = gtk4::Label::new(Some("—"));
        let mem_bar = gtk4::ProgressBar::new();
        let mem_lbl = gtk4::Label::new(Some("—"));
        let disk_bar = gtk4::ProgressBar::new();
        let disk_lbl = gtk4::Label::new(Some("—"));

        let recent_list = gtk4::ListBox::new();
        recent_list.set_selection_mode(gtk4::SelectionMode::None);
        recent_list.add_css_class("boxed-list");

        let events_list = gtk4::ListBox::new();
        events_list.set_selection_mode(gtk4::SelectionMode::None);
        events_list.add_css_class("boxed-list");

        // ── container stat cards row ─────────────────────────────────────────
        let nav1 = on_navigate.clone();
        let nav2 = on_navigate.clone();
        let nav3 = on_navigate.clone();
        let nav4 = on_navigate.clone();

        // Paused starts with dim-label; apply_fast toggles to warning when > 0.
        let running_card =
            make_stat_card(&running_lbl, &gettext("Running"), "success", move || {
                nav1("containers:running")
            });
        let paused_card = make_stat_card(&paused_lbl, &gettext("Paused"), "dim-label", move || {
            nav2("containers:paused")
        });
        let stopped_card =
            make_stat_card(&stopped_lbl, &gettext("Stopped"), "dim-label", move || {
                nav3("containers:stopped")
            });
        let errored_card =
            make_stat_card(&errored_lbl, &gettext("Errors"), "dim-label", move || {
                nav4("containers:errors")
            });

        let container_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        container_row.set_hexpand(true);
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
        resource_row.set_hexpand(true);
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

        // ── host resources section ───────────────────────────────────────────
        let host_group = adw::PreferencesGroup::new();
        host_group.set_title(&gettext("Host Resources"));
        host_group.add(&make_progress_row(&gettext("CPU"), &cpu_bar, &cpu_lbl));
        host_group.add(&make_progress_row(&gettext("Memory"), &mem_bar, &mem_lbl));
        host_group.add(&make_progress_row(&gettext("Disk"), &disk_bar, &disk_lbl));

        let usage_spinner = gtk4::Spinner::new();
        usage_spinner.set_spinning(true);
        usage_spinner.set_halign(gtk4::Align::Center);
        usage_spinner.set_valign(gtk4::Align::Center);
        usage_spinner.set_height_request(48);

        let last_updated_lbl = gtk4::Label::new(None);
        last_updated_lbl.add_css_class("caption");
        last_updated_lbl.add_css_class("dim-label");

        let refresh_spinner = gtk4::Spinner::new();
        refresh_spinner.set_visible(false);
        refresh_spinner.set_valign(gtk4::Align::Center);

        let footer_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        footer_box.set_halign(gtk4::Align::End);
        footer_box.set_margin_top(4);
        footer_box.append(&refresh_spinner);
        footer_box.append(&last_updated_lbl);

        let storage_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        storage_box.append(&storage_group);
        storage_box.append(&host_group);

        let usage_stack = gtk4::Stack::new();
        usage_stack.add_named(&usage_spinner, Some("loading"));
        usage_stack.add_named(&storage_box, Some("content"));
        usage_stack.set_visible_child_name("content");

        // ── section headings ─────────────────────────────────────────────────
        let container_status_title = gtk4::Label::new(Some(&gettext("Container Status")));
        container_status_title.add_css_class("heading");
        container_status_title.set_halign(gtk4::Align::Start);

        let resources_title = gtk4::Label::new(Some(&gettext("Resources")));
        resources_title.add_css_class("heading");
        resources_title.set_halign(gtk4::Align::Start);

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

        // ── two-column layout: left = storage/host, right = recent containers/events ──
        let left_col = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        left_col.set_hexpand(true);
        left_col.set_valign(gtk4::Align::Start);
        left_col.append(&usage_stack);

        let right_col = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        right_col.set_hexpand(true);
        right_col.set_valign(gtk4::Align::Start);
        right_col.append(&recent_title);
        right_col.append(&recent_list);
        right_col.append(&events_title);
        right_col.append(&events_list);

        let columns_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
        columns_box.set_hexpand(true);
        columns_box.append(&left_col);
        columns_box.append(&right_col);

        // ── outer scrollable box ──────────────────────────────────────────────
        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(16);
        content.set_margin_end(16);
        content.set_hexpand(true);
        content.append(&spinner);
        content.append(&container_status_title);
        content.append(&container_row);
        content.append(&resources_title);
        content.append(&resource_row);
        content.append(&columns_box);
        content.append(&footer_box);

        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_hscrollbar_policy(gtk4::PolicyType::Never);
        scroll.set_vexpand(true);
        scroll.set_hexpand(true);
        scroll.set_child(Some(&content));

        // The sidebar_box is the actual widget added to the ViewStack.
        let sidebar_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        sidebar_box.set_hexpand(true);
        sidebar_box.append(&scroll);

        Self(Rc::new(Inner {
            widget: sidebar_box,
            spinner,
            usage_stack,
            running_lbl,
            paused_lbl,
            stopped_lbl,
            errored_lbl,
            running_btn: running_card,
            paused_btn: paused_card,
            stopped_btn: stopped_card,
            errored_btn: errored_card,
            images_lbl,
            volumes_lbl,
            networks_lbl,
            images_bar,
            images_size_lbl,
            volumes_bar,
            volumes_size_lbl,
            cpu_bar,
            cpu_lbl,
            mem_bar,
            mem_lbl,
            disk_bar,
            disk_lbl,
            recent_list,
            events_list,
            last_updated_lbl,
            refresh_spinner,
            use_cases: Arc::new(DashboardUseCase {
                container: container_uc,
                network: network_uc,
            }),
            on_loading: Rc::new(on_loading),
            on_toast: Rc::new(on_toast),
            on_navigate,
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
                Ok(snapshot) => {
                    Self::apply_fast(
                        &inner,
                        &snapshot.containers,
                        snapshot.networks_count,
                        &snapshot.events,
                    );
                    // Show timestamp immediately after the fast load so the user
                    // knows the container data is current; updated again after system_df.
                    Self::update_last_updated(&inner);
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
        inner.refresh_spinner.set_visible(true);
        inner.refresh_spinner.set_spinning(true);

        let use_cases = inner.use_cases.clone();
        let inner_weak = Rc::downgrade(&inner);
        spawn_driver_task(
            use_cases,
            |uc| {
                let usage = uc.network.system_df()?;
                let host = uc.network.host_stats().unwrap_or_default();
                Ok::<_, gtk_cross_platform::infrastructure::containers::error::ContainerError>((
                    usage, host,
                ))
            },
            move |result| {
                let Some(inner) = inner_weak.upgrade() else {
                    return;
                };
                inner.refresh_spinner.set_visible(false);
                inner.refresh_spinner.set_spinning(false);
                match result {
                    Ok((usage, host)) => {
                        Self::apply_usage(&inner, &usage, host.disk_total_bytes);
                        Self::apply_host_stats(&inner, &host);
                        Self::update_last_updated(&inner);
                        Self::schedule_refresh(inner);
                    }
                    Err(e) => {
                        (inner.on_toast)(&format!("{}: {e}", gettext("Usage stats failed")));
                        inner.usage_stack.set_visible_child_name("content");
                        // Keep auto-refresh alive so the dashboard recovers when the driver
                        // becomes reachable again (e.g. after a temporary socket timeout).
                        Self::schedule_refresh(inner);
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

        // Atomic CSS replacement avoids a brief intermediate state on repeated reloads.
        inner
            .paused_lbl
            .set_css_classes(&["title-1", if paused > 0 { "warning" } else { "dim-label" }]);
        inner
            .errored_lbl
            .set_css_classes(&["title-1", if errored > 0 { "error" } else { "dim-label" }]);

        // Update accessible labels to include the numeric value so screen readers
        // announce e.g. "4 Running" rather than just "Running".
        let running_desc = gettext("Running");
        let paused_desc = gettext("Paused");
        let stopped_desc = gettext("Stopped");
        let errors_desc = gettext("Errors");
        inner
            .running_btn
            .update_property(&[gtk4::accessible::Property::Label(&format!(
                "{running} {running_desc}"
            ))]);
        inner
            .paused_btn
            .update_property(&[gtk4::accessible::Property::Label(&format!(
                "{paused} {paused_desc}"
            ))]);
        inner
            .stopped_btn
            .update_property(&[gtk4::accessible::Property::Label(&format!(
                "{stopped} {stopped_desc}"
            ))]);
        inner
            .errored_btn
            .update_property(&[gtk4::accessible::Property::Label(&format!(
                "{errored} {errors_desc}"
            ))]);

        inner.networks_lbl.set_text(&networks_count.to_string());

        // Recent containers — clear and repopulate
        while let Some(child) = inner.recent_list.first_child() {
            inner.recent_list.remove(&child);
        }
        for c in containers.iter().take(8) {
            let row = adw::ActionRow::new();
            row.set_title(&glib::markup_escape_text(&c.name));
            row.set_subtitle(&glib::markup_escape_text(&c.image));
            row.add_suffix(&status_badge::new(&c.status));
            inner.recent_list.append(&row);
        }
        if !containers.is_empty() {
            let see_all = adw::ActionRow::new();
            see_all.set_title(&gettext("See all containers"));
            see_all.set_activatable(true);
            see_all.add_suffix(&gtk4::Image::from_icon_name("go-next-symbolic"));
            let nav = inner.on_navigate.clone();
            see_all.connect_activated(move |_| nav("containers"));
            inner.recent_list.append(&see_all);
        } else {
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
            let relative_ts = format_relative_time(&ev.timestamp);
            row.set_subtitle(&glib::markup_escape_text(&format!(
                "{} · {}",
                ev.actor, relative_ts
            )));
            inner.events_list.append(&row);
        }
        if events.is_empty() {
            let row = adw::ActionRow::new();
            row.set_title(&gettext("No recent events"));
            inner.events_list.append(&row);
        }
    }

    fn apply_usage(inner: &Inner, usage: &SystemUsage, disk_total_bytes: u64) {
        inner.images_lbl.set_text(&usage.images_total.to_string());
        inner.volumes_lbl.set_text(&usage.volumes_total.to_string());

        // When disk_total_bytes is available, bars show each type as a percentage
        // of the total physical disk so the user can see actual disk pressure.
        // Falls back to 0 when disk size is unavailable (e.g. inside Flatpak sandbox).
        let img_frac = if disk_total_bytes > 0 {
            (usage.images_size as f64 / disk_total_bytes as f64).min(1.0)
        } else {
            0.0
        };
        let vol_frac = if disk_total_bytes > 0 {
            (usage.volumes_size as f64 / disk_total_bytes as f64).min(1.0)
        } else {
            0.0
        };
        inner.images_bar.set_fraction(img_frac);
        inner.volumes_bar.set_fraction(vol_frac);
        let img_pct = (img_frac * 100.0).round() as u32;
        let vol_pct = (vol_frac * 100.0).round() as u32;
        let img_size_text = format!("{} ({}%)", fmt_bytes(usage.images_size), img_pct);
        let vol_size_text = format!("{} ({}%)", fmt_bytes(usage.volumes_size), vol_pct);
        inner.images_size_lbl.set_text(&img_size_text);
        inner.volumes_size_lbl.set_text(&vol_size_text);

        let bar_tooltip = gettext("Percentage of total disk space used by container storage");
        inner.images_bar.set_tooltip_text(Some(&bar_tooltip));
        inner.volumes_bar.set_tooltip_text(Some(&bar_tooltip));

        // Expose value text so screen readers announce the storage size.
        let images_label = gettext("Images");
        let volumes_label = gettext("Volumes");
        inner
            .images_bar
            .update_property(&[gtk4::accessible::Property::ValueText(&format!(
                "{images_label} — {img_size_text}"
            ))]);
        inner
            .volumes_bar
            .update_property(&[gtk4::accessible::Property::ValueText(&format!(
                "{volumes_label} — {vol_size_text}"
            ))]);

        inner.usage_stack.set_visible_child_name("content");
    }

    fn apply_host_stats(inner: &Inner, stats: &HostStats) {
        let cpu_pct = stats.cpu_percent.round() as u32;
        let mem_pct = stats.mem_percent.round() as u32;
        let disk_pct = stats.disk_percent.round() as u32;

        let cpu_frac = stats.cpu_percent / 100.0;
        let mem_frac = stats.mem_percent / 100.0;
        let disk_frac = stats.disk_percent / 100.0;

        inner.cpu_bar.set_fraction(cpu_frac);
        inner.mem_bar.set_fraction(mem_frac);
        inner.disk_bar.set_fraction(disk_frac);

        set_bar_level(&inner.cpu_bar, cpu_frac);
        set_bar_level(&inner.mem_bar, mem_frac);
        set_bar_level(&inner.disk_bar, disk_frac);

        inner.cpu_lbl.set_text(&format!("{cpu_pct}%"));
        inner.mem_lbl.set_text(&format!("{mem_pct}%"));
        inner.disk_lbl.set_text(&format!("{disk_pct}%"));

        let cpu_label = gettext("CPU");
        let mem_label = gettext("Memory");
        let disk_label = gettext("Disk");
        inner
            .cpu_bar
            .update_property(&[gtk4::accessible::Property::ValueText(&format!(
                "{cpu_label} — {cpu_pct}%"
            ))]);
        inner
            .mem_bar
            .update_property(&[gtk4::accessible::Property::ValueText(&format!(
                "{mem_label} — {mem_pct}%"
            ))]);
        inner
            .disk_bar
            .update_property(&[gtk4::accessible::Property::ValueText(&format!(
                "{disk_label} — {disk_pct}%"
            ))]);
    }
}
