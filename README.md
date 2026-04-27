# GTK Cross-Platform

<p align="center">
  A native GNOME container manager built entirely on the GNOME ecosystem —
  GTK4, LibAdwaita, GLib, GIO — written in Rust, distributed as Flatpak.
</p>

<p align="center">
  <a href="LICENSE"><img alt="GPL-3.0-or-later" src="https://img.shields.io/badge/license-GPL--3.0--or--later-blue.svg"/></a>
  <img alt="GNOME Platform 48" src="https://img.shields.io/badge/GNOME%20Platform-48-4A86CF.svg"/>
  <img alt="Rust" src="https://img.shields.io/badge/language-Rust-CE412B.svg"/>
  <img alt="Flatpak" src="https://img.shields.io/badge/distribution-Flatpak-4A90D9.svg"/>
</p>

---

## What is this?

GTK Cross-Platform is a container management desktop application — think Docker Desktop, but native to the GNOME
desktop.
It connects to Docker, Podman, or containerd and lets you inspect containers, images, volumes, and networks from a clean
Adwaita UI.

The project is also a **reference implementation** of GNOME application development best practices: hexagonal
architecture in Rust, full GNOME HIG compliance, responsive layout targeting both desktop and GNOME Mobile (Phosh /
postmarketOS), accessibility, internationalization, and Flatpak-first distribution.

Every library used — GTK4, LibAdwaita, GLib, GIO — is part of the GNOME project. No Electron, no Qt, no web views.

---

## Screenshots

> Add screenshots to `docs/screenshots/` and link them here.

---

## Features

### Container runtimes

- Docker, Podman, and containerd via a unified port/adapter interface
- Auto-detection of the available runtime at startup
- Containers, images, volumes, and networks — list, inspect, start, stop, remove, prune

### GNOME-native UI

