---
description: Apply UI/architecture patterns identified from the Requester project — GSettings schema, CommonActions constants, signal-block guard, AdwClamp, SplitButton, Default+PartialEq on domain models, ContainerStatus converters, EmptyState component, ToastUtil, CSS dark split, bind_with_mapping for theme, GtkStack slide transitions, GtkSignalListItemFactory for ColumnView, accessibility roles on custom widgets, Cancellable lifecycle, destructive-toast undo pattern, sidebar-width-fraction persistence, and GtkShortcutController in XML. Runs make fmt and make test as exit criteria.
---

# Apply Requester Patterns

> Architecture rules, layer boundaries, threading model, A11Y, and i18n requirements are in
> `CLAUDE.md`. Read it before writing a single line of code.
>
> This command applies patterns identified from the Requester GTK4/Adwaita/Vala project.
> Each group is independent unless stated otherwise. Run `make test` after every group.

---

## Platform support matrix

This project targets **Linux**, **macOS**, **Windows (MSYS2/MINGW64)**, and **GNOME Mobile**
(Phosh/postmarketOS). Every change in this command must work correctly on all four. Groups with
platform-specific behaviour are marked with the affected platform in bold.

| Platform                 | GTK backend                 | GSettings backend                  | Dark mode source                    |
|--------------------------|-----------------------------|------------------------------------|-------------------------------------|
| Linux (Flatpak / native) | Wayland / X11               | dconf                              | GNOME Shell `color-scheme` portal   |
| macOS                    | Quartz (GTK4 macOS backend) | keyfile (`~/.config/`)             | macOS Dark Mode (system appearance) |
| Windows (MSYS2/MINGW64)  | Win32                       | keyfile (`%APPDATA%/glib-2.0/...`) | Windows 10/11 dark mode setting     |
| GNOME Mobile (Phosh)     | Wayland                     | dconf                              | GNOME Shell `color-scheme` portal   |

`adw::StyleManager::get_default()` abstracts over these sources — `is_dark()` and
`connect_dark_notify()` return the correct value on every platform without extra code.
`<Primary>` in accelerator strings maps to **Cmd** on macOS and **Ctrl** on
Linux/Windows/Mobile — always use `<Primary>`, never `<Control>`, to keep shortcuts
consistent across platforms.

---

## Baseline

```sh
make test   # must be green before touching anything
make lint   # zero warnings required
make fmt    # no diffs required
```

Record the test count. No regressions are acceptable.

---

## Group A — Domain model quality (`src/core/domain/`)

**Platform impact: none — pure Rust, no platform concerns.**

These changes are pure Rust — no GTK, no infrastructure imports.

### A1 — `#[derive(Default, PartialEq)]` on all domain models

Add `Default` and `PartialEq` to every domain struct and enum in `src/core/domain/`.

Files: `container.rs`, `image.rs`, `volume.rs`, `network.rs`.

```rust
// Before
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    ...
}

// After
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Container {
    ...
}
```

**Why `Default`:** allows `Container::default()` as a placeholder in `ListStore` rows before
the driver returns data, mirroring `HarRequest.empty()` from Requester.

**Why `PartialEq`:** enables `old_list != new_list` comparisons to skip redundant UI refreshes.
The `ContainerStats` struct (sparkline windows) is exempt — rolling windows are not meaningfully
comparable.

For enums (`ContainerStatus`, `ImageStatus`), `Default` requires nominating the zero-state
variant. Use `#[default]`:

```rust
#[derive(Debug, Clone, Default, PartialEq)]
pub enum ContainerStatus {
    #[default]
    Unknown,
    Running,
    // ...
}
```

Write one inline `#[cfg(test)]` test per file verifying `Container::default() == Container::default()`.

### A2 — Add converters to `ContainerStatus`

`ContainerStatus` is mapped to CSS classes and icon names in multiple view files today.
Centralise all three mappings inside the enum.

```rust
// src/core/domain/container.rs
impl ContainerStatus {
    /// Returns the Adwaita CSS accent class for StatusBadge.
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Running => "success",
            Self::Paused => "warning",
            Self::Stopped | Self::Exited => "dim-label",
            Self::Dead | Self::Error => "error",
            Self::Created | Self::Restarting => "accent",
            Self::Unknown => "dim-label",
        }
    }

    /// Returns the symbolic icon name for sidebar rows.
    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Running => "media-playback-start-symbolic",
            Self::Paused => "media-playback-pause-symbolic",
            Self::Stopped
            | Self::Exited => "media-playback-stop-symbolic",
            Self::Dead
            | Self::Error => "dialog-error-symbolic",
            Self::Restarting => "view-refresh-symbolic",
            Self::Created
            | Self::Unknown => "emblem-default-symbolic",
        }
    }

    /// Returns the short display label shown alongside the color badge.
    pub fn display_label(&self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::Paused => "Paused",
            Self::Stopped => "Stopped",
            Self::Exited => "Exited",
            Self::Dead => "Dead",
            Self::Error => "Error",
            Self::Created => "Created",
            Self::Restarting => "Restarting",
            Self::Unknown => "Unknown",
        }
    }
}
```

After adding the methods, grep all views for inline `match status` / `if status ==` patterns
that duplicate these mappings and replace them with calls to `status.css_class()`,
`status.icon_name()`, and `status.display_label()`.

Write three inline unit tests: one per method, exercising `Running` and `Unknown`.

---

## Group B — `actions` module (`src/window/actions.rs`)

**Platform impact: none — GLib action names are platform-agnostic strings.**

Create a new file `src/window/actions.rs` with string constants for every GLib action name
used in the project. This eliminates magic strings and makes typos into compile errors.

