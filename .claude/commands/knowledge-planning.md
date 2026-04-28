---
description: Knowledge planning — plan and apply documentation improvements for this GTK4/Rust/Flatpak project. Covers README/CONTRIBUTING regeneration and targeted documentation fixes. This command is self-contained.
---

# /project:knowledge-planning

Plan and apply documentation improvements for this project. This command unifies two responsibilities
from the knowledge-lifecycle sub-cycle:

1. **Knowledge planning** — deciding which documentation updates are needed, in what order, and why
2. **Knowledge application** — applying specific targeted fixes to existing documents

Run it fresh without prior conversation context. Read all source files before writing anything.

---

## When to use

- After running `/project:knowledge-audit` — apply the gaps it found
- After a significant code change — update README/CONTRIBUTING to reflect the new state
- Before a release — ensure all docs are accurate and complete
- As a standalone improvement — generate fresh README.md following current OSS standards

---

## Mode A: Apply Targeted Fixes

Apply all documentation fixes identified by `/project:knowledge-audit`. Read the files listed
under each fix before editing them. Apply every fix in the order listed. Run `make fmt` and
`make lint` at the end to verify nothing in the Rust source was accidentally disturbed.
Do **not** change any Rust source files — this mode is documentation-only.

### Fix 1 — GOVERNANCE.md: replace deleted workflow reference

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

### Fix 2 — Replace placeholder `github.com/example` URLs everywhere

The following files contain `https://github.com/example/gtk-cross-platform` which is a placeholder.
Replace every occurrence with `https://github.com/BrunoMartinsCorrea/gtk-cross-platform`.

Files to update:

- `CHANGELOG.md` (two link-reference lines at the bottom)
- `SECURITY.md` (GitHub Security Advisory URL in §Reporting a vulnerability)
- `CODE_OF_CONDUCT.md` (GitHub Security Advisory URL in §Reporting)

---

### Fix 3 — CLAUDE.md: add SOURCE_DATADIR to config constants list

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

### Fix 4 — CLAUDE.md: add 720sp breakpoint to the responsive breakpoints table

**File:** `CLAUDE.md` §Responsive breakpoints

`data/resources/window.ui` defines a fourth `AdwBreakpoint` at `max-width: 720sp` that collapses
`AdwNavigationSplitView` and reveals `AdwViewSwitcherBar`. It is absent from the table.

Replace the current breakpoints table with:

| Breakpoint          | Condition | Effect                                                                                            |
|---------------------|-----------|---------------------------------------------------------------------------------------------------|
| Desktop normal      | > 768 sp  | Margins 48 sp                                                                                     |
| Desktop compact     | ≤ 768 sp  | Margins 32 sp                                                                                     |
| Split-view collapse | ≤ 720 sp  | `AdwNavigationSplitView` collapses; `AdwViewSwitcherBar` revealed; `AdwViewSwitcher` (top) hidden |
| Tablet              | ≤ 600 sp  | Margins 24 sp                                                                                     |
| GNOME Mobile        | ≤ 360 sp  | Margins 16 sp                                                                                     |

---

### Fix 5 — CLAUDE.md: fix po/meson.build reference in i18n section

**File:** `CLAUDE.md` §i18n / Localization

The project uses Cargo/build.rs for GResource compilation, not Meson. Replace:

```
- `po/meson.build` — runs `i18n.gettext()` to compile `.po` → `.mo`
```

With:

```
- `po/POTFILES` — lists `.rs` and `.ui` files containing translatable strings
- `po/LINGUAS` — lists active locale codes (20+ community translations; see po/LINGUAS)
```

---

### Fix 6 — CLAUDE.md: add gettext-rs and glib-build-tools to §Dependencies

**File:** `CLAUDE.md` §Dependencies

`Cargo.toml` is the source of truth. The current paragraph omits `gettext-rs = "0.7"` (i18n
runtime) and `glib-build-tools = "0.20"` (build-time GResource compiler). Append both to the
dependency sentence.

---

### Fix 7 — CLAUDE.md: fix composition root claim

**File:** `CLAUDE.md` §Key types (`GtkCrossPlatformApp`) and §Architecture

Find:

```
`activate()` is the only place where concrete types are wired to ports.
```

Replace with:

```
`activate()` is the primary place where concrete types are wired to ports. `MainWindow` may also
re-wire the runtime adapter when the user switches runtimes at runtime via the runtime switcher.
```

---

### Fix 8 — CLAUDE.md: update threading section — app.rs also calls spawn_driver_task

**File:** `CLAUDE.md` §Threading

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

### Fix 9 — CLAUDE.md: update §Project Structure to add missing files and directories

Add all files and directories added since the initial implementation: use cases, ports/use_cases/,
dynamic_driver.rs, actions.rs, empty_state.rs, list_factory.rs, toast_util.rs, objects/, dashboard_view.rs,
style-dark.css, GSettings schema.

---

### Fix 10 — CLAUDE.md: add missing slash commands to the table

Add entries for commands that exist in `.claude/commands/` but have no table entry in CLAUDE.md.

---

### Fix 11 — CLAUDE.md: document key undocumented Makefile targets

Add Quality Gates (local), macOS distributable, and Release subsections to §Build and Run Commands.

---

### Fix 12 — CONTRIBUTING.md: fix rustup apt install command

**File:** `CONTRIBUTING.md` §Development environment → Linux

`rustup` is not available as an apt package on Ubuntu/Debian. Replace:

