---
description: Implement the v0.2 MVP feature set derived from Doca.zip design specs — search/filter, pull image, create container wizard, stats, inspect, env vars, terminal, compose grouping, dashboard, runtime switcher, about dialog, menu popover, empty states, logs follow, and network/volume create dialog completeness.
---

# v0.2 MVP Implementation

> Architecture rules, threading model, layer boundaries, A11Y, and i18n requirements are in `CLAUDE.md`.
> The design source of truth is `Doca.zip` in the repo root. Extract it and read all files before starting.

## Step 0 — Read the design

```sh
unzip -o Doca.zip -d /tmp/doca
```

Read these files before writing a single line of code:

| File                        | What it contains                                                                                                   |
|-----------------------------|--------------------------------------------------------------------------------------------------------------------|
| `Feature Gap Analysis.html` | Priority matrix: P1 · P2 · P3 · Done; v0.2/v0.3/v0.4/v1.0 roadmap                                                  |
| `Container Manager v2.html` | Full interactive visual mockup of the v0.2 UI (desktop + mobile)                                                   |
| `Container Manager.html`    | v0.1 baseline prototype                                                                                            |
| `cm-core.jsx`               | Mock data shapes, domain constants, all shared components (StatusBadge, PR, Toast, Spinner, IconBtn, PillBtn)      |
| `cm-app.jsx`                | Dashboard, Sidebar (search + compose grouping + Add context button), runtime switcher, dialog orchestration        |
| `cm-features.jsx`           | Terminal, Stats (sparklines), Inspect (JSON), Image Layers, detail panes per resource type                         |
| `cm-dialogs.jsx`            | Pull image (3-phase), Create container wizard (4-step), Create volume, Create network, About dialog, MenuPopover   |
| `tweaks-panel.jsx`          | Floating design-tweak control panel (TweaksPanel, TweakSection, TweakSlider, TweakToggle, TweakRadio, TweakSelect) |

---

## Design reference — screens & domain models

### Application layout

```
┌──────────────────────────────────────────┐
│ ≡  Container Manager  [Containers|Images|Volumes|Networks|Home]  ⟳ [runtime]  ⋯ │
├──────────┬───────────────────────────────┤
│ Sidebar  │ Detail Pane                   │
│ 320 px   │ (tab bar + content)           │
│ search   │                               │
│ list     │                               │
└──────────┴───────────────────────────────┘
Mobile: single-pane (list ↔ detail), bottom tab bar (Home/Containers/Images/Volumes/Networks)
```

### Detail-pane tabs per resource

| Resource  | Tabs                                     |
|-----------|------------------------------------------|
| Container | Info · Logs · Stats · Terminal · Inspect |
| Image     | Info · Layers                            |
| Volume    | (single view — no tabs)                  |
| Network   | (single view — no tabs)                  |
| Home      | Dashboard (full-width)                   |

### Color tokens (design reference only — map to GNOME Adwaita in Rust)

| Design token | Light             | Dark                    | GNOME mapping      |
|--------------|-------------------|-------------------------|--------------------|
| `--accent`   | `#3584e4`         | `#78aeed`               | `@accent_color`    |
| `--ok`       | `#26a269`         | `#57e389`               | `@success_color`   |
| `--warn`     | `#905b00`         | `#f8e45c`               | `@warning_color`   |
| `--err`      | `#c01c28`         | `#ff7b7b`               | `@error_color`     |
| `--txt2`     | `rgba(0,0,0,.55)` | `rgba(255,255,255,.55)` | `@dim_label_color` |

### StatusBadge colors (design → Adwaita CSS class)

