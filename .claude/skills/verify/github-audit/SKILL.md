---
name: verify:github-audit
version: 1.0.0
description: Audit the repository across six dimensions: CI/CD, security, distribution, docs, licensing, community
---

# verify:github-audit

Audit this GTK4/Rust/Flatpak repository across the six dimensions below. This skill is
self-contained — run it without prior conversation context.

**Repository:** `gtk-cross-platform`
**Stack:** Rust 2024 edition · GTK4 0.9 · libadwaita 0.7 · glib/gio 0.20 · gettext-rs 0.7
**Project type:** GNOME desktop application (Flatpak), hexagonal architecture, cross-platform
target (Linux, macOS, Windows, GNOME Mobile)

---

## What to read before auditing

Read the following files in full before issuing any diagnosis:

- `CLAUDE.md` — architecture rules, threading, HIG, i18n, breakpoints
- `Cargo.toml` and `Cargo.lock` — actual dependency versions
- `com.example.GtkCrossPlatform.json` — Flatpak manifest (SDK, sandbox permissions)
- `.github/workflows/ci.yml` — lint and unit test pipeline
- `.github/workflows/flatpak.yml` — Flatpak build + nightly publishing
- `.github/workflows/editorconfig.yml` — `.editorconfig` validation
- `Makefile` — local build/test/lint/flatpak targets
- `meson.build` and `meson_options.txt` — alternative build system (legacy)
- `README.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CHANGELOG.md`
- `data/resources/window.ui` — breakpoints and widget templates
- `data/resources/style.css` — touch targets and CSS classes
- `tests/` — `container_driver_test.rs`, `greet_use_case_test.rs`, `widget_test.rs`, `unit/`
- `src/` complete — actual vs. documented architecture

---

## Dimension 1 — Maintainability

**Analyze:**

- `src/` structure — coherence with the hexagonal architecture documented in `CLAUDE.md`:
  `core/` (pure domain), `ports/` (traits), `infrastructure/` (adapters), `window/` (UI)
- Dead files, commented code, `TODO` / `FIXME` not tracked in any issue
- `README.md` — sections present: Overview, Quickstart, Development, Architecture, Contributing,
  Versioning, License; CI badges; documented `make` commands
- `CONTRIBUTING.md` — contribution guide updated with PR flow, commit style,
  local build requirements
- `CODE_OF_CONDUCT.md`, `CHANGELOG.md`, `AUTHORS`, `GOVERNANCE.md` — presence and quality
- Rustdoc (`///`) on public types in `src/ports/` and `src/core/domain/`; absence of
  unnecessary long blocks in internal code
- Tests — location in `tests/` (`container_driver_test.rs`, `greet_use_case_test.rs`,
  `widget_test.rs`) and inline `#[cfg(test)]` modules in `src/core/`; descriptive naming;
  Arrange-Act-Assert pattern; coverage of all `IContainerDriver` methods via `MockContainerDriver`

**Expected best practices:**

- `README.md` with architecture section mentioning `AdwNavigationSplitView`, `AdwBreakpoint`,
  `GestureLongPress`, `spawn_driver_task`, and the four runtime adapters
