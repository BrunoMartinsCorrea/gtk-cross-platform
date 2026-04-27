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
make test-integration # Integration tests only (container_driver + greet_use_case)
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
| Split-view collapse | ≤ 720 sp   | `AdwNavigationSplitView` collapses; `AdwViewSwitcherBar` revealed; `AdwViewSwitcher` (top) hidden |
| Tablet              | ≤ 600 sp   | Margins 24 sp                                                                       |
| GNOME Mobile        | ≤ 360 sp   | Margins 16 sp                                                                       |

**Touchscreen:** minimum 44×44 sp touch targets on interactive elements; `gtk4::GestureLongPress` provides touchscreen
alternative to right-click context.

**Runtime detection order** (implemented in `src/infrastructure/containers/factory.rs`):

| Order | Check                                           | Runtime                               |
|-------|-------------------------------------------------|---------------------------------------|
| 1     | `/var/run/docker.sock` accessible               | Docker                                |
| 2     | `/run/user/{uid}/podman/podman.sock` accessible | Podman (rootless)                     |
| 3     | `/run/podman/podman.sock` accessible            | Podman (root)                         |
| 4     | `nerdctl version` exits 0                       | containerd/nerdctl                    |
| —     | None found                                      | `ContainerError::RuntimeNotAvailable` |

When adding a new runtime adapter, register it after order 4 in `ContainerDriverFactory::detect()`.

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
`PermissionDenied` and `ParseError` variants; `AppLogger::info` for `NotFound`; `AppLogger::warning` for all
other `ContainerError` variants. Normalised via `log_container_error()` in `error.rs`.

**Dependencies:** gtk4 = 0.9 (feature `v4_12`), libadwaita = 0.7 (feature `v1_4`), glib = 0.20, gio = 0.20, gettext-rs = 0.7 (i18n runtime), serde_json = 1 (JSON inspect + syntax highlighting), async-channel = 2 (cross-thread messaging). Build dependency: glib-build-tools = 0.20 (GResource compiler). Minimum runtime: GTK4 ≥ 4.12, LibAdwaita ≥ 1.4.

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

- Use `async_channel::bounded(1)` — never `std::sync::mpsc` or `tokio` channels
- `tokio` is banned: it conflicts with the GLib event loop
- Views (`src/window/views/`) are the primary layer that calls `spawn_driver_task`. `MainWindow` may also call it for
  window-scoped actions (e.g., system prune) that do not belong to any single resource view.
  `src/app.rs` may also call it for app-lifecycle actions that execute before the window is ready.
- The `cb` callback runs on the GTK main loop via `glib::spawn_local`

## Design Standards

