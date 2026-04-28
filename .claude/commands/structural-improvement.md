---
description: Structural improvement — remove dead code, useless comments, and duplicated helpers across the four resource views; consolidates shared utilities into window/utils/. Runs make fmt and make test as exit criteria.
---

# Structural Improvement

> Read `CLAUDE.md` before writing a single line of code.
> This command is self-contained — run it fresh without prior conversation context.

---

## Baseline

```sh
make test   # must pass before touching anything
make lint   # zero warnings required
make fmt    # no diffs required
```

Record the test count. No regressions are acceptable.

---

## Group A — Remove dead code suppressed by `#[allow(dead_code)]`

For each item below: remove the symbol **and** its `#[allow(dead_code)]` attribute.
After removing each item, run `make lint` to confirm no callsite was missed.

### A1 — Unused action constants (`src/window/actions.rs:12–15`)

```rust
// Remove these two constants — no handlers are wired for them:
#[allow(dead_code)]
pub const FOCUS_SEARCH: &str = "win.focus-search";
#[allow(dead_code)]
pub const CLEAR_SEARCH: &str = "win.clear-search";
```

Grep for `FOCUS_SEARCH` and `CLEAR_SEARCH` across the project before deleting.
If a callsite exists, implement the missing handler instead of deleting.

### A2 — `StatusBadge::update` (`src/window/components/status_badge.rs:23`)

Remove the `update()` function — badges are constructed fresh in `connect_bind`; no view
calls this helper. Grep for `status_badge.*update\|update.*status_badge` to verify.

### A3 — `ToastUtil::show_with_action` (`src/window/components/toast_util.rs:37`)

Remove only `show_with_action`. Grep `show_with_action` to confirm no callsites.

> **Note:** `show_destructive` is **not** dead code — it is called from
> `src/window/main_window.rs` (prune-system confirmation). Remove its
> `#[allow(dead_code)]` attribute and keep the method.

### A4 — `resource_row::new` (`src/window/components/resource_row.rs:7`)

Remove the `new()` constructor — only `icon_button()` is used.
Grep `resource_row::new\b` to verify no callsite.

### A5 — `list_factory::make_factory` (`src/window/components/list_factory.rs`)

Remove the entire `make_factory` function (and `list_factory.rs` if it becomes empty).
Update `src/window/components/mod.rs` to remove the `pub mod list_factory;` line.
Grep `list_factory\|make_factory` to verify no callsite.

### A6 — `EmptyState::no_results` and `no_selection` (`src/window/components/empty_state.rs:17,31`)

Remove both methods. Views call `EmptyState::new()` directly and set properties inline.
Grep `no_results\|no_selection` to verify no callsite.

### A7 — `_use_filter_containers_in_tests_only` (`src/window/views/containers_view.rs`)

`filter_containers` lives in `src/core/domain/container.rs` and is already tested there
and in `tests/search_filter_test.rs`. The view uses `CustomFilter` inline and never calls
`filter_containers` directly. The wrapper only prevents a dead-import warning.

Remove both the wrapper **and** the `filter_containers` import from the `use` block at the
top of `containers_view.rs`. Run `make test` to confirm no regression.

---

## Group B — Create shared utility modules

### B1 — Create `src/window/utils/mod.rs`

```rust
pub mod format;
pub mod store;
```

### B2 — Create `src/window/utils/format.rs`

Consolidate the three `fmt_bytes` / `format_size` implementations from
`images_view.rs:500`, `volumes_view.rs:476`, and `dashboard_view.rs:42`
into a single canonical function.
Use the `dashboard_view` implementation as the base (it correctly handles the `< 1 MiB` case):

```rust
pub fn fmt_bytes(bytes: u64) -> String {
    const GIB: u64 = 1_073_741_824;
    const MIB: u64 = 1_048_576;
    if bytes >= GIB {
        format!("{:.1} GB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.0} MB", bytes as f64 / MIB as f64)
    } else {
        format!("{bytes} B")
    }
}
```

### B3 — Create `src/window/utils/store.rs`

