# /project:apply-docs-fixes

Apply all documentation fixes identified by `/project:docs-audit`. This command is self-contained —
run it fresh without prior conversation context.

Read the files listed under each fix before editing them. Apply every fix in the order listed.
Run `make fmt` and `make lint` at the end to verify nothing in the Rust source was accidentally
disturbed. Do **not** change any Rust source files — this command is documentation-only.

---

## Fix 1 — GOVERNANCE.md: replace deleted workflow reference

**File:** `GOVERNANCE.md` §Releases

`.github/workflows/flatpak.yml` was deleted; the nightly release workflow is now
`.github/workflows/release.yml`. Replace the stale filename.

Change:
```
`.github/workflows/flatpak.yml`
```
To:
```
`.github/workflows/release.yml`
```

---

## Fix 2 — Replace placeholder `github.com/example` URLs everywhere

The following files contain `https://github.com/example/gtk-cross-platform` which is a placeholder.
Replace every occurrence with `https://github.com/BrunoMartinsCorrea/gtk-cross-platform`.

Files to update:
- `CHANGELOG.md` (two link-reference lines at the bottom)
- `SECURITY.md` (GitHub Security Advisory URL in §Reporting a vulnerability)
- `CODE_OF_CONDUCT.md` (GitHub Security Advisory URL in §Reporting)

---

## Fix 3 — CLAUDE.md: add SOURCE_DATADIR to config constants list

**File:** `CLAUDE.md` §Key types → `config` entry

`src/config.rs` exports `SOURCE_DATADIR` (set in `build.rs`; used in `src/app.rs` for icon theme
path setup at runtime). Add it to the constants list.

Find the config bullet in the Key types section:
```
- `config` (`src/config.rs`) — compile-time constants from `build.rs` env vars: `APP_ID`, `VERSION`, `PROFILE`, `LOCALEDIR`, `PKGDATADIR`, `GETTEXT_PACKAGE`.
```
Replace with:
```
- `config` (`src/config.rs`) — compile-time constants from `build.rs` env vars: `APP_ID`, `VERSION`, `PROFILE`, `LOCALEDIR`, `PKGDATADIR`, `SOURCE_DATADIR`, `GETTEXT_PACKAGE`.
```

---

## Fix 4 — CLAUDE.md: add 720sp breakpoint to the responsive breakpoints table

**File:** `CLAUDE.md` §Responsive breakpoints

`data/resources/window.ui` defines a fourth `AdwBreakpoint` at `max-width: 720sp` that collapses
`AdwNavigationSplitView` and reveals `AdwViewSwitcherBar`. It is absent from the table.

Find the breakpoints table and add the missing row. The table currently has:

| Breakpoint | Condition | Margin |
|------------------|------------------|--------|
| Desktop normal | > 768 sp | 48 sp |
| Desktop compact | ≤ 768 sp | 32 sp |
| Tablet | ≤ 600 sp | 24 sp |
| GNOME Mobile | ≤ 360 sp | 16 sp |

Replace with:

| Breakpoint | Condition | Effect |
|----------------------|------------|----------------------------------------------|
| Desktop normal | > 768 sp | Margins 48 sp |
| Desktop compact | ≤ 768 sp | Margins 32 sp |
| Split-view collapse | ≤ 720 sp | `AdwNavigationSplitView` collapses; `AdwViewSwitcherBar` revealed; `AdwViewSwitcher` (top) hidden |
| Tablet | ≤ 600 sp | Margins 24 sp |
| GNOME Mobile | ≤ 360 sp | Margins 16 sp |

---

## Fix 5 — CLAUDE.md: fix po/meson.build reference in i18n section

**File:** `CLAUDE.md` §i18n / Localization

The project uses Cargo/build.rs for GResource compilation, not Meson. `po/meson.build` does not
exist. The `.po` → `.mo` compilation pipeline is not yet wired.

Find the bullet:
```
- `po/meson.build` — runs `i18n.gettext()` to compile `.po` → `.mo`
```
Replace with:
```
- `po/POTFILES` — lists `.rs` and `.ui` files containing translatable strings
- `po/LINGUAS` — lists active locale codes (20+ community translations; see po/LINGUAS)
```

(Note: `po/POTFILES` and `po/LINGUAS` are already listed later in the same section — remove the
duplicate if present after applying this fix.)

---

## Fix 6 — CLAUDE.md: add gettext-rs and glib-build-tools to §Dependencies

**File:** `CLAUDE.md` §Dependencies

`Cargo.toml` is the source of truth. The current paragraph omits `gettext-rs = "0.7"` (i18n
runtime) and `glib-build-tools = "0.20"` (build-time GResource compiler).

Find the §Dependencies sentence that begins "Dependencies: gtk4 = 0.9" and append both omissions:

```
**Dependencies:** gtk4 = 0.9 (feature `v4_12`), libadwaita = 0.7 (feature `v1_4`), glib = 0.20, gio = 0.20, gettext-rs = 0.7 (i18n runtime), serde_json = 1 (JSON inspect + syntax highlighting), async-channel = 2 (cross-thread messaging). Build dependency: glib-build-tools = 0.20 (GResource compiler). Minimum runtime: GTK4 ≥ 4.12, LibAdwaita ≥ 1.4.
```

---

## Fix 7 — CLAUDE.md: fix composition root claim

**File:** `CLAUDE.md` §Key types (`GtkCrossPlatformApp`) and §Architecture

The claim "activate() is the only place where concrete types are wired to ports" is no longer
accurate. `src/window/main_window.rs` calls `ContainerDriverFactory::detect_specific()` for the
runtime-switcher feature.

Find:
```
`activate()` is the only place where concrete types are wired to ports.
```
Replace with:
```
`activate()` is the primary place where concrete types are wired to ports. `MainWindow` may also
re-wire the runtime adapter when the user switches runtimes at runtime via the runtime switcher.
```

Apply the same correction in the Architecture layer table comment for `src/app.rs`.

---

## Fix 8 — CLAUDE.md: update threading section — app.rs also calls spawn_driver_task

**File:** `CLAUDE.md` §Threading

`src/app.rs` calls `spawn_driver_task` for app-scoped actions. The threading section only names
views and MainWindow as callers.

Find:
```
- Views (`src/window/views/`) are the primary layer that calls `spawn_driver_task`. `MainWindow` may also call it for
  window-scoped actions (e.g., system prune) that do not belong to any single resource view.
```
Replace with:
```
- Views (`src/window/views/`) are the primary layer that calls `spawn_driver_task`. `MainWindow` may also call it for
  window-scoped actions (e.g., system prune) that do not belong to any single resource view.
  `src/app.rs` may also call it for app-lifecycle actions that execute before the window is ready.
```

---

## Fix 9 — CLAUDE.md: update §Project Structure to add missing files and directories

**File:** `CLAUDE.md` §Project Structure

Many files and directories added after the initial implementation are absent from the tree. Update
the `src/` block to reflect the actual file tree.

In the `core/use_cases/` subtree, replace:
```
      mod.rs
      greet_use_case.rs                  # GreetUseCase — pure domain logic, no GTK
```
With:
```
      mod.rs
      container_use_case.rs              # ContainerUseCase — list/inspect/lifecycle operations
      greet_use_case.rs                  # GreetUseCase — pure domain logic, no GTK
      image_use_case.rs                  # ImageUseCase — list/pull/remove/inspect
      network_use_case.rs                # NetworkUseCase — list/create/remove
      volume_use_case.rs                 # VolumeUseCase — list/create/remove
```

In the `ports/` subtree, after `i_greeting_service.rs` add:
```
    use_cases/
      mod.rs
      i_container_use_case.rs            # IContainerUseCase port consumed by ContainersView
      i_image_use_case.rs                # IImageUseCase port consumed by ImagesView
      i_network_use_case.rs              # INetworkUseCase port consumed by NetworksView
      i_volume_use_case.rs               # IVolumeUseCase port consumed by VolumesView
```

In the `infrastructure/containers/` subtree, after `mock_driver.rs` add:
```
      dynamic_driver.rs                  # DynamicDriver — wraps Arc<dyn IContainerDriver> for runtime switching
```

In the `window/` subtree:
- After `mod.rs` add:
```
    actions.rs                           # CommonActions — window-scoped GAction constants
```
- In `components/` subtree, after `confirm_dialog.rs` add:
```
      empty_state.rs                     # EmptyState — adw::StatusPage wrapper for empty list states
      list_factory.rs                    # GtkSignalListItemFactory helpers for ColumnView/ListView
      toast_util.rs                      # ToastUtil — fire-and-forget adw::Toast helpers
```
- After the `components/` block, add a new `objects/` block:
```
    objects/
      mod.rs
      container_object.rs                # ContainerObject — GObject wrapper for Container domain model
      image_object.rs                    # ImageObject — GObject wrapper for Image domain model
      network_object.rs                  # NetworkObject — GObject wrapper for Network domain model
      volume_object.rs                   # VolumeObject — GObject wrapper for Volume domain model
```
- In the `views/` subtree, after the existing views add:
```
      dashboard_view.rs                  # DashboardView — system overview: running containers, disk usage
```

In the `data/` subtree:
- After `style.css` add:
```
    style-dark.css                       # Dark-mode overrides (loaded via AdwStyleManager)
  com.example.GtkCrossPlatform.gschema.xml  # GSettings schema (sidebar-width, last-runtime)
```

In the `po/` subtree, replace:
```
  pt_BR.po                               # Brazilian Portuguese translation (example)
```
With:
```
  pt_BR.po                               # Brazilian Portuguese
  *.po                                   # 20+ community translations (see po/LINGUAS for full list)
```