```rust
// src/window/actions.rs

pub mod app {
    pub const QUIT: &str = "app.quit";
    pub const ABOUT: &str = "app.about";
    pub const PREFERENCES: &str = "app.preferences";
}

pub mod win {
    pub const REFRESH: &str = "win.refresh";
    pub const PRUNE_SYSTEM: &str = "win.prune-system";
    pub const FOCUS_SEARCH: &str = "win.focus-search";
    pub const CLEAR_SEARCH: &str = "win.clear-search";
    pub const CLOSE: &str = "window.close";
}
```

Then in `src/window/mod.rs`, add `pub mod actions;`.

Replace every string literal that matches an action name in `src/app.rs` and
`src/window/main_window.rs` with the corresponding constant.

```rust
// Before
app.set_accels_for_action("app.quit", & ["<Primary>q"]);

// After
use crate::window::actions;
app.set_accels_for_action(actions::app::QUIT, & ["<Primary>q"]);
```

No tests required for this group — the compiler verifies correctness.

---

## Group C — `EmptyState` component (`src/window/components/empty_state.rs`)

**Platform impact: none — `adw::StatusPage` and `adw::Clamp` render identically on all
platforms with the Adwaita theme.**

Each resource view currently duplicates `adw::StatusPage` construction for empty lists,
no-results states, and empty detail panes. Extract into a reusable builder.

```rust
// src/window/components/empty_state.rs
use adw::prelude::*;
use gettextrs::gettext;

pub struct EmptyState;

impl EmptyState {
    /// Empty list — no resources exist yet.
    pub fn no_items(icon: &str, title: &str, body: &str) -> adw::StatusPage {
        let page = adw::StatusPage::new();
        page.set_icon_name(Some(icon));
        page.set_title(&gettext(title));
        page.set_description(Some(&gettext(body)));
        page
    }

    /// No-results — list is non-empty but search filter yields nothing.
    pub fn no_results(query: &str) -> adw::StatusPage {
        let page = adw::StatusPage::new();
        page.set_icon_name(Some("edit-find-symbolic"));
        page.set_title(&gettext("No Results"));
        page.set_description(Some(&format!(
            "{} \u{201c}{}\u{201d}",
            gettext("No matches for"),
            query
        )));
        page
    }

    /// Empty selection — detail pane when nothing is selected.
    pub fn no_selection(icon: &str, title: &str, body: &str) -> adw::StatusPage {
        Self::no_items(icon, title, body)
    }
}
```

Add `pub mod empty_state;` to `src/window/components/mod.rs`.

After creating the component, replace every inline `adw::StatusPage::new()` construction in
the four resource views with `EmptyState::no_items(...)`, `EmptyState::no_results(...)`,
or `EmptyState::no_selection(...)` as appropriate.

Wrap all `adw::StatusPage` instances used as detail-pane content inside an `adw::Clamp` with
`maximum_size = 480` so they do not stretch on wide monitors:

```rust
let clamp = adw::Clamp::new();
clamp.set_maximum_size(480);
clamp.set_child(Some( & empty_state));
```

---

## Group D — `ToastUtil` helper (`src/window/components/toast_util.rs`)

**Platform impact: none — `adw::Toast` renders identically on all platforms.**

Views currently build `adw::Toast` objects inline. Centralise toast construction with four
variants covering the full toast taxonomy:

```rust
// src/window/components/toast_util.rs
use adw::prelude::*;
use gettextrs::gettext;

pub struct ToastUtil;

impl ToastUtil {
    /// Transient confirmation (default 2 s). Use for non-destructive feedback.
    pub fn show(overlay: &adw::ToastOverlay, message: &str) {
        let toast = adw::Toast::new(message);
        overlay.add_toast(toast);
    }

    /// Persistent error (timeout = 0, stays until dismissed). Use for failures.
    pub fn show_error(overlay: &adw::ToastOverlay, message: &str) {
        let toast = adw::Toast::new(message);
        toast.set_timeout(0);
        overlay.add_toast(toast);
    }

    /// Destructive confirmation with Undo button (timeout = 10 s, HIGH priority).
    /// The Requester uses this for delete operations: the action fires immediately,
    /// then the undo button reverses it within the 10 s window.
    ///
    /// `undo_action_name` must be a registered `gio::SimpleAction` on the window
    /// (e.g. "win.undo-remove-container").
    pub fn show_destructive(
        overlay: &adw::ToastOverlay,
        message: &str,
        undo_action_name: &str,
    ) {
        let toast = adw::Toast::new(message);
        toast.set_priority(adw::ToastPriority::High);
        toast.set_timeout(10);
        toast.set_button_label(Some(&gettext("Undo")));
        toast.set_action_name(Some(undo_action_name));
        overlay.add_toast(toast);
    }

    /// Inline action toast (e.g. "Image pulled — View" or "Config missing — Open Preferences").
    pub fn show_with_action(
        overlay: &adw::ToastOverlay,
        message: &str,
        action_label: &str,
        action_name: &str,
    ) {
        let toast = adw::Toast::new(message);
        toast.set_button_label(Some(&gettext(action_label)));
        toast.set_action_name(Some(action_name));
        overlay.add_toast(toast);
    }
}
```

Add `pub mod toast_util;` to `src/window/components/mod.rs`.

Replace ad-hoc `adw::Toast::new(...)` + `overlay.add_toast(...)` pairs in all four views
with the corresponding `ToastUtil` call.

For container remove and image remove operations, switch from `show()` to `show_destructive()`
with a `"win.undo-remove-{resource}"` action. Implement the undo action as a `gio::SimpleAction`
in the view that re-adds the resource (or at minimum shows a toast explaining it cannot be
undone after the operation completes, disabling the undo button).

