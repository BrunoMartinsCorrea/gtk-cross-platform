---
description: Apply GTK4/GLib idiomatic patterns from docs/conceptual-improvements.md — GObject wrappers for domain models, GListModel + SignalListItemFactory, FilterListModel + CustomFilter, reactive property bindings, and CustomSorter. Replaces the imperative ListBox rebuild pattern across all four resource views.
---

# Apply Conceptual Improvements

> Read `CLAUDE.md` and `docs/conceptual-improvements.md` before writing a single line of code.
> Architecture rules, layer boundaries, threading model, A11Y, and i18n requirements are in `CLAUDE.md`.
> The conceptual analysis and migration diagrams are in `docs/conceptual-improvements.md`.

Run `make test` after every phase. Run `make fmt-fix` before the final commit.

---

## Prerequisites

Verify these conditions before starting:

- `src/window/objects/` does **not** exist (greenfield for GObject wrappers)
- `src/window/components/list_factory.rs` exists (reuse it — do not rewrite)
- `src/core/domain/container.rs`, `image.rs`, `volume.rs`, `network.rs` are plain Rust structs
- All views use `gtk4::ListBox` + imperative `append()` / `remove()` in `populate()` functions

---

## Phase 1 — GObject wrappers for domain models

**Goal:** Create `src/window/objects/` with four GObject types that wrap the plain domain structs.
This is the prerequisite for all other phases.

### Rules

- GObject wrappers live exclusively in `src/window/objects/` — never in `src/core/`
- Domain structs (`Container`, `Image`, `Volume`, `Network`) remain unchanged
- Each wrapper exposes GObject properties using `#[derive(Properties)]` from `glib`
- Each wrapper provides a `from_domain(x: &DomainType) -> Self` constructor
- No GTK imports inside `src/window/objects/` — only `glib`, `gio`, and domain types

### Files to create

**`src/window/objects/mod.rs`**

```rust
mod container_object;
mod image_object;
mod network_object;
mod volume_object;

pub use container_object::ContainerObject;
pub use image_object::ImageObject;
pub use network_object::NetworkObject;
pub use volume_object::VolumeObject;
```

**`src/window/objects/container_object.rs`**

Expose these properties: `id` (String), `name` (String), `status` (String), `image` (String),
`short_id` (String), `compose_project` (String — empty string when `None`).

```rust
use glib::prelude::*;
use glib::subclass::prelude::*;
use glib::Properties;
use std::cell::RefCell;
use crate::core::domain::container::Container;

mod imp {
    use super::*;

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::ContainerObject)]
    pub struct ContainerObject {
        #[property(get, set)] pub id: RefCell<String>,
        #[property(get, set)] pub name: RefCell<String>,
        #[property(get, set)] pub status: RefCell<String>,
        #[property(get, set)] pub image: RefCell<String>,
        #[property(get, set)] pub short_id: RefCell<String>,
        #[property(get, set)] pub compose_project: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ContainerObject {
        const NAME: &'static str = "GtkCrossPlatformContainerObject";
        type Type = super::ContainerObject;
    }

    impl ObjectImpl for ContainerObject {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }
        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }
        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }
}

glib::wrapper! {
    pub struct ContainerObject(ObjectSubclass<imp::ContainerObject>);
}

impl ContainerObject {
    pub fn from_domain(c: &Container) -> Self {
        let obj: Self = glib::Object::new();
        obj.set_id(c.id.clone());
        obj.set_name(c.name.clone());
        obj.set_status(c.status.to_string());
        obj.set_image(c.image.clone());
        obj.set_short_id(c.short_id.clone());
        obj.set_compose_project(c.compose_project.clone().unwrap_or_default());
        obj
    }
}
```

Follow the same pattern for `ImageObject`, `VolumeObject`, `NetworkObject`, exposing the
most-used display fields from each domain struct.

**`ImageObject` properties:** `id`, `tags` (String — first tag or empty), `size` (u64 → use
`i64` for GObject compat), `created` (String — formatted date).

**`VolumeObject` properties:** `name`, `driver`, `mountpoint`, `scope`, `size_bytes` (i64).

**`NetworkObject` properties:** `id`, `name`, `driver`, `scope`, `containers_count` (i64).

### Wire into src/window/mod.rs

Add `pub mod objects;` to `src/window/mod.rs`.

### Verification

```sh
make build   # must compile with zero warnings
make test    # all existing tests must pass
```

---

