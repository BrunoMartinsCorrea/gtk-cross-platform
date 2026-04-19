# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-04-18

### Added

- Initial GTK4 + LibAdwaita application skeleton in Vala
- `GtkCrossPlatformApp` (app lifecycle) and `MainWindow` (primary UI)
- Responsive layout via three `Adw.Breakpoint` thresholds: 768 sp / 600 sp / 360 sp
- Touch support: `Gtk.GestureLongPress` alternative to right-click; 44×44 sp minimum touch targets
- GNU gettext i18n pipeline (`po/POTFILES`, `_()` wrappers, Brazilian Portuguese translation)
- Tux mascot as scalable app icon (`data/icons/hicolor/scalable/apps/`)
- Flatpak manifest targeting GNOME Platform 48 (Wayland + X11 fallback)
- Cross-platform build targets: Linux Flatpak, macOS (Homebrew), Windows (MSYS2), GNOME Mobile aarch64
- Hexagonal architecture: `core/` (domain), `ports/` (interfaces), `infrastructure/` (adapters)
- `AppLogger` structured logging wrapper over `GLib.log` with DEBUG/INFO/WARNING/ERROR levels
- Development profile activates `G_MESSAGES_DEBUG=all` at runtime
- `GreetUseCase` unit tests with `GLib.Test` (no GTK dependency in test executable)
- `meson_options.txt` options: `profile` (default/development) and `platform` (linux/macos/windows/mobile)

[Unreleased]: https://github.com/example/gtk-cross-platform/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/example/gtk-cross-platform/releases/tag/v0.1.0