---

## Fix 10 — CLAUDE.md: add missing slash commands to the table

**File:** `CLAUDE.md` §Slash Commands

Three files in `.claude/commands/` have no table entry. Add them.

Add these rows to the slash commands table:

| Command | When to use |
|---|---|
| `/project:implement-v02-mvp` | Implement the v0.2 MVP feature set from Doca.zip design specs (search, pull, wizard, stats, inspect, terminal, compose, dashboard, runtime switcher) |
| `/project:redesign-claude-setup` | Survey the current command set and generate an improved set covering the full FLOSS development lifecycle |
| `/project:testing-audit` | Audit and plan the full test strategy for the project |

Also add the entry for `.claude/commands/` in the Project Structure `.claude/` block:
```
    commands/
      ...
      implement-v02-mvp.md               # /project:implement-v02-mvp
      redesign-claude-setup.md           # /project:redesign-claude-setup
      testing-audit.md                   # /project:testing-audit
```

---

## Fix 11 — CLAUDE.md: document key undocumented Makefile targets

**File:** `CLAUDE.md` §Build and Run Commands

Add a new subsection after the Flatpak commands:

```
**Quality gates (local):**

```sh
make validate           # Run all local checks: check-version, check-potfiles, validate-metainfo, validate-desktop, lint, lint-i18n, fmt
make validate-metainfo  # appstreamcli validate on metainfo XML
make validate-desktop   # desktop-file-validate on .desktop file
make check-version      # Verify Cargo.toml version == metainfo.xml version
make check-potfiles     # Verify all files with gettext() are listed in po/POTFILES
make deny               # cargo deny check (license + security advisories)
make spell-check        # typos . (spell-check all tracked files)
make check-unused-deps  # cargo machete (detect unused dependencies)
```

**macOS distributable:**

```sh
make dmg               # Build macOS .app bundle + .dmg (requires dylibbundler, create-dmg)
```
```

---

## Fix 12 — CONTRIBUTING.md: fix rustup apt install command

**File:** `CONTRIBUTING.md` §Development environment → Linux

`rustup` is not available as an apt package on Ubuntu/Debian. Replace the single apt command with
two separate steps:

Find:
```sh
sudo apt install libgtk-4-dev libadwaita-1-dev gettext rustup
rustup toolchain install stable
```
Replace with:
```sh
sudo apt install libgtk-4-dev libadwaita-1-dev gettext
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install stable
```

---

## Fix 13 — CONTRIBUTING.md: align PR checklist with PR template

**File:** `CONTRIBUTING.md` §Pull requests

The PR section has 4 steps. The `.github/PULL_REQUEST_TEMPLATE.md` has 9 checklist items.
Replace the current 4-step numbered list with a checklist that matches the PR template:

```
**Before opening a PR, verify:**

1. Fork and create a branch from `main`
2. Write or update tests for new domain logic (in `tests/` or inline `#[cfg(test)]` in `src/core/`)
3. Run all checks:
   ```sh
   make lint && make lint-i18n && make fmt && make test
   ```
4. For UI changes: cite the [GNOME HIG](https://developer.gnome.org/hig/) section in the PR description

**PR checklist (matches `.github/PULL_REQUEST_TEMPLATE.md`):**

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
```

---

## Fix 14 — README.md: fix setup-macos description

**File:** `README.md` §macOS

Find:
```
make setup-macos         # prints Homebrew install instructions
```
Replace with:
```
make setup-macos         # installs GTK4 stack via Homebrew + Rust via rustup
```

---

## Fix 15 — README.md: note glib-compile-schemas in getting-started prerequisites

**File:** `README.md` §Requirements or §Getting started

`make run` invokes `make schema` which requires `glib-compile-schemas`. Add it to the Requirements
table as a build-time dependency:

Find the Requirements table and add one row:
```
| glib-compile-schemas | part of `libglib2.0-dev` / `glib2-devel` / `glib` (Homebrew) |
```

---

## Verification

After applying all fixes:

1. Confirm no `.github/workflows/flatpak.yml` references remain:
   ```sh
   grep -r "flatpak\.yml" . --include="*.md" --include="*.json"
   ```
2. Confirm no `github.com/example` URLs remain:
   ```sh
   grep -r "github.com/example" . --include="*.md"
   ```
3. Confirm `SOURCE_DATADIR` appears in CLAUDE.md:
   ```sh
   grep "SOURCE_DATADIR" CLAUDE.md
   ```
4. Confirm `720sp` breakpoint appears in CLAUDE.md:
   ```sh
   grep "720" CLAUDE.md
   ```
5. Confirm `po/meson.build` no longer appears in CLAUDE.md as a real file path:
   ```sh
   grep "meson.build" CLAUDE.md
   ```
6. Run `make fmt` to confirm Rust source is untouched.

Report any fix that could not be applied cleanly, with the reason.