```sh
sudo apt install libgtk-4-dev libadwaita-1-dev gettext rustup
rustup toolchain install stable
```

With:

```sh
sudo apt install libgtk-4-dev libadwaita-1-dev gettext
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install stable
```

---

### Fix 13 — CONTRIBUTING.md: align PR checklist with PR template

Replace the current 4-step PR section with a full checklist matching `.github/PULL_REQUEST_TEMPLATE.md`.

---

### Fix 14 — README.md: fix setup-macos description

Change: `make setup-macos # prints Homebrew install instructions`
To: `make setup-macos # installs GTK4 stack via Homebrew + Rust via rustup`

---

### Fix 15 — README.md: note glib-compile-schemas in getting-started prerequisites

Add `glib-compile-schemas` to Requirements table as a build-time dependency (`part of libglib2.0-dev / glib2-devel / glib (Homebrew)`).

---

## Mode B: Regenerate README.md

Update `README.md` to reflect the current project state. Also update `CONTRIBUTING.md` if it
contains stale paths, module names, or missing commit scopes.

> Architecture, layer rules, and breakpoints are in `CLAUDE.md` — reference them, do not duplicate.

### Step 1 — Gather current facts

Read these files before writing anything:

- `Cargo.toml` — exact dependency versions (gtk4, libadwaita, glib, gio, async-channel)
- `Makefile` — all available targets with their descriptions
- `com.example.GtkCrossPlatform.json` — GNOME Platform runtime version
- `data/com.example.GtkCrossPlatform.metainfo.xml` — AppStream summary and description
- `src/` — real module structure (core, ports, infrastructure, window)
- `CLAUDE.md` — architecture layers, key types, breakpoints, dependencies
- `CONTRIBUTING.md` — commit convention, scope table
- `CHANGELOG.md` — current version and release history

### Step 2 — Required sections (in this order)

1. **Title + tagline** — one sentence: what it is, for whom
2. **Badges** — license, platform, language, Flatpak distribution
3. **What is this?** — 2–3 honest sentences; no placeholder claims
4. **Screenshots / demo** — placeholder if none exists yet
5. **Features** — grouped by category (container management, UI, platform support, distribution)
6. **Architecture** — include the hexagonal layer table from `CLAUDE.md`; add a C4 context ASCII diagram
7. **Requirements** — exact runtime versions derived from `Cargo.toml` and the Flatpak manifest
8. **Getting started** — minimum path from clone to running (`make setup && make run`)
9. **Build reference** — every Makefile target with a one-line description
10. **Project structure** — `src/`, `tests/`, `data/`, `po/` with inline notes
11. **Guidelines** — table with real links (see below)
12. **Contributing** — link to `CONTRIBUTING.md`
13. **License**

### Step 3 — Mandatory declarations

- State explicitly: **"No Electron, no Qt, no web views."**
- State which container runtimes are supported: Docker, Podman, containerd
- State which platforms are targeted: Linux (Flatpak / native), macOS, Windows, GNOME Mobile (Phosh/postmarketOS)
- State the architectural pattern: **Hexagonal Architecture (Ports & Adapters)**

### Step 4 — Guidelines table (mandatory, with links)

| Scope                       | Guideline                                                                                                 |
|-----------------------------|-----------------------------------------------------------------------------------------------------------|
| UI/UX design                | [GNOME Human Interface Guidelines](https://developer.gnome.org/hig/)                                      |
| Widgets                     | [LibAdwaita patterns](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/stable/) — `Adw.*` over raw GTK |
| Accessibility (A11Y)        | [GNOME Accessibility](https://wiki.gnome.org/Accessibility) — semantic roles, labels, keyboard nav        |
| Internationalization (I18N) | [GNU gettext](https://www.gnu.org/software/gettext/) — all visible text via `gettext()`                   |
| Responsiveness              | `AdwBreakpoint` at 360 / 600 / 720 / 768 sp                                                              |
| Dark / Light theme          | `AdwStyleManager` — automatic, no manual toggle                                                           |
| Distribution                | [Flatpak](https://docs.flatpak.org/) with least-privilege finish-args                                     |
| Commits                     | [Conventional Commits](https://www.conventionalcommits.org/)                                              |
| Changelog                   | [Keep a Changelog](https://keepachangelog.com/)                                                           |
| Rust                        | [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)                                        |

### Step 5 — Rewrite README.md

Rewrite from scratch applying the structure above. Follow the human-first principles from `CLAUDE.md`
under "Documentation Philosophy": short sections, lists over paragraphs, code blocks for all commands,
logical reading flow (what → why → how to use → how to contribute).

### Quality checklist

- [ ] All dependency versions match `Cargo.toml`
- [ ] All Makefile targets listed actually exist in `Makefile`
- [ ] Guideline links are real, non-placeholder URLs
- [ ] GNOME ecosystem exclusivity stated explicitly ("No Electron, no Qt, no web views")
- [ ] A11Y, I18N, responsiveness, dark/light theme and contrast all mentioned
- [ ] Architecture table matches `CLAUDE.md`
- [ ] Container runtimes and target platforms listed
- [ ] No stale information (wrong module paths, removed targets, outdated versions)

---

## Verification

After applying all fixes:

1. `grep -r "flatpak\.yml" . --include="*.md"` — must return no results
2. `grep -r "github.com/example" . --include="*.md"` — must return no results
3. `grep "SOURCE_DATADIR" CLAUDE.md` — must find the entry
4. `grep "720" CLAUDE.md` — must find the breakpoint row
5. `make fmt` — confirm Rust source is untouched
