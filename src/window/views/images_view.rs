// SPDX-License-Identifier: GPL-3.0-or-later
use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use adw::prelude::*;
use gettextrs::{gettext, pgettext};
use glib;
use gtk4::gio;

use gtk_cross_platform::core::domain::image::Image;
use gtk_cross_platform::infrastructure::containers::background::spawn_driver_task;
use gtk_cross_platform::infrastructure::containers::error::log_container_error;
use gtk_cross_platform::infrastructure::logging::app_logger::AppLogger;
use gtk_cross_platform::ports::use_cases::i_image_use_case::IImageUseCase;

use crate::window::components::{confirm_dialog, detail_pane, empty_state::EmptyState, resource_row};
use crate::window::objects::ImageObject;

const LOG_DOMAIN: &str = concat!(env!("APP_ID"), ".view.images");

type OnToastDestructive = Rc<dyn Fn(&str, &str)>;

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
    use_case: Arc<dyn IImageUseCase>,
    on_toast: Rc<dyn Fn(&str)>,
    on_toast_destructive: OnToastDestructive,
    on_loading: Rc<dyn Fn(bool)>,
    on_run_image: Rc<dyn Fn(&str)>,
    loading: Cell<bool>,
    loaded: Cell<bool>,
    selection_handler: std::cell::RefCell<Option<glib::SignalHandlerId>>,
    list_cancellable: std::cell::RefCell<Option<gio::Cancellable>>,
    detail_cancellable: std::cell::RefCell<Option<gio::Cancellable>>,
}

#[derive(Clone)]
pub struct ImagesView(Rc<Inner>);

