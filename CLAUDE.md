# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Run Commands

**Local development (Cargo):**
```sh
make setup        # Fetch Cargo dependencies (run once)
make build        # Compile with cargo build
make run          # Build and run the application
make test         # Run unit + integration tests (cargo test)
make lint         # Run cargo clippy -- -D warnings
make fmt          # Check formatting (cargo fmt --check)
make fmt-fix      # Auto-format code (cargo fmt)
make run-mobile   # Run with GTK_DEBUG=interactive to simulate narrow screen
make clean        # Remove build artifacts
```

**Flatpak distribution:**
```sh
make flatpak-build      # Build Flatpak x86_64 (force-clean, installs deps from Flathub)
make flatpak-run        # Build and run in Flatpak sandbox
make flatpak-install    # Build and install the Flatpak
make flatpak-build-arm  # Cross-compile for aarch64 (GNOME Mobile / PinePhone)
```

**Cross-platform setup (one-time per platform):**
```sh
make setup-macos    # Print Homebrew install instructions for macOS
make setup-windows  # Print MSYS2/MINGW64 install instructions for Windows
```

The Flatpak manifest is `com.example.GtkCrossPlatform.json`. The build profile
(`default` or `development`) and target platform (`linux`, `macos`, `windows`, `mobile`)
are controlled via environment variables passed to `cargo build` (see `build.rs`).

## Architecture

This is a **cross-platform** GTK4 + Adwaita desktop application written in **Rust**,
targeting Linux, macOS, Windows, and GNOME Mobile (Phosh/postmarketOS).

**App ID:** `com.example.GtkCrossPlatform` — replace with a real reverse-domain ID before publishing.

The codebase follows **Hexagonal Architecture** (Ports & Adapters):

| Layer | Path | Rule |
|---|---|---|
| Domain (core) | `src/core/` | No GTK/Adw/GLib imports; pure business logic |
| Ports | `src/ports/` | Rust traits consumed by core and UI |
| Adapters | `src/infrastructure/` | Implement ports; may use GLib/IO, never GTK |
| UI | `src/window/` | GTK/Adw widgets only; depends on ports |
| Composition root | `src/app.rs` → `activate()` | Only place where concrete types are wired |

**Key types:**
- `GtkCrossPlatformApp` (`src/app.rs`) — GLib subclass of `adw::Application`, owns app lifecycle and the composition root (`activate()` wires all dependencies).
- `MainWindow` (`src/window/main_window.rs`) — GLib subclass of `adw::ApplicationWindow`, uses `CompositeTemplate` backed by `data/resources/window.ui`. Breakpoints and layout are declared in the UI file; `GestureLongPress` is wired in Rust.
- `GreetUseCase` (`src/core/use_cases/greet_use_case.rs`) — domain use case; zero external imports.
- `IGreetingService` (`src/ports/i_greeting_service.rs`) — port trait used by the UI layer.
- `GreetingService` (`src/infrastructure/greeting/greeting_service.rs`) — adapter implementing `IGreetingService`, delegates to `GreetUseCase`.
- `AppLogger` (`src/infrastructure/logging/app_logger.rs`) — thin wrapper around `glib::g_debug!` / `g_info!` / `g_warning!` / `g_critical!`.
- `config` (`src/config.rs`) — compile-time constants from `build.rs` env vars: `APP_ID`, `VERSION`, `PROFILE`, `LOCALEDIR`, `PKGDATADIR`, `SOURCE_DATADIR`, `GETTEXT_PACKAGE`.

**Responsive breakpoints** (declared in `data/resources/window.ui`):
| Breakpoint       | Condition        | Margin |
|------------------|------------------|--------|
| Desktop normal   | > 768 sp         | 48 sp  |
| Desktop compact  | ≤ 768 sp         | 32 sp  |
| Tablet           | ≤ 600 sp         | 24 sp  |
| GNOME Mobile     | ≤ 360 sp         | 16 sp  |

**Touchscreen:** minimum 44×44 sp touch targets on interactive elements; `gtk4::GestureLongPress` provides touchscreen alternative to right-click context.

**Dependencies:** gtk4 = 0.11 (feature `v4_14`), libadwaita = 0.9 (feature `v1_6`), glib = 0.22, gio = 0.22. Minimum runtime: GTK4 ≥ 4.14, LibAdwaita ≥ 1.6.

