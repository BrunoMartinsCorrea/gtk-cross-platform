---
description: Decompose src/window/ into reusable components and per-resource views, shrinking main_window.rs to a thin shell.
---

Refactor `src/window/` into the component + view hierarchy below. Do not touch `src/core/`,
`src/ports/`, or `src/infrastructure/` unless strictly required to expose data that components need.

> Layer rules, breakpoints, and key types are in `CLAUDE.md`. Only `src/window/` is in scope here.

## Target file structure

```
src/window/
├── mod.rs
├── main_window.rs                  ← thin shell only; no row builders or populate_* methods
├── components/
│   ├── mod.rs
│   ├── status_badge.rs             ← colored pill for ContainerStatus (A11Y: text + color)
│   ├── resource_row.rs             ← adw::ActionRow builder + icon_button helper
│   ├── detail_pane.rs              ← scrollable adw::PreferencesGroup key-value grid
│   └── confirm_dialog.rs           ← adw::AlertDialog wrapper for destructive confirmations
└── views/
    ├── mod.rs
    ├── containers_view.rs          ← sidebar list + detail pane for containers
    ├── images_view.rs
    ├── volumes_view.rs
    └── networks_view.rs
```

**Rule:** components know nothing about each other; views compose components; `main_window.rs`
composes views. No widget module may import a sibling widget module.

## Component contracts

### `StatusBadge` — `src/window/components/status_badge.rs`

```rust
impl StatusBadge {
    pub fn new(status: &ContainerStatus) -> Self;
    pub fn set_status(&self, status: &ContainerStatus);
}
```

- Applies `.status-badge.<css_class>` from `ContainerStatus::css_class()`
- A11Y: `set_accessible_role(AccessibleRole::Status)` + `set_tooltip_text(status.label())`
- I18N: status label through `gettext!()`
- Must show both color AND text label — never color alone

### `ResourceRow` — `src/window/components/resource_row.rs`

```rust
impl ResourceRow {
    pub fn new(icon: &str, title: &str, subtitle: &str) -> Self;
    pub fn set_trailing(&self, widget: &impl IsA<gtk4::Widget>);
}

pub fn icon_button(icon_name: &str, tooltip: &str) -> gtk4::Button;
```

- All text pre-translated at call site with `gettext!()`
- `icon_button` sets both `set_tooltip_text` and `update_property(&[Property::Label(...)])`

### `DetailPane` — `src/window/components/detail_pane.rs`

```rust
pub struct PropertyGroup {
    pub title: String,           // translated group header
    pub rows: Vec<(String, String)>, // (translated label, value)
}

impl DetailPane {
    pub fn new() -> Self;
    pub fn set_groups(&self, groups: Vec<PropertyGroup>);
    pub fn clear(&self);
}
```

- Uses `adw::PreferencesGroup` + `adw::ActionRow` pairs
- Tab order top-to-bottom; each row subtitle is the accessible value

### `ConfirmDialog` — `src/window/components/confirm_dialog.rs`

```rust
impl ConfirmDialog {
    pub async fn ask(
        parent: &impl IsA<gtk4::Widget>,
        heading: &str,
        body: &str,
        confirm_label: &str,
    ) -> bool;
}
```

- Wraps `adw::AlertDialog` — do not override its focus trap or Escape key handling

## View contracts

Each view is a GLib struct owning a `gtk4::ListBox(.boxed-list-separate)`, a `DetailPane`,
and an `adw::Spinner`. Views are the **only** layer allowed to call `spawn_driver_task`.

```rust
// Same pattern for ContainersView, ImagesView, VolumesView, NetworksView
impl ContainersView {
    pub fn new(driver: Arc<dyn IContainerDriver>) -> Self;
    pub fn reload(&self);
}
```

## `main_window.rs` final shape

```rust
pub struct MainWindow {
    #[template_child] toast_overlay: TemplateChild<adw::ToastOverlay>,
    #[template_child] split_view: TemplateChild<adw::NavigationSplitView>,
    #[template_child] view_stack: TemplateChild<adw::ViewStack>,
    containers_view: OnceCell<ContainersView>,
    images_view: OnceCell<ImagesView>,
    volumes_view: OnceCell<VolumesView>,
    networks_view: OnceCell<NetworksView>,
}
```

No row builders, no `populate_*` methods, no dialog logic.

## I18N rules (enforced in all new code)

- Every string literal passed to a widget: `gettext!("…")`
- Same English word for different contexts: `pgettext!("container action", "Remove")`
- Counts: `ngettext!("1 container", "{n} containers", n)` — never `format!("{n} containers")`
- Directional icons: `widget.set_direction(gtk4::TextDirection::Ltr)` to prevent RTL flip
- `po/POTFILES` must include every `.rs` file under `src/window/`

## A11Y rules (enforced in all new code)

- Icon-only buttons: `set_tooltip_text` AND `update_property(&[Property::Label(...)])`
- After a destructive action: move focus to the next row, or to the empty-state widget if empty
- After a dialog closes: return focus to the widget that triggered it
- List reload announcement: use `adw::Toast` or a hidden `Label` with `AccessibleRole::Status`
- `Escape` must dismiss any open dialog — do not override `adw::AlertDialog` default

## Implementation order

Run `make test` after each step. Do not proceed to the next step if tests fail.

1. Extract `StatusBadge` — no signals, no async; add unit test for `css_class()`
2. Extract `ConfirmDialog` — wrapper only; test with mock parent
3. Extract `DetailPane` — pure data display; test `set_groups` + `clear`
4. Extract `ResourceRow` + `icon_button` helper
5. Migrate `ContainersView` — move all container list/detail/action logic out of `main_window.rs`
6. Migrate `ImagesView`, `VolumesView`, `NetworksView` — same pattern
7. Slim `main_window.rs` to final shell
8. Audit I18N — run `xgettext`; verify `po/POTFILES` covers all new `.rs` files under `src/window/`
9. Audit A11Y — keyboard-only navigation; verify `Escape` dismisses dialogs; verify focus returns after close
10. Audit breakpoints — resize from 1200 sp to 320 sp; verify each `AdwBreakpoint` fires correctly

## Exit criteria

- [ ] `make test` passes
- [ ] `make lint` reports zero warnings (`cargo clippy -- -D warnings`)
- [ ] `make fmt` shows no diff
- [ ] `main_window.rs` contains no row builders, `populate_*` methods, or dialog logic
- [ ] Every icon-only button has `set_tooltip_text` + `update_property(Property::Label(...))`
- [ ] All user-visible strings wrapped in `gettext!()`
- [ ] `tests/widget_test.rs` has at least one test per new component
- [ ] No widget module imports a sibling widget module
