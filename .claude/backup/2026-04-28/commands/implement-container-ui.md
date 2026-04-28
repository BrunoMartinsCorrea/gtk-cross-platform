---
description: Implement the GTK4/Adwaita UI layer for the container management feature (containers, images, volumes, networks views).
---

Implement or update the GTK4/Adwaita UI layer for container management.

> Architecture rules, layer boundaries, threading model, and breakpoints are in `CLAUDE.md`.
> The driver port (`IContainerDriver`) and async bridge (`spawn_driver_task`) are already
> implemented in `src/infrastructure/containers/`.

## Feature → UI mapping

### Containers

| Driver method               | Widget                                             | Trigger                  |
|-----------------------------|----------------------------------------------------|--------------------------|
| `list_containers(all=true)` | `gtk4::ListBox` in Containers tab                  | startup + Refresh button |
| `start_container`           | `▶` icon button in `adw::ActionRow` suffix         | click                    |
| `stop_container`            | `■` icon button in `adw::ActionRow` suffix         | click                    |
| `restart_container`         | button in detail pane                              | click                    |
| `remove_container`          | `🗑` icon button + `adw::AlertDialog` confirmation | click → confirm          |
| `inspect_container`         | detail pane key-value grid                         | row selection            |

### Images / Volumes / Networks

- `list_*` → `gtk4::ListBox` in the respective tab
- `remove_*` → `🗑` icon button + `adw::AlertDialog` confirmation

### System

- `ping` / `version` → window subtitle: `"Docker"` / `"Podman"` / `"containerd"`
- `prune_system` → hamburger menu item + `adw::AlertDialog` confirmation

## Widget hierarchy

```
AdwApplicationWindow (MainWindow)
  AdwToastOverlay (toast_overlay)
    AdwNavigationSplitView (split_view)
      sidebar: AdwNavigationPage
        AdwToolbarView
          top: AdwHeaderBar
            GtkMenuButton (open-menu-symbolic)
            AdwViewSwitcher (view_switcher_top)      ← hidden at ≤ 360 sp
            GtkSpinner (spinner)
            GtkButton (refresh_button)
          content: AdwViewStack (view_stack)
            page "containers": ContainersView
            page "images":     ImagesView
            page "volumes":    VolumesView
            page "networks":   NetworksView
          bottom: AdwViewSwitcherBar [reveal=false]  ← revealed at ≤ 360 sp
      content: AdwNavigationPage
        AdwToolbarView
          top: AdwHeaderBar (detail_header_bar)
          content: GtkStack (detail_stack)
            "empty": AdwStatusPage
            "detail": GtkScrolledWindow → GtkBox (detail_content)

AdwBreakpoint (max-width: 720sp)
  → split_view.collapsed = true
  → view_switcher_bar.reveal = true
  → view_switcher_top.visible = false
```

Each list row uses `adw::ActionRow` inside `gtk4::ListBox(.boxed-list-separate)`:

```
AdwActionRow
  [prefix] StatusBadge (.status-badge .running/.stopped/.paused/.error)
  [title]  resource name (e.g., container name or image tag)
  [subtitle] metadata (image ref, short ID, driver, mountpoint, etc.)
  [suffix] primary action GtkButton + remove GtkButton
```

## Threading pattern — do not deviate

```
GTK Main Thread                          Worker Thread
─────────────                            ─────────────
begin_loading()
spawn_driver_task(driver, task, cb) ───▶ std::thread::spawn { task(driver) → tx.send }
                                    ◀─── async_channel::bounded(1)
glib::spawn_local { rx.recv() → cb }
end_loading() + update_ui
```

Rules:

- **Never** call GTK from outside the main thread
- **Always** use `async_channel::bounded(1)` — not `std` channels or `tokio`
- The callback `cb` executes on the GTK main loop via `glib::spawn_local`
- Views (`src/window/views/`) are the **only** layer that calls `spawn_driver_task`
- `tokio` is banned — it conflicts with the GLib main loop