Consolidate `find_store_position` from all four views into a generic function.
The only difference between the four copies is the GObject type in `and_downcast`:

```rust
use gtk4::glib;
use gtk4::gio;

pub fn find_store_position<T, F>(store: &gio::ListStore, pred: F) -> Option<u32>
where
    T: glib::ObjectType + glib::object::IsA<glib::Object>,
    F: Fn(&T) -> bool,
{
    (0..store.n_items()).find(|&i| {
        store
            .item(i)
            .and_downcast::<T>()
            .map_or(false, |o| pred(&o))
    })
}
```

### B4 — Expose `clear_box` from `src/window/components/mod.rs`

Remove the four identical private `fn clear_box` functions from:

- `src/window/views/containers_view.rs:2146`
- `src/window/views/images_view.rs:844`
- `src/window/views/volumes_view.rs:595`
- `src/window/views/networks_view.rs:679`

Add a single public function to `src/window/components/mod.rs`:

```rust
pub fn clear_box(b: &gtk4::Box) {
    while let Some(child) = b.first_child() {
        b.remove(&child);
    }
}
```

### B5 — Register utils in `src/window/mod.rs`

Add `pub mod utils;` to `src/window/mod.rs`.

---

## Group C — Wire views to the shared utils

For each of the four views (`containers_view.rs`, `images_view.rs`, `volumes_view.rs`,
`networks_view.rs`):

### C1 — Replace `fmt_bytes` / `format_size`

1. Delete the local `fn fmt_bytes` (or `format_size`) definition.
2. Add import: `use crate::window::utils::format::fmt_bytes;`
3. The call sites are already using `fmt_bytes(...)` — no further changes needed.

For `dashboard_view.rs`:

1. Delete `fn format_size`.
2. Add import: `use crate::window::utils::format::fmt_bytes;`
3. Replace every `format_size(...)` call with `fmt_bytes(...)`.

### C2 — Replace `find_store_position`

1. Delete the local `fn find_store_position` definition from each view.
2. Add import: `use crate::window::utils::store::find_store_position;`
3. Update each call site to pass the concrete GObject type:

   ```rust
   // containers_view:
   find_store_position::<ContainerObject, _>(&inner.store, |o| o.id() == id)

   // images_view:
   find_store_position::<ImageObject, _>(&inner.store, |o| o.id() == id)

   // volumes_view (uses `name`, not `id`):
   find_store_position::<VolumeObject, _>(&inner.store, |o| o.name() == name)

   // networks_view:
   find_store_position::<NetworkObject, _>(&inner.store, |o| o.id() == id)
   ```

### C3 — Replace `clear_box`

1. Delete the local `fn clear_box` from each view.
2. Add import: `use crate::window::components::clear_box;`
3. Call sites already use `clear_box(...)` — no changes needed.

---

## Group D — Fix useless comments

Comments that only restate what the identifier already expresses. Replace or remove:

### D1 — `containers_view.rs:305`

```rust
// Before:
// Badge label (placeholder CSS; updated in connect_bind)
let badge = gtk4::Label::builder()...

// After: remove the comment entirely — the code is self-explanatory.
```

### D2 — `containers_view.rs:504`

```rust
// Before:
// Store widget refs for connect_bind access
unsafe {
item.set_data("badge", badge);

// After: explain the non-obvious constraint (why unsafe set_data is needed):
// GObject carries no typed fields; set_data is the GTK4/Rust idiom for
// passing widget refs from connect_setup into connect_bind closures.
unsafe {
item.set_data("badge", badge);
```

### D3 — `networks_view.rs` suffix-walk block

Identify the 4-line comment block describing `adw::ActionRow` suffix layout before the
sibling-walk loop. Replace with a single line naming the invariant:

```rust
// Remove button is the sole suffix added at setup time; find it by sibling walk.
```

---

## Exit criteria

```sh
make fmt     # zero diffs
make lint    # zero warnings — especially no unused-import warnings
make test    # same count as baseline, all green
```

If `make lint` emits any `unused import` warnings, remove those imports.
If `make test` emits fewer tests than the baseline, a callsite was deleted — restore it.
