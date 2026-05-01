# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Run Commands

`make` without arguments runs the application (`.DEFAULT_GOAL := run`). `make setup` configures
the environment idempotently for the detected platform.

**Local development (Cargo):**

```sh
make                  # Configure environment and run (default goal)
make setup            # Configure environment for the detected platform (idempotent)
make build            # Compile with cargo build (debug)
make build-release    # Compile with cargo build --release
make run              # Build and run the application
make test             # Run all tests (unit + integration + i18n)
make test-unit        # Unit tests only (cargo nextest --lib)
make test-integration # Integration tests only (cargo nextest, all tests/ except i18n and widget)
make lint             # Run cargo clippy -- -D warnings
make fmt              # Check formatting (cargo fmt --check)
make fmt-fix          # Auto-format code (cargo fmt)
make run-mobile       # Run with GTK_DEBUG=interactive to simulate narrow screen
make clean            # Remove build artifacts (Cargo + Flatpak)
make clean-all        # Remove all generated files including icons
make cache-info       # Show sizes of Cargo and Flatpak caches
make cache-prune      # Prune Cargo registry cache (installs cargo-cache if needed)
```

**Pre-PR quality gate:**

```sh
make ci               # Run the full CI pipeline locally (validate + test); mirrors ci.yml
```

**Flatpak distribution:**

```sh
make dist-flatpak      # Build Flatpak x86_64 (force-clean, installs deps from Flathub)
make dist-flatpak-run  # Build and run in Flatpak sandbox
make dist-flatpak-install  # Build and install the Flatpak
make dist-flatpak-arm  # Cross-compile for aarch64 (GNOME Mobile / PinePhone)

# Backwards-compatible aliases:
make flatpak-build     # alias for dist-flatpak
make flatpak-run       # alias for dist-flatpak-run
make flatpak-install   # alias for dist-flatpak-install
make flatpak-build-arm # alias for dist-flatpak-arm
```

**Cross-platform setup (one-time per platform):**

```sh
make setup-macos    # Install GTK4 stack via Homebrew + Rust via rustup (macOS, idempotent)
make setup-linux    # Install GTK4 dev libraries via apt or dnf (Linux, idempotent)
make setup-windows  # Print MSYS2/MINGW64 install instructions for Windows
```

**Quality gates (local):**

```sh
make validate           # Run all local quality gates (format + lint + metadata + i18n + deps)
make validate-metainfo  # appstreamcli validate on metainfo XML
make validate-desktop   # desktop-file-validate on .desktop file
make check-version      # Verify Cargo.toml version == metainfo.xml version
make check-potfiles     # Verify all files with gettext() are listed in po/POTFILES
make audit              # cargo audit (security advisories)
make deny               # cargo deny check (license + security advisories)
make spell-check        # typos . (spell-check all tracked files)
make check-unused-deps  # cargo machete (detect unused dependencies)
```

**macOS distributable:**

```sh
make dist-macos        # Build macOS .app bundle + .dmg (requires dylibbundler, create-dmg)
make dmg               # alias for dist-macos
```

**Release:**

```sh
make release           # Full release: ci → dist → tag → GitHub Release (all 4 artifacts)
make release-tag       # Create and push git tag for the current version
make release-github    # Create GitHub Release with Flatpak x86_64 + aarch64 + DMG + Windows ZIP
```

The Flatpak manifest is `com.example.GtkCrossPlatform.json`. The build profile
(`default` or `development`) and target platform (`linux`, `macos`, `windows`, `mobile`)
are controlled via environment variables passed to `cargo build` (see `build.rs`).

`NEXTEST_PROFILE` (default: `default`) controls the nextest profile. CI passes
`make test-unit NEXTEST_PROFILE=ci` to activate fail-fast mode.

## Architecture

This is a **cross-platform** GTK4 + Adwaita desktop application written in **Rust**,
targeting Linux, macOS, Windows, and GNOME Mobile (Phosh/postmarketOS).

**App ID:** `com.example.GtkCrossPlatform` — replace with a real reverse-domain ID before publishing.

The codebase follows **Hexagonal Architecture** (Ports & Adapters):