## Phase 2 — GListModel + SignalListItemFactory (flat views first)

**Goal:** Migrate `ImagesView`, `VolumesView`, `NetworksView` from `ListBox` + `populate()` to
`gio::ListStore<XObject>` + `gtk4::ListView` + `SignalListItemFactory`.

Start with these three flat views before tackling `ContainersView` (Phase 2b, more complex due to
compose-project grouping).

### Migration pattern (apply to each flat view)

1. Replace the `gtk4::ListBox` field with `gio::ListStore<XObject>` (owned by `Inner`).
2. Replace the `populate()` function with `refresh_store(store: &gio::ListStore<XObject>, items: Vec<DomainType>)`.
3. Wire a `gtk4::SignalListItemFactory` via `make_factory()` from `list_factory.rs` (see Factory
   setup below).
4. Wrap the store in **`gtk4::SingleSelection`** — all four resource views drive a detail pane from
   the selected item. `NoSelection` would silently break the detail pane.
5. Create a `gtk4::ListView` bound to the selection model.
6. Replace the old `ListBox` container in the UI template or build the `ListView` programmatically.

```
gio::ListStore<XObject>
    ↓
gtk4::SingleSelection           ← required: drives the detail pane
    ↓
gtk4::ListView + SignalListItemFactory
```

### Factory setup (via list_factory.rs)

`list_factory.rs` exposes `make_factory<S, B>(setup, bind)`. Both closures receive a
`&glib::Object` that must be downcast to `gtk4::ListItem`. Use it as follows:

```rust
use crate::window::components::list_factory::make_factory;

let factory = make_factory(
    |_, obj| {
        let item = obj.downcast_ref::<gtk4::ListItem>().unwrap();
        let row = adw::ActionRow::new();
        item.set_child(Some(&row));
    },
    |_, obj| {
        let item = obj.downcast_ref::<gtk4::ListItem>().unwrap();
        let row = item.child().and_downcast::<adw::ActionRow>().unwrap();
        let image_obj = item.item().and_downcast::<ImageObject>().unwrap();
        row.set_title(&image_obj.tags());
        row.set_subtitle(&image_obj.size().to_string());
    },
);
```

### Detail pane rewiring

The current `connect_row_selected` on `ListBox` must be replaced with
`connect_selection_changed` on `SingleSelection` after migration:

```rust
selection.connect_selection_changed(glib::clone!(
    #[weak] inner,
    move |sel, _, _| {
        if let Some(obj) = sel.selected_item().and_downcast::<ImageObject>() {
            inner.detail_pane.show_image(&obj);
        }
    }
));
```

### Signal blocking during programmatic refresh

`ImagesView` and `VolumesView` currently use `list.block_signal(id)` / `unblock_signal(id)` to
prevent the `connect_row_selected` handler from firing when the list is rebuilt programmatically.
After migration, apply the same guard on the `SingleSelection`:

```rust
// In Inner, store the handler id from connect_selection_changed:
let handler_id = selection.connect_selection_changed(…);
inner.selection_handler_id = Some(handler_id);

// During refresh_store:
inner.selection.block_signal(inner.selection_handler_id.as_ref().unwrap());
inner.store.remove_all();
for item in items { inner.store.append(&XObject::from_domain(&item)); }
inner.selection.unblock_signal(inner.selection_handler_id.as_ref().unwrap());
```

### Rules

- Each `ListStore` lives in `Inner` — one store per view
- `refresh_store` calls `store.remove_all()` then appends in a loop — equivalent to the current
  `populate()` but operates on the model, not on widgets
- The factory closure must hold only a **weak reference** to any `Rc<Inner>` — never a strong ref
- Row height must respect the 44 sp minimum touch target (`adw::ActionRow` already satisfies this)
- All user-visible strings in factory `connect_bind` must be wrapped in `gettext()`

---

## Phase 2b — ContainersView migration

**Goal:** Migrate `ContainersView` from its multi-`ListBox` / `active_lists: RefCell<Vec<ListBox>>`
pattern to a single `gio::ListStore<ContainerObject>` + `ListView`. Compose-project grouping is
handled in Phase 5 via `SortListModel` + `set_header_factory`; do not reimplement `group_by_compose()`
here — remove it and leave grouping flat until Phase 5.

**Prerequisite:** Phase 2 complete for the three flat views.

### Migration steps