- `CHANGELOG.md` following [Keep a Changelog](https://keepachangelog.com/) with per-version entries
- All public types in `ports/` documented with `///`; no `println!` or `dbg!` in production code
- `tests/container_driver_test.rs` covering all `IContainerDriver` methods via mock

---

## Dimension 2 — CI/CD

**Analyze:**

- `.github/workflows/ci.yml` — steps present: `cargo fmt --check`, `cargo clippy -D warnings`,
  `cargo test --lib`; absence of explicit `permissions:` block; absence of Cargo dependency cache
  (`Swatinem/rust-cache` or `actions/cache`); integration tests in `tests/` not run
  (require Wayland/X11 display)
- `.github/workflows/flatpak.yml` — triggers: `push` (main) and `pull_request` (main); build via
  `flatpak/flatpak-github-actions/flatpak-builder@v6`; generation of `cargo-sources.json` with
  `flatpak-cargo-generator.py` for offline build; nightly release publishing via `gh release`
  only on push to main (`Publish Nightly` step); **no** aarch64 cross-compilation in
  the workflow (available locally only via `make flatpak-build-arm`); no explicit Cargo cache
  (only flatpak-builder cache via `cache-key`)
- `.github/workflows/editorconfig.yml` — `.editorconfig` validation
- Absence of `cargo audit` in any workflow
- Absence of Rust version matrix (`stable`, `beta`) or platform matrix in workflows
- Absence of `workflow_dispatch` in workflows for manual builds
- Absence of `.github/dependabot.yml` for automatic crate and Actions updates

**Expected best practices:**

- Complete pipeline: `fmt → clippy → test → flatpak-build → release`
- Cargo cache in `ci.yml` via `Swatinem/rust-cache` (reduces lint time from ~3 min to ~30 s)
- `permissions: contents: read` as default in `ci.yml`; `contents: write` only in the release
  job in `flatpak.yml` (already present)
- `workflow_dispatch` added to `flatpak.yml` for manual builds without push
- Separate job for GTK widget tests (`widget_test.rs` marked `#[ignore]`) with Xvfb virtual display
- `cargo audit` as mandatory step in `ci.yml`, failing on HIGH/CRITICAL vulnerabilities
- Dependabot or Renovate configured for `cargo` and `github-actions`

---

## Dimension 3 — Versioning

**Analyze:**

- Commit convention in git history — Conventional Commits (`feat:`, `fix:`, `chore:`,
  `BREAKING CHANGE:`)?
- Version alignment: `Cargo.toml [package].version` ↔ git tag ↔ `CHANGELOG.md` ↔
  `com.example.GtkCrossPlatform.json` (`<release>` in metainfo)
- `.gitignore` — adequate for Rust (`target/`), GTK (`_build/`, `build/`), IDEs (.iml, .idea,
  .vscode), macOS (`.DS_Store`), environment variables (`.env`)
- `.gitattributes` — present; normalizes line endings (`* text=auto eol=lf`); marks
  `*.flatpak` and `*.gresource` as binary; marks `Cargo.lock` as `merge=text`; marks
  `po/*.po` for GitHub Linguist; **missing**: `diff=po` attribute for readable `.po` diffs
- Semantic tags (`v0.1.0`) associated with GitHub Releases

**Expected best practices:**

- Conventional Commits adopted and documented in `CONTRIBUTING.md`
- `Cargo.toml` version bumped on each release; `CHANGELOG.md` updated in the same commit
- `.gitignore` covering `target/`, `_build/`, generated `*.gresource`, `.env`, `*.swp`
- `.gitattributes` with `po/*.po diff=po` added
- No compiled artifacts (`.gresource`, binaries) in git history

---

## Dimension 4 — Distribution

**Analyze:**

- `com.example.GtkCrossPlatform.json` — Flatpak manifest: pinned SDK (`org.gnome.Sdk//48`),
  pinned runtime, offline dependencies via `cargo-sources.json`, sandbox permissions (Wayland,
  X11, IPC; no unnecessary `--device=dri`)
- App ID `com.example.GtkCrossPlatform` — placeholder; must be replaced with a real ID before
  publishing to Flathub
- `data/com.example.GtkCrossPlatform.metainfo.xml` — AppStream metainfo: valid, with
  updated `<release>`, screenshots, description, GNOME categories
- `data/com.example.GtkCrossPlatform.desktop` — desktop entry: categories, `TryExec`,
  `StartupWMClass`
- GitHub Releases — Flatpak bundle published automatically as nightly release on each push
  to main; `nightly` tag recreated on each build (not versioned)
- aarch64 cross-compilation (GNOME Mobile / PinePhone) available locally only
  via `make flatpak-build-arm` — **not** automated in CI
- Makefile distribution targets: `flatpak-build`, `flatpak-run`, `flatpak-install`,
  `flatpak-build-arm`
- Meson build system (`meson.build` + `meson_options.txt`) present but apparently
  legacy — primary build uses Cargo; analyze whether it is synced or can be removed

**Expected best practices:**

- Real App ID (reverse-domain) before any Flathub publishing
- `metainfo.xml` validated with `appstream-util validate` in CI
- `desktop` file validated with `desktop-file-validate` in CI
- Flatpak bundle attached to GitHub Release with versioned name (`gtk-cross-platform-v0.1.0.flatpak`)
  in semantic releases; nightly release kept separate
- aarch64 job in `flatpak.yml` via `flatpak-github-actions` for GNOME Mobile builds
- Clarify role of `meson.build` or remove it if dead code

---

## Dimension 5 — Security

**Analyze:**

- `cargo audit` — absent from workflows
- Dependencies with known vulnerabilities (infer from `Cargo.lock` + current date)
- Flatpak sandbox permissions in manifest — principle of least privilege: no
  `--filesystem=home`, no `--device=all`, no unnecessary `--share=network`
- `SECURITY.md` — responsible disclosure policy present with contact
- Hardcoded secrets in workflows (tokens, deploy credentials) — verify
- `.github/dependabot.yml` — absent; no automatic Actions and Cargo crate updates
- Secret scanning in CI (`gitleaks`, `trufflehog`, or GitHub Secret Scanning)
- Workflow permissions: `ci.yml` without explicit `permissions:` block; `flatpak.yml` with
  `permissions: contents: write` at job level (correct for release)

**Expected best practices:**

- `cargo audit` as mandatory CI step, failing on HIGH/CRITICAL vulnerabilities
- Flatpak sandbox without permissions beyond Wayland, X11 fallback, IPC
- `SECURITY.md` with response deadline and report channel (email or GitHub Security Advisory)
- Dependabot configured for `cargo` and `github-actions` in `.github/dependabot.yml`
- Explicit `permissions: contents: read` in `ci.yml` (principle of least privilege)
- No hardcoded secrets; all tokens via `secrets.*` in workflows

---

## Dimension 6 — Tooling Configuration

**Analyze:**

- `.editorconfig` — present; covers `indent_style`, `indent_size`, `end_of_line`,
  `insert_final_newline`, `trim_trailing_whitespace` for `.rs`, `.toml`, `.yml`, `.md`, `.ui`
- `.gitattributes` — present; normalizes EOL, marks binaries and `Cargo.lock`; missing `diff=po`
- `rustfmt.toml` — absent; no explicit formatting configuration beyond default `cargo fmt`
- Clippy lints — verify whether `#![deny(clippy::all)]` or `[lints]` is in `Cargo.toml`; look for
  silent unjustified `#[allow(clippy::...)]`
- Pre-commit hooks — `pre-commit` framework or scripts in `.git/hooks/` for `cargo fmt` and
  `cargo clippy` locally — verify presence
- `Makefile` — well-defined targets (`setup`, `build`, `run`, `test`, `lint`, `fmt`, `fmt-fix`,
  `run-mobile`, `flatpak-build`, `flatpak-run`, `flatpak-install`, `flatpak-build-arm`,
  `setup-macos`, `setup-windows`); absence of `make help` with one-line descriptions
- Build environment variables — `APP_ID`, `PROFILE`, `PKGDATADIR`, `LOCALEDIR` defined
  in `build.rs`; no `.env` with real values in repository
- `meson.build` — present at root but apparently legacy (primary build is Cargo); analyze
  sync with `Cargo.toml`

**Expected best practices:**

- `.editorconfig` validated in CI (workflow `editorconfig.yml` already present — verify covered rules)
- `rustfmt.toml` with minimal explicit configuration (`edition = "2024"`)
- Clippy lints centralized in `Cargo.toml [lints]` (Rust 1.73+), not scattered per file
- `make help` prints all targets with a one-line description
- `po/*.po diff=po` added to `.gitattributes`
- No real `.env` in repository; required variables documented in `CONTRIBUTING.md`

---

## Delivery format

```
# Audit: gtk-cross-platform

## Executive Summary
<paragraph with overall diagnosis and change priority>

## Scorecard
| Dimension       | Score (0–10) | Status    |
|-----------------|--------------|-----------|
| Maintainability | X            | 🔴/🟡/🟢 |
| CI/CD           | X            | 🔴/🟡/🟢 |
| Versioning      | X            | 🔴/🟡/🟢 |
| Distribution    | X            | 🔴/🟡/🟢 |
| Security        | X            | 🔴/🟡/🟢 |
| Tooling         | X            | 🔴/🟡/🟢 |

## Analysis by Dimension
<for each dimension: Current State → Gaps → Recommendations>

## Restructuring Plan

### Quick Wins (1–3 days)
<list of high-impact, low-effort changes>

### Medium-Term Improvements (1–4 weeks)
<prioritized list>

### Structural Initiatives (1–3 months)
<list with dependencies between items>

## Files to Create / Modify
| File                                 | Action                     | Reason                              |
|--------------------------------------|----------------------------|-------------------------------------|
| .github/dependabot.yml               | create                     | automatic crate updates             |
| rustfmt.toml                         | create                     | explicit formatting configuration   |
| ...                                  | ...                        | ...                                 |
```

---

## Constraints

- Base all recommendations on what is **observable in the repository**; do not assume what is
  not present
- When a file is inaccessible (e.g., real secrets), explicitly state the analysis is partial
- Prioritize recommendations by: **security > CI/CD > distribution > maintainability**
- Use status emojis only in the scorecard; keep the rest technical and direct
- Do not repeat gaps already covered by `/verify:compliance-audit` (i18n, A11Y, breakpoints,
  threading, hexagonal architecture) — focus on the GitHub repository aspects of this audit
