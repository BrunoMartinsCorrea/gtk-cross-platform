---
description: Diagnose and fix dashboard loading performance — lazy view init, deferred system_df, debounced search, and elimination of duplicate driver calls.
---

Optimize the dashboard and all resource views for faster perceived startup and smoother
interaction. This command is self-contained — run it without any prior conversation context.

> Architecture rules, threading model, and layer boundaries are in `CLAUDE.md`.
> Read it before touching any file.

---

## Context: Why this command exists

On startup, `main_window.rs::activate()` calls `refresh_all()`, which fires `reload()` on
every view simultaneously — dashboard, containers, images, volumes, networks. This causes:

1. **8 driver API calls at startup** — including duplicates (`container.list()` and
   `network.list()` each called twice: once in the dashboard and once in the respective view).
2. **`system_df()` blocks the first paint** — Docker's `/system/df` endpoint enumerates all
   container, image, and volume disk usage synchronously; on large environments this takes
   several seconds and blocks dashboard rendering.
3. **Hidden views load eagerly** — the user sees the dashboard first, but images, volumes,
   and networks tabs all fetch their data in the background immediately, competing for threads
   and socket bandwidth.
4. **Search has no debounce** — `repopulate()` fires on every keystroke, causing unnecessary
   re-renders on large lists.

---

## What to read before implementing

Read these files in full before making any change:

- `CLAUDE.md` — architecture rules, threading model, layer boundaries
- `src/window/main_window.rs` — `refresh_all()` and how views are wired
- `src/window/views/dashboard_view.rs` — current loading sequence (lines with `spawn_driver_task`)
- `src/window/views/containers_view.rs` — `reload()` + search handler
- `src/window/views/images_view.rs` — `reload()` pattern
- `src/window/views/volumes_view.rs` — `reload()` pattern
- `src/window/views/networks_view.rs` — `reload()` pattern
- `src/infrastructure/containers/background.rs` — `spawn_driver_task` implementation
- `src/infrastructure/containers/docker_driver.rs` — `system_df()` call site

Do **not** modify: `CLAUDE.md`, `src/core/domain/`, `src/ports/`, `src/infrastructure/`,
any existing test, or any `.ui` file unless a breakpoint must change.

---

## Fix 1 — Lazy view loading (startup)

**File:** `src/window/main_window.rs`

**Problem:** `refresh_all()` is called unconditionally on activation, loading all 5 views.

**Fix:** Replace the initial `refresh_all()` call with `reload_visible_page()`, which loads
only the currently selected page. Wire remaining views to load on first navigation.

### Implementation

Add a method `reload_visible_page()` that reads the current `AdwViewStack` page name and
calls `reload()` only on the matching view:

```rust
fn reload_visible_page(&self) {
    let imp = self.imp();
    let page = imp.view_stack.visible_child_name().unwrap_or_default();
    match page.as_str() {
        "dashboard"  => imp.dashboard_view.get().map(|v| v.reload()),
        "containers" => imp.containers_view.get().map(|v| v.reload()),
        "images"     => imp.images_view.get().map(|v| v.reload()),
        "volumes"    => imp.volumes_view.get().map(|v| v.reload()),
        "networks"   => imp.networks_view.get().map(|v| v.reload()),
        _            => None,
    };
}
```

Wire a `notify::visible-child` signal on `AdwViewStack` so switching tabs for the first time
triggers `reload()` on that view — but only once (guard with a `Cell<bool>` field per view
named `loaded`):

```rust
// Inside each view's imp struct
loaded: Cell<bool>,
```

In `reload()`, set `self.imp().loaded.set(true)` after the first fetch. In the
`notify::visible-child` handler, call `v.reload()` only if `!v.imp().loaded.get()`.

**Keep `refresh_all()`** for the manual Refresh button — it must reload all views regardless
of `loaded` state (and reset `loaded` to `false` before reloading so re-navigation stays
consistent after a manual refresh).