1. Replace `active_lists: RefCell<Vec<gtk4::ListBox>>` with `store: gio::ListStore<ContainerObject>`.
2. Remove the `group_by_compose()` function — Phase 5 replaces it.
3. Apply the same `make_factory` / `SingleSelection` / `connect_selection_changed` pattern from
   Phase 2 flat views.
4. Keep `connect_row_selected` → `connect_selection_changed` replacement and signal blocking (same
   as Phase 2).

### Verification

```sh
make test
make run   # containers list flat (no grouping yet); detail pane still works
```

---

## Phase 3 — FilterListModel + CustomFilter for reactive search

**Goal:** Replace `repopulate()` on every keystroke with a `gtk4::FilterListModel` that filters
in-place without recreating widgets.

**Prerequisite:** Phase 2 completed for the target view.

### Migration pattern

For each migrated view, after creating the `gio::ListStore<XObject>`:

```rust
let filter = gtk4::CustomFilter::new(glib::clone!(
    #[weak] search_entry,
    move |obj| {
        let query = search_entry.text().to_lowercase();
        if query.is_empty() { return true; }
        let item = obj.downcast_ref::<ImageObject>().unwrap();
        item.tags().to_lowercase().contains(&query)
    }
));
let filter_model = gtk4::FilterListModel::new(Some(store.clone()), Some(filter.clone()));
search_entry.connect_changed(glib::clone!(
    #[weak] filter,
    move |_| filter.changed(gtk4::FilterChange::Different)
));
// Pass filter_model (not store) to SingleSelection from Phase 2
let selection = gtk4::SingleSelection::new(Some(filter_model.clone()));
```

### Domain filter integration

`filter_containers()` in `src/core/domain/container.rs` is the only public domain filter function
— it is pure and remains testable. `Image`, `Volume`, and `Network` have no equivalent public
filter functions; inline `CustomFilter` closures are sufficient for those views. Do not delete
`filter_containers()` — it is independently tested.

### Empty state

When `filter_model.n_items() == 0`, show the `EmptyState` component. Discriminate between two
distinct states using the search query:

```rust
filter_model.connect_items_changed(glib::clone!(
    #[weak] inner,
    move |model, _, _, _| {
        let empty = model.n_items() == 0;
        inner.list_view.set_visible(!empty);
        inner.empty_state.set_visible(empty);
        if empty {
            let query = inner.search_entry.text();
            if query.is_empty() {
                inner.empty_state.set_title(&gettext("No images"));
                inner.empty_state.set_description(None);
            } else {
                inner.empty_state.set_title(&gettext("No results"));
                inner.empty_state.set_description(Some(&format!(
                    "{} «{}»", gettext("No items match"), query
                )));
            }
        }
    }
));
```

### Verification

```sh
make test
make run   # type in search box — list must filter without full rebuild
```

---

## Phase 4 — Reactive property bindings for UI state

**Goal:** Replace imperative `set_sensitive()` / `set_visible()` calls that update buttons and
labels based on container state with declarative `bind_property()` bindings.

**Prerequisite:** Phase 1 (GObject wrappers with properties).

### Target bindings (apply where the pattern already exists imperatively)

```rust
// "Start" button disabled when container is already running
container_object
    .bind_property("status", &start_btn, "sensitive")
    .transform_to(|_, status: String| Some(status != "running"))
    .sync_create()
    .build();

// "Stop" button disabled when container is not running
container_object
    .bind_property("status", &stop_btn, "sensitive")
    .transform_to(|_, status: String| Some(status == "running"))
    .sync_create()
    .build();

// Status label kept in sync automatically
container_object
    .bind_property("status", &status_label, "label")
    .sync_create()
    .build();
```

### Binding lifecycle in a recycled ListView

`ListView` recycles widgets — the same row widget is rebound to different data objects as the user
scrolls. Without cleanup, each `connect_bind` call layers a new binding on top of the old one.
Store bindings in the `ListItem` via `set_data` and drop them in `connect_unbind`:

```rust
factory.connect_bind(|_, obj| {
    let item = obj.downcast_ref::<gtk4::ListItem>().unwrap();
    let row = item.child().and_downcast::<adw::ActionRow>().unwrap();
    let container = item.item().and_downcast::<ContainerObject>().unwrap();

    let b1 = container
        .bind_property("status", &start_btn, "sensitive")
        .transform_to(|_, status: String| Some(status != "running"))
        .sync_create()
        .build();
    let b2 = container
        .bind_property("status", &stop_btn, "sensitive")
        .transform_to(|_, status: String| Some(status == "running"))
        .sync_create()
        .build();
    let b3 = container
        .bind_property("status", &status_label, "label")
        .sync_create()
        .build();

    // Store so connect_unbind can drop them
    unsafe { item.set_data("bindings", vec![b1, b2, b3]); }
});

factory.connect_unbind(|_, obj| {
    let item = obj.downcast_ref::<gtk4::ListItem>().unwrap();
    // Drop stored bindings; the Vec<Binding> destructor calls unbind() on each
    unsafe { item.steal_data::<Vec<glib::Binding>>("bindings"); }
});
```

### Rules

- Apply bindings inside the factory `connect_bind` closure, not at construction time
- **Always** pair `connect_bind` with `connect_unbind` when storing bindings — omitting
  `connect_unbind` causes stale bindings to accumulate silently as rows are recycled
- Keep `set_sensitive()` calls for states that depend on **multiple** properties simultaneously
  (those cannot be expressed as a single `bind_property`)

### Verification

```sh
make test
make run   # start/stop buttons must reflect container state without manual refresh
```

---

## Phase 5 — CustomSorter + SortListModel

**Goal:** Add interactive sorting and restore the compose-project grouping for `ContainersView`
using `gtk4::SortListModel` instead of manual `group_by_compose()`.

**Prerequisite:** Phase 2 and 3 complete.

### Default sorter (all views)

```rust
let sorter = gtk4::CustomSorter::new(move |a, b| {
    let a = a.downcast_ref::<ContainerObject>().unwrap();
    let b = b.downcast_ref::<ContainerObject>().unwrap();
    // Running first, then alphabetical by name
    match (a.status().as_str(), b.status().as_str()) {
        ("running", "running") => a.name().cmp(&b.name()).into(),
        ("running", _) => gtk4::Ordering::Smaller,
        (_, "running") => gtk4::Ordering::Larger,
        _ => a.name().cmp(&b.name()).into(),
    }
});
let sort_model = gtk4::SortListModel::new(Some(filter_model), Some(sorter));
```

### Compose grouping for ContainersView

Replace the manual `group_by_compose()` function with a `CustomSorter` that sorts by
`compose_project` first, then by `name` within each group. Add section headers using
`gtk4::ListView`'s header factory (`set_header_factory`) — available since GTK 4.12.

```rust
let section_sorter = gtk4::CustomSorter::new(move |a, b| {
    let a = a.downcast_ref::<ContainerObject>().unwrap();
    let b = b.downcast_ref::<ContainerObject>().unwrap();
    let proj_a = a.compose_project();
    let proj_b = b.compose_project();
    match (proj_a.is_empty(), proj_b.is_empty()) {
        (false, true) => gtk4::Ordering::Smaller,   // compose groups first
        (true, false) => gtk4::Ordering::Larger,
        _ => proj_a.cmp(&proj_b).then(a.name().cmp(&b.name())).into(),
    }
});
```

The header factory creates an `adw::ActionRow` (or plain `gtk4::Label`) displaying the
`compose_project` name. Use `list_item.position()` and compare with the previous item to decide
whether to render a header.

### Verification

```sh
make test
make run   # containers grouped by compose project, running containers first within each group
```

---

## Exit criteria

All phases complete when:

- [ ] `make build` produces zero warnings
- [ ] `make test` passes all tests
- [ ] `make lint` passes (no clippy warnings)
- [ ] `make fmt` shows no diff
- [ ] Visual smoke test: all four resource views (containers, images, volumes, networks) display
      correctly; search filters reactively; compose grouping is preserved
- [ ] No `ListBox` + imperative `append()`/`remove()` pattern remains in
      `src/window/views/` (grep-verify)
- [ ] `src/window/objects/` contains four files: `container_object.rs`, `image_object.rs`,
      `volume_object.rs`, `network_object.rs`

```sh
# Final verification commands
make fmt-fix
make build
make test
make lint
grep -r "\.append(" src/window/views/ | grep -v "//"  # should be empty or only non-list appends
```

---

## What NOT to change

- `src/core/domain/` — domain structs remain plain Rust; no GObject imports
- `src/ports/` — port traits are unchanged
- `src/infrastructure/` — driver adapters are unchanged
- `spawn_driver_task` threading model — unchanged
- The `filter_containers()` domain function — keep it; it's independently tested
- `list_factory.rs` — extend it if needed, but do not delete it