- **Adwaita design language** — follows every [GNOME Human Interface Guideline](https://developer.gnome.org/hig/)
- **Responsive layout** — four breakpoints (> 768 sp desktop → 360 sp GNOME Mobile), declared in UI templates
- **Wayland-first** — Wayland socket primary, X11 fallback via `--socket=fallback-x11`
- **Dark / light theme** — automatic via `AdwStyleManager`; no manual theme switching needed
- **High contrast** — tested against GNOME's high-contrast mode
- **Touch support** — `GestureLongPress` as the right-click equivalent; all interactive elements ≥ 44 × 44 sp
- **Keyboard navigation** — full keyboard accessibility; no interaction requires a pointer

### Developer experience

- **Hexagonal architecture** — domain logic is completely decoupled from GTK and IO
- **Internationalization (i18n)** — GNU gettext pipeline; Brazilian Portuguese included
- **Accessibility (a11y)** — semantic widget hierarchy; labels on every interactive element
- **Flatpak sandbox** — Wayland + IPC only; no broad filesystem or network permissions
- **Conventional Commits** — structured commit history, automated changelog via Keep a Changelog

---

## Architecture

The codebase enforces a strict layering rule: **inner layers never import outer layers**.

```
┌─────────────────────────────────────────────────────────┐
│  UI  (src/window/)       GTK4 · LibAdwaita · GLib       │
│  — widgets, views, components, CompositeTemplate         │
├─────────────────────────────────────────────────────────┤
│  Ports  (src/ports/)     pure Rust traits                │
│  — IContainerDriver · IGreetingService                   │
├─────────────────────────────────────────────────────────┤
│  Infrastructure  (src/infrastructure/)   GLib · GIO      │
│  — Docker · Podman · containerd adapters                 │
│  — HTTP over Unix socket · async background tasks        │
├─────────────────────────────────────────────────────────┤
│  Domain  (src/core/)     zero external deps              │
│  — Container · Image · Volume · Network models           │
│  — use cases: pure functions, fully unit-testable        │
└─────────────────────────────────────────────────────────┘
```

Composition root: `src/app.rs` → `activate()`. This is the only place where concrete types are wired to their ports.

---

## Requirements

| Component              | Minimum version                                                     |
|------------------------|---------------------------------------------------------------------|
| GTK4                   | ≥ 4.12                                                              |
| LibAdwaita             | ≥ 1.4                                                               |
| Rust                   | stable (2024 edition)                                               |
| glib-compile-schemas   | part of `libglib2.0-dev` / `glib2-devel` / `glib` (Homebrew)       |
| Container runtime      | Docker ≥ 20 · Podman ≥ 4 · containerd ≥ 1.6 (one of these)         |

---

## Getting started

```sh
git clone https://github.com/BrunoMartinsCorrea/gtk-cross-platform.git
cd gtk-cross-platform
make setup   # fetch Cargo dependencies
make run     # build and launch
```

---

## Build reference

### Local (Cargo)

```sh
make setup        # fetch dependencies (once)
make build        # cargo build
make run          # build and run
make test         # cargo test — unit + integration
make lint         # cargo clippy -- -D warnings
make fmt          # check formatting (cargo fmt --check)
make fmt-fix      # auto-format (cargo fmt)
make run-mobile   # simulate GNOME Mobile (GTK_DEBUG=interactive, 360 sp)
make clean        # remove build artifacts
```

### Flatpak

```sh
make flatpak-build      # build x86_64 Flatpak (GNOME Platform 48)
make flatpak-run        # build and run in sandbox
make flatpak-install    # install locally
make flatpak-build-arm  # cross-compile for aarch64 (PinePhone / Librem 5)
```

### macOS (Homebrew)

```sh
make setup-macos         # installs GTK4 stack via Homebrew + Rust via rustup
make setup && make build
```

The Adwaita theme is applied on macOS and Windows — native platform look is intentionally not used. This is a GNOME
application.

### Windows (MSYS2 / MINGW64)

```sh
make setup-windows   # prints required pacman commands
```

---

## Project layout

```
src/
  main.rs                        # entry point: GResource, i18n, app launch
  app.rs                         # GtkCrossPlatformApp + composition root
  config.rs                      # compile-time constants from build.rs
  lib.rs                         # library root
  core/                          # domain — no GTK, no IO
    domain/                      # Container, Image, Volume, Network models
    use_cases/                   # business logic (pure functions)
  ports/                         # Rust traits (IContainerDriver, IGreetingService)
  infrastructure/
    containers/                  # Docker, Podman, containerd adapters
    greeting/                    # GreetingService adapter
    logging/                     # AppLogger (wraps GLib structured logging)
  window/
    main_window.rs               # MainWindow (CompositeTemplate over window.ui)
    views/                       # ContainersView, ImagesView, VolumesView, NetworksView
    components/                  # ResourceRow, StatusBadge, DetailPane, ConfirmDialog
tests/
  container_driver_test.rs       # integration tests (real driver or mock)
  widget_test.rs                 # GTK widget tests
data/
  resources/
    window.ui                    # MainWindow layout + AdwBreakpoint declarations
    style.css                    # application-scoped CSS overrides
  icons/hicolor/scalable/apps/
    com.example.GtkCrossPlatform.svg
  com.example.GtkCrossPlatform.desktop
  com.example.GtkCrossPlatform.metainfo.xml
po/
  POTFILES                       # source files with translatable strings
  LINGUAS                        # active locale list
  pt_BR.po                       # Brazilian Portuguese
com.example.GtkCrossPlatform.json   # Flatpak manifest (GNOME Platform 48)
```

---

## Design principles

This project explicitly follows these standards and guidelines:

- [GNOME Human Interface Guidelines](https://developer.gnome.org/hig/) — the primary design authority for every UI
  decision
- [LibAdwaita patterns](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/stable/) — `Adw.*` widgets over raw GTK
  equivalents wherever an equivalent exists
- [GNOME Accessibility](https://wiki.gnome.org/Accessibility) — semantic widget hierarchy, proper labelling,
  keyboard-only navigation
- [GNU gettext i18n](https://www.gnu.org/software/gettext/) — every user-visible string is wrapped in `gettext()`
- [Flatpak sandboxing](https://docs.flatpak.org/en/latest/sandbox-permissions.html) — least-privilege finish args; no
  `--filesystem=home` or broad network grants
- [Conventional Commits](https://www.conventionalcommits.org/) — structured commit messages for changelog automation
- [Keep a Changelog](https://keepachangelog.com/) — human-readable changelog at `CHANGELOG.md`
- **Code style**: minimal comments — code is self-documenting through naming; comments only appear when the *why* is
  non-obvious

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for environment setup, commit conventions, UI guidelines, translation workflow,
and pull request expectations.

All contributions are expected to:

- Pass `make test` and `make lint`
- Cite the relevant [GNOME HIG](https://developer.gnome.org/hig/) section for UI changes
- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) for public interfaces
- Keep `src/core/` and `src/ports/` free of GTK, GIO, and any IO imports

---

## License

GPL-3.0-or-later — see [LICENSE](LICENSE).

This project is free software. Contributions, forks, and redistribution are welcome under the same terms.