This project follows the [GNOME Human Interface Guidelines (HIG)](https://developer.gnome.org/hig/). When adding or
modifying UI:

- Use Adwaita widgets (`adw::*`) over raw GTK widgets whenever an equivalent exists
- Declare layout in `.ui` files (Composite Templates); wire signals and closures in Rust
- Follow GNOME naming conventions, spacing, and layout patterns
- Support both Wayland and X11; never assume a specific display server
- Add an `AdwBreakpoint` in `window.ui` for every new layout change at 360/600/768 sp thresholds
- Use `adw::ToastOverlay` for transient feedback instead of dialogs where appropriate
- All touch targets must be ≥ 44×44 sp; never rely on `hover` as the sole state indicator
- Avoid menus activated only by right-click — provide `gtk4::GestureLongPress` equivalent
- Icon-only buttons must have both `set_tooltip_text` (keyboard-visible) and
  `update_property(&[gtk4::accessible::Property::Label(...)])` — tooltip alone is insufficient for screen readers
- Focus management: after a destructive action (remove), move focus to the next row or the empty-state widget; after any
  dialog closes, return focus to the widget that triggered it
- Never use color alone to convey state — `StatusBadge` must show a text label alongside the color indicator

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

When updating the README, use `/project:update-readme`.

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
      http_over_unix.rs                  # HTTP client routed through a Unix domain socket
      background.rs                      # spawn_driver_task — bridges blocking driver calls to GTK main loop
      error.rs                           # ContainerError type
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
      list_factory.rs                    # GtkSignalListItemFactory helpers for ColumnView/ListView
      toast_util.rs                      # ToastUtil — fire-and-forget adw::Toast helpers
    objects/
      mod.rs
      container_object.rs                # ContainerObject — GObject wrapper for Container domain model
      image_object.rs                    # ImageObject — GObject wrapper for Image domain model
      network_object.rs                  # NetworkObject — GObject wrapper for Network domain model
      volume_object.rs                   # VolumeObject — GObject wrapper for Volume domain model
    views/
      mod.rs
      containers_view.rs                 # Sidebar list + detail pane for containers
      images_view.rs                     # Sidebar list + detail pane for images
      volumes_view.rs                    # Sidebar list + detail pane for volumes
      networks_view.rs                   # Sidebar list + detail pane for networks
      dashboard_view.rs                  # DashboardView — system overview: running containers, disk usage
tests/
  container_driver_test.rs              # Integration tests using MockContainerDriver
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
  com.example.GtkCrossPlatform.gschema.xml  # GSettings schema (sidebar-width, last-runtime)
  com.example.GtkCrossPlatform.metainfo.xml  # AppStream metainfo (GNOME Software)
po/
  POTFILES                               # Source files with translatable strings
  LINGUAS                                # Active locale list
  pt_BR.po                               # Brazilian Portuguese
  *.po                                   # 20+ community translations (see po/LINGUAS for full list)
com.example.GtkCrossPlatform.json        # Flatpak manifest (GNOME Platform 48, rust-stable)
Cargo.toml                               # Rust manifest (gtk4, libadwaita, glib, gio, gettext-rs)
build.rs                                 # Injects APP_ID/PROFILE/PKGDATADIR/LOCALEDIR; compiles GResource
Makefile                                 # Convenience wrappers for Cargo/Flatpak commands
.claude/
  settings.json                          # Project-level permissions (cargo/make allow-list)
  settings.local.json                    # Local overrides — gitignored
  commands/
    update-readme.md                     # /project:update-readme
    refactor-components.md               # /project:refactor-components
    add-runtime-driver.md                # /project:add-runtime-driver <runtime>
    implement-container-ui.md            # /project:implement-container-ui
    scaffold-oss-docs.md                 # /project:scaffold-oss-docs
    compliance-audit.md                  # /project:compliance-audit
    github-audit.md                      # /project:github-audit
    concept-audit.md                     # /project:concept-audit
    hexagonal-refactor.md                # /project:hexagonal-refactor
    add-quality-gates.md                 # /project:add-quality-gates
    optimize-dashboard-loading.md        # /project:optimize-dashboard-loading
    release-audit.md                     # /project:release-audit
    apply-requester-patterns.md          # /project:apply-requester-patterns
    apply-conceptual-improvements.md     # /project:apply-conceptual-improvements
    docs-audit.md                         # /project:docs-audit
    apply-docs-fixes.md                  # /project:apply-docs-fixes
    implement-v02-mvp.md                 # /project:implement-v02-mvp
    redesign-claude-setup.md             # /project:redesign-claude-setup
    testing-audit.md                     # /project:testing-audit
    cleanup-code.md                      # /project:cleanup-code
    test-quality-guardrail.md            # /project:test-quality-guardrail
    redesign-makefile.md                 # /project:redesign-makefile
    dist-audit.md                        # /project:dist-audit
    sync-pull-request.md                 # /project:sync-pull-request
    content-audit.md                     # /project:content-audit
    plan-content-improvements.md         # /project:plan-content-improvements
    create-prompt.md                     # /project:create-prompt <context>
  docs/
    content-audit.md                     # Generated by /project:content-audit; managed automatically
    content-improvement-plan.md          # Generated by /project:plan-content-improvements; managed automatically
  prompts/
    INDEX.md                             # Index of all generated prompts
    *.md                                 # Generated by /project:create-prompt; stored here permanently
docs/
  compliance-plan.md                     # 18 documented-vs-implemented gaps + guardrails
.prompt/                                 # Legacy prompt templates (gitignored; superseded by .claude/commands/)
```

## Slash Commands

Invoke with `/project:<name>` inside a Claude Code session. Each command is self-contained —
it does not require reading prior conversation context to execute correctly.

| Command                              | When to use                                                                             |
|--------------------------------------|-----------------------------------------------------------------------------------------|
| `/project:update-readme`             | README or CONTRIBUTING.md needs updating after structural changes                       |
| `/project:refactor-components`       | Decompose `src/window/` into finer-grained components and views                         |
| `/project:add-runtime-driver <name>` | Add a new container runtime adapter (e.g., `nerdctl`, `lima`)                           |
| `/project:implement-container-ui`    | Implement or overhaul the GTK4/Adwaita UI layer for container management                |
| `/project:scaffold-oss-docs`         | Generate OSS documentation structure for any repository                                 |
| `/project:update-gitignore`          | Audit `.gitignore` against detected extensions and community best practices, then apply |
| `/project:compliance-audit`          | Audit documented concepts vs. implementation; outputs gap report with severity          |
| `/project:github-audit`              | Audit the repository across six dimensions (CI/CD, security, distribution, etc.)        |
| `/project:concept-audit`             | Find internal conceptual inconsistencies: naming, layer leaks, trait contracts, threading |
| `/project:hexagonal-refactor`        | Introduce driver ports (inbound traits) + container use cases; wire views through use cases |
| `/project:add-quality-gates`         | Implement all missing CI quality gates (AppStream, desktop, deny, typos, nextest, coverage) |
| `/project:release-audit`             | Audit the cross-platform release pipeline (workflows, bundling, artifacts, publishing)  |
| `/project:optimize-dashboard-loading` | Fix startup latency: lazy view loading, deferred `system_df`, debounced search, duplicate-call elimination |
| `/project:apply-requester-patterns`  | Apply GTK4/Adwaita patterns from Requester: GSettings, CommonActions, EmptyState, ToastUtil, AdwClamp, SplitButton, domain derives, CSS split |
| `/project:apply-conceptual-improvements` | Migrate ListBox→GListModel+SignalListItemFactory, add GObject wrappers, FilterListModel, reactive bindings, CustomSorter |
| `/project:docs-audit`                    | Audit documentation layer for staleness, cross-document inconsistencies, coverage gaps, and README/CLAUDE.md accuracy  |
| `/project:apply-docs-fixes`              | Apply all fixes identified by `/project:docs-audit` (URLs, stale paths, missing constants, breakpoint table, PR checklist) |
| `/project:implement-v02-mvp`             | Implement v0.2 MVP features from Doca.zip specs (search, pull wizard, stats, terminal, compose, dashboard, runtime switcher) |
| `/project:redesign-claude-setup`         | Survey and regenerate the full `.claude/commands/` set for the complete FLOSS development lifecycle |
| `/project:testing-audit`                 | Audit and plan the full Rust test strategy; identify coverage gaps and missing test categories |
| `/project:cleanup-code`                  | Remove dead code (`#[allow(dead_code)]`), useless comments, and duplicated view helpers; extract shared `window/utils/` module |
| `/project:test-quality-guardrail`        | Guardrail de qualidade: audita testes contra princípios universais, identifica antipadrões e aplica abstrações (Builder, Object Mother, fixtures compartilhadas, parametrização) |
| `/project:redesign-makefile`             | Reestrutura o Makefile como instrumento SOLID de ciclo de vida: agregadores genéricos delegam para específicos, `make` sem args configura e roda automaticamente |
| `/project:dist-audit`                    | Audit distributed artifacts (Flatpak, macOS DMG, Windows ZIP) for runtime completeness, identity consistency, store compliance, and first-install UX |
| `/project:sync-pull-request`             | Create or update the PR for the current branch — enforces title/body/base contracts and reports state, review status, and CI checks |
| `/project:content-audit`                 | Audit content quality: human-first readability, AI-free codebase, Mermaid-over-ASCII diagrams, comment quality, terminology consistency, and placeholder hygiene |
| `/project:plan-content-improvements`     | Build or refresh a prioritised improvement plan from the latest content audit; tracks status, effort, and quick wins |
| `/project:create-prompt <context>`       | Generate a high-quality, self-contained AI prompt for a given context; saves to `.claude/prompts/` |

## Dependency Versioning Rule

When bumping a dependency in `Cargo.toml`, update the version table in §Dependencies in the same
commit. `Cargo.toml` is the source of truth — CLAUDE.md must never diverge from it.

## Testing

- `make test` — runs all tests via `cargo nextest` (unit + integration + i18n).
- `make test-unit` — unit tests only (`cargo nextest --lib`).
- `make test-integration` — integration tests only (`container_driver_test` + `greet_use_case_test`).
- `make test-i18n` — i18n structural tests only.
- `make test-nextest` — alias for `make test` (backwards compatibility).
- `make coverage` — runs `cargo llvm-cov` summary for lib + integration tests (manual tool; not part of CI).
- `NEXTEST_PROFILE` — pass `NEXTEST_PROFILE=ci` to activate fail-fast mode (`.config/nextest.toml`).

**Test layers:**

| Layer | Location | Rule |
|-------|----------|------|
| Unit | `#[cfg(test)]` inline in `src/core/` and `src/infrastructure/logging/` | No `gtk4` / `adw` imports |
| Integration | `tests/container_driver_test.rs`, `tests/greet_use_case_test.rs`, `tests/i18n_test.rs` | Public API only; use `MockContainerDriver` |
| Widget | `tests/widget_test.rs` | Marked `#[ignore]`; run with `xvfb-run cargo test --test widget_test -- --test-threads=1 --ignored` |

**Rules:**
- `tests/unit/` does **not** exist — domain unit tests live inline via `#[cfg(test)]` (Rust convention for accessing private internals).
- Domain tests (`src/core/`) must not import `gtk4`, `adw`, or `glib`.
- Tests for a new use case go in `tests/<use_case>_test.rs` or inline in `src/core/use_cases/<use_case>.rs`.
- `MockContainerDriver` is the sole driver used in all integration tests — never use Docker/Podman sockets in CI.
- Widget tests require a display; CI runs them only with `xvfb-run` on explicit request (not part of default CI pipeline).
