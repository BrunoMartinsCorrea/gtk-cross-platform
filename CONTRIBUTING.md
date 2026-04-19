# Contributing

## Development Environment

### Linux

```sh
sudo apt install valac meson ninja-build \
    libgtk-4-dev libadwaita-1-dev gettext
make setup && make build && make test
```

### macOS

```sh
make setup-macos
make setup && make build
```

### Windows

```sh
make setup-windows   # prints required MSYS2/MINGW64 pacman commands
```

## Project Structure

```
src/
  app.vala                          # composition root — wires up dependencies
  window/main_window.vala           # UI adapter (GTK layer)
  ports/i_greeting_service.vala     # interface definitions (no GTK imports)
  core/use_cases/                   # domain logic (no GTK imports)
  infrastructure/                   # concrete adapters (greeting, logging)
tests/
  unit/core/                        # GLib.Test unit tests (no GTK dep)
po/                                 # gettext translations
data/icons/                         # application icons
```

**Rule:** `src/core/` and `src/ports/` must never import GTK or LibAdwaita.

## Commit Conventions

This project follows [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add dark mode support
fix: correct breakpoint at 600 sp
i18n: add French (fr) translation
test: cover GreetUseCase edge cases
chore: update GNOME Platform to 48
```

## Adding a New Language

1. Generate translation template (from project root):
   ```sh
   ninja -C build gtk-cross-platform-pot
   ```
2. Create the new locale file:
   ```sh
   cd po && msginit -l <locale> -i gtk-cross-platform.pot -o <locale>.po
   ```
3. Add `<locale>` to `po/LINGUAS`
4. Translate strings in `<locale>.po`
5. Build to compile `.po` → `.mo`: `make build`

## UI Guidelines

- Use `Adw.*` widgets over raw GTK equivalents wherever possible
- All touch targets must be ≥ 44×44 sp (GNOME HIG)
- Add `Adw.Breakpoint` entries for every layout change at 360 / 600 / 768 sp
- Never use `hover` as the sole state indicator — touchscreens have no hover
- Test portrait and landscape orientations with `make run-mobile`

## Pull Requests

1. Fork and create a branch from `main`
2. Write or update tests in `tests/unit/` for new domain logic
3. Ensure `make test` passes before opening a PR
4. Reference the relevant GNOME HIG section for UI changes

## References

- [GNOME Human Interface Guidelines](https://developer.gnome.org/hig/)
- [Vala Language Reference](https://vala.dev)
- [LibAdwaita API docs](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/stable/)
- [GTK4 API docs](https://docs.gtk.org/gtk4/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Keep a Changelog](https://keepachangelog.com/)