## CSS semantic classes

```css
/* data/resources/style.css — loaded via GResource in app.rs::startup() */
.status-badge.running {
    color: @success_color;
}

.status-badge.paused {
    color: @warning_color;
}

.status-badge.stopped,
.status-badge.exited {
    color: alpha(@view_fg_color, 0.5);
}

.status-badge.dead,
.status-badge.error {
    color: @error_color;
}
```

`StatusBadge` must show both color AND a visible text label — never color alone.

## I18N requirements

- Every string literal passed to a widget: `gettext!("…")`
- Same English word for different contexts: `pgettext!("container action", "Remove")`
- Counts: `ngettext!("1 container", "{n} containers", n)` — never `format!("{n} containers")`
- Directional icons: `widget.set_direction(gtk4::TextDirection::Ltr)` to prevent RTL flip
- `.ui` template files: `translatable="yes"` on every `<property>` with user-visible text
- `po/POTFILES` must list every `.rs` under `src/window/` and every `.ui` file

## A11Y requirements

- Icon-only buttons must have both `set_tooltip_text` AND `update_property(&[Property::Label(...)])`
- `StatusBadge`: apply `set_accessible_role(AccessibleRole::Status)`
- After a destructive action: move focus to the next row, or empty-state widget if list is empty
- After a dialog closes: return focus to the widget that triggered it
- List reload: announce via `adw::Toast` or a hidden `Label` with `AccessibleRole::Status`

## Test strategy

### Layer 1 — Domain (no GTK, no daemon required)

`src/core/domain/container.rs` → `#[cfg(test)]` module:

- `test_status_from_state`, `test_status_css_class`, `test_status_labels`
- `test_port_display`, `test_stats_memory_mb`

### Layer 2 — Mock driver (no GTK, no daemon required)

`tests/container_driver_test.rs` using `MockContainerDriver`:

- `test_mock_list_containers`, `test_mock_start_stop`
- `test_mock_list_images`, `test_factory_unavailable_error`

### Layer 3 — Widget tests (require display / mark `#[ignore]` in CI)

`tests/widget_test.rs`:

- `test_adw_action_row_title`
- `test_status_badge_css_class`

CI command: `xvfb-run cargo test -- --test-threads=1`

## Implementation order

Run `make test` after each step — do not proceed if tests fail:

1. Domain tests in `src/core/domain/container.rs`
2. `MockContainerDriver` + `tests/container_driver_test.rs`
3. `tests/widget_test.rs` skeleton (marked `#[ignore]`)
4. Add `async-channel` to `Cargo.toml` if absent; implement `src/infrastructure/containers/background.rs`
5. Update `data/resources/style.css` + `data/resources/resources.gresource.xml`
6. Redesign `data/resources/window.ui` with the hierarchy above; add `AdwBreakpoint` entries
7. Redesign `src/window/main_window.rs` — views injected, no business logic
8. Update `src/app.rs` — call `ContainerDriverFactory::detect()` and register the CSS provider in `startup()`
9. `cargo check` → `make test` → `make run` (verify visually)

## Exit criteria

- [ ] `make test` passes
- [ ] `make lint` reports zero warnings
- [ ] `make run` launches the window and shows the runtime name in the title/subtitle
- [ ] Containers / Images / Volumes / Networks tabs each show their list
- [ ] Row selection updates the detail pane with key-value properties
- [ ] Refresh button reloads the list; spinner is visible during the async fetch
- [ ] Remove action shows `adw::AlertDialog` confirmation before calling the driver
- [ ] Window resizes correctly at 720 sp (split view collapses) and 360 sp (tab bar moves to bottom)
- [ ] All user-visible strings wrapped in `gettext!()`
- [ ] All icon-only buttons have `set_tooltip_text` + `update_property(Property::Label(...))`
- [ ] `StatusBadge` shows text label alongside color