---

## Group E — Signal-block guard for reactive updates

**Platform impact: none — `glib::signal::signal_handler_block` is a core GLib API available
on all platforms.**

When a view updates a widget programmatically (e.g., re-selecting an item after a list
refresh), the widget emits `notify::` signals that trigger handlers, causing unwanted
re-entrancy. Use `glib::signal::signal_handler_block` / `signal_handler_unblock` paired
with the stored `SignalHandlerId`.

Identify every location in the four resource views where a `ListBox`, `DropDown`, or
`SelectionModel` is updated programmatically after a driver call, and guard them:

```rust
// Pattern to apply wherever programmatic updates trigger handlers
glib::signal::signal_handler_block( & widget, & handler_id);
widget.set_selected(new_index);   // or set_selected_rows, etc.
glib::signal::signal_handler_unblock( & widget, & handler_id);
```

The `handler_id: glib::SignalHandlerId` must be stored alongside the widget reference
(typically as a field of the view's `imp` struct or inside a `Rc<Cell<...>>`).

Focus on the `selected-item` / `notify::selected` signal handlers in the sidebar list
of each view, which are the most common source of re-entrancy bugs.

No new tests required, but run `make test` to confirm no regressions.

---

## Group F — `bind_property` for theme synchronisation in `src/app.rs`

**Platform impact: none — `adw::StyleManager` abstracts dark-mode detection on all
platforms (GNOME Shell portal on Linux/Mobile, macOS Dark Mode via Quartz backend,
Windows 10/11 dark mode via Win32 backend). The `connect_color_scheme_notify` callback
fires correctly on all four targets with no platform-specific code.**

The application already reads `adw::StyleManager::get_default()` but performs no active
binding. Add a binding that keeps the app color scheme in sync with the system dark-mode
preference using `bind_property_full()` with a transform closure:

```rust
// src/app.rs — inside setup_actions() or activate()
let style_manager = adw::StyleManager::get_default();

// Keep a GSettings instance for the "color-scheme" key.
// Schema registration is in Group G; for now, fall back to direct StyleManager binding.
style_manager.connect_color_scheme_notify( | sm| {
// Future: persist selection to GSettings here (Group G).
let _ = sm.color_scheme();
});
```

This is a placeholder. The full binding (GSettings ↔ StyleManager) requires Group G.
Do not skip ahead — implement the binding after Group G is complete.

---

## Group G — GSettings schema (`data/com.example.GtkCrossPlatform.gschema.xml`)

**Platform impact: requires platform-specific setup steps (see below). The GSettings API
itself works on all platforms but the backend and tooling differ.**

> Also wires `sidebar-width-fraction` to `AdwNavigationSplitView` — see "sidebar width
> persistence" at the end of this group.

### Backend behaviour per platform

| Platform        | GSettings backend                                  | Config location                |
|-----------------|----------------------------------------------------|--------------------------------|
| Linux (Flatpak) | dconf                                              | GNOME dconf database           |
| Linux (native)  | dconf (falls back to keyfile if dconf unavailable) | `~/.config/`                   |
| macOS           | keyfile (dconf is Linux-only)                      | `~/.config/glib-2.0/settings/` |
| Windows (MSYS2) | keyfile                                            | `%APPDATA%/glib-2.0/settings/` |
| GNOME Mobile    | dconf                                              | GNOME dconf database           |

All four platforms read and write settings via the same `gio::Settings` API — only the
storage location differs. No platform-conditional code is needed in Rust.

### Schema file

Create `data/com.example.GtkCrossPlatform.gschema.xml`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<schemalist>
    <schema id="com.example.GtkCrossPlatform" path="/com/example/GtkCrossPlatform/">

        <!-- Window geometry -->
        <key name="window-width" type="i">
            <default>1200</default>
            <summary>Window width</summary>
        </key>
        <key name="window-height" type="i">
            <default>700</default>
            <summary>Window height</summary>
        </key>
        <key name="window-maximized" type="b">
            <default>false</default>
            <summary>Whether the window is maximized</summary>
        </key>

        <!-- Appearance -->
        <key name="color-scheme" type="s">
            <choices>
                <choice value="default"/>
                <choice value="force-light"/>
                <choice value="force-dark"/>
            </choices>
            <default>'default'</default>
            <summary>Application color scheme</summary>
        </key>

        <!-- Runtime preference -->
        <key name="preferred-runtime" type="s">
            <default>''</default>
            <summary>Last selected container runtime ('docker', 'podman', 'containerd', or '')</summary>
        </key>

        <!-- Layout — fraction of the window width occupied by the sidebar (0.0–1.0) -->
        <key name="sidebar-width-fraction" type="d">
            <range min="0.15" max="0.5"/>
            <default>0.25</default>
            <summary>Sidebar width as a fraction of the window width</summary>
        </key>

    </schema>
</schemalist>
```

> **Why `sidebar-width-fraction` (double) and not `sidebar-width` (integer)?**
> `AdwNavigationSplitView` does **not** expose a `sidebar-width` integer property.
> The available layout properties are `min-sidebar-width` (float, sp), `max-sidebar-width`
> (float, sp), and `sidebar-width-fraction` (float, 0..1). A direct `gio::Settings::bind`
> only works when the GSettings type exactly matches the GObject property type.
> Binding an integer key to a float property would panic at runtime on every platform.
> Use `sidebar-width-fraction` (type `"d"`) bound directly to the widget property of the
> same name — no `bind_with_mapping` needed.

### Linux / Flatpak distribution — `data/meson.build`

This file is used **only** for the Flatpak build pipeline. macOS uses `make dmg`;
Windows uses MSYS2 + the local schema directory. Create `data/meson.build`:

```meson
# data/meson.build — Linux/Flatpak only
install_data(
  'com.example.GtkCrossPlatform.gschema.xml',
  install_dir: get_option('datadir') / 'glib-2.0' / 'schemas',
)

gnome.compile_schemas(
  depend_files: 'com.example.GtkCrossPlatform.gschema.xml',
)
```

### Local development (`make` workflow) — all platforms

> **Windows note:** `glib-compile-schemas` ships in the `glib2` MSYS2 package. If it is not
> in PATH, run: `pacman -S mingw-w64-x86_64-glib2` in the MSYS2 MINGW64 shell. The
> `setup-windows` instructions in `Makefile` must be updated to include this package.

Update `Makefile`:

```makefile
SCHEMA_DIR := $(shell pwd)/data

schema:
	glib-compile-schemas $(SCHEMA_DIR)

run: schema build
	GSETTINGS_SCHEMA_DIR=$(SCHEMA_DIR) cargo run

run-mobile: schema build
	GSETTINGS_SCHEMA_DIR=$(SCHEMA_DIR) GTK_DEBUG=interactive cargo run
```

> **Syntax note:** The `VAR=value command` inline env-var syntax is POSIX sh and works in
> bash, zsh, and MSYS2 MINGW64. It does **not** work in CMD.exe or PowerShell natively.
> The project documents MSYS2 MINGW64 as the Windows dev shell — this is sufficient. Users
> on Windows who use a different shell must set `GSETTINGS_SCHEMA_DIR` manually.

Add `data/gschemas.compiled` to `.gitignore`.

### macOS `.app` bundle — `make dmg`

The existing `make dmg` target in `Makefile` already runs `glib-compile-schemas` for the
system schemas bundled from Homebrew. After adding the project schema, extend the `dmg`
target to also copy it into the bundle before compiling:

```makefile
	# In the dmg target, after "mkdir -p $(BUNDLE)/Contents/Resources/share/glib-2.0/schemas/"
	cp data/com.example.GtkCrossPlatform.gschema.xml \
		"$(BUNDLE)/Contents/Resources/share/glib-2.0/schemas/"
	glib-compile-schemas "$(BUNDLE)/Contents/Resources/share/glib-2.0/schemas/"
```

This ensures the app schema is available inside the bundle without relying on the system
schema directory.

### Wire GSettings in `src/app.rs`

```rust
use gio::Settings;

// In GtkCrossPlatformAppImpl::activate():
let settings = Settings::new("com.example.GtkCrossPlatform");

// Persist + restore window geometry
let window_width  = settings.int("window-width");
let window_height = settings.int("window-height");
let maximized     = settings.boolean("window-maximized");
win.set_default_size(window_width, window_height);
if maximized { win.maximize(); }

settings.bind("window-width", & win, "default-width")
.flags(gio::SettingsBindFlags::DEFAULT)
.build();
settings.bind("window-height", & win, "default-height")
.flags(gio::SettingsBindFlags::DEFAULT)
.build();
settings.bind("window-maximized", & win, "maximized")
.flags(gio::SettingsBindFlags::DEFAULT)
.build();
```

### Wire color-scheme binding after Group F placeholder

```rust
// Map GSettings string → adw::ColorScheme
let style_manager = adw::StyleManager::get_default();
let sm = style_manager.clone();
settings.connect_changed(Some("color-scheme"), move | s, _ | {
let scheme = match s.string("color-scheme").as_str() {
"force-dark" => adw::ColorScheme::ForceDark,
"force-light" => adw::ColorScheme::ForceLight,
_ => adw::ColorScheme::Default,
};
sm.set_color_scheme(scheme);
});
// Apply on startup
{
let scheme = match settings.string("color-scheme").as_str() {
"force-dark" => adw::ColorScheme::ForceDark,
"force-light" => adw::ColorScheme::ForceLight,
_ => adw::ColorScheme::Default,
};
style_manager.set_color_scheme(scheme);
}
```

### Persist runtime preference via GSettings

The current implementation persists the runtime preference as a plain file in
`glib::user_config_dir()`. Replace it with GSettings for consistency.

> **Migration note:** existing users who have a file-based preference will not have it
> automatically migrated — the first run after this change will fall back to auto-detect.
> This is acceptable: auto-detect picks the correct runtime in the common single-runtime
> case. If a seamless migration is needed, add a one-time migration in `activate()` that
> reads the file, writes it to GSettings, then deletes the file.

```rust
// Read
let runtime_name = settings.string("preferred-runtime");
if ! runtime_name.is_empty() { /* use it */ }

// Write (on runtime switch)
settings.set_string("preferred-runtime", selected_name).ok();
```

Remove `save_runtime_pref()`, `load_runtime_pref()`, and `runtime_pref_path()` from
`main_window.rs` after wiring the GSettings reads/writes.

### Persist `AdwNavigationSplitView` sidebar width fraction

Bind the `sidebar-width-fraction` GSettings key to the split view's
`sidebar-width-fraction` property. Both are type `f64` — no mapping closure needed.

```rust
// src/window/main_window.rs — after the split_view is available
let settings = gio::Settings::new("com.example.GtkCrossPlatform");
settings
.bind("sidebar-width-fraction", & split_view, "sidebar-width-fraction")
.flags(gio::SettingsBindFlags::DEFAULT)
.build();
```

Keep `max-sidebar-width="320"` in `window.ui` as a hard ceiling and keep
`min-sidebar-width` at a sensible floor (240 sp). The GSettings binding controls the
*initial* fraction; the hard constraints in XML prevent the sidebar from becoming
unusable regardless of what GSettings returns.

---

## Group H — `adw::AboutWindow` with `issue_url` and runtime credits

**Platform impact: none — `adw::AboutWindow` renders identically on all platforms.**

> **Before implementing:** check `src/app.rs` — `show_about_window()` may already include
> `issue_url`, `website`, `add_link`, and the runtime credit section via `spawn_driver_task`.
> If so, verify completeness against the spec below rather than re-implementing from scratch.
> Duplicate registrations of `add_link` or `add_credit_section` are silently ignored by
> libadwaita but produce confusing duplicated rows in the dialog — audit carefully.

The about window must include:

```rust
// src/app.rs — about_action handler
let about = adw::AboutWindow::builder()
.application_name( & gettext("Container Manager"))
.application_icon("com.example.GtkCrossPlatform")
.version(config::VERSION)
.comments( & gettext(
"A native GNOME application for managing Docker, Podman, and containerd containers.",
))
.developer_name("Container Manager Contributors")
.website("https://github.com/your-org/gtk-cross-platform")
.issue_url("https://github.com/your-org/gtk-cross-platform/issues")
.license_type(gtk4::License::Gpl30)
.copyright("© 2026 Container Manager Contributors")
.translator_credits( & gettext("translator-credits"))
.transient_for(active_window.as_ref().unwrap())
.build();

// Toolkit section (compile-time constants — no async needed)
about.add_credit_section(
Some( & gettext("Toolkit")),
& [ & format!(
    "GTK {}.{} / Libadwaita {}.{}",
    gtk4::major_version(), gtk4::minor_version(),
    adw::major_version(), adw::minor_version(),
)],
);

about.add_link( & gettext("Source Code"), "https://github.com/your-org/gtk-cross-platform");
about.add_link( & gettext("Report an Issue"), "https://github.com/your-org/gtk-cross-platform/issues");
```

For the runtime version credit, load it asynchronously via `spawn_driver_task`. If the
driver call fails, show `"Unknown"`:

```rust
let about_weak = about.downgrade();
spawn_driver_task(
driver.clone(),
| d| d.version(),
move | result| {
let version = result.unwrap_or_else( | _ | gettext("Unknown"));
if let Some(about_ref) = about_weak.upgrade() {
about_ref.add_credit_section(
Some( & gettext("Container Runtime")),
& [ & version],
);
}
},
);

about.present();
```

---

## Group I — CSS split: `style.css` and `style-dark.css`

**Platform impact:**

- **macOS:** `adw::StyleManager::is_dark()` follows macOS Dark Mode via the GTK4 Quartz backend. `connect_dark_notify`
  fires correctly when the user toggles Appearance in System Settings.
- **Windows:** follows Windows 10/11 dark mode setting via the Win32 backend. Same behaviour as macOS — no extra code
  needed.
- **Linux / Mobile:** follows the GNOME Shell `color-scheme` portal. Already the primary target.
- The `gtk::style_context_add_provider_for_display` / `style_context_remove_provider_for_display` APIs are
  GDK-backend-agnostic and work identically on all platforms.

The single `data/resources/style.css` mixes layout rules and color overrides.
Split it:

1. Keep `style.css` for layout-only rules (touch targets, spacing, status badge shape,
   monospace font, column view borders).
2. Create `data/resources/style-dark.css` for `@define-color` overrides or rule variants
   that only apply in dark mode.

Add `style-dark.css` to `data/resources/resources.gresource.xml`:

```xml

<file>style-dark.css</file>
```

Load it conditionally in `src/app.rs` (or wherever `style.css` is loaded):

```rust
let display = gtk4::gdk::Display::default ().expect("no display");
let provider = gtk4::CssProvider::new();
provider.load_from_resource("/com/example/GtkCrossPlatform/style.css");
gtk4::style_context_add_provider_for_display(
& display,
& provider,
gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
);

// Dark override — load and unload as theme changes
let dark_provider = gtk4::CssProvider::new();
dark_provider.load_from_resource("/com/example/GtkCrossPlatform/style-dark.css");

let dp = dark_provider.clone();
adw::StyleManager::get_default().connect_dark_notify( move | sm| {
if sm.is_dark() {
gtk4::style_context_add_provider_for_display(
& gtk4::gdk::Display::default().unwrap(),
&dp,
gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
);
} else {
gtk4::style_context_remove_provider_for_display(
& gtk4::gdk::Display::default().unwrap(),
&dp,
);
}
});
// Apply on startup
if adw::StyleManager::get_default().is_dark() {
gtk4::style_context_add_provider_for_display(
& display,
& dark_provider,
gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
);
}
```

If `style.css` currently has no dark-specific rules, create `style-dark.css` as an empty
file with a comment header — it signals the structure is ready for future additions.

---

## Group J — `AdwClamp` in detail panes

**Platform impact: none — `adw::Clamp` renders identically on all platforms and is
particularly beneficial on macOS and Windows where windows commonly open maximised.**

Detail panes in the four resource views use `adw::PreferencesGroup` inside a
`gtk::ScrolledWindow`. On wide monitors, the property rows stretch to full width, breaking
the GNOME HIG layout intent. Wrap the `adw::PreferencesGroup` (and any `adw::StatusPage`
used as detail content) in an `adw::Clamp`:

```rust
// In each show_detail() function — wrap the outermost content widget
let clamp = adw::Clamp::new();
clamp.set_maximum_size(720);
clamp.set_tightening_threshold(600);
clamp.set_child(Some( & prefs_group));  // or &status_page

let scrolled = gtk4::ScrolledWindow::new();
scrolled.set_vexpand(true);
scrolled.set_hexpand(true);
scrolled.set_child(Some( & clamp));
```

Apply to `containers_view.rs`, `images_view.rs`, `volumes_view.rs`, `networks_view.rs`.

The `detail_pane.rs` component builder (`src/window/components/detail_pane.rs`) should
apply `AdwClamp` internally so callers do not need to add it manually — update the
component if `build_detail_pane()` is the central factory.

---

## Group K — `GtkStack` slide transitions in sidebar navigation

**Platform impact: none — `GtkStack` transitions are rendered by GTK4 and behave
identically on all platforms. The animation is driven by GLib's frame clock, which is
backed by Wayland/X11 on Linux, Core Animation on macOS, and the Win32 compositor on
Windows.**

When navigating from the list pane to the detail pane on mobile (collapsed
`AdwNavigationSplitView`), the content area currently has no slide animation. Add a
`GtkStack` with `slide-left-right` transition where context changes are navigational
(list → detail, resource type change) as opposed to content switches (tab changes within
a detail pane, which keep their existing notebook transitions).

In `window.ui`, update the `AdwNavigationSplitView` sidebar page:

```xml

<object class="GtkStack" id="sidebar_stack">
    <property name="transition-type">slide-left-right</property>
    <property name="transition-duration">200</property>
</object>
```

If the sidebar already uses a flat list, this change applies when switching resource views
in the `AdwViewStack`. Verify the current transition type in `window.ui` and set it to
`slide-left-right` if it is currently `none` or `crossfade`.

---

## Group L — `SplitButton` for multi-action buttons

**Platform impact: none — `adw::SplitButton` renders identically on all platforms.**

In `images_view.rs`, the "Run" button on the image detail pane is a primary action with a
related secondary action ("Push…" stub). Replace the standalone `gtk::Button` with
`adw::SplitButton`:

```rust
let run_split = adw::SplitButton::new();
run_split.set_label( & gettext("Run"));
run_split.set_icon_name("media-playback-start-symbolic");
run_split.set_tooltip_text(Some( & gettext("Run a container from this image")));

let push_action = gio::SimpleAction::new("push-stub", None);
let push_menu = gio::Menu::new();
push_menu.append(Some( & gettext("Push to registry…")), Some("win.push-stub"));
run_split.set_menu_model(Some( & push_menu));

run_split.connect_clicked(clone!(@weak self as view => move |_| {
    view.show_create_container_dialog_with_image(&image_id);
}));
```

A11Y: `set_tooltip_text` is not sufficient for icon-only buttons. Also call:

```rust
run_split.update_property( & [gtk4::accessible::Property::Label(
& gettext("Run container from image"),
)]);
```

---

## Group M — `GtkSignalListItemFactory` for `ColumnView` (modern list pattern)

**Platform impact: none — `GtkSignalListItemFactory` and `GtkColumnView` are core GTK4
APIs available on all platforms. `GtkListBox::bind_model()` was deprecated in GTK 4.10 on
all platforms equally.**

The factory separates widget creation (setup) from data binding (bind), allowing GTK to
reuse widgets as rows scroll off-screen:

```rust
// Reusable factory builder — put in src/window/components/list_factory.rs
use gtk4::prelude::*;

pub fn make_factory<S, B>(setup: S, bind: B) -> gtk4::SignalListItemFactory
where
    S: Fn(&gtk4::SignalListItemFactory, &gtk4::ListItem) + 'static,
    B: Fn(&gtk4::SignalListItemFactory, &gtk4::ListItem) + 'static,
{
    let factory = gtk4::SignalListItemFactory::new();
    factory.connect_setup(setup);
    factory.connect_bind(bind);
    factory
}
```

Apply this pattern in `DataTable`-like contexts — specifically the environment variables
section in `containers_view.rs` (currently uses a manual `gtk::ListBox`). Replace with
`gtk4::ColumnView`:

```rust
// Setup: create the empty widget once per visible row slot
let name_factory = make_factory(
| _, item| {
let entry = gtk4::Entry::new();
entry.add_css_class("transparent");
item.set_child(Some( & entry));
},
| _, item| {
let entry = item.child().and_downcast::< gtk4::Entry > ().unwrap();
if let Some(env_var) = item.item().and_downcast::< glib::BoxedAnyObject > () {
entry.set_text( & env_var.borrow::<EnvVar > ().name);
}
},
);

let name_col = gtk4::ColumnViewColumn::new(Some( & gettext("Name")), Some(name_factory));
column_view.append_column( & name_col);
```

**Model:** Use `gio::ListStore` backed by a `glib::BoxedAnyObject` wrapping the domain type
(e.g., `EnvVar`). `gio::ListStore` is `GObject`-aware and integrates directly with
`GtkSelectionModel`.

**Do not use `GtkListBox::bind_model()`** for any new list code — it is a deprecated API.
Existing usages in the four resource views should be migrated opportunistically (not
required to complete this command, but flag each one with a `// TODO: migrate to ColumnView`
comment).

