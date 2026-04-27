// SPDX-License-Identifier: GPL-3.0-or-later
use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use adw::prelude::*;
use gettextrs::{gettext, pgettext};
use glib;
use gtk4::gio;

use gtk_cross_platform::core::domain::network::CreateNetworkOptions;
use gtk_cross_platform::infrastructure::containers::background::spawn_driver_task;
use gtk_cross_platform::infrastructure::containers::error::log_container_error;
use gtk_cross_platform::infrastructure::logging::app_logger::AppLogger;
use gtk_cross_platform::ports::use_cases::i_network_use_case::INetworkUseCase;

use crate::window::components::{
    clear_box, confirm_dialog, detail_pane, empty_state::EmptyState, resource_row,
};
use crate::window::objects::NetworkObject;
use crate::window::utils::store::find_store_position;

const LOG_DOMAIN: &str = concat!(env!("APP_ID"), ".view.networks");

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
    use_case: Arc<dyn INetworkUseCase>,
    on_toast: Rc<dyn Fn(&str)>,
    on_loading: Rc<dyn Fn(bool)>,
    loading: Cell<bool>,
    loaded: Cell<bool>,
    selection_handler: std::cell::RefCell<Option<glib::SignalHandlerId>>,
    list_cancellable: std::cell::RefCell<Option<gio::Cancellable>>,
}

#[derive(Clone)]
pub struct NetworksView(Rc<Inner>);