**GResource:** UI files are compiled into `compiled.gresource` at build time via `glib-build-tools` (see `build.rs`). Resources are registered in `main()` before the application starts via `gio::resources_register_include!`.

**Flatpak sandbox permissions:** Wayland socket, X11 fallback, IPC. (`--device=dri` removed — re-add if GPU access is needed.)

## i18n / Localization

All user-visible strings must use `gettextrs::gettext("...")`. The pipeline:

- `po/POTFILES` — lists `.rs` and `.ui` files containing translatable strings
- `po/LINGUAS` — lists active locale codes (e.g., `pt_BR`)
- `po/meson.build` — runs `i18n.gettext()` to compile `.po` → `.mo`
- `po/pt_BR.po` — example Brazilian Portuguese translation

The text domain is bound in `main()` via `gettextrs::bindtextdomain` / `textdomain` using `config::GETTEXT_PACKAGE` and `config::LOCALEDIR`.

To add a new language: run `msginit -l <locale>` in `po/`, add the locale to `LINGUAS`, translate strings, run `make build`.

## Design Standards

This project follows the [GNOME Human Interface Guidelines (HIG)](https://developer.gnome.org/hig/). When adding or modifying UI:

- Use Adwaita widgets (`adw::*`) over raw GTK widgets whenever an equivalent exists
- Declare layout in `.ui` files (Composite Templates); wire signals and closures in Rust
- Follow GNOME naming conventions, spacing, and layout patterns
- Support both Wayland and X11; never assume a specific display server
- Add an `AdwBreakpoint` in `window.ui` for every new layout change at 360/600/768 sp thresholds
- Use `adw::ToastOverlay` for transient feedback instead of dialogs where appropriate
- All touch targets must be ≥ 44×44 sp; never rely on `hover` as the sole state indicator
- Avoid menus activated only by right-click — provide `gtk4::GestureLongPress` equivalent

## Project Structure

```
src/
  main.rs                                # Entry point: registers GResource, sets up i18n, runs app
  app.rs                                 # GtkCrossPlatformApp (adw::Application subclass) + composition root
  config.rs                              # Compile-time constants from build.rs env vars
  lib.rs                                 # Library root: exports core, infrastructure, ports, config
  window/
    mod.rs
    main_window.rs                       # MainWindow (CompositeTemplate over window.ui)
  core/
    mod.rs
    use_cases/
      mod.rs
      greet_use_case.rs                  # GreetUseCase — pure domain logic, no GTK
  ports/
    mod.rs
    i_greeting_service.rs                # IGreetingService trait
  infrastructure/
    mod.rs
    greeting/
      mod.rs
      greeting_service.rs                # GreetingService — implements IGreetingService
    logging/
      mod.rs
      app_logger.rs                      # AppLogger — glib log wrapper
tests/
  greet_use_case_test.rs                 # Integration test: GreetUseCase (no GTK deps)
data/
  resources/
    resources.gresource.xml              # GResource manifest
    window.ui                            # MainWindow template (layout + breakpoints)
  icons/hicolor/scalable/apps/
    com.example.GtkCrossPlatform.svg     # Application icon (scalable)
  com.example.GtkCrossPlatform.desktop   # Desktop entry (app launcher, Flathub)
  com.example.GtkCrossPlatform.metainfo.xml  # AppStream metainfo (GNOME Software)
  meson.build                            # Installs icons, desktop entry, metainfo
po/
  POTFILES                               # Source files with translatable strings
  LINGUAS                                # Active locale list
  pt_BR.po                               # Brazilian Portuguese translation (example)
com.example.GtkCrossPlatform.json        # Flatpak manifest (GNOME Platform 48, rust-stable)
Cargo.toml                               # Rust manifest (gtk4 0.11, adw 0.9, glib/gio 0.22)
build.rs                                 # Injects APP_ID/PROFILE/PKGDATADIR/LOCALEDIR; compiles GResource
meson.build                              # Meson build (Vala→C path, kept for Flatpak compatibility)
meson_options.txt                        # Options: profile (default/development), platform
Makefile                                 # Convenience wrappers for Cargo/Flatpak commands
```

## Testing

- `make test` runs all tests via `cargo test`.
- Tests in `tests/` are integration tests; inline `#[cfg(test)]` modules in `src/core/` are unit tests.
- Domain tests (`core/`) must not import `gtk4` or `adw`.
- Tests for a new use case go in `tests/<use_case>_test.rs` or inline in `src/core/use_cases/<use_case>.rs`.