Add `pub mod list_factory;` to `src/window/components/mod.rs`.

---

## Group N — Accessibility roles on custom widgets and `.ui` files

**Platform impact: none — GTK4 accessibility is backed by AT-SPI2 on Linux, NSAccessibility
on macOS, and UIA on Windows. The same `set_accessible_role()` and `update_property()` calls
produce the correct platform-native accessibility tree on all targets.**

Every custom widget must declare its accessible role, and every interactive element in
`.ui` files must have an accessibility label. Missing roles and labels make the application
unusable with screen readers.

### N1 — `set_accessible_role()` on custom GTK widget subclasses

In `src/window/components/status_badge.rs` and any other `gtk::Widget` subclass defined
in the project, set the accessible role in the class init:

```rust
// src/window/components/status_badge.rs — inside ObjectImpl::class_init()
fn class_init(klass: &mut Self::Class) {
    klass.set_accessible_role(gtk4::AccessibleRole::Status);
    // ... rest of class init
}
```

Accessible roles to use per widget type:

| Widget                     | Role                                                            |
|----------------------------|-----------------------------------------------------------------|
| `StatusBadge`              | `gtk4::AccessibleRole::Status`                                  |
| `ResourceRow` (action row) | `gtk4::AccessibleRole::ListItem`                                |
| Icon-only toolbar buttons  | `gtk4::AccessibleRole::Button` (already default, but add label) |
| Search entry               | `gtk4::AccessibleRole::SearchBox`                               |