impl ImagesView {
    pub fn new(
        use_case: Arc<dyn IImageUseCase>,
        detail_content: gtk4::Box,
        detail_stack: gtk4::Stack,
        on_toast: impl Fn(&str) + 'static,
        on_toast_destructive: impl Fn(&str, &str) + 'static,
        on_loading: impl Fn(bool) + 'static,
        on_run_image: impl Fn(&str) + 'static,
    ) -> Self {
        let store = gio::ListStore::new::<ImageObject>();

        let search_entry = gtk4::SearchEntry::new();
        search_entry.set_hexpand(true);
        search_entry.set_placeholder_text(Some(&gettext("Search images…")));

        let search_bar = gtk4::SearchBar::new();
        search_bar.set_search_mode(false);
        search_bar.set_show_close_button(true);
        search_bar.set_child(Some(&search_entry));

        // Filter: matches against primary tag and short_id
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
            let img = obj.downcast_ref::<ImageObject>().unwrap();
            img.tags().to_ascii_lowercase().contains(&q)
                || img.short_id().to_ascii_lowercase().contains(&q)
        });

        let filter_model =
            gtk4::FilterListModel::new(Some(store.clone()), Some(filter.clone()));

        // Sort alphabetically by primary tag
        let sorter = gtk4::CustomSorter::new(|a, b| {
            let a = a.downcast_ref::<ImageObject>().unwrap();
            let b = b.downcast_ref::<ImageObject>().unwrap();
            a.tags().cmp(&b.tags()).into()
        });
        let sort_model =
            gtk4::SortListModel::new(Some(filter_model.clone()), Some(sorter));

        let selection = gtk4::SingleSelection::new(Some(sort_model.clone()));
        selection.set_autoselect(false);

        let list_view = gtk4::ListView::new(Some(selection.clone()), None::<gtk4::SignalListItemFactory>);
        list_view.add_css_class("boxed-list");
        list_view.set_hexpand(true);
        list_view.set_show_separators(true);

        // Pull image button
        let pull_btn = gtk4::Button::new();
        pull_btn.set_icon_name("list-add-symbolic");
        pull_btn.set_label(&gettext("Pull Image"));
        pull_btn.add_css_class("pill");
        pull_btn.set_halign(gtk4::Align::Center);
        pull_btn.set_margin_top(4);
        pull_btn.set_margin_bottom(4);
        pull_btn.set_tooltip_text(Some(&gettext("Pull a new image")));
        pull_btn.update_property(&[gtk4::accessible::Property::Label(&gettext("Pull image"))]);

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
            "image-x-generic-symbolic",
            "No Images",
            "No local images found.",
        );
        empty_status.set_vexpand(true);

        let list_stack = gtk4::Stack::new();
        list_stack.set_vexpand(true);
        list_stack.add_named(&scroll, Some("list"));
        list_stack.add_named(&empty_status, Some("empty"));

        let sidebar_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        sidebar_box.append(&search_bar);
        sidebar_box.append(&pull_btn);
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
            on_toast_destructive: Rc::new(on_toast_destructive),
            on_loading: Rc::new(on_loading),
            on_run_image: Rc::new(on_run_image),
            loading: Cell::new(false),
            loaded: Cell::new(false),
            selection_handler: std::cell::RefCell::new(None),
            list_cancellable: std::cell::RefCell::new(None),
            detail_cancellable: std::cell::RefCell::new(None),
        });

        let view = Self(inner);
        view.wire_signals(&search_bar, &pull_btn);
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

    fn wire_signals(&self, search_bar: &gtk4::SearchBar, pull_btn: &gtk4::Button) {
        // ── Factory ──────────────────────────────────────────────────────────
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
                    &pgettext("image action", "Remove image"),
                );
                row.add_suffix(&remove_btn);

                // Wire remove at setup time; reads current item at click time
                let item_weak = item.downgrade();
                let iw2 = iw.clone();
                remove_btn.connect_clicked(move |btn| {
                    let Some(item) = item_weak.upgrade() else { return };
                    let Some(inner) = iw2.upgrade() else { return };
                    let Some(img_obj) = item.item().and_downcast::<ImageObject>() else { return };

                    let id = img_obj.id();
                    let tag = img_obj.tags();
                    let idx = find_store_position(&inner.store, &id);
                    let body = gettext("Remove image \"{tag}\"?").replace("{tag}", &tag);
                    let inner2 = inner.clone();
                    confirm_dialog::ask(
                        btn,
                        &gettext("Remove Image?"),
                        &body,
                        &pgettext("image action", "Remove"),
                        move || {
                            let log = AppLogger::new(LOG_DOMAIN);
                            log.debug(&format!("Removing image {id}"));
                            let use_case = inner2.use_case.clone();
                            let id2 = id.clone();
                            let cb = inner2.clone();
                            spawn_driver_task(
                                use_case,
                                move |uc| uc.remove(&id2, false),
                                move |result| match result {
                                    Ok(()) => {
                                        (cb.on_toast_destructive)(
                                            &gettext("Image removed"),
                                            "win.undo-remove-image",
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

                item.set_child(Some(&row));
            });
        }

        factory.connect_bind(|_, obj| {
            let item = obj.downcast_ref::<gtk4::ListItem>().unwrap();
            let row = item.child().and_downcast::<adw::ActionRow>().unwrap();
            let img_obj = item.item().and_downcast::<ImageObject>().unwrap();

            let mut subtitle =
                format!("{} · {}", img_obj.short_id(), fmt_bytes(img_obj.size() as u64));
            if img_obj.is_dangling() {
                subtitle.push_str(&format!(" · {}", gettext("dangling")));
            }
            row.set_title(&img_obj.tags());
            row.set_subtitle(&subtitle);
        });

        self.0.list_view.set_factory(Some(&factory));

        // ── Selection → detail pane ──────────────────────────────────────────
        let iw = inner_weak.clone();
        let handler_id = self.0.selection.connect_selection_changed(move |sel, _, _| {
            let Some(inner) = iw.upgrade() else { return };
            if let Some(obj) = sel.selected_item().and_downcast::<ImageObject>() {
                show_detail(&inner, &obj);
            } else {
                inner.detail_stack.set_visible_child_name("empty");
            }
        });
        *self.0.selection_handler.borrow_mut() = Some(handler_id);

        // ── Empty state watcher ───────────────────────────────────────────────
        {
            let iw = inner_weak.clone();
            self.0.filter_model.connect_items_changed(move |model, _, _, _| {
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
                if key == gtk4::gdk::Key::f
                    && mods.contains(gtk4::gdk::ModifierType::CONTROL_MASK)
                {
                    if let Some(sb) = sb_weak.upgrade() {
                        sb.set_search_mode(true);
                    }
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            });
            self.0.sidebar_box.add_controller(key_ctrl);
        }

        // Pull image dialog
        {
            let iw = inner_weak.clone();
            pull_btn.connect_clicked(move |btn| {
                let Some(inner) = iw.upgrade() else { return };
                show_pull_dialog(btn, inner);
            });
        }
    }
}

// ── Reload ────────────────────────────────────────────────────────────────────

fn reload_impl(inner: Rc<Inner>, focus_after_remove: Option<u32>) {
    let log = AppLogger::new(LOG_DOMAIN);
    log.info("Loading images list");
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
                Ok(images) => {
                    log.info(&format!("Loaded {} images", images.len()));
                    refresh_store(&cb, &images);
                    if let Some(idx) = focus_after_remove {
                        focus_after_store_update(&cb, idx);
                    }
                }
                Err(ref e) => {
                    log_container_error(&log, e);
                    (cb.on_toast)(&format!("{}: {e}", gettext("Failed to load images")));
                }
            }
        },
    );
}

// ── Store helpers ──────────────────────────────────────────────────────────────

fn refresh_store(inner: &Rc<Inner>, images: &[Image]) {
    // Block selection handler during programmatic store update
    if let Some(id) = inner.selection_handler.borrow().as_ref() {
        inner.selection.block_signal(id);
    }
    inner.store.remove_all();
    for img in images {
        inner.store.append(&ImageObject::from_domain(img));
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
    if let Some(obj) = inner.selection.selected_item().and_downcast::<ImageObject>() {
        show_detail(inner, &obj);
    }
}

fn find_store_position(store: &gio::ListStore, id: &str) -> Option<u32> {
    (0..store.n_items()).find(|&i| {
        store
            .item(i)
            .and_downcast::<ImageObject>()
            .map(|o| o.id() == id)
            .unwrap_or(false)
    })
}

fn update_empty_state(inner: &Rc<Inner>, n_items: u32) {
    if n_items == 0 {
        let is_searching = !inner.search_entry.text().is_empty();
        if is_searching {
            inner.empty_status.set_icon_name(Some("edit-find-symbolic"));
            inner.empty_status.set_title(&gettext("No Results"));
            inner.empty_status.set_description(Some(&format!(
                "{} \"{}\"",
                gettext("No images matched"),
                inner.search_entry.text()
            )));
        } else {
            inner
                .empty_status
                .set_icon_name(Some("image-x-generic-symbolic"));
            inner.empty_status.set_title(&gettext("No Images"));
            inner
                .empty_status
                .set_description(Some(&gettext("No local images found.")));
        }
        inner.list_stack.set_visible_child_name("empty");
    } else {
        inner.list_stack.set_visible_child_name("list");
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub fn fmt_bytes(b: u64) -> String {
    if b >= 1_073_741_824 {
        format!("{:.1} GB", b as f64 / 1_073_741_824.0)
    } else {
        format!("{:.0} MB", b as f64 / 1_048_576.0)
    }
}

// ── Detail pane ───────────────────────────────────────────────────────────────

fn show_detail(inner: &Rc<Inner>, obj: &ImageObject) {
    clear_box(&inner.detail_content);

    let created_str = {
        let secs = obj.created();
        if secs > 0 {
            let days = secs / 86_400;
            let hours = (secs % 86_400) / 3_600;
            if days > 0 {
                format!("{days}d ago")
            } else {
                format!("{hours}h ago")
            }
        } else {
            "—".to_string()
        }
    };

    // Action buttons row
    let btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    btn_row.set_margin_top(8);
    btn_row.set_margin_bottom(4);
    btn_row.set_margin_start(12);
    btn_row.set_margin_end(12);

    let run_split = adw::SplitButton::new();
    run_split.set_label(&gettext("Run"));
    run_split.add_css_class("suggested-action");
    run_split.add_css_class("pill");
    run_split.set_tooltip_text(Some(&gettext(
        "Create and start a container from this image",
    )));
    run_split.update_property(&[gtk4::accessible::Property::Label(&gettext(
        "Run container from image",
    ))]);

    let push_menu = gio::Menu::new();
    push_menu.append(
        Some(&gettext("Push to registry\u{2026}")),
        Some("win.push-stub"),
    );
    run_split.set_menu_model(Some(&push_menu));

    let on_run = inner.on_run_image.clone();
    let img_tag = obj.tags();
    run_split.connect_clicked(move |_| {
        on_run(&img_tag);
    });
    btn_row.append(&run_split);
    inner.detail_content.append(&btn_row);

    let notebook = gtk4::Notebook::new();
    notebook.set_vexpand(true);
    notebook.set_margin_top(4);

    // ── Info tab ──────────────────────────────────────────────
    let in_use_str = if obj.in_use() {
        gettext("Yes")
    } else {
        gettext("No")
    };
    let pane = detail_pane::build(&[detail_pane::PropertyGroup {
        title: String::new(),
        rows: vec![
            (gettext("Tag"), obj.tags()),
            (gettext("ID"), obj.id()),
            (gettext("Size"), fmt_bytes(obj.size() as u64)),
            (gettext("Created"), created_str),
            (
                gettext("Digest"),
                if obj.digest().is_empty() {
                    "—".to_string()
                } else {
                    obj.digest()
                },
            ),
            (gettext("In Use"), in_use_str),
        ],
    }]);
    let info_scroll = gtk4::ScrolledWindow::new();
    info_scroll.set_hscrollbar_policy(gtk4::PolicyType::Never);
    info_scroll.set_child(Some(&pane));
    let info_label = gtk4::Label::new(Some(&gettext("Info")));
    notebook.append_page(&info_scroll, Some(&info_label));

    // ── Layers tab ────────────────────────────────────────────
    let layers_outer = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    let layers_stack = gtk4::Stack::new();

    let spinner = gtk4::Spinner::new();
    spinner.set_halign(gtk4::Align::Center);
    spinner.set_valign(gtk4::Align::Center);
    spinner.set_spinning(true);
    layers_stack.add_named(&spinner, Some("loading"));

    let layers_list = gtk4::ListBox::new();
    layers_list.add_css_class("boxed-list-separate");
    let layers_scroll = gtk4::ScrolledWindow::new();
    layers_scroll.set_hscrollbar_policy(gtk4::PolicyType::Never);
    layers_scroll.set_vexpand(true);
    layers_scroll.set_child(Some(&layers_list));
    let layers_content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    layers_content.append(&layers_scroll);
    let total_lbl = gtk4::Label::new(None);
    total_lbl.add_css_class("dim-label");
    total_lbl.set_margin_top(4);
    total_lbl.set_margin_bottom(8);
    layers_content.append(&total_lbl);
    layers_stack.add_named(&layers_content, Some("content"));
    layers_outer.append(&layers_stack);

    let layers_label = gtk4::Label::new(Some(&gettext("Layers")));
    notebook.append_page(&layers_outer, Some(&layers_label));

    inner.detail_content.append(&notebook);
    inner.detail_stack.set_visible_child_name("detail");

    // Load layers asynchronously
    if let Some(c) = inner.detail_cancellable.borrow_mut().take() {
        c.cancel();
    }
    let detail_c = gio::Cancellable::new();
    *inner.detail_cancellable.borrow_mut() = Some(detail_c.clone());

    let uc = inner.use_case.clone();
    let img_id = obj.id();
    let on_toast = inner.on_toast.clone();
    let layers_stack_w = layers_stack.downgrade();
    let layers_list_w = layers_list.downgrade();
    let total_lbl_w = total_lbl.downgrade();
    spawn_driver_task(
        uc,
        move |uc| uc.layers(&img_id),
        move |result| {
            if detail_c.is_cancelled() {
                return;
            }
            let Some(stack) = layers_stack_w.upgrade() else {
                return;
            };
            match result {
                Ok(layers) => {
                    if let Some(list) = layers_list_w.upgrade() {
                        let total: u64 = layers.iter().map(|l| l.size).sum();
                        for layer in &layers {
                            let cmd_truncated = if layer.cmd.len() > 60 {
                                format!("{}…", &layer.cmd[..60])
                            } else {
                                layer.cmd.clone()
                            };
                            let row = adw::ActionRow::new();
                            row.set_title(&glib::markup_escape_text(&cmd_truncated));
                            row.set_subtitle(&format!(
                                "<span font_family=\"monospace\">{}</span> · {}",
                                glib::markup_escape_text(&layer.id),
                                fmt_bytes(layer.size)
                            ));
                            row.set_subtitle_selectable(true);
                            list.append(&row);
                        }
                        if let Some(lbl) = total_lbl_w.upgrade() {
                            lbl.set_text(&format!("{}: {}", gettext("Total"), fmt_bytes(total)));
                        }
                    }
                    stack.set_visible_child_name("content");
                }
                Err(e) => {
                    on_toast(&format!("{}: {e}", gettext("Failed to load layers")));
                    stack.set_visible_child_name("content");
                }
            }
        },
    );
}

// ── Pull image dialog ─────────────────────────────────────────────────────────

fn show_pull_dialog(parent: &gtk4::Button, inner: Rc<Inner>) {
    let Some(root) = parent.root().and_downcast::<gtk4::Window>() else {
        return;
    };

    let dialog = gtk4::Window::new();
    dialog.set_title(Some(&gettext("Pull Image")));
    dialog.set_transient_for(Some(&root));
    dialog.set_modal(true);
    dialog.set_default_size(440, 240);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    let header_bar = adw::HeaderBar::new();
    let cancel_btn = gtk4::Button::with_label(&gettext("Cancel"));
    cancel_btn.add_css_class("flat");
    header_bar.pack_start(&cancel_btn);
    let pull_btn = gtk4::Button::with_label(&gettext("Pull"));
    pull_btn.add_css_class("suggested-action");
    pull_btn.set_sensitive(false);
    header_bar.pack_end(&pull_btn);
    content.append(&header_bar);

    let input_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    input_box.set_margin_top(12);
    input_box.set_margin_bottom(12);
    input_box.set_margin_start(12);
    input_box.set_margin_end(12);

    let img_group = adw::PreferencesGroup::new();
    img_group.set_title(&gettext("Image Reference"));
    img_group.set_description(Some(&gettext("e.g. ubuntu:22.04, ghcr.io/org/app:v1")));
    let ref_row = adw::EntryRow::new();
    ref_row.set_title(&gettext("Registry/tag"));
    img_group.add(&ref_row);
    input_box.append(&img_group);

    let progress_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    progress_box.set_margin_top(12);
    progress_box.set_margin_bottom(12);
    progress_box.set_margin_start(12);
    progress_box.set_margin_end(12);
    progress_box.set_visible(false);

    let spinner = gtk4::Spinner::new();
    spinner.set_halign(gtk4::Align::Center);
    spinner.set_spinning(false);
    let progress_lbl = gtk4::Label::new(Some(&gettext("Pulling…")));
    progress_lbl.add_css_class("dim-label");
    progress_box.append(&spinner);
    progress_box.append(&progress_lbl);

    content.append(&input_box);
    content.append(&progress_box);
    dialog.set_child(Some(&content));

    let pull_btn_weak = pull_btn.downgrade();
    ref_row.connect_changed(move |row| {
        if let Some(btn) = pull_btn_weak.upgrade() {
            btn.set_sensitive(!row.text().is_empty());
        }
    });

    let dw = dialog.downgrade();
    cancel_btn.connect_clicked(move |_| {
        if let Some(d) = dw.upgrade() {
            d.close();
        }
    });

    let ref_w = ref_row.downgrade();
    let input_w = input_box.downgrade();
    let progress_w = progress_box.downgrade();
    let spinner_w = spinner.downgrade();
    let pull_btn_w = pull_btn.downgrade();
    let cancel_w = cancel_btn.downgrade();
    let dialog_w = dialog.downgrade();
    let cb = inner.clone();
    pull_btn.connect_clicked(move |_| {
        let reference = ref_w
            .upgrade()
            .map(|r| r.text().to_string())
            .unwrap_or_default();
        if reference.is_empty() {
            return;
        }

        if let Some(ib) = input_w.upgrade() {
            ib.set_visible(false);
        }
        if let Some(pb) = progress_w.upgrade() {
            pb.set_visible(true);
        }
        if let Some(sp) = spinner_w.upgrade() {
            sp.set_spinning(true);
        }
        if let Some(pb) = pull_btn_w.upgrade() {
            pb.set_sensitive(false);
        }
        if let Some(cb_btn) = cancel_w.upgrade() {
            cb_btn.set_sensitive(false);
        }

        let uc = cb.use_case.clone();
        let reference2 = reference.clone();
        let cb2 = cb.clone();
        let dw2 = dialog_w.clone();
        let spinner_w2 = spinner_w.clone();
        let progress_w2 = progress_w.clone();
        spawn_driver_task(
            uc,
            move |uc| uc.pull(&reference2),
            move |result| {
                if let Some(sp) = spinner_w2.upgrade() {
                    sp.set_spinning(false);
                }
                match result {
                    Ok(()) => {
                        (cb2.on_toast)(&format!("{}: {reference}", gettext("Image pulled")));
                        reload_impl(cb2.clone(), None);
                        if let Some(d) = dw2.upgrade() {
                            d.close();
                        }
                    }
                    Err(ref e) => {
                        log_container_error(&AppLogger::new(LOG_DOMAIN), e);
                        (cb2.on_toast)(&format!("{}: {e}", gettext("Pull failed")));
                        if let Some(pb) = progress_w2.upgrade() {
                            pb.set_visible(false);
                        }
                        if let Some(d) = dw2.upgrade() {
                            d.close();
                        }
                    }
                }
            },
        );
    });

    let key_ctrl = gtk4::EventControllerKey::new();
    let dw3 = dialog.downgrade();
    key_ctrl.connect_key_pressed(move |_, key, _, _| {
        if key == gtk4::gdk::Key::Escape {
            if let Some(d) = dw3.upgrade() {
                d.close();
            }
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    dialog.add_controller(key_ctrl);

    dialog.present();
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn clear_box(b: &gtk4::Box) {
    while let Some(child) = b.first_child() {
        b.remove(&child);
    }
}