| Layer            | Path                        | Rule                                         |
|------------------|-----------------------------|----------------------------------------------|
| Domain (core)    | `src/core/`                 | No GTK/Adw/GLib imports; pure business logic |
| Ports            | `src/ports/`                | Rust traits consumed by core and UI          |
| Adapters         | `src/infrastructure/`       | Implement ports; may use GLib/IO, never GTK  |
| UI               | `src/window/`               | GTK/Adw widgets only; depends on ports       |
| Composition root | `src/app.rs` → `activate()` | Primary place where concrete types are wired; `MainWindow` may re-wire for runtime switching |

**Key types:**

- `GtkCrossPlatformApp` (`src/app.rs`) — GLib subclass of `adw::Application`; composition root — `activate()` is the
  primary place concrete types are wired to ports. `MainWindow` may also re-wire the runtime adapter when the user
  switches runtimes via the runtime switcher.
- `MainWindow` (`src/window/main_window.rs`) — GLib subclass of `adw::ApplicationWindow`; thin shell over
  `AdwNavigationSplitView` + `AdwViewStack`; delegates all logic to views.
- `IContainerDriver` (`src/ports/i_container_driver.rs`) — port trait implemented by Docker, Podman, containerd, and
  Mock adapters.
- `ContainerDriverFactory` (`src/infrastructure/containers/factory.rs`) — auto-detects the available runtime and returns
  `Arc<dyn IContainerDriver>`.
- `spawn_driver_task` (`src/infrastructure/containers/background.rs`) — runs a blocking driver call on a thread pool and
  delivers the result back to the GTK main loop via `async-channel`.
- `ContainersView` / `ImagesView` / `VolumesView` / `NetworksView` (`src/window/views/`) — own the sidebar list and
  detail pane for each resource type; the only layer that calls `spawn_driver_task`.
- `MockContainerDriver` (`src/infrastructure/containers/mock_driver.rs`) — in-memory driver used in all integration
  tests.
- `AppLogger` (`src/infrastructure/logging/app_logger.rs`) — thin wrapper around `glib::g_debug!` / `g_info!` /
  `g_warning!` / `g_critical!`; supports sub-domain loggers via `subdomain()` and structured fields via
  `log_with_fields()`.
- `log_container_error` (`src/infrastructure/containers/error.rs`) — normalises log level per `ContainerError`
  variant; call this at the call site (views/app.rs), not inside the error constructor.
- `config` (`src/config.rs`) — compile-time constants from `build.rs` env vars: `APP_ID`, `VERSION`, `PROFILE`,
  `LOCALEDIR`, `PKGDATADIR`, `SOURCE_DATADIR`, `GETTEXT_PACKAGE`.

**Responsive breakpoints** (declared in `data/resources/window.ui`):
| Breakpoint          | Condition  | Effect                                                                              |
|---------------------|------------|-------------------------------------------------------------------------------------|
| Desktop normal      | > 768 sp   | Margins 48 sp                                                                       |
| Desktop compact     | ≤ 768 sp   | Margins 32 sp                                                                       |
| Split-view collapse | ≤ 900 sp   | `AdwNavigationSplitView` collapses; `AdwViewSwitcherBar` revealed; `AdwViewSwitcher` (top) hidden |
| Tablet              | ≤ 600 sp   | Margins 24 sp                                                                       |
| GNOME Mobile        | ≤ 360 sp   | Margins 16 sp                                                                       |

**Touchscreen:** minimum 44×44 sp touch targets on interactive elements; `gtk4::GestureLongPress` provides touchscreen
alternative to right-click context.

**Runtime detection order** (implemented in `src/infrastructure/containers/factory.rs`):

| Order | Check                                                        | Runtime                               |
|-------|--------------------------------------------------------------|---------------------------------------|
| 1     | `/var/run/docker.sock` or `~/.rd/docker.sock` accessible     | Docker (or Rancher Desktop on macOS)  |
| 2     | `CONTAINER_HOST` env var socket path accessible              | Podman (explicit override)            |
| 3     | `/run/user/{uid}/podman/podman.sock` accessible              | Podman (rootless, Linux)              |
| 4     | `/run/podman/podman.sock` accessible                         | Podman (root, Linux)                  |
| 5     | `~/.local/share/containers/podman/machine/default/podman.sock` | Podman 5.x (macOS)                 |
| 6     | `~/.local/share/containers/podman/machine/qemu/podman.sock`  | Podman 4.x (macOS)                   |
| 7     | `nerdctl version` exits 0                                    | containerd/nerdctl                    |
| —     | None found                                                   | `ContainerError::RuntimeNotAvailable` |