### N2 — Accessibility labels in `.ui` files

Every button, toggle, or interactive element that lacks visible text must have an
`<accessibility>` block in its `.ui` declaration:

```xml
<!-- data/resources/window.ui or component .ui files -->
<object class="GtkButton" id="refresh_btn">
    <property name="icon-name">view-refresh-symbolic</property>
    <accessibility>
        <property name="label" translatable="yes">Refresh</property>
    </accessibility>
</object>
```

Audit `data/resources/window.ui` and all component `.ui` files. For every icon-only
button, toggle button, or image widget, add the `<accessibility>` block if absent.

### N3 — `update_property` in Rust code

For buttons created in Rust (not in `.ui`), the existing `set_tooltip_text()` call is
insufficient. Add `update_property` alongside every `set_tooltip_text`:

```rust
// Pattern to apply to every icon-only button in the four resource views
btn.set_tooltip_text(Some( & gettext("Remove container")));
btn.update_property( & [gtk4::accessible::Property::Label(
& gettext("Remove container"),
)]);
```

Grep for `set_tooltip_text` in `src/window/views/` and add the corresponding
`update_property` call immediately after each one.

---

## Group O — `Cancellable` lifecycle for async operations

**Platform impact: none — `gio::Cancellable` is a core GIO API available on all
platforms. The cancel-on-destroy pattern prevents stale-callback bugs on every platform.**