---

## Fix 2 — Split dashboard loading: fast path + deferred system_df

**File:** `src/window/views/dashboard_view.rs`

**Problem:** The dashboard `spawn_driver_task` closure runs four sequential calls:
`container.list()`, `system_df()`, `network.list()`, `events()`. `system_df()` is the
heaviest and blocks all four results.

**Fix:** Split into two sequential tasks:

### Task A — Fast path (fires immediately)

```rust
spawn_driver_task(use_case.clone(), |uc| {
    let containers = uc.container.list(true)?;
    let networks   = uc.network.list()?;
    let events     = uc.network.events(None, Some(10))?;
    Ok((containers, networks, events))
}, |result| {
    // Populate container count, network count, recent events immediately
    self.update_fast_widgets(result);
    // Then trigger the slow path
    self.load_usage_stats();
});
```

### Task B — Deferred (fires from inside Task A callback)

```rust
fn load_usage_stats(&self) {
    let use_case = self.imp().use_case.get().cloned().expect("use case set");
    spawn_driver_task(use_case, |uc| uc.network.system_df(), |result| {
        self.update_usage_card(result);
    });
}
```

The usage/disk card must show a placeholder spinner while Task B is running. Wrap it in a
`GtkStack` with pages `"loading"` (spinner) and `"content"` (actual data), and switch pages
in `update_usage_card()`.

**Threading rule:** both tasks follow the existing `spawn_driver_task` pattern — no
deviation. Never call `system_df()` from the GTK main thread.

---

## Fix 3 — Eliminate duplicate driver calls

**File:** `src/window/main_window.rs`

**Problem:** With Fix 1, lazy loading already prevents eager duplicate calls on startup.
However, when the user clicks the global Refresh button, `refresh_all()` still triggers both
dashboard and the individual views — causing `container.list()` and `network.list()` to be
called twice.

**Fix:** When `refresh_all()` is triggered, skip `containers_view.reload()` and
`networks_view.reload()` if the dashboard is the current page (the dashboard result covers
the same data). Add a parameter to `refresh_all()`:

```rust
pub fn refresh_all(&self) {
    // Dashboard always refreshes (it is the coordinator for container + network counts)
    if let Some(v) = imp.dashboard_view.get() { v.reload(); }

    // Only refresh non-visible views if they have already been loaded at least once.
    // If a view was never shown, refreshing it wastes a driver call.
    for view in [&imp.containers_view, &imp.images_view, &imp.volumes_view, &imp.networks_view] {
        if let Some(v) = view.get() {
            if v.imp().loaded.get() { v.reload(); }
        }
    }
}
```

This way, tabs the user has never visited are never fetched by the Refresh button either.

---

## Fix 4 — Debounce search/filter input

**Files:** `src/window/views/containers_view.rs` (and any other view with a search entry)

**Problem:** The search entry connects `changed` directly to `repopulate()`, which iterates
the full list on every keystroke.

**Fix:** Wrap the `changed` handler in a `glib::timeout_add_local_once` debounce of 150 ms.
Cancel the previous timer if a new keystroke arrives before it fires.

```rust
// In the view's imp struct
search_debounce: RefCell<Option<glib::SourceId>>,
```

```rust
// In the search entry `changed` signal handler
let imp = self.imp();
// Cancel previous pending debounce
if let Some(id) = imp.search_debounce.borrow_mut().take() {
    id.remove();
}
// Schedule new debounce
let self_weak = self.downgrade();
*imp.search_debounce.borrow_mut() = Some(glib::timeout_add_local_once(
    std::time::Duration::from_millis(150),
    move || {
        if let Some(s) = self_weak.upgrade() {
            s.repopulate();
        }
    },
));
```

Apply to every view that has a search/filter `gtk4::SearchEntry` or `gtk4::Entry`.

---

## Fix 5 — In-progress guard (prevent concurrent duplicate tasks)