When adding a new runtime adapter, register it after order 7 in `ContainerDriverFactory::detect()`.

**G_LOG_DOMAIN convention** (`AppLogger` sub-domain hierarchy):

| Layer          | Domain                                          | Example env filter                              |
|----------------|-------------------------------------------------|-------------------------------------------------|
| App / startup  | `com.example.GtkCrossPlatform`                  | `G_MESSAGES_DEBUG=com.example.GtkCrossPlatform` |
| Infrastructure | `…GtkCrossPlatform.containers`                  | (future; reserved for driver adapters)          |
|                | `…GtkCrossPlatform.background`                  | (spawn_driver_task thread bridge)               |
| Views          | `…GtkCrossPlatform.view.containers`             |                                                 |
|                | `…GtkCrossPlatform.view.images`                 |                                                 |
|                | `…GtkCrossPlatform.view.volumes`                |                                                 |
|                | `…GtkCrossPlatform.view.networks`               |                                                 |

GLib prefix-matches `G_MESSAGES_DEBUG` against the log domain (see `gmessages.c`, `should_be_printed`), so
`G_MESSAGES_DEBUG=com.example.GtkCrossPlatform` enables every sub-domain listed above. `G_MESSAGES_DEBUG=all`
enables all GLib messages globally and is set automatically when `config::PROFILE == "development"`.

Log level mapping: GLib has no TRACE level — `AppLogger::trace` maps to `g_debug!`. Use `AppLogger::critical` for
`PermissionDenied` and `ParseError` variants; `AppLogger::info` for `NotFound` and `AlreadyExists`; `AppLogger::warning`
for all other `ContainerError` variants. Normalised via `log_container_error(&logger, &err)` in `error.rs`.

**Dependencies:** gtk4 = 0.9 (feature `v4_12`), libadwaita = 0.7 (feature `v1_4`), glib = 0.20, gio = 0.20, gettext-rs = 0.7 (i18n runtime), serde_json = 1 (JSON inspect + syntax highlighting), async-channel = 2 (cross-thread messaging), sysinfo = 0.33 (CPU/memory/disk stats for dashboard). Build dependency: glib-build-tools = 0.20 (GResource compiler). Minimum runtime: GTK4 ≥ 4.12, LibAdwaita ≥ 1.4.

**GResource:** UI files are compiled into `compiled.gresource` at build time via `glib-build-tools` (see `build.rs`).
Resources are registered in `main()` before the application starts via `gio::resources_register_include!`.

**Flatpak sandbox permissions:** Wayland socket, X11 fallback, IPC. (`--device=dri` removed — re-add if GPU access is
needed.)

## i18n / Localization

All user-visible strings must use `gettextrs::gettext("...")`. The pipeline:

- `po/POTFILES` — lists `.rs` and `.ui` files containing translatable strings
- `po/LINGUAS` — lists active locale codes (20+ community translations; see `po/LINGUAS`)
- `po/pt_BR.po` — example Brazilian Portuguese translation (20+ additional locales in `po/`)

The text domain is bound in `main()` via `gettextrs::bindtextdomain` / `textdomain` using `config::GETTEXT_PACKAGE` and
`config::LOCALEDIR`.

To add a new language: run `msginit -l <locale>` in `po/`, add the locale to `LINGUAS`, translate strings, run
`make build`.

**Context disambiguation:** when the same English word maps to different concepts in target languages, use
`pgettext!("context", "string")`. Example: `pgettext!("container action", "Remove")` vs
`pgettext!("image action", "Remove")`.

**Plurals:** use `ngettext!("1 container", "{n} containers", n)` — never `format!("{n} containers")`.

**RTL layout:** wrap directional icons (`arrow-left`, `arrow-right`) with
`gtk4::Widget::set_direction(gtk4::TextDirection::Ltr)` so they don't flip incorrectly in RTL locales.

## Threading

All blocking driver calls must go through `spawn_driver_task` (`src/infrastructure/containers/background.rs`). Never
call any GTK function from outside the GTK main thread.

```
GTK Main Thread                          Worker Thread
─────────────                            ─────────────
begin_loading()
spawn_driver_task(driver, task, cb) ───▶ std::thread::spawn { task(driver) → tx.send }
                                    ◀─── async_channel::bounded(1)
glib::spawn_local { rx.recv() → cb }
end_loading() + update_ui
```

