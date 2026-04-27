# Contributing

## Development environment

### Linux

```sh
sudo apt install libgtk-4-dev libadwaita-1-dev gettext
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install stable
make setup && make build && make test
```

### macOS

```sh
make setup-macos
make setup && make build
```

### Windows (MSYS2 / MINGW64)

```sh
make setup-windows   # prints required pacman commands
```

---

## Project structure

```
src/core/        # domain logic — zero external deps (no GTK, no IO)
src/ports/       # Rust traits consumed by core and UI
src/infrastructure/  # adapters: container runtimes, logging
src/window/      # GTK/Adwaita widgets, views, components
tests/           # integration tests
po/              # gettext translations
data/            # UI templates, icons, desktop/metainfo files
```

**Rule:** `src/core/` and `src/ports/` must never import `gtk4`, `adw`, `gio`, or any IO library.

---

## Commit conventions

This project follows [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add container log streaming view
fix: correct breakpoint threshold at 600 sp
i18n: add French (fr) translation
a11y: add accessible label to status badge
test: cover DockerDriver error paths
chore: update GNOME Platform to 48
```

---

## Testing

`make test` runs all tests. There are three test suites:

| Suite | File | Requires display? |
|---|---|---|
| Integration — container driver | `tests/container_driver_test.rs` | No |
| i18n structural validation | `tests/i18n_test.rs` | No |
| GTK widget tests | `tests/widget_test.rs` | Yes (`#[ignore]`) |

**Container driver tests** use `MockContainerDriver` — no runtime needs to be installed:

```sh
cargo test --test container_driver_test
```

**Widget tests** require a display and must be run explicitly:

```sh
# Linux with virtual framebuffer
xvfb-run cargo test --test widget_test -- --test-threads=1 --ignored

# macOS
cargo test --test widget_test -- --test-threads=1 --ignored
```

**Domain tests** (`src/core/`) must not import `gtk4` or `adw`. New use case tests go in `tests/<use_case>_test.rs` or inline `#[cfg(test)]` in `src/core/use_cases/`.

---

## Adding a translation

1. Run `make build` to regenerate the `.pot` template
2. Create the locale file:
   ```sh
   cd po && msginit -l <locale> -i gtk-cross-platform.pot -o <locale>.po
   ```
3. Add `<locale>` to `po/LINGUAS`
4. Translate strings in `<locale>.po`
5. Run `make build` to compile `.po` → `.mo`

---

## UI guidelines

- Use `adw::*` widgets over raw GTK equivalents wherever an equivalent exists
- All interactive elements must be ≥ 44 × 44 sp (GNOME HIG touch target minimum)
- Add an `AdwBreakpoint` for every layout change at 360 / 600 / 768 sp
- Never use `hover` as the sole state indicator — touchscreens have no hover
- Provide `GestureLongPress` as the touch equivalent for any right-click action
- Test portrait and landscape on mobile: `make run-mobile`
- Cite the relevant [GNOME HIG](https://developer.gnome.org/hig/) section in your PR description

---

## Pull requests

**Before opening a PR:**

1. Fork and create a branch from `main`
2. Write or update tests for new domain logic (in `tests/` or inline `#[cfg(test)]` in `src/core/`)
3. Run all checks:
   ```sh
   make lint && make lint-i18n && make fmt && make test
   ```
4. For UI changes: cite the relevant [GNOME HIG](https://developer.gnome.org/hig/) section in the PR description

**PR checklist (mirrors `.github/PULL_REQUEST_TEMPLATE.md`):**

- [ ] `make fmt` passes (`cargo fmt --check`)
- [ ] `make lint` passes (`cargo clippy -- -D warnings`)
- [ ] `make lint-i18n` passes (`msgfmt` validates all `.po` files)
- [ ] `make test` passes (`cargo test`)
- [ ] All user-visible strings use `gettext!()` / `pgettext!()` / `ngettext!()`
- [ ] Blocking driver calls go through `spawn_driver_task` — no direct GTK calls from worker threads
- [ ] New interactive widgets have `set_tooltip_text` **and** `accessible::Property::Label` set
- [ ] Touch targets on new interactive elements are ≥ 44×44 sp
- [ ] `src/core/` and `src/ports/` do not import `gtk4`, `adw`, or any IO library
- [ ] Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/)

---

## Reporting a security issue

Do **not** open a public issue. Follow the process described in [SECURITY.md](SECURITY.md).

---

## Project governance

Maintainership criteria, decision-making process, and release policy are documented in [GOVERNANCE.md](GOVERNANCE.md).

---

## References

- [GNOME Human Interface Guidelines](https://developer.gnome.org/hig/)
- [LibAdwaita API docs](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/stable/)
- [GTK4 API docs](https://docs.gtk.org/gtk4/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Keep a Changelog](https://keepachangelog.com/)