impl NetworksView {
    pub fn new(
        use_case: Arc<dyn INetworkUseCase>,
        detail_content: gtk4::Box,
        detail_stack: gtk4::Stack,
        on_toast: impl Fn(&str) + 'static,
        on_loading: impl Fn(bool) + 'static,
    ) -> Self {
        let store = gio::ListStore::new::<NetworkObject>();

        let search_entry = gtk4::SearchEntry::new();
        search_entry.set_hexpand(true);
        search_entry.set_placeholder_text(Some(&gettext("Search networks…")));

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
            let net = obj.downcast_ref::<NetworkObject>().unwrap();
            net.name().to_ascii_lowercase().contains(&q)
                || net.driver().to_ascii_lowercase().contains(&q)
                || net.scope().to_ascii_lowercase().contains(&q)
        });

        let filter_model = gtk4::FilterListModel::new(Some(store.clone()), Some(filter.clone()));

        let sorter = gtk4::CustomSorter::new(|a, b| {
            let a = a.downcast_ref::<NetworkObject>().unwrap();
            let b = b.downcast_ref::<NetworkObject>().unwrap();
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
        create_btn.set_label(&gettext("New Network"));
        create_btn.add_css_class("pill");
        create_btn.set_halign(gtk4::Align::Center);
        create_btn.set_margin_top(4);
        create_btn.set_margin_bottom(4);
        create_btn.set_tooltip_text(Some(&gettext("Create a new network")));
        create_btn.update_property(&[gtk4::accessible::Property::Label(&gettext("New network"))]);

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
            "network-wired-symbolic",
            "No Networks",
            "No networks found.",
        );
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
                    &pgettext("network action", "Remove network"),
                );
                row.add_suffix(&remove_btn);

                let item_weak = item.downgrade();
                let iw2 = iw.clone();
                remove_btn.connect_clicked(move |btn| {
                    let Some(item) = item_weak.upgrade() else {
                        return;
                    };
                    let Some(inner) = iw2.upgrade() else { return };
                    let Some(net_obj) = item.item().and_downcast::<NetworkObject>() else {
                        return;
                    };

                    let id = net_obj.id();
                    let name = net_obj.name();
                    let idx =
                        find_store_position::<NetworkObject, _>(&inner.store, |o| o.id() == id);
                    let body = gettext("Remove network \"{name}\"?").replace("{name}", &name);
                    let inner2 = inner.clone();
                    confirm_dialog::ask(
                        btn,
                        &gettext("Remove Network?"),
                        &body,
                        &pgettext("network action", "Remove"),
                        move || {
                            let log = AppLogger::new(LOG_DOMAIN);
                            log.debug(&format!("Removing network {id}"));
                            let use_case = inner2.use_case.clone();
                            let id2 = id.clone();
                            let cb = inner2.clone();
                            spawn_driver_task(
                                use_case,
                                move |uc| uc.remove(&id2),
                                move |result| match result {
                                    Ok(()) => {
                                        (cb.on_toast)(&gettext("Network removed"));
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
            let net_obj = item.item().and_downcast::<NetworkObject>().unwrap();

            let subnet = if net_obj.subnet().is_empty() {
                "—".to_string()
            } else {
                net_obj.subnet()
            };
            let container_label = if net_obj.containers_count() == 1 {
                format!("1 {}", gettext("container"))
            } else {
                format!("{} {}", net_obj.containers_count(), gettext("containers"))
            };
            let subtitle = format!(
                "{} · {} · {} · {}",
                net_obj.driver(),
                net_obj.scope(),
                subnet,
                container_label
            );
            row.set_title(&net_obj.name());
            row.set_subtitle(&subtitle);

            // Remove button is the sole suffix added at setup time; find it by sibling walk.
            let mut child = row.last_child();
            while let Some(w) = child {
                if w.is::<gtk4::Button>() {
                    w.set_visible(!is_system_network(&net_obj.name()));
                    break;
                }
                child = w.prev_sibling();
            }
        });

        self.0.list_view.set_factory(Some(&factory));

        // Selection → detail pane
        let iw = inner_weak.clone();
        let handler_id = self
            .0
            .selection
            .connect_selection_changed(move |sel, _, _| {
                let Some(inner) = iw.upgrade() else { return };
                if let Some(obj) = sel.selected_item().and_downcast::<NetworkObject>() {
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

        // Create network dialog
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
    log.info("Loading networks list");
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
                Ok(networks) => {
                    log.info(&format!("Loaded {} networks", networks.len()));
                    refresh_store(&cb, &networks);
                    if let Some(idx) = focus_after_remove {
                        focus_after_store_update(&cb, idx);
                    }
                }
                Err(ref e) => {
                    log_container_error(&log, e);
                    (cb.on_toast)(&format!("{}: {e}", gettext("Failed to load networks")));
                }
            }
        },
    );
}

// ── Store helpers ─────────────────────────────────────────────────────────────

fn refresh_store(
    inner: &Rc<Inner>,
    networks: &[gtk_cross_platform::core::domain::network::Network],
) {
    if let Some(id) = inner.selection_handler.borrow().as_ref() {
        inner.selection.block_signal(id);
    }
    inner.store.remove_all();
    for net in networks {
        inner.store.append(&NetworkObject::from_domain(net));
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
        .and_downcast::<NetworkObject>()
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
                gettext("No networks matched"),
                inner.search_entry.text()
            )));
        } else {
            inner
                .empty_status
                .set_icon_name(Some("network-wired-symbolic"));
            inner.empty_status.set_title(&gettext("No Networks"));
            inner
                .empty_status
                .set_description(Some(&gettext("No networks found.")));
        }
        inner.list_stack.set_visible_child_name("empty");
    } else {
        inner.list_stack.set_visible_child_name("list");
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn is_system_network(name: &str) -> bool {
    matches!(name, "bridge" | "host" | "none")
}

// ── Detail pane ───────────────────────────────────────────────────────────────

fn show_detail(inner: &Rc<Inner>, obj: &NetworkObject) {
    clear_box(&inner.detail_content);

    let subnet = if obj.subnet().is_empty() {
        "—".to_string()
    } else {
        obj.subnet()
    };
    let gateway = if obj.gateway().is_empty() {
        "—".to_string()
    } else {
        obj.gateway()
    };
    let internal_str = if obj.internal() {
        gettext("Yes")
    } else {
        gettext("No")
    };

    let pane = detail_pane::build(&[detail_pane::PropertyGroup {
        title: String::new(),
        rows: vec![
            (gettext("Name"), obj.name()),
            (gettext("Driver"), obj.driver()),
            (gettext("Scope"), obj.scope()),
            (gettext("Subnet"), subnet),
            (gettext("Gateway"), gateway),
            (gettext("Internal"), internal_str),
            (gettext("Containers"), obj.containers_count().to_string()),
        ],
    }]);
    inner.detail_content.append(&pane);

    if is_system_network(&obj.name()) {
        let hint = EmptyState::in_clamp(EmptyState::no_items(
            "dialog-information-symbolic",
            "System Network",
            "System network — cannot be removed.",
        ));
        inner.detail_content.append(&hint);
    }

    inner.detail_stack.set_visible_child_name("detail");
}

// ── Create network dialog ─────────────────────────────────────────────────────

fn show_create_dialog(trigger: &impl gtk4::prelude::IsA<gtk4::Widget>, inner: Rc<Inner>) {
    let window = trigger.root().and_downcast::<gtk4::Window>();
    let dialog = adw::MessageDialog::new(window.as_ref(), Some(&gettext("New Network")), None);
    dialog.add_response("cancel", &gettext("Cancel"));
    dialog.add_response("create", &pgettext("network action", "Create"));
    dialog.set_response_appearance("create", adw::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("create"));
    dialog.set_close_response("cancel");

    let form = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    form.set_margin_top(8);

    let name_entry = gtk4::Entry::new();
    name_entry.set_placeholder_text(Some(&gettext("Network name")));
    name_entry.set_activates_default(true);
    form.append(&name_entry);

    let driver_label = gtk4::Label::new(Some(&gettext("Driver")));
    driver_label.set_halign(gtk4::Align::Start);
    driver_label.add_css_class("caption");
    form.append(&driver_label);

    let drivers = gtk4::StringList::new(&["bridge", "overlay", "macvlan", "ipvlan"]);
    let driver_drop = gtk4::DropDown::new(Some(drivers), gtk4::Expression::NONE);
    driver_drop.set_selected(0);
    form.append(&driver_drop);

    let subnet_entry = gtk4::Entry::new();
    subnet_entry.set_placeholder_text(Some("172.20.0.0/16"));
    form.append(&subnet_entry);

    let subnet_hint = gtk4::Label::new(Some(&gettext("Subnet (optional, CIDR)")));
    subnet_hint.set_halign(gtk4::Align::Start);
    subnet_hint.add_css_class("caption");
    subnet_hint.add_css_class("dim-label");
    form.append(&subnet_hint);

    dialog.set_extra_child(Some(&form));

    let on_toast = inner.on_toast.clone();
    let trigger_weak = trigger.upcast_ref::<gtk4::Widget>().downgrade();
    dialog.connect_response(None, move |_, response| {
        if let Some(w) = trigger_weak.upgrade() {
            w.grab_focus();
        }
        if response != "create" {
            return;
        }
        let name = name_entry.text().trim().to_string();
        if name.is_empty() {
            return;
        }
        let driver = driver_drop
            .selected_item()
            .and_downcast::<gtk4::StringObject>()
            .map(|s| s.string().to_string())
            .unwrap_or_else(|| "bridge".to_string());
        let subnet_raw = subnet_entry.text().trim().to_string();
        let subnet = if subnet_raw.is_empty() {
            None
        } else if is_valid_cidr(&subnet_raw) {
            Some(subnet_raw)
        } else {
            on_toast(&gettext(
                "Invalid subnet — expected CIDR notation (e.g. 172.20.0.0/16)",
            ));
            return;
        };
        let opts = CreateNetworkOptions {
            name: name.clone(),
            driver,
            subnet,
        };
        let log = AppLogger::new(LOG_DOMAIN);
        log.debug(&format!("Creating network {name}"));
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
                        (cb.on_toast)(&gettext("Network created"));
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

fn is_valid_cidr(s: &str) -> bool {
    let Some((addr, prefix)) = s.split_once('/') else {
        return false;
    };
    let parts: Vec<&str> = addr.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    if !parts.iter().all(|p| p.parse::<u8>().is_ok()) {
        return false;
    }
    prefix.parse::<u8>().map(|p| p <= 32).unwrap_or(false)
}
