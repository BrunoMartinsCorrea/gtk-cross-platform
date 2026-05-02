// SPDX-License-Identifier: GPL-3.0-or-later
use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use adw::prelude::*;
use gettextrs::{gettext, pgettext};
use glib;
use gtk4::gio;

use gtk_cross_platform::core::domain::volume::CreateVolumeOptions;
use gtk_cross_platform::infrastructure::containers::background::spawn_driver_task;
use gtk_cross_platform::infrastructure::containers::error::log_container_error;
use gtk_cross_platform::infrastructure::logging::app_logger::AppLogger;
use gtk_cross_platform::ports::use_cases::i_volume_use_case::IVolumeUseCase;

use crate::window::components::{
    clear_box, confirm_dialog, detail_pane, empty_state::EmptyState, resource_row,
};
use crate::window::objects::VolumeObject;
use crate::window::utils::format::fmt_bytes;
use crate::window::utils::store::find_store_position;

const LOG_DOMAIN: &str = concat!(env!("APP_ID"), ".view.volumes");

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
    use_case: Arc<dyn IVolumeUseCase>,
    on_toast: Rc<dyn Fn(&str)>,
    on_loading: Rc<dyn Fn(bool)>,
    loading: Cell<bool>,
    loaded: Cell<bool>,
    selection_handler: std::cell::RefCell<Option<glib::SignalHandlerId>>,
    list_cancellable: std::cell::RefCell<Option<gio::Cancellable>>,
}

#[derive(Clone)]
pub struct VolumesView(Rc<Inner>);