Every async operation that crosses a widget boundary (spawned via `spawn_driver_task`)
must be associated with a `gio::Cancellable` that is:

1. **Reset** before starting a new operation of the same type.
2. **Cancelled** when the widget is destroyed or navigated away from.

This prevents "use-after-free" style bugs where a callback fires after the widget has
been dropped.

### O1 — Per-view `Cancellable` field

Add one `Cancellable` per long-running operation category in each view's `imp` struct:

```rust
// In each view's imp struct (e.g., containers_view.rs)
use std::cell::Cell;
use gio::Cancellable;

pub struct ContainersViewImp {
    // ... existing fields
    pub list_cancellable: Cell<Option<Cancellable>>,
    pub detail_cancellable: Cell<Option<Cancellable>>,
}
```

### O2 — Reset before dispatch

Before calling `spawn_driver_task`, cancel the previous operation and create a fresh
`Cancellable`:

```rust
fn load_list(&self) {
    // Cancel any in-flight list load
    if let Some(c) = self.imp().list_cancellable.take() {
        c.cancel();
    }
    let cancellable = Cancellable::new();
    self.imp().list_cancellable.set(Some(cancellable.clone()));

    spawn_driver_task(
        self.use_case.clone(),
        move |uc| uc.list(true),
        clone!(@weak self as view => move |result| {
            view.imp().list_cancellable.set(None);
            // ... update UI
        }),
    );
}
```