**Files:** all views (`containers_view.rs`, `images_view.rs`, `volumes_view.rs`,
`networks_view.rs`, `dashboard_view.rs`)

**Problem:** If the user clicks Refresh while a previous `reload()` is still in flight, a
second `spawn_driver_task` is queued, causing two concurrent fetches and a double-update of
the list.

**Fix:** Add a boolean guard to each view's imp struct:

```rust
loading: Cell<bool>,
```

At the start of `reload()`:
```rust
if self.imp().loading.get() { return; }
self.imp().loading.set(true);
self.begin_loading();
```

In the `spawn_driver_task` callback (both success and error branches):
```rust
self.imp().loading.set(false);
self.end_loading();
```

The Refresh button in `main_window.rs` is already disabled while `loading_count > 0`, so
this guard is an additional per-view safety layer, not a replacement.

---

## Constraints

- **Never** call any GTK function from outside the GTK main thread.
- **Never** use `tokio`, `std::sync::mpsc`, or raw threads — always `spawn_driver_task` +
  `async_channel::bounded(1)` + `glib::spawn_local`.
- **Never** access `IContainerDriver` or `IContainerUseCase` directly from views — go through
  the existing port (`spawn_driver_task`).
- Dashboard's Task B (`system_df`) must be started from inside Task A's GTK-thread callback —
  never from the worker thread.
- Search debounce timer must be cancelled on `Cell::take()` before creating a new one —
  never leak `SourceId` values.
- Do not add any new crate dependencies — all primitives needed (`glib::timeout_add_local_once`,
  `Cell`, `RefCell`) are already available.

---

## Implementation order

Run `make test` after each step — do not proceed if tests fail:

1. Add `loaded: Cell<bool>` to each view's imp struct; update `reload()` to set it; add
   `notify::visible-child` handler in `main_window.rs`
2. Implement `reload_visible_page()` in `main_window.rs`; replace startup `refresh_all()`
   with `reload_visible_page()`; update `refresh_all()` to skip unvisited views
3. Add `loading: Cell<bool>` guard to every view's `reload()`
4. Split dashboard `spawn_driver_task` into Task A (fast) + Task B (deferred `system_df`);
   add spinner placeholder for the usage card
5. Add search debounce to every view with a search entry
6. Run `make test` + `make lint` + `make run`; verify manually:
   - Startup: only dashboard API call fires (check with `G_MESSAGES_DEBUG=all make run`)
   - Tab switch: view loads on first visit, not before
   - Refresh: all previously-visited tabs refresh; unvisited tabs do not
   - Dashboard: container/network counts appear quickly; disk usage card shows spinner, then updates
   - Search: typing quickly in a container list fires `repopulate()` once, not once per character

---

## Exit criteria

- [ ] `make test` passes (all unit + integration tests)
- [ ] `make lint` reports zero warnings (`cargo clippy -- -D warnings`)
- [ ] `make fmt` shows no diff (`cargo fmt --check`)
- [ ] On startup with `G_MESSAGES_DEBUG=all make run`, only **one** `spawn_driver_task` is
      logged (dashboard fast path) — not five
- [ ] Switching to a tab for the first time triggers exactly one `spawn_driver_task` for that view
- [ ] Switching to a tab that was already visited does **not** trigger a new fetch
- [ ] The Refresh button causes exactly one fetch per previously-visited view
- [ ] Dashboard container/network counts appear before the disk usage card
- [ ] Disk usage card shows a spinner while `system_df()` is in flight
- [ ] Typing rapidly in any search field triggers `repopulate()` once (150 ms after the last keystroke)
- [ ] `loading: Cell<bool>` guard prevents concurrent duplicate fetches (verified by clicking
      Refresh twice in quick succession and checking logs for a single request)
- [ ] No new crate added to `Cargo.toml`
- [ ] No `tokio`, `std::sync::mpsc`, or raw thread usage introduced