impl VolumesView {
    pub fn new(
        use_case: Arc<dyn IVolumeUseCase>,
        detail_content: gtk4::Box,
        detail_stack: gtk4::Stack,
        on_toast: impl Fn(&str) + 'static,
        on_loading: impl Fn(bool) + 'static,
    ) -> Self {
        let store = gio::ListStore::new::<VolumeObject>();

        let search_entry = gtk4::SearchEntry::new();
        search_entry.set_hexpand(true);
        search_entry.set_placeholder_text(Some(&gettext("Search volumes…")));

        let search_bar = gtk4::SearchBar::new();
        search_bar.set_search_mode(false);
        search_bar.set_show_close_button(true);
        search_bar.set_child(Some(&search_entry));

        let se_weak = search_entry.downgrade();
        let filter = gtk4::CustomFilter::new(move |obj| {
            let Some(entry) = se_weak.upgrade() else {
                return true;
            };
            let query = entry.text();
            if query.is_empty() {
                return true;
            }
            let q = query.to_ascii_lowercase();
            let vol = obj.downcast_ref::<VolumeObject>().unwrap();
            vol.name().to_ascii_lowercase().contains(&q)
                || vol.driver().to_ascii_lowercase().contains(&q)
        });

        let filter_model = gtk4::FilterListModel::new(Some(store.clone()), Some(filter.clone()));

        let sorter = gtk4::CustomSorter::new(|a, b| {
            let a = a.downcast_ref::<VolumeObject>().unwrap();
            let b = b.downcast_ref::<VolumeObject>().unwrap();
            a.name().cmp(&b.name()).into()
        });
        let sort_model = gtk4::SortListModel::new(Some(filter_model.clone()), Some(sorter));

        let selection = gtk4::SingleSelection::new(Some(sort_model.clone()));
        selection.set_autoselect(false);

        let list_view =
            gtk4::ListView::new(Some(selection.clone()), None::<gtk4::SignalListItemFactory>);
        list_view.add_css_class("boxed-list");
        list_view.set_hexpand(true);
        list_view.set_show_separators(true);

        let create_btn = gtk4::Button::new();
        create_btn.set_label(&gettext("New Volume"));
        create_btn.add_css_class("pill");
        create_btn.set_halign(gtk4::Align::Center);
        create_btn.set_margin_top(4);
        create_btn.set_margin_bottom(4);
        create_btn.set_tooltip_text(Some(&gettext("Create a new volume")));
        create_btn.update_property(&[gtk4::accessible::Property::Label(&gettext("New volume"))]);

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

        let empty_status =
            EmptyState::no_items("drive-harddisk-symbolic", "No Volumes", "No volumes found.");
        empty_status.set_vexpand(true);

        let list_stack = gtk4::Stack::new();
        list_stack.set_vexpand(true);
        list_stack.add_named(&scroll, Some("list"));
        list_stack.add_named(&empty_status, Some("empty"));

        let sidebar_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        sidebar_box.append(&search_bar);
        sidebar_box.append(&create_btn);
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
            on_toast: Rc::new(on_toast),
            on_loading: Rc::new(on_loading),
            loading: Cell::new(false),
            loaded: Cell::new(false),
            selection_handler: std::cell::RefCell::new(None),
            list_cancellable: std::cell::RefCell::new(None),
        });

        let view = Self(inner);
        view.wire_signals(&search_bar, &create_btn);
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
        self.0.detail_stack.set_visible_child_name("empty");
    }

    fn wire_signals(&self, search_bar: &gtk4::SearchBar, create_btn: &gtk4::Button) {
        let inner_weak = Rc::downgrade(&self.0);
        let factory = gtk4::SignalListItemFactory::new();

        {
            let iw = inner_weak.clone();
            factory.connect_setup(move |_, obj| {
                let item = obj.downcast_ref::<gtk4::ListItem>().unwrap();

                let row = adw::ActionRow::new();
                row.set_activatable(true);
                row.set_focusable(true);

                let remove_btn = resource_row::icon_button(
                    "user-trash-symbolic",
                    &pgettext("volume action", "Remove volume"),
                );
                row.add_suffix(&remove_btn);

                let item_weak = item.downgrade();
                let iw2 = iw.clone();
                remove_btn.connect_clicked(move |btn| {
                    let Some(item) = item_weak.upgrade() else {
                        return;
                    };
                    let Some(inner) = iw2.upgrade() else { return };
                    let Some(vol_obj) = item.item().and_downcast::<VolumeObject>() else {
                        return;
                    };

                    let name = vol_obj.name();
                    let idx =
                        find_store_position::<VolumeObject, _>(&inner.store, |o| o.name() == name);
                    let body = gettext("Remove volume \"{name}\"? All data will be lost.")
                        .replace("{name}", &name);
                    let inner2 = inner.clone();
                    confirm_dialog::ask(
                        btn,
                        &gettext("Remove Volume?"),
                        &body,
                        &pgettext("volume action", "Remove"),
                        move || {
                            let log = AppLogger::new(LOG_DOMAIN);
                            log.debug(&format!("Removing volume {name}"));
                            let use_case = inner2.use_case.clone();
                            let name2 = name.clone();
                            let cb = inner2.clone();
                            spawn_driver_task(
                                use_case,
                                move |uc| uc.remove(&name2, false),
                                move |result| match result {
                                    Ok(()) => {
                                        (cb.on_toast)(&gettext("Volume removed"));
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

                item.set_child(Some(&row));
            });
        }

        factory.connect_bind(|_, obj| {
            let item = obj.downcast_ref::<gtk4::ListItem>().unwrap();
            let row = item.child().and_downcast::<adw::ActionRow>().unwrap();
            let vol_obj = item.item().and_downcast::<VolumeObject>().unwrap();

            let mut subtitle = vol_obj.driver();
            if vol_obj.size_bytes() >= 0 {
                subtitle.push_str(&format!(" · {}", fmt_bytes(vol_obj.size_bytes() as u64)));
            }
            if !vol_obj.in_use() {
                subtitle.push_str(&format!(" · {}", gettext("unused")));
            }
            row.set_title(&vol_obj.name());
            row.set_subtitle(&subtitle);
        });

        self.0.list_view.set_factory(Some(&factory));

        // Selection → detail pane
        let iw = inner_weak.clone();
        let handler_id = self
            .0
            .selection
            .connect_selection_changed(move |sel, _, _| {
                let Some(inner) = iw.upgrade() else { return };
                if let Some(obj) = sel.selected_item().and_downcast::<VolumeObject>() {
                    show_detail(&inner, &obj);
                } else {
                    inner.detail_stack.set_visible_child_name("empty");
                }
            });
        *self.0.selection_handler.borrow_mut() = Some(handler_id);

        // Empty state watcher
        {
            let iw = inner_weak.clone();
            self.0
                .filter_model
                .connect_items_changed(move |model, _, _, _| {
                    let Some(inner) = iw.upgrade() else { return };
                    update_empty_state(&inner, model.n_items());
                });
        }

        // Search filter
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

        // Create volume dialog
        {
            let iw = inner_weak.clone();
            create_btn.connect_clicked(move |btn| {
                let Some(inner) = iw.upgrade() else { return };
                show_create_dialog(btn, inner);
            });
        }
    }
}

// ── Reload ────────────────────────────────────────────────────────────────────

fn reload_impl(inner: Rc<Inner>, focus_after_remove: Option<u32>) {
    let log = AppLogger::new(LOG_DOMAIN);
    log.info("Loading volumes list");
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
        |uc| uc.list(),
        move |result| {
            if cancellable.is_cancelled() {
                return;
            }
            let log = AppLogger::new(LOG_DOMAIN);
            cb.loading.set(false);
            cb.loaded.set(true);
            (cb.on_loading)(false);
            match result {
                Ok(volumes) => {
                    log.info(&format!("Loaded {} volumes", volumes.len()));
                    refresh_store(&cb, &volumes);
                    if let Some(idx) = focus_after_remove {
                        focus_after_store_update(&cb, idx);
                    }
                }
                Err(ref e) => {
                    log_container_error(&log, e);
                    (cb.on_toast)(&format!("{}: {e}", gettext("Failed to load volumes")));
                }
            }
        },
    );
}

// ── Store helpers ─────────────────────────────────────────────────────────────

fn refresh_store(inner: &Rc<Inner>, volumes: &[gtk_cross_platform::core::domain::volume::Volume]) {
    if let Some(id) = inner.selection_handler.borrow().as_ref() {
        inner.selection.block_signal(id);
    }
    inner.store.remove_all();
    for vol in volumes {
        inner.store.append(&VolumeObject::from_domain(vol));
    }
    if let Some(id) = inner.selection_handler.borrow().as_ref() {
        inner.selection.unblock_signal(id);
    }
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
    if let Some(obj) = inner
        .selection
        .selected_item()
        .and_downcast::<VolumeObject>()
    {
        show_detail(inner, &obj);
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
                gettext("No volumes matched"),
                inner.search_entry.text()
            )));
        } else {
            inner
                .empty_status
                .set_icon_name(Some("drive-harddisk-symbolic"));
            inner.empty_status.set_title(&gettext("No Volumes"));
            inner
                .empty_status
                .set_description(Some(&gettext("No volumes found.")));
        }
        inner.list_stack.set_visible_child_name("empty");
    } else {
        inner.list_stack.set_visible_child_name("list");
    }
}

// ── Detail pane ───────────────────────────────────────────────────────────────

fn show_detail(inner: &Rc<Inner>, obj: &VolumeObject) {
    clear_box(&inner.detail_content);

    let size_str = if obj.size_bytes() >= 0 {
        fmt_bytes(obj.size_bytes() as u64)
    } else {
        "—".to_string()
    };
    let in_use_str = if obj.in_use() {
        gettext("Yes")
    } else {
        format!("{} — {}", gettext("No"), gettext("can be pruned"))
    };

    let pane = detail_pane::build(&[detail_pane::PropertyGroup {
        title: String::new(),
        rows: vec![
            (gettext("Name"), obj.name()),
            (gettext("Driver"), obj.driver()),
            (gettext("Mountpoint"), obj.mountpoint()),
            (gettext("Size"), size_str),
            (gettext("In Use"), in_use_str),
        ],
    }]);
    inner.detail_content.append(&pane);
    inner.detail_stack.set_visible_child_name("detail");
}

// ── Create volume dialog ──────────────────────────────────────────────────────

fn show_create_dialog(trigger: &impl gtk4::prelude::IsA<gtk4::Widget>, inner: Rc<Inner>) {
    let window = trigger.root().and_downcast::<gtk4::Window>();
    let dialog = adw::MessageDialog::new(window.as_ref(), Some(&gettext("New Volume")), None);
    dialog.add_response("cancel", &gettext("Cancel"));
    dialog.add_response("create", &pgettext("volume action", "Create"));
    dialog.set_response_appearance("create", adw::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("create"));
    dialog.set_close_response("cancel");

    let form = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    form.set_margin_top(8);

    let entry = gtk4::Entry::new();
    entry.set_placeholder_text(Some(&gettext("Volume name")));
    entry.set_activates_default(true);
    form.append(&entry);

    let driver_label = gtk4::Label::new(Some(&gettext("Driver")));
    driver_label.set_halign(gtk4::Align::Start);
    driver_label.add_css_class("caption");
    form.append(&driver_label);

    let drivers = gtk4::StringList::new(&["local", "nfs", "tmpfs"]);
    let driver_drop = gtk4::DropDown::new(Some(drivers), gtk4::Expression::NONE);
    driver_drop.set_selected(0);
    form.append(&driver_drop);

    dialog.set_extra_child(Some(&form));

    let trigger_weak = trigger.upcast_ref::<gtk4::Widget>().downgrade();
    dialog.connect_response(None, move |_, response| {
        if let Some(w) = trigger_weak.upgrade() {
            w.grab_focus();
        }
        if response != "create" {
            return;
        }
        let name = entry.text().trim().to_string();
        if name.is_empty() {
            return;
        }
        let driver = driver_drop
            .selected_item()
            .and_downcast::<gtk4::StringObject>()
            .map(|s| s.string().to_string())
            .unwrap_or_else(|| "local".to_string());
        let opts = CreateVolumeOptions {
            name: name.clone(),
            driver,
            labels: Default::default(),
        };
        let log = AppLogger::new(LOG_DOMAIN);
        log.debug(&format!("Creating volume {name}"));
        let use_case = inner.use_case.clone();
        let cb = inner.clone();
        (inner.on_loading)(true);
        spawn_driver_task(
            use_case,
            move |uc| uc.create(&opts),
            move |result| {
                (cb.on_loading)(false);
                match result {
                    Ok(_) => {
                        (cb.on_toast)(&gettext("Volume created"));
                        reload_impl(cb.clone(), None);
                    }
                    Err(ref e) => {
                        log_container_error(&AppLogger::new(LOG_DOMAIN), e);
                        (cb.on_toast)(&format!("{}: {e}", gettext("Create failed")));
                    }
                }
            },
        );
    });
    dialog.present();
}