### O3 — Cancel on widget destroy

Connect a `destroy` signal handler to cancel all in-flight operations:

```rust
// In the view's constructed() or setup_signals()
self .connect_destroy( | view| {
if let Some(c) = view.imp().list_cancellable.take()   { c.cancel(); }
if let Some(c) = view.imp().detail_cancellable.take() { c.cancel(); }
});
```

### O4 — Ignore results from cancelled operations

In every `spawn_driver_task` callback, check whether the cancellable was cancelled before
updating the UI:

```rust
// In the callback closure
move | result| {
// If the view was destroyed or navigated away, cancellable was taken → None
if view.imp().list_cancellable.borrow().is_none() {
return; // stale result, discard
}
// ... safe to update UI
}
```

Write one integration test in `tests/cancellable_test.rs` verifying that cancelling a
`MockContainerDriver` operation before the callback fires does not panic or update state.

---

## Group P — `GtkShortcutController` in XML for component-local shortcuts

**Platform impact:**

- **macOS:** The `<Primary>` modifier maps to the **Cmd** key on macOS and **Ctrl** on
  Linux/Windows/Mobile. Always use `<Primary>`, **never `<Control>`**, in shortcut trigger
  strings. Using `<Control>f` instead of `<Primary>f` would make the focus-search shortcut
  Ctrl+F on macOS — inconsistent with the rest of the codebase and with macOS conventions.
- All other platforms: `<Primary>` = Ctrl, identical to `<Control>`. No behavioural difference.
- The `GtkShortcutController` XML element and `scope` attribute are GTK4 core APIs,
  available identically on all platforms.

Application-level shortcuts (`Ctrl+Q`, `F5`, etc.) belong in `src/app.rs` via
`set_accels_for_action`. Component-local shortcuts (e.g., `Primary+F` to focus search within
a sidebar, `Escape` to clear search) belong in the `.ui` file of the component itself,
using `GtkShortcutController`.

Add the following to `data/resources/window.ui` for the main window's search shortcut:

```xml
<!-- Inside the AdwNavigationSplitView sidebar page or the containing widget -->
<child>
    <object class="GtkShortcutController">
        <property name="scope">local</property>
        <child>
            <object class="GtkShortcut">
                <property name="trigger">&lt;Primary&gt;f</property>
                <property name="action">action(win.focus-search)</property>
            </object>
        </child>
    </object>
</child>
```

