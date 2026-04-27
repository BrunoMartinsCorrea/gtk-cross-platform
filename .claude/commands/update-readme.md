---
description: Generate or update README.md (and CONTRIBUTING.md if stale) following modern OSS standards — human-first, AI-context-rich.
---

Update `README.md` to reflect the current project state. Also update `CONTRIBUTING.md` if it
contains stale paths, module names, or missing commit scopes.

> Architecture, layer rules, and breakpoints are in `CLAUDE.md` — reference them, do not duplicate.

## Step 1 — Gather current facts

Read these files before writing anything:

- `Cargo.toml` — exact dependency versions (gtk4, libadwaita, glib, gio, async-channel)
- `Makefile` — all available targets with their descriptions
- `com.example.GtkCrossPlatform.json` — GNOME Platform runtime version
- `data/com.example.GtkCrossPlatform.metainfo.xml` — AppStream summary and description
- `src/` — real module structure (core, ports, infrastructure, window)
- `CLAUDE.md` — architecture layers, key types, breakpoints, dependencies
- `CONTRIBUTING.md` — commit convention, scope table
- `CHANGELOG.md` — current version and release history

## Step 2 — Required sections (in this order)

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
11. **Guidelines** — table with real links (see Step 4)
12. **Contributing** — link to `CONTRIBUTING.md`
13. **License**

## Step 3 — Mandatory declarations

- State explicitly: **"No Electron, no Qt, no web views."**
- State which container runtimes are supported: Docker, Podman, containerd
- State which platforms are targeted: Linux (Flatpak / native), macOS, Windows, GNOME Mobile (Phosh/postmarketOS)
- State the architectural pattern: **Hexagonal Architecture (Ports & Adapters)**

## Step 4 — Guidelines table (mandatory, with links)

| Scope                       | Guideline                                                                                                 |
|-----------------------------|-----------------------------------------------------------------------------------------------------------|
| UI/UX design                | [GNOME Human Interface Guidelines](https://developer.gnome.org/hig/)                                      |
| Widgets                     | [LibAdwaita patterns](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/stable/) — `Adw.*` over raw GTK |
| Accessibility (A11Y)        | [GNOME Accessibility](https://wiki.gnome.org/Accessibility) — semantic roles, labels, keyboard nav        |
| Internationalization (I18N) | [GNU gettext](https://www.gnu.org/software/gettext/) — all visible text via `gettext()`                   |
| Responsiveness              | `AdwBreakpoint` at 360 / 600 / 768 sp                                                                     |
| Dark / Light theme          | `AdwStyleManager` — automatic, no manual toggle                                                           |
| Distribution                | [Flatpak](https://docs.flatpak.org/) with least-privilege finish-args                                     |
| Commits                     | [Conventional Commits](https://www.conventionalcommits.org/)                                              |
| Changelog                   | [Keep a Changelog](https://keepachangelog.com/)                                                           |
| Rust                        | [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)                                        |

## Step 5 — Rewrite README.md

Rewrite from scratch applying the structure above. Do not preserve the old structure — only
keep factual content that is still accurate. Follow the human-first principles from `CLAUDE.md`
under "Documentation Philosophy":

- Short sections with clear headers
- Lists over paragraphs
- Code blocks for all commands
- Logical reading flow: what it is → why → how to use → how to contribute

## Quality checklist

- [ ] All dependency versions match `Cargo.toml`
- [ ] All Makefile targets listed actually exist in `Makefile`
- [ ] Guideline links are real, non-placeholder URLs
- [ ] GNOME ecosystem exclusivity stated explicitly ("No Electron, no Qt, no web views")
- [ ] A11Y, I18N, responsiveness, dark/light theme and contrast all mentioned
- [ ] Architecture table matches `CLAUDE.md`
- [ ] Container runtimes and target platforms listed
- [ ] No stale information (wrong module paths, removed targets, outdated versions)