- `tokio` is banned — conflicts with the GLib event loop; use `async_channel::bounded(1)` instead
- Views are the primary callers; `MainWindow` and `src/app.rs` may also call it for window/app-scoped actions

## Design Standards

Follow the [GNOME HIG](https://developer.gnome.org/hig/). Use `adw::*` widgets over raw GTK; declare layout in `.ui`
files (Composite Templates); wire signals in Rust. Support Wayland and X11.

Key A11Y rules (full checklist in `.claude/rules/standards/accessibility.md`):
- Add an `AdwBreakpoint` for every layout change at 360/600/768 sp thresholds; touch targets ≥ 44×44 sp
- Icon-only buttons need both `set_tooltip_text` AND `update_property(&[Property::Label(...)])`
- Never use color alone to convey state — `StatusBadge` always pairs color with a text label

## Documentation Philosophy

This project is **human-first**: every document — README, CONTRIBUTING, prompts, changelogs — must be easy to read and
navigate for humans before it optimizes for anything else.

### Human-first principles

- **Short sections with clear headers** — the reader should be able to scan and land on what they need without reading
  everything
- **Lists over paragraphs** — prefer bullet points for features, requirements, and instructions; reserve prose for
  context that genuinely needs it
- **Code blocks for all commands** — no inline code for multi-step instructions
- **Logical reading flow** — what it is → why use it → how to use it → how to contribute; never bury the "what" after
  the "how"
- **Express opinions** — documents should declare preferences and commitments, not just enumerate facts. Say "this is a
  GNOME application — Adwaita theme applies on macOS by design", not "the theme may vary by platform"

### AI-context richness (secondary, never at the cost of readability)

Documents must carry enough named context for an AI agent to reconstruct a rich mental model of the project without
reading the source:

- Name every widget pattern used (`AdwNavigationSplitView`, `AdwBreakpoint`, `GestureLongPress`)
- Name every architectural concept (`Hexagonal Architecture`, `Ports & Adapters`, `CompositeTemplate`)
- Name runtimes, SDKs, and constraints explicitly (`GNOME Platform 48`, `org.gnome.Sdk`, `GTK ≥ 4.12`)
- Name every design scope the project targets (A11Y, I18N, responsiveness, dark/light theme, high contrast, touch,
  Wayland-first, Flatpak)
- Never omit breakpoint values, licensing, or layer rules from documentation

When updating the README, use `/meta:knowledge-planning`.

### Separation: docs/ vs .claude/docs/

- **`docs/`** — human-authored documents intended for contributors and maintainers. Must follow human-first principles.
  Keep this folder minimal: only documents a developer would actively seek out (roadmap, architecture decisions, gap
  tracking).
- **`.claude/docs/reports/`** — AI-generated audit outputs (audit reports, analysis, code review reports). Claude artifacts exempt from
  human-first requirements. Skills and prompts must write their reports here, never to `docs/`.
- **`.claude/docs/reference/`** — Static reference material for AI agents (GNOME HIG, GTK learning resources). Rarely changes.

A file in `docs/` written by an AI agent is a violation of this rule — move it to `.claude/docs/reports/`.

### Code comments

Default to **no comments**. Add a comment only when the *why* is non-obvious: a hidden constraint, a subtle invariant, a
workaround for a specific external bug, behavior that would genuinely surprise a future reader. If removing the comment
would not confuse anyone, do not write it. Never write comments that describe *what* the code does — well-named
identifiers already do that.

## Project Structure

```
src/
  main.rs                                # Entry point: registers GResource, sets up i18n, runs app
  app.rs                                 # GtkCrossPlatformApp (adw::Application subclass) + composition root
  config.rs                              # Compile-time constants from build.rs env vars
  lib.rs                                 # Library root: exports core, infrastructure, ports, config
  core/
    mod.rs
    domain/
      mod.rs
      container.rs                       # Container, ContainerStatus, ContainerStats domain models
      image.rs                           # Image domain model
      volume.rs                          # Volume domain model
      network.rs                         # Network, PruneReport, SystemUsage domain models
    use_cases/
      mod.rs
      container_use_case.rs              # ContainerUseCase — list/inspect/lifecycle operations
      greet_use_case.rs                  # GreetUseCase — pure domain logic, no GTK
      image_use_case.rs                  # ImageUseCase — list/pull/remove/inspect
      network_use_case.rs                # NetworkUseCase — list/create/remove
      volume_use_case.rs                 # VolumeUseCase — list/create/remove
  ports/
    mod.rs
    i_container_driver.rs                # IContainerDriver trait (unified runtime interface)
    i_greeting_service.rs                # IGreetingService trait
    use_cases/
      mod.rs
      i_container_use_case.rs            # IContainerUseCase port consumed by ContainersView
      i_image_use_case.rs                # IImageUseCase port consumed by ImagesView
      i_network_use_case.rs              # INetworkUseCase port consumed by NetworksView
      i_volume_use_case.rs               # IVolumeUseCase port consumed by VolumesView
  infrastructure/
    mod.rs
    containers/
      mod.rs
      factory.rs                         # Auto-detects available runtime; returns Arc<dyn IContainerDriver>
      docker_driver.rs                   # Docker adapter (HTTP over Unix socket)
      podman_driver.rs                   # Podman adapter
      containerd_driver.rs               # containerd adapter
      mock_driver.rs                     # In-memory mock for tests
      dynamic_driver.rs                  # DynamicDriver — wraps Arc<dyn IContainerDriver> for runtime switching
      host_stats.rs                      # read_host_stats() — CPU/memory/disk from sysinfo (used by DashboardView)
      http_over_unix.rs                  # HTTP client routed through a Unix domain socket
      background.rs                      # spawn_driver_task — bridges blocking driver calls to GTK main loop
      error.rs                           # ContainerError type + log_container_error()
    greeting/
      mod.rs
      greeting_service.rs                # GreetingService — implements IGreetingService
    logging/
      mod.rs
      app_logger.rs                      # AppLogger — thin wrapper over glib structured logging
  window/
    mod.rs
    actions.rs                           # CommonActions — window-scoped GAction constants
    main_window.rs                       # Thin shell: wires views into AdwNavigationSplitView
    components/
      mod.rs
      status_badge.rs                    # Colored pill label for ContainerStatus (A11Y: text + color)
      resource_row.rs                    # Base adw::ActionRow builder + icon_button helper
      detail_pane.rs                     # Scrollable key-value property grid (adw::PreferencesGroup)
      confirm_dialog.rs                  # adw::MessageDialog wrapper for destructive confirmations
      empty_state.rs                     # EmptyState — adw::StatusPage wrapper for empty list states
      toast_util.rs                      # ToastUtil — fire-and-forget adw::Toast helpers
    objects/
      mod.rs
      container_object.rs                # ContainerObject — GObject wrapper for Container domain model
      image_object.rs                    # ImageObject — GObject wrapper for Image domain model
      network_object.rs                  # NetworkObject — GObject wrapper for Network domain model
      volume_object.rs                   # VolumeObject — GObject wrapper for Volume domain model
    utils/
      mod.rs
      format.rs                          # fmt_bytes() — human-readable byte sizes (B / MB / GB)
      store.rs                           # find_store_position() — typed linear scan over gio::ListStore
    views/
      mod.rs
      containers_view.rs                 # Sidebar list + detail pane for containers
      images_view.rs                     # Sidebar list + detail pane for images
      volumes_view.rs                    # Sidebar list + detail pane for volumes
      networks_view.rs                   # Sidebar list + detail pane for networks
      dashboard_view.rs                  # DashboardView — system overview: running containers, disk usage
tests/
  support/mod.rs                        # Shared test helpers (fixture builders, assertions)
  cancellable_test.rs                   # Task cancellation
  compose_grouping_test.rs              # Compose project grouping
  compose_lifecycle_test.rs             # Compose start/stop lifecycle
  container_driver_test.rs              # Core driver operations with MockContainerDriver
  container_lifecycle_test.rs           # Container start/stop/remove lifecycle
  container_logs_test.rs                # Log streaming
  container_stats_test.rs               # Stats polling
  create_container_test.rs              # Container creation options
  dashboard_test.rs                     # DashboardView data assembly
  env_masking_test.rs                   # Environment variable masking (secrets)
  greet_use_case_test.rs                # GreetUseCase domain logic
  i18n_test.rs                          # i18n structural (po files, POTFILES, LINGUAS)
  image_layers_test.rs                  # Image layer inspection
  inspect_test.rs                       # Container/image inspect JSON
  pull_image_streaming_test.rs          # pull progress streaming
  pull_image_test.rs                    # Image pull
  runtime_switcher_test.rs              # Runtime switching via DynamicDriver
  search_filter_test.rs                 # Search/filter in list views
  system_events_test.rs                 # Docker/Podman event stream
  terminal_test.rs                      # Terminal output integration
  widget_test.rs                        # GTK widget tests (require display; marked #[ignore])
data/
  resources/
    resources.gresource.xml              # GResource manifest
    window.ui                            # MainWindow template (AdwNavigationSplitView + AdwBreakpoint)
    style.css                            # Application-scoped CSS (status badge colors, touch targets)
    style-dark.css                       # Dark-mode overrides (loaded via AdwStyleManager)
  icons/hicolor/scalable/apps/
    com.example.GtkCrossPlatform.svg     # Application icon (scalable)
  com.example.GtkCrossPlatform.desktop   # Desktop entry (app launcher, Flathub)
  com.example.GtkCrossPlatform.gschema.xml  # GSettings schema (sidebar-width-fraction, preferred-runtime)
  com.example.GtkCrossPlatform.metainfo.xml  # AppStream metainfo (GNOME Software)
po/
  POTFILES                               # Source files with translatable strings
  LINGUAS                                # Active locale list
  pt_BR.po                               # Brazilian Portuguese
  *.po                                   # 20+ community translations (see po/LINGUAS for full list)
com.example.GtkCrossPlatform.json        # Flatpak manifest (GNOME Platform 48, rust-stable)
Cargo.toml                               # Rust manifest (gtk4, libadwaita, glib, gio, gettext-rs, sysinfo)
build.rs                                 # Injects APP_ID/PROFILE/PKGDATADIR/LOCALEDIR; compiles GResource
Makefile                                 # Convenience wrappers for Cargo/Flatpak commands
.claude/
  settings.json                          # Project-level permissions (cargo/make allow-list)
  settings.local.json                    # Local overrides — gitignored
  skills/plan/                           # Planning & scoping workflows (invoke: /plan:<name>)
  skills/build/                          # Implementation & coding workflows (invoke: /build:<name>)
  skills/verify/                         # Quality, audit & review workflows (invoke: /verify:<name>)
  skills/release/                        # Delivery & distribution workflows (invoke: /release:<name>)
  skills/meta/                           # Claude tooling meta-workflows (invoke: /meta:<name>)
  agents/                                # Autonomous subagent prompts (scope-prefixed: plan--, build--, verify--, release--, meta--, domain--)
  rules/domain/container-management.md  # globs: src/core/domain/**, src/infrastructure/containers/**, src/ports/**
  rules/standards/language.md            # globs: **/*.rs — threading, async, logging, i18n
  rules/standards/interface.md           # globs: src/ports/**, src/infrastructure/containers/**
  rules/standards/verification.md        # globs: tests/**, src/**/tests/**
  rules/standards/observability.md       # globs: src/infrastructure/logging/**, src/**/*.rs
  rules/standards/makefile.md            # globs: Makefile
  rules/standards/accessibility.md       # globs: src/window/**, data/resources/**
  docs/reports/                          # AI-generated audit outputs (artifact-structure-proposal.md, content-audit.md, conceptual-improvements.md, design-vs-implementation.md, test-quality-audit.md)
  docs/reference/                        # Static reference material (gtk-sources.md, adwaita.md)
  prompts/                               # Stored reusable prompts (INDEX.md + *.md)
docs/
  compliance-plan.md                     # 18 documented-vs-implemented gaps + guardrails (human-authored, human-facing)
.prompt/                                 # Legacy prompt templates (gitignored; superseded by .claude/skills/)
```

## Dependency Versioning Rule

When bumping a dependency in `Cargo.toml`, update the version table in §Dependencies in the same
commit. `Cargo.toml` is the source of truth — CLAUDE.md must never diverge from it.

## Testing

`make test` / `make test-unit` / `make test-integration` / `make test-i18n`. Pass `NEXTEST_PROFILE=ci` for fail-fast.
`make coverage` runs `cargo llvm-cov` (not part of CI).

| Layer | Location | Rule |
|-------|----------|------|
| Unit | `#[cfg(test)]` inline in `src/core/` | No `gtk4` / `adw` imports |
| Integration | `tests/*.rs` | Public API only; `MockContainerDriver` — never real sockets |
| Widget | `tests/widget_test.rs` | `#[ignore]`; needs display (`xvfb-run`) |

`tests/unit/` does **not** exist — domain unit tests live inline (Rust convention for private access).