**Scope values:**

- `local` — only active when the containing widget has keyboard focus (use for
  component-specific shortcuts)
- `global` — active whenever the window has focus (use sparingly; prefer
  `set_accels_for_action` for global app shortcuts to keep them discoverable)

**Apply to:**

- Sidebar search focus: `<Primary>f` → `win.focus-search` (scope: `local`)
- Search bar dismiss: `Escape` → `win.clear-search` (scope: `local`)
- Detail refresh: `<Primary>r` is already a global accel → do not duplicate in XML

Register the `win.focus-search` and `win.clear-search` actions in `main_window.rs` if they
do not exist, and add their names to `src/window/actions.rs` (Group B).

---

## Exit Criteria

Run all checks in this order:

```sh
make fmt      # cargo fmt --check — zero diffs
make lint     # cargo clippy -- -D warnings — zero warnings
make test     # cargo test — same count as baseline, zero failures
```

Verify the schema compiles cleanly on every platform before marking done:

```sh
# Linux / macOS (in repo root)
glib-compile-schemas --strict --dry-run data/

# Windows (MSYS2 MINGW64)
glib-compile-schemas --strict --dry-run data/
```

### Checklist

- [ ] `Container`, `Image`, `Volume`, `Network` all derive `Default` and `PartialEq`
- [ ] `ContainerStatus` has `css_class()`, `icon_name()`, `display_label()` methods
- [ ] `src/window/actions.rs` exists with all action name constants (including `FOCUS_SEARCH`, `CLEAR_SEARCH`); no magic
  string literals in `app.rs`/`main_window.rs`
- [ ] `EmptyState` component exists in `src/window/components/empty_state.rs`; all four views use it
- [ ] `ToastUtil` exists in `src/window/components/toast_util.rs`; four views use it
- [ ] `data/com.example.GtkCrossPlatform.gschema.xml` exists with keys: `window-width`, `window-height`,
  `window-maximized`, `color-scheme`, `preferred-runtime`, `sidebar-width-fraction` (type `"d"`)
- [ ] Schema validates: `glib-compile-schemas --strict --dry-run data/` exits 0
- [ ] `make schema` compiles `data/gschemas.compiled`; `data/gschemas.compiled` is in `.gitignore`
- [ ] `make run` and `make run-mobile` set `GSETTINGS_SCHEMA_DIR=$(SCHEMA_DIR)` before launching
- [ ] `data/meson.build` created for Flatpak/Linux distribution (schema install + compile)
- [ ] `make dmg` copies `com.example.GtkCrossPlatform.gschema.xml` into the `.app` bundle before `glib-compile-schemas`
- [ ] `Makefile` `setup-windows` includes `mingw-w64-x86_64-glib2` for `glib-compile-schemas`
- [ ] Window geometry is persisted via GSettings (width, height, maximized)
- [ ] Preferred runtime is read/written via GSettings (replaces file-based `save_runtime_pref` / `load_runtime_pref`)
- [ ] Color-scheme preference is persisted and applied via GSettings; `adw::StyleManager` updated on startup and on key
  change
- [ ] `settings.bind("sidebar-width-fraction", &split_view, "sidebar-width-fraction")` — type `f64` on both sides, no
  mapping closure
- [ ] `adw::AboutWindow` has `issue_url`, `website`, `translator_credits`, runtime credit section, and two `add_link`
  entries — no duplicate entries if already present
- [ ] `style-dark.css` exists in `data/resources/` and is registered in `resources.gresource.xml`
- [ ] Dark CSS provider is added/removed via `connect_dark_notify` and applied correctly on startup
- [ ] Detail panes in all four resource views use `adw::Clamp` with `maximum-size = 720`
- [ ] `EmptyState` status pages are wrapped in `adw::Clamp` with `maximum-size = 480`
- [ ] `SplitButton` is used for the Run action in `images_view.rs`
- [ ] `signal_handler_block` / `signal_handler_unblock` guards are applied to programmatic list selections in at least
  two views; `SignalHandlerId` stored as a view field
- [ ] `ToastUtil::show_destructive()` is used for container remove and image remove (10 s timeout, HIGH priority, Undo
  button)
- [ ] `src/window/components/list_factory.rs` exists with `make_factory()` helper; existing `bind_model()` calls are
  marked with `// TODO: migrate to ColumnView`
- [ ] `StatusBadge` (and any other `gtk::Widget` subclass) calls `set_accessible_role()` in class init
- [ ] Every icon-only button in `.ui` files has an `<accessibility><property name="label">` block
- [ ] Every `set_tooltip_text()` in Rust code has a matching `update_property(Property::Label(...))` call immediately
  after it
- [ ] Each resource view's `imp` struct has `list_cancellable` and `detail_cancellable` fields
- [ ] `spawn_driver_task` calls cancel the previous `Cancellable` before dispatching
- [ ] Views connect `destroy` signal to cancel all in-flight cancellables
- [ ] `tests/cancellable_test.rs` exists with at least one test verifying no panic on cancel
- [ ] `data/resources/window.ui` has a `GtkShortcutController` for `win.focus-search` using `<Primary>f` (not
  `<Control>f`), scope `local`
- [ ] `win.focus-search` and `win.clear-search` action names are in `src/window/actions.rs`
- [ ] `src/core/domain/` has zero imports from `gtk4`, `adw`, `glib`, `ports`, or `infrastructure`
- [ ] `make fmt` — no diffs
- [ ] `make lint` — zero warnings
- [ ] `make test` — zero failures, count ≥ baseline