| Status               | Design color     | Adwaita approach  |
|----------------------|------------------|-------------------|
| running              | green (#26a269)  | `success` accent  |
| paused               | orange (#905b00) | `warning` accent  |
| stopped / exited     | gray             | `dim-label` class |
| dead / error         | red (#c01c28)    | `error` accent    |
| created / restarting | blue (accent)    | `accent` class    |

### Domain data shapes (from `cm-core.jsx`)

#### Container

```
id, name, image, status (running|paused|stopped|exited|dead|error|created|restarting),
ports (string), command (string), compose (project name or null),
created (ISO timestamp), restart (always|no|unless-stopped|on-failure), env []
```

#### Image

```
id (sha256:…), short_id, name, tag, size (bytes → MB), created (unix),
digest (optional), in_use (bool — has running container using this image),
labels {}
```

> ⚠️ `in_use` is in the design but missing from the Rust `Image` domain struct.

#### Volume

```
name, driver (local|nfs|tmpfs), mountpoint, created, labels {}, scope,
size (bytes — human display "2.3 GB"), in_use (bool)
```

> ⚠️ `size` and `in_use` are in the design but missing from the Rust `Volume` domain struct.

#### Network

```
id, name, driver (bridge|host|null|overlay|macvlan|ipvlan),
scope, internal (bool), created, subnet (optional), gateway (optional),
containers_count (u64 — number of connected containers)
```

> ⚠️ `containers_count` is in the design but missing from the Rust `Network` domain struct.

#### Dashboard stats

```
system: { cpu_percent, mem_percent, disk_percent }
containers: { running, paused, stopped, errored }
recent_events: [{ ts, type, action, actor, id }]
```

#### Image layer

```
id (sha256 prefix), cmd (Dockerfile command), size (bytes)
```

> New domain type required for image layers feature.

#### ContainerEvent

```
timestamp (HH:MM:SS or ISO), event_type (container|image|volume|network),
action (start|stop|pull|create|…), actor (container/image name), actor_id (truncated id)
```

---

## Implementation status — v0.2 complete

Run `make test` before adding anything. All 148 non-widget tests must be green.

```
148 non-widget tests:
  56  inline unit tests (lib)
  92  integration tests across 15 test files
 + 5  widget tests (ignored; run with xvfb-run --ignored)
```

### Completed features

| Feature                                          | Location                                            | Tests                              |
|--------------------------------------------------|-----------------------------------------------------|------------------------------------|
| Global search/filter (Ctrl+F)                    | `containers_view.rs`                                | `search_filter_test.rs` (10)       |
| Pull image dialog — basic                        | `images_view.rs`                                    | `pull_image_test.rs` (6)           |
| Pull image — layer streaming                     | `images_view.rs`                                    | `pull_image_streaming_test.rs` (4) |
| Create container wizard (4-step)                 | `containers_view.rs`                                | `create_container_test.rs` (7)     |
| Container stats tab (sparklines, 60s)            | `containers_view.rs`                                | `container_stats_test.rs` (6)      |
| Container inspect tab (JSON + copy)              | `containers_view.rs`                                | `inspect_test.rs` (7)              |
| Container env vars + secret masking              | `containers_view.rs`                                | `env_masking_test.rs` (11)         |
| Compose stack grouping (collapsible)             | `containers_view.rs`                                | `compose_grouping_test.rs` (9)     |
| Logs backend (port + mock)                       | `infrastructure/containers/`                        | `container_logs_test.rs` (4)       |
| Logs tab UI (timestamps, copy, refresh)          | `containers_view.rs` (`build_logs_tab`)             | (backend tests)                    |
| Terminal backend (port + mock)                   | `infrastructure/containers/`                        | `terminal_test.rs` (4)             |
| Terminal tab UI (shell selector, entry, history) | `containers_view.rs` (`build_terminal_tab`)         | (backend tests)                    |
| `IContainerUseCase::exec()`                      | `i_container_use_case.rs` + `container_use_case.rs` | `terminal_test.rs`                 |
| Create volume dialog                             | `volumes_view.rs` (`show_create_dialog`)            | —                                  |
| Create network dialog                            | `networks_view.rs` (`show_create_dialog`)           | —                                  |
| Dashboard backend (system_df, prune)             | `infrastructure/containers/`                        | `dashboard_test.rs` (3)            |
| Dashboard UI (Home tab)                          | `dashboard_view.rs`                                 | (backend tests)                    |
| Runtime switcher backend (`detect_specific`)     | `factory.rs`                                        | `runtime_switcher_test.rs` (3)     |
| Runtime switcher widget (`adw::HeaderBar`)       | `main_window.rs`                                    | (backend tests)                    |

---

## Remaining work — implement in priority order

All items below are specified in the Doca.zip design but not yet in the codebase.
Implement in the order listed; confirm `make test` passes after each group.

---

### Group A — Domain model gaps (prerequisite for B, C, D)

These fields are in the design but absent from the Rust domain structs.
Add them before implementing any UI that depends on them.

#### A1 — `Image.in_use` (`src/core/domain/image.rs`)

```rust
pub struct Image {
    // existing fields ...
    pub in_use: bool,   // true when at least one container uses this image
}
```

Update `MockContainerDriver::list_images()` and all driver impls to populate it.

#### A2 — `Volume.size_bytes` + `Volume.in_use` (`src/core/domain/volume.rs`)

```rust
pub struct Volume {
    // existing fields ...
    pub size_bytes: Option<u64>,  // None when size is unavailable from the runtime
    pub in_use: bool,
}
```

Docker API: `VolumeUsageData.Size` + `VolumeUsageData.RefCount > 0`.
Update all drivers and mock.

#### A3 — `Network.containers_count` (`src/core/domain/network.rs`)

```rust
pub struct Network {
    // existing fields ...
    pub containers_count: u64,  // number of containers currently connected
}
```

Docker API: `len(NetworkResource.Containers)`.
Update all drivers and mock.

#### A4 — `ImageLayer` new type (`src/core/domain/image.rs`)

```rust
pub struct ImageLayer {
    pub id: String,      // sha256 prefix (12 chars)
    pub cmd: String,     // Dockerfile command that created this layer
    pub size: u64,       // bytes
}
```

Add `inspect_image_layers` to `IContainerDriver`:

```rust
fn inspect_image_layers(&self, id: &str) -> Result<Vec<ImageLayer>, ContainerError>;
```

Add `layers` to `IImageUseCase`:

```rust
fn layers(&self, id: &str) -> Result<Vec<ImageLayer>, ContainerError>;
```

Implement in `ImageUseCase` (delegate to driver). Implement mock returning deterministic layers.
Write tests in `tests/image_layers_test.rs` (minimum 3 tests).

---

### Group B — Sidebar row enhancements

Design specifies visual cues in list rows that are not yet rendered.

#### B1 — Image row: "dangling" badge

When `img.in_use == false` and `img.tags.is_empty()` (or tag is `<none>:<none>`):

- Append a small label or CSS class `dim-label` with text `gettext("dangling")` to the subtitle.
- Example subtitle: `"abc12345 · 7.8 MB · dangling"`.

#### B2 — Volume row: size + "unused" badge

Current subtitle only shows driver. Extend to:

- `"{driver} · {size_human}" + if !in_use { " · unused" }"`
- Helper: `fn fmt_bytes(b: u64) -> String` — format as "142 MB" or "2.3 GB".

#### B3 — Network row: containers count

Current subtitle shows driver and subnet. Extend to:

- `"{driver} · {subnet} · {containers_count} container(s)"`.

#### B4 — Compose group header: running badge

Design (`cm-features.jsx` Sidebar) shows a badge on each compose group header
of the form **"N/M running"** (e.g., "3/4 running").

Implementation:

- In `containers_view.rs`, when building the compose group `adw::ActionRow` header, count
  containers in the group with `status == ContainerStatus::Running`.
- Build subtitle string: `format!("{}/{} running", running, total)`.
- Use `pgettext!("compose group", "{running}/{total} running")` for i18n.

#### B5 — Sidebar: empty states

The design defines two distinct empty-state components for the sidebar:

**EmptyList** — shown when the resource list is empty (no Docker objects exist):

- Each tab has its own message:
    - Containers: icon `system-run-symbolic`, title `gettext("No Containers")`, body
      `gettext("No containers found. Run a container to get started.")`
    - Images: icon `drive-optical-symbolic`, title `gettext("No Images")`, body
      `gettext("No images found. Pull an image to get started.")`
    - Volumes: icon `drive-harddisk-symbolic`, title `gettext("No Volumes")`, body `gettext("No volumes found.")`
    - Networks: icon `network-wired-symbolic`, title `gettext("No Networks")`, body `gettext("No networks found.")`
- Use `adw::StatusPage` with icon, title, and description.

**No-results state** — shown when search returns no matches (list is non-empty but filter yields nothing):

- Icon: `edit-find-symbolic`
- Title: `gettext("No Results")`
- Body: `format!("{} "{}"", gettext("No matches for"), search_query)`
- Use `adw::StatusPage`.

**EmptySelection** — shown in the detail pane when no item is selected:

- Each tab has its own message matching the sidebar's EmptyList prompt.
- Use `adw::StatusPage`.
- Containers: icon `system-run-symbolic`, title `gettext("No Container Selected")`, body
  `gettext("Select a container from the list.")`
- Images, Volumes, Networks: analogous.

---

### Group C — Detail pane completeness

#### C1 — Image detail: additional properties + action buttons

`show_detail()` in `images_view.rs` currently shows: Tag, ID, Size, Created, Digest.

Add the following rows:

- **In Use**: `gettext("Yes")` / `gettext("No")`.

Add action buttons above the property group (same pattern as containers):

- **[Run]** button: opens the 4-step Create Container Wizard with `image` pre-filled.
- **[Push…]** button: stub — show `adw::Toast` with `gettext("Push not yet implemented")`.
- **[Remove]** button: already present in the row; mirror it here in the detail pane.

A11Y: each icon-only button must have `set_tooltip_text` + `update_property(Property::Label(...))`.

Design ref: `cm-features.jsx` → `ImageDetail` component.

#### C2 — Image detail: Layers tab

Add a 2nd tab **"Layers"** to `show_detail()` in `images_view.rs`:

- Tab layout: vertical list in `gtk4::ScrolledWindow`.
- Each layer row: `adw::ActionRow`
    - Title: first 60 chars of `layer.cmd` (truncated with "…").
    - Subtitle: `layer.id` (monospace) + `" · " + fmt_bytes(layer.size)`.
- Below the list: cumulative size footer label.
- Load via `spawn_driver_task` → `IImageUseCase::layers()`.
- Show `adw::Spinner` while loading; error via toast.
- All strings use `gettext!()`.

Design ref: `cm-features.jsx` → `LayersView` component.

#### C3 — Volume detail: Size + In Use

`show_detail()` in `volumes_view.rs` currently shows: Name, Driver, Mountpoint.

Add:

- **Size**: `fmt_bytes(vol.size_bytes.unwrap_or(0))` or `"—"` when None.
- **In Use**: `gettext("Yes")` / `gettext("No")`.

#### C4 — Network detail: Gateway + Internal + Containers

`show_detail()` in `networks_view.rs` currently shows: Name, Driver, Scope, Subnet.

Add:

- **Gateway**: `net.gateway.as_deref().unwrap_or("—")`.
- **Internal**: `gettext("Yes")` / `gettext("No")`.
- **Containers**: `format!("{}", net.containers_count)`.

---

### Group D — Compose stack lifecycle (P1)

Design ref: `Feature Gap Analysis.html` → "Start / Stop de stack" (P1).

The design specifies that the collapsible compose group header should have action buttons to
start or stop all containers in the stack at once.

**Port addition** — add to `IContainerUseCase`:

```rust
fn start_all(&self, ids: &[&str]) -> Result<Vec<Result<(), ContainerError>>, ContainerError>;
fn stop_all(&self, ids: &[&str], timeout_secs: Option<u32>) -> Result<Vec<Result<(), ContainerError>>, ContainerError>;
```

Implement in `ContainerUseCase` (loop over IDs, collect results).

**UI** — in the compose group header row (inside `containers_view.rs`):

- Add **[Stop All]** / **[Start All]** icon-only buttons (context-dependent: show Stop when all running, Start when none
  running, both when mixed).
- Buttons call `spawn_driver_task` with the appropriate batch use-case method.
- After completion, reload the container list and show a toast with count: `"3 containers stopped"`.
- A11Y: `set_tooltip_text` + `update_property(Property::Label(...))` on both buttons.

Write tests in `tests/compose_lifecycle_test.rs` (minimum 3 tests).

---

### Group E — Rename container (P2)

Design ref: `Feature Gap Analysis.html` → "Rename container" (P2).

**Port** — already present: `IContainerUseCase::rename(id, new_name)`.

**UI** — in the container Info tab, make the container name a clickable/editable title:

- Replace the static `PR { k: "Name", v: name }` row with an `adw::EntryRow` (inline editable).
- On Enter or focus-out: if text changed, call `spawn_driver_task` → `IContainerUseCase::rename()`.
- On success: update the row title in the sidebar + show toast `"Container renamed"`.
- On error: revert text to original name + toast with error.

---

### Group F — Event stream panel (P2)

Design ref: `Feature Gap Analysis.html` → "Stream de eventos" (P2); `cm-app.jsx` → dashboard recent events.

Add `system_events` to `IContainerDriver`:

```rust
pub struct ContainerEvent {
    pub timestamp: String,   // HH:MM:SS or ISO
    pub event_type: String,  // "container" | "image" | "volume" | "network"
    pub action: String,      // "start" | "stop" | "pull" | "create" | ...
    pub actor: String,       // container/image name
    pub actor_id: String,    // truncated id
}

fn system_events(&self, since: Option<i64>, limit: Option<usize>) -> Result<Vec<ContainerEvent>, ContainerError>;
```

Add to `INetworkUseCase` (or create `ISystemUseCase`):

```rust
fn events(&self, since: Option<i64>, limit: Option<usize>) -> Result<Vec<ContainerEvent>, ContainerError>;
```

**Dashboard integration**: replace mock "Recent Events" in `dashboard_view.rs` with real data loaded via
`spawn_driver_task`.

**Event log panel** (optional — P2 stretch goal): a collapsible bottom panel in the main window with live event rows.

Write tests in `tests/system_events_test.rs` (minimum 3 tests).

---

### Group G — About dialog (`adw::AboutWindow`)

Design ref: `cm-dialogs.jsx` → `AboutDialog` component. This is listed as "Done" in the Feature
Gap Analysis but is absent from the Rust codebase. It must be triggered from the hamburger menu.

**Trigger:** `MenuPopover` "About Container Manager" item → opens `adw::AboutWindow`.

**Fields to populate:**

```rust
adw::AboutWindow::builder()
.application_name(gettext("Container Manager"))
.application_icon("com.example.GtkCrossPlatform")
.version(config::VERSION)
.comments(gettext("A native GNOME application for managing Docker, Podman, and containerd containers."))
.developer_name("Container Manager Contributors")
.website("https://github.com/your-org/gtk-cross-platform")
.issue_url("https://github.com/your-org/gtk-cross-platform/issues")
.license_type(gtk::License::Gpl30)
.copyright("© 2026 Container Manager Contributors")
.build()
```

**Runtime info section** (`add_other_credits`):

- Section "Runtime": runtime name + version from `driver.version()` (load via `spawn_driver_task` before showing the
  dialog — fall back to `gettext("Unknown")` on error).
- Section "Toolkit": `format!("GTK {}", gtk4::major_version())` (compile-time constants available via
  `gtk4::major_version()` / `gtk4::minor_version()`).

**Links** (use `add_link`):

- `gettext("Source Code on GitHub")` → website URL.
- `gettext("Report an Issue")` → issue URL.

**Behavior:**

- Present via `window.present()` — `adw::AboutWindow` is transient for `main_window`.
- No need for custom close button; `adw::AboutWindow` provides its own.

Design ref: `cm-dialogs.jsx` → `AboutDialog` (360×540 modal — adapt to `adw::AboutWindow` idiom).

---

### Group H — Menu popover completeness

Design ref: `cm-dialogs.jsx` → `MenuPopover`.

The hamburger menu (≡ button in `adw::HeaderBar`) exposes three items:

| Item                    | Icon                          | Action                                                                                      |
|-------------------------|-------------------------------|---------------------------------------------------------------------------------------------|
| Prune System…           | `user-trash-symbolic`         | Open `adw::MessageDialog` to confirm, then `spawn_driver_task` → `INetworkUseCase::prune()` |
| Preferences             | `preferences-system-symbolic` | Show `adw::Toast` with `gettext("Preferences not yet implemented")`                         |
| About Container Manager | `help-about-symbolic`         | Open `adw::AboutWindow` (Group G)                                                           |

**Current state:** Prune System is wired. "Preferences" and "About" items are missing.

**Implementation in `main_window.rs`:**

- Add `gtk::PopoverMenu` (or `adw::OverlaySplitView` toggle) to the `adw::HeaderBar` start widget.
- Use `gio::Menu` + `gio::SimpleAction` for each item (follows GNOME HIG for header menus).
- Wire `app.about` action to show `adw::AboutWindow`.
- Wire `app.preferences` action to show the toast.
- Separator between Prune and Preferences in the menu model.

**A11Y:** Menu button must have `set_tooltip_text(gettext("Main menu"))` + `update_property(Property::Label(...))`.

---

### Group I — Logs tab: follow mode + clear

Design ref: `cm-features.jsx` → `LogsView` toolbar.

The design specifies two behaviors currently absent from `build_logs_tab`:

#### I1 — Follow toggle

- Add a `gtk::ToggleButton` labeled `gettext("Follow")` with icon `go-bottom-symbolic`.
- When toggled on: after each refresh or new data load, programmatically scroll the `ScrolledWindow` to the bottom (
  `scrolled.vadjustment().set_value(scrolled.vadjustment().upper())`).
- When toggled off: no auto-scroll (user can freely scroll back through history).
- Default: on when container is running, off when stopped.
- A11Y: `set_tooltip_text(gettext("Auto-scroll to newest log line"))` + `update_property(Property::Label(...))`.

> **Note:** The existing implementation uses a "Timestamps" toggle and "Refresh" button.
> Both are correct GNOME HIG additions absent from the React prototype.
> Keep Timestamps + Copy + Refresh; add Follow as a 4th toolbar item.

#### I2 — Clear button

- Add an icon-only `gtk::Button` with icon `edit-clear-all-symbolic`.
- On click: clear the `TextView`'s buffer (`buffer.set_text("")`).
- A11Y: `set_tooltip_text(gettext("Clear log output"))` + `update_property(Property::Label(...))`.

---

### Group J — Terminal: command history navigation

Design ref: `cm-features.jsx` → `TerminalView`.

The design specifies Arrow Up / Arrow Down to cycle through the last 50 commands entered in the
terminal. The Rust implementation's "shell selector, entry, history" item confirms history was
intended but may not include keyboard navigation.

**Implementation in `build_terminal_tab`:**

- Maintain a `Vec<String>` history (max 50 entries) and a cursor index in a `Rc<RefCell<...>>`.
- Connect `gtk::EventControllerKey` to the `gtk::Entry`:
    - `ArrowUp`: move cursor back; set entry text to `history[cursor]`.
    - `ArrowDown`: move cursor forward; set entry text to `history[cursor]` or clear when past end.
- On command submit: push to history, reset cursor to `history.len()`.

**"Exit" command handling:**

- When user types `exit` in the terminal entry:
    - Append `gettext("Session closed.")` to the output `TextView`.
    - Set `entry.set_sensitive(false)` and `run_btn.set_sensitive(false)`.
    - Show a dim label: `gettext("Reconnect by selecting the container again.")`.

---

### Group K — Conditional remove protection for system networks

Design ref: `cm-features.jsx` → `NetworkRow`, `NetworkDetail`.

Default Docker/Podman networks (`bridge`, `host`, `none`) must not be removable.

**Sidebar row** (`networks_view.rs`):

- Hide (or `set_sensitive(false)`) the Remove button for rows where `net.name` is in
  `["bridge", "host", "none"]`.

**Detail pane** (`networks_view.rs`):

- Hide the Remove action button for the same networks.
- Show `adw::StatusPage` hint instead: `gettext("System network — cannot be removed.")`.

---

### Group L — Create dialog port completeness

#### L1 — `CreateVolumeDialog`: driver parameter

The design (`cm-dialogs.jsx` → `CreateVolumeDialog`) shows a **Driver** select
(options: `local`, `nfs`, `tmpfs`; default: `local`).

The Rust port signature is:

```rust
fn create_volume(&self, name: &str, labels: HashMap<String, String>) -> Result<Volume, ContainerError>;
```

This signature is sufficient — pass the driver as a label key `"driver"` or extend the dialog
to pass it as a separate field via a new `CreateVolumeOptions` struct:

```rust
pub struct CreateVolumeOptions {
    pub name: String,
    pub driver: String,       // "local" | "nfs" | "tmpfs"
    pub labels: HashMap<String, String>,
}
```

Update `IContainerDriver::create_volume`, all five drivers, `IVolumeUseCase`, and `VolumeUseCase`.

**UI addition in `volumes_view.rs` `show_create_dialog()`:**

- Add an `adw::ComboRow` (or `gtk::DropDown`) with options `["local", "nfs", "tmpfs"]`.
- Default selection: `local`.
- Pass selected driver to `CreateVolumeOptions::driver`.

#### L2 — `CreateNetworkDialog`: driver + subnet parameters

The design (`cm-dialogs.jsx` → `CreateNetworkDialog`) shows:

- **Driver** select: `bridge`, `overlay`, `macvlan`, `ipvlan` (default: `bridge`).
- **Subnet** text input: optional, placeholder `"172.20.0.0/16"`.

The Rust port signature is:

```rust
fn create_network(&self, name: &str) -> Result<Network, ContainerError>;
```

Extend to:

```rust
pub struct CreateNetworkOptions {
    pub name: String,
    pub driver: String,            // "bridge" | "overlay" | "macvlan" | "ipvlan"
    pub subnet: Option<String>,    // CIDR notation, validated client-side
}

fn create_network(&self, opts: &CreateNetworkOptions) -> Result<Network, ContainerError>;
```

Update `IContainerDriver::create_network`, all five drivers, `INetworkUseCase`, and `NetworkUseCase`.

**UI addition in `networks_view.rs` `show_create_dialog()`:**

- Add `adw::ComboRow` for driver (options: `bridge`, `overlay`, `macvlan`, `ipvlan`; default: `bridge`).
- Add `adw::EntryRow` for optional subnet (validate CIDR on submit; show inline error via `adw::Toast` if invalid).
- Pass both to `CreateNetworkOptions`.

---

### Group M — Dashboard: system metrics refresh

Design ref: `cm-features.jsx` → `Dashboard` → Host Resources panel.

The design updates CPU/Memory/Disk metrics every 2000ms via a mock random-walk. In Rust,
implement a periodic refresh:

- After the initial `spawn_driver_task` → `system_df()` completes and populates the gauges,
  schedule a repeat: use `glib::timeout_add_seconds_local(30, ...)` (30 s — not 2 s; the
  dashboard is not a real-time tool; keep it lightweight).
- Cancel the timeout (store the `SourceId` in `Rc<Cell<Option<SourceId>>>`) when the Home tab
  loses visibility (connect to `adw::ViewStack::notify::visible-child`).
- Show a small "Last updated HH:MM:SS" label in the bottom-right of the Host Resources card.

---

## Port and domain signatures — use these exactly

### `IContainerDriver` trait (`src/ports/i_container_driver.rs`)

```rust
pub trait IContainerDriver: Send + Sync {
    // Runtime detection
    fn runtime_name(&self) -> &str;
    fn is_available(&self) -> bool;
    fn ping(&self) -> Result<(), ContainerError>;
    fn version(&self) -> Result<String, ContainerError>;

    // Container lifecycle
    fn list_containers(&self, all: bool) -> Result<Vec<Container>, ContainerError>;
    fn inspect_container(&self, id: &str) -> Result<Container, ContainerError>;
    fn start_container(&self, id: &str) -> Result<(), ContainerError>;
    fn stop_container(&self, id: &str, timeout_secs: Option<u32>) -> Result<(), ContainerError>;
    fn restart_container(&self, id: &str, timeout_secs: Option<u32>) -> Result<(), ContainerError>;
    fn pause_container(&self, id: &str) -> Result<(), ContainerError>;
    fn unpause_container(&self, id: &str) -> Result<(), ContainerError>;
    fn remove_container(&self, id: &str, force: bool, remove_volumes: bool) -> Result<(), ContainerError>;
    fn create_container(&self, opts: &CreateContainerOptions) -> Result<String, ContainerError>;
    fn rename_container(&self, id: &str, new_name: &str) -> Result<(), ContainerError>;

    // Container info
    fn container_logs(&self, id: &str, tail: Option<u32>, timestamps: bool) -> Result<String, ContainerError>;
    fn container_stats(&self, id: &str) -> Result<ContainerStats, ContainerError>;
    fn inspect_container_json(&self, id: &str) -> Result<String, ContainerError>;
    fn exec_in_container(&self, id: &str, cmd: &[&str]) -> Result<String, ContainerError>;

    // Image operations
    fn list_images(&self) -> Result<Vec<Image>, ContainerError>;
    fn pull_image(&self, reference: &str) -> Result<(), ContainerError>;
    fn pull_image_streaming(&self, reference: &str, tx: std::sync::mpsc::SyncSender<PullProgress>) -> Result<(), ContainerError>;
    fn cancel_pull(&self);
    fn remove_image(&self, id: &str, force: bool) -> Result<(), ContainerError>;
    fn tag_image(&self, source: &str, target: &str) -> Result<(), ContainerError>;
    fn inspect_image(&self, id: &str) -> Result<Image, ContainerError>;
    fn inspect_image_layers(&self, id: &str) -> Result<Vec<ImageLayer>, ContainerError>;  // TO ADD (Group A4)

    // Volume operations
    fn list_volumes(&self) -> Result<Vec<Volume>, ContainerError>;
    fn create_volume(&self, opts: &CreateVolumeOptions) -> Result<Volume, ContainerError>;  // CHANGED (Group L1)
    fn remove_volume(&self, name: &str, force: bool) -> Result<(), ContainerError>;

    // Network operations
    fn list_networks(&self) -> Result<Vec<Network>, ContainerError>;
    fn create_network(&self, opts: &CreateNetworkOptions) -> Result<Network, ContainerError>;  // CHANGED (Group L2)
    fn remove_network(&self, id: &str) -> Result<(), ContainerError>;

    // System
    fn system_df(&self) -> Result<SystemUsage, ContainerError>;
    fn prune_system(&self, volumes: bool) -> Result<PruneReport, ContainerError>;
    fn system_events(&self, since: Option<i64>, limit: Option<usize>) -> Result<Vec<ContainerEvent>, ContainerError>;  // TO ADD (Group F)
}
```

### Use case ports (`src/ports/use_cases/`)

```rust
// i_container_use_case.rs — current + additions
pub trait IContainerUseCase: Send + Sync {
    fn list(&self, all: bool) -> Result<Vec<Container>, ContainerError>;
    fn inspect(&self, id: &str) -> Result<Container, ContainerError>;
    fn start(&self, id: &str) -> Result<(), ContainerError>;
    fn stop(&self, id: &str, timeout_secs: Option<u32>) -> Result<(), ContainerError>;
    fn restart(&self, id: &str, timeout_secs: Option<u32>) -> Result<(), ContainerError>;
    fn pause(&self, id: &str) -> Result<(), ContainerError>;
    fn unpause(&self, id: &str) -> Result<(), ContainerError>;
    fn remove(&self, id: &str, force: bool, remove_volumes: bool) -> Result<(), ContainerError>;
    fn create(&self, opts: &CreateContainerOptions) -> Result<String, ContainerError>;
    fn rename(&self, id: &str, new_name: &str) -> Result<(), ContainerError>;
    fn logs(&self, id: &str, tail: Option<u32>, timestamps: bool) -> Result<String, ContainerError>;
    fn stats(&self, id: &str) -> Result<ContainerStats, ContainerError>;
    fn inspect_json(&self, id: &str) -> Result<String, ContainerError>;
    fn exec(&self, id: &str, cmd: &[&str]) -> Result<String, ContainerError>;
    fn start_all(&self, ids: &[&str]) -> Result<Vec<Result<(), ContainerError>>, ContainerError>;  // TO ADD (Group D)
    fn stop_all(&self, ids: &[&str], timeout_secs: Option<u32>) -> Result<Vec<Result<(), ContainerError>>, ContainerError>;  // TO ADD (Group D)
}

// i_image_use_case.rs — current + additions
pub trait IImageUseCase: Send + Sync {
    fn list(&self) -> Result<Vec<Image>, ContainerError>;
    fn pull(&self, reference: &str) -> Result<(), ContainerError>;
    fn remove(&self, id: &str, force: bool) -> Result<(), ContainerError>;
    fn tag(&self, source: &str, target: &str) -> Result<(), ContainerError>;
    fn inspect(&self, id: &str) -> Result<Image, ContainerError>;
    fn layers(&self, id: &str) -> Result<Vec<ImageLayer>, ContainerError>;  // TO ADD (Group A4)
}

// i_volume_use_case.rs — updated signature
pub trait IVolumeUseCase: Send + Sync {
    fn list(&self) -> Result<Vec<Volume>, ContainerError>;
    fn create(&self, opts: &CreateVolumeOptions) -> Result<Volume, ContainerError>;  // CHANGED (Group L1)
    fn remove(&self, name: &str, force: bool) -> Result<(), ContainerError>;
}

// i_network_use_case.rs — current + additions
pub trait INetworkUseCase: Send + Sync {
    fn list(&self) -> Result<Vec<Network>, ContainerError>;
    fn create(&self, opts: &CreateNetworkOptions) -> Result<Network, ContainerError>;  // CHANGED (Group L2)
    fn remove(&self, id: &str) -> Result<(), ContainerError>;
    fn system_df(&self) -> Result<SystemUsage, ContainerError>;
    fn prune(&self, volumes: bool) -> Result<PruneReport, ContainerError>;
    fn events(&self, since: Option<i64>, limit: Option<usize>) -> Result<Vec<ContainerEvent>, ContainerError>;  // TO ADD (Group F)
}
```

### `DynamicDriver` (`src/infrastructure/containers/dynamic_driver.rs`)

Hot-swappable wrapper — delegates everything to inner. When adding new `IContainerDriver` methods, add corresponding
delegation here.

```rust
pub struct DynamicDriver {
    inner: RwLock<Arc<dyn IContainerDriver>>
}
impl DynamicDriver {
    pub fn new(driver: Arc<dyn IContainerDriver>) -> Self;
    pub fn swap(&self, new_driver: Arc<dyn IContainerDriver>);
}
impl IContainerDriver for DynamicDriver { /* delegates */ }
```

### Factory (`src/infrastructure/containers/factory.rs`)

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeKind { Docker, Podman, Containerd }

impl ContainerDriverFactory {
    pub fn detect() -> Result<Arc<dyn IContainerDriver>, ContainerError>;
    pub fn detect_specific(name: &str) -> Result<Arc<dyn IContainerDriver>, ContainerError>;
    pub fn with_runtime(kind: RuntimeKind) -> Result<Arc<dyn IContainerDriver>, ContainerError>;
    pub fn available_runtimes() -> Vec<(RuntimeKind, String)>;
}
```

### Runtime preference persistence (`src/window/main_window.rs`)

```rust
pub fn load_runtime_pref() -> Option<String>;  // "docker" | "podman" | "containerd"
fn save_runtime_pref(name: &str);              // ~/.config/com.example.GtkCrossPlatform/runtime
```

---

## Driver implementation table

All five drivers must implement every method of `IContainerDriver`:

| Driver     | File                                                         |
|------------|--------------------------------------------------------------|
| Docker     | `src/infrastructure/containers/docker_driver.rs`             |
| Podman     | `src/infrastructure/containers/podman_driver.rs`             |
| containerd | `src/infrastructure/containers/containerd_driver.rs`         |
| Mock       | `src/infrastructure/containers/mock_driver.rs` (all tests)   |
| Dynamic    | `src/infrastructure/containers/dynamic_driver.rs` (hot-swap) |

When adding new port methods:

1. Add to `IContainerDriver` trait.
2. Implement in all five drivers (Docker, Podman, containerd, Mock, Dynamic).
3. Add to `IImageUseCase` / `IContainerUseCase` / etc. as appropriate.
4. Implement in the corresponding `*UseCase` struct.
5. Write tests.

---

## Implemented detail — v0.2 reference

### Container tabs (5 total, `containers_view.rs`)

1. **Info** (`build_info_tab`) — actions (Start/Stop, Pause/Unpause, Restart, Remove) + property grid (Name, ID, Image,
   Command, Status, Ports, Restart, Compose) + environment section with secret masking (keys matching
   `/pass|secret|key|token/i` → "••••••••").
2. **Stats** (`build_stats_tab`) — 4 sparklines (CPU, Mem, Net In/Out), 60-point rolling window, current values table; "
   Not running" state.
3. **Inspect** (`build_inspect_tab`) — syntax-highlighted JSON, Copy button; loads via
   `IContainerUseCase::inspect_json()`.
4. **Logs** (`build_logs_tab`, `:1162`) — toolbar (Timestamps toggle, Copy, Refresh, Follow toggle [Group I1],
   Clear [Group I2]), `gtk4::Stack` spinner→content, loads via `IContainerUseCase::logs(id, Some(200), timestamps)`.
5. **Terminal** (`build_terminal_tab`, `:1292`) — shell DropDown (`["sh","bash"]`), Entry + Run button + command history
   navigation (Arrow Up/Down, max 50 [Group J]); disabled when not running; appends output (never clears); loads via
   `IContainerUseCase::exec()`.

### Image detail — current

Single property pane: Tag, ID, Size, Created, Digest. Remove button in list row only.
No Layers tab yet (Group C2). No [Run] or [Push] buttons yet (Group C1). No In Use row (Group C1).

### Volume detail — current

Single property pane: Name, Driver, Mountpoint. No Size or In Use (Group C3).

### Network detail — current

Single property pane: Name, Driver, Scope, Subnet. No Gateway, Internal, or Containers count (Group C4).

---

## Logs tab — `build_logs_tab` (`containers_view.rs:1162`)

- 4th notebook tab: `"Logs"`.
- Toolbar: Timestamps `ToggleButton`, Follow `ToggleButton` (Group I1), Copy icon button, Clear icon button (Group I2),
  Refresh icon button — all with `set_tooltip_text` + `update_property(Property::Label(...))`.
- `gtk4::Stack`: "loading" (`gtk4::Spinner`) / "content" (`ScrolledWindow` + `TextView` monospace non-editable).
- Errors via `adw::ToastOverlay`; never inline label.
- Works for stopped containers — tab is never disabled.

## Terminal tab — `build_terminal_tab` (`containers_view.rs:1292`)

- 5th notebook tab: `"Terminal"`.
- `ScrolledWindow` + `TextView` monospace non-editable (output area, append-only).
- Input row: `gtk4::DropDown` (sh/bash) + `gtk4::Entry` + Run `gtk4::Button`.
- Command history: `Vec<String>` (max 50), navigated with `EventControllerKey` ArrowUp/ArrowDown (Group J).
- `set_sensitive(false)` on Entry and Run when container is not running; dim hint label explains.
- Run calls `spawn_driver_task` → `IContainerUseCase::exec(id, &[shell, "-c", cmd])`.
- Run button has `set_tooltip_text` + `update_property(Property::Label(...))`.
- Typing `exit` closes the session: appends "Session closed." and disables input (Group J).

---

## Roadmap (for reference — not in scope for this command)

| Version | Theme          | Key features                                                                                                                                                                                                                                                   |
|---------|----------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| v0.2    | MVP (done)     | Search, pull, wizard, stats, inspect, env, terminal, compose, dashboard, switcher                                                                                                                                                                              |
| v0.3    | Core           | Image layers, Run/Push buttons, dangling badge, volume size+in_use, network containers_count, compose lifecycle, rename, event stream, About dialog, empty states, follow mode, command history, conditional remove, create dialog completeness, compose badge |
| v0.4    | Differentiator | Volume file browser, CVE scanning (Trivy), Push image, Build image, Registry auth, Pods (Podman), Compose editor                                                                                                                                               |
| v1.0    | Platform       | Kubernetes basic, extensions API, system notifications, Flathub release                                                                                                                                                                                        |

---

## Test execution requirements

```sh
make test          # cargo test — 148 non-widget tests, zero failures
make lint          # cargo clippy -- -D warnings, zero warnings
make fmt           # cargo fmt --check, no diffs
```

Widget tests (5 tests, `#[ignore]`):

```sh
xvfb-run cargo test --test widget_test -- --test-threads=1 --ignored
```

**Mock driver rules:**

- Deterministic — no `thread::sleep`, no random data.
- Use `AtomicBool`/`AtomicUsize` for state, not timers.
- Must implement every `IContainerDriver` method — no regressions.

---

## Implementation order (for remaining work)

```
1. Baseline: make test → 148 green
2. Group A: domain model gaps (in_use, size, containers_count, ImageLayer)
3. make test green
4. Group B: sidebar row enhancements (dangling, size, containers count, compose badge, empty states)
5. Group C: detail pane completeness (image In Use + Layers tab, volume size+in_use, network completeness)
6. Group L: create dialog port completeness (CreateVolumeOptions, CreateNetworkOptions + UI driver/subnet)
7. make test green
8. Group D: compose stack lifecycle (start_all / stop_all + UI)
9. make test green
10. Group E: rename container inline
11. Group F: event stream (system_events port + dashboard integration)
12. Group G: About dialog (adw::AboutWindow)
13. Group H: menu popover completeness (Preferences stub + About trigger)
14. Group I: logs follow toggle + clear button
15. Group J: terminal command history + exit command
16. Group K: conditional remove protection for system networks
17. Group M: dashboard system metrics refresh timer
18. make test green
19. make lint && make fmt
```

---

## Exit criteria — v0.2 complete / v0.3 pending

### v0.2 — done

- [x] `make test` 148 non-widget tests, zero failures
- [x] `make lint` zero warnings
- [x] `make fmt` no diffs
- [x] Feature A — pull image streaming
- [x] Feature B — Logs tab UI (`build_logs_tab`)
- [x] Feature C — `IContainerUseCase::exec()` + Terminal tab UI (`build_terminal_tab`)
- [x] Feature D — Dashboard UI (`dashboard_view.rs`)
- [x] Feature E — Runtime switcher (`main_window.rs`)
- [x] Create volume dialog + create network dialog
- [x] `MockContainerDriver` implements all port methods (no regressions)
- [x] No GTK in `src/core/`; no tokio; `async_channel::bounded` on GTK side
- [x] Every user-visible string uses `gettext!()` or `pgettext!()`
- [x] Every icon-only button has `set_tooltip_text` + `update_property(Property::Label(...))`

### v0.3 — pending

- [ ] `Image.in_use` field (Group A1)
- [ ] `Volume.size_bytes` + `Volume.in_use` fields (Group A2)
- [ ] `Network.containers_count` field (Group A3)
- [ ] `ImageLayer` type + `inspect_image_layers` port + `IImageUseCase::layers()` + tests (Group A4)
- [ ] Image row: dangling badge (Group B1)
- [ ] Volume row: size + unused badge (Group B2)
- [ ] Network row: containers count (Group B3)
- [ ] Compose group header: "N/M running" badge (Group B4)
- [ ] Empty states: EmptyList + EmptySelection + no-results for all resource tabs (Group B5)
- [ ] Image detail: In Use row + [Run] button + [Push…] stub (Group C1)
- [ ] Image detail: Layers tab (Group C2)
- [ ] Volume detail: Size + In Use rows (Group C3)
- [ ] Network detail: Gateway + Internal + Containers rows (Group C4)
- [ ] Compose stack lifecycle (start_all / stop_all port + UI) (Group D)
- [ ] Rename container inline (Group E)
- [ ] Event stream (`system_events` port + dashboard + tests) (Group F)
- [ ] About dialog (`adw::AboutWindow` with runtime version + links) (Group G)
- [ ] Menu popover: Preferences stub + About trigger (Group H)
- [ ] Logs tab: Follow toggle + Clear button (Group I)
- [ ] Terminal: Arrow Up/Down history + `exit` command handling (Group J)
- [ ] System network remove protection (bridge/host/none) (Group K)
- [ ] `CreateVolumeOptions` (driver) + dialog driver select (Group L1)
- [ ] `CreateNetworkOptions` (driver + subnet) + dialog fields (Group L2)
- [ ] Dashboard: 30s refresh timer + "Last updated" label (Group M)
