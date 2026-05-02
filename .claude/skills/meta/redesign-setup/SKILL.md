---
name: meta:redesign-setup
version: 1.0.0
description: Survey the current Claude command set and project quality standards,
    then generate an improved command set covering the complete FLOSS
    development lifecycle from environment setup through distribution.
---

# meta:redesign-setup

Audit the existing Claude slash-command set against FLOSS best practices, survey the
project's architecture rules and quality standards, identify lifecycle gaps, and generate
the missing command files so that AI-assisted development is safe, predictable, and
auditable end-to-end.

## Global guardrails (apply for the entire execution)

- **Never use placeholders** — every path, function, and module name in the generated
  commands must come from what was found in Phases 1–2. No "your-use-case", no "TODO".
- **Never move files between layers** without explicit user confirmation.
- **All file paths** are relative to the repository root (no `~/` or `../`).
- **Threading**: any I/O reference in a generated command must name `spawn_driver_task`
  and explicitly prohibit `tokio` and `std::sync::mpsc`.
- **i18n**: every user-visible string requirement must cite `gettext!`, `pgettext!`, or
  `ngettext!` by name — never "wrap in a translation function".
- **A11Y**: every icon-only button requirement must cite both `set_tooltip_text` and
  `update_property(&[gtk4::accessible::Property::Label(...)])`.
- **Architecture**: any new file under `src/infrastructure/` must carry the guardrail
  "zero `gtk4` / `adw` imports — this file must not import any GTK type".
- **`cargo check` after every generated file** — stop and report the error if it fails.
- **Exit criteria in every command** — minimum 8 checkboxes covering: build, test, lint,
  and the command's specific guardrail.

---

## Phase 1 — Command-set survey (read-only)

Read every file in `.claude/commands/` and the Slash Commands table in `CLAUDE.md`.

For each command file extract:

- The `description:` line from frontmatter
- Phase structure: number of phases and whether each has a stop-and-wait gate
- Exit-criteria: count and form (numbered checklist vs prose)
- Guardrails present: A11Y, i18n, threading, architecture, none
- Whether it references real file paths or uses abstract layer names
- Whether it accepts `$ARGUMENTS`

Then print the following structured report. Fill every cell with real data — no guessing.

```
=== COMMAND-SET SURVEY ===

Existing commands (<N> total):

  <command-filename>
    Description:      <frontmatter description>
    Phases / gates:   <e.g. "3 phases, gate after phase 1">
    Exit criteria:    <N checklist items>
    Guardrails:       <comma-separated: A11Y | i18n | threading | architecture | none>
    Real file paths:  <Yes / No>
    Accepts $ARGS:    <Yes / No>

  (repeat for each command)

Strengths of the current set:
  + <list observed strengths — only if genuinely present>

Gaps — lifecycle stages with no covering command:
  1.  [CI]          No command to add CI quality gates (coverage, cargo-audit, Xvfb widget tests)
  2.  [I18N]        No command to audit i18n completeness (all user strings wrapped in gettext)
  3.  [RELEASE]     No command for release preparation (CHANGELOG, version bump, signed tag)
  4.  [A11Y]        No command for accessibility audit (walk all interactive widgets)
  5.  [USE-CASE]    No command for adding domain use cases with full hexagonal wiring
  6.  [BOUNDARY]    No command for detecting hexagonal layer boundary violations
  7.  [SECURITY]    No cargo-audit / cargo-deny in any CI workflow
  8.  [COVERAGE]    No code coverage reporting (tarpaulin / llvm-cov)
  9.  [WIDGET-CI]   Widget tests (tests/widget_test.rs) not run in CI — need Xvfb
  10. [MULTI-ARCH]  No macOS / Windows / ARM matrix in GitHub Actions
  11. [HOOKS]       Pre-commit hooks not documented or enforced
  12. [CHANGELOG]   No CHANGELOG automation from Conventional Commits
  13. [LICENSE]     No dependency license audit (cargo-deny [licenses])
  14. [ERROR-PATHS] No driver-specific error-path tests (mock HTTP 500/timeout/malformed JSON)

  (remove any gap that already has coverage; add gaps discovered during reading)

Config / docs issues found:
  - <file>: <description of issue>
  (examples found in real projects:
    Cargo.toml: edition = "2024" — should be "2021"
    po/POTFILES: stale Vala entries — project is Rust
    data/com.example.GtkCrossPlatform.metainfo.xml: homepage = https://example.com placeholder
    No VISION.md despite being listed as scaffold-oss-docs output
    No clippy.toml, rustfmt.toml, deny.toml, .github/CODEOWNERS)

New commands to generate:
  add-use-case.md
  audit-layer-boundaries.md
  audit-i18n.md
  audit-a11y.md
  add-ci-quality-gates.md
  prepare-release.md

Commands to update with missing sections:
  add-runtime-driver.md     — add error-path test section (mock HTTP 500/timeout)
  implement-container-ui.md — add error-path test requirements for views
  scaffold-oss-docs.md      — add CODEOWNERS, deny.toml, clippy.toml to generated file list

Proceed? (stop here — wait for user confirmation before Phase 2)
```

---

## Phase 2 — Architecture and quality standards survey (read-only)

Gather the technical facts that will be embedded as guardrails in the generated commands.
Read each file listed below — do not guess values; read them.

Files to read:

- `Cargo.toml` — dependency versions, edition field, bin/lib structure
- `CLAUDE.md` — layer table, Threading section, i18n pipeline, A11Y requirements, breakpoints
- `.github/workflows/ci.yml` — existing CI jobs, triggers, missing gates
- `.github/workflows/flatpak.yml` — Flatpak runtime version, published bundle name
- `src/ports/i_container_driver.rs` — full trait surface (method names by category)
- `src/core/use_cases/greet_use_case.rs` — existing use-case struct/impl/test pattern
- `tests/container_driver_test.rs` — MockContainerDriver usage pattern and assertion style
- `po/POTFILES` — current source file list (detect stale non-Rust entries)
- `data/com.example.GtkCrossPlatform.metainfo.xml` — detect placeholder URL and release entry

Then print:

```
=== ARCHITECTURE SURVEY ===

Project: <name> | App ID: <app-id> | License: <license>
GTK stack: gtk4 = <version> (features: <features>), libadwaita = <version>, glib = <version>
Rust edition: <edition>  <"WARN: should be 2021" if edition != "2021">
GNOME Platform runtime: <version from flatpak manifest>

Layer rules (from CLAUDE.md):
  src/core/           — <rule>
  src/ports/          — <rule>
  src/infrastructure/ — <rule>
  src/window/         — <rule>
  src/app.rs          — <composition root rule>

Threading invariant:
  <exact rule from CLAUDE.md — spawn_driver_task, async_channel, glib::spawn_local>
  Only <path> may call spawn_driver_task.

i18n pipeline:
  <gettext!/pgettext!/ngettext! — cite exact macro names>
  po/POTFILES status: <N entries; stale entries: list them>
  Active locales (po/LINGUAS): <list>

A11Y requirements:
  <exact requirements from CLAUDE.md — cite Property::Label, AccessibleRole, touch targets>

IContainerDriver surface (src/ports/i_container_driver.rs):
  <N methods across categories: runtime identity, containers, images, volumes, networks, system>

Use-case pattern (src/core/use_cases/greet_use_case.rs):
  <describe the struct/impl/cfg(test) pattern in one sentence>

MockContainerDriver test pattern (tests/container_driver_test.rs):
  <describe the factory + call + assert pattern in one sentence>

CI gaps (from .github/workflows/ci.yml):
  - <list each missing job: security, coverage, widget-tests, deny, multi-platform>

Files to create alongside new commands:
  deny.toml       — cargo-deny configuration (licenses + bans)
  clippy.toml     — clippy lint configuration
  rustfmt.toml    — formatting configuration
  .github/CODEOWNERS — automatic review routing

Real paths confirmed for generated commands:
  src/core/use_cases/         src/ports/                  src/infrastructure/containers/
  src/window/views/           .github/workflows/ci.yml    po/POTFILES
  Cargo.toml                  deny.toml (to create)       .github/CODEOWNERS (to create)

Proceed? (stop here — wait for user confirmation before generating any file)
```

---

## Phase 3 — Generate improved command files

Generate files in this exact order (each `cargo check` must pass before continuing):

1. `.claude/commands/add-use-case.md`
2. `.claude/commands/audit-layer-boundaries.md`
3. `.claude/commands/audit-i18n.md`
4. `.claude/commands/audit-a11y.md`
5. `.claude/commands/add-ci-quality-gates.md`
6. `.claude/commands/prepare-release.md`
7. Update `.claude/commands/add-runtime-driver.md` — append error-path test section
8. Update `.claude/commands/implement-container-ui.md` — append error-path view test requirements
9. Update `.claude/commands/scaffold-oss-docs.md` — add CODEOWNERS, deny.toml, clippy.toml to Phase 2

Use exact values discovered in Phase 2 for all dependency versions, file paths, method names,
and CI job structure. Do not use placeholders.

---

### 3.1 — `add-use-case.md`

**Usage:** `/meta:add-use-case <use-case-name>`  
**Example:** `/meta:add-use-case list-running-containers`

**What it does:** Creates a new domain use case following the hexagonal pattern — struct
in `src/core/use_cases/<name>_use_case.rs`, a new port trait in `src/ports/` if the use
case requires an I/O boundary, wiring in `src/app.rs::activate()`, and inline unit tests.

**The command must include these exact steps:**

Step 1 — Read existing pattern.

- Read `src/core/use_cases/greet_use_case.rs` — understand struct, execute(), #[cfg(test)] block.
- Read `src/core/use_cases/mod.rs` — understand how modules are declared.
- Read `src/ports/i_container_driver.rs` — check whether the new use case can reuse an existing port.
- Read `src/app.rs` — understand where Arc<dyn IContainerDriver> is wired.

Step 2 — Create the use-case file.

- Path: `src/core/use_cases/<name>_use_case.rs`
- Struct: `pub struct <Name>UseCase;`
- Constructor: `pub fn new() -> Self`
- Method: `pub fn execute(&self, ...) -> Result<<Output>, <Name>Error>`
- Error type: `#[derive(Debug, thiserror::Error)] pub enum <Name>Error` — in the same file.
- Inline tests (`#[cfg(test)] mod tests`): minimum 3 — happy path, error path, edge case.
- Zero `gtk4` / `adw` / `glib` / `gio` imports — violation is a CI compilation error.

Step 3 — If a new port is needed.

- Path: `src/ports/i_<name>.rs`
- A `pub trait I<Name>` with the minimal surface the use case needs.
- Zero `gtk4` / `adw` / `glib` / `gio` imports.
- Add `pub mod i_<name>;` to `src/ports/mod.rs`.

Step 4 — Register the module.

- Add `pub mod <name>_use_case;` to `src/core/use_cases/mod.rs`.

Step 5 — Wire in the composition root (only if the use case is callable from the UI).

- In `src/app.rs::activate()`: instantiate and pass as `Arc<dyn I<Port>>` — never a concrete type.

Step 6 — Verify.

- Run `cargo check` — must pass with zero errors.
- Run `make test` — all tests including new ones must pass.
- Run `make lint` — zero warnings.

**Exit criteria:**

- [ ] `cargo check` passes with zero errors
- [ ] `make test` passes (all tests including new ones)
- [ ] `make lint` reports zero warnings
- [ ] Use-case file has zero `gtk4` / `adw` / `glib` / `gio` / `std::io` / `std::fs` imports
- [ ] Port trait file (if created) has zero `gtk4` / `adw` / `glib` / `gio` imports
- [ ] Inline `#[cfg(test)]` block has at least 3 tests: happy path, error path, edge case
- [ ] `src/core/use_cases/mod.rs` declares the new module with `pub mod`
- [ ] Wiring in `src/app.rs::activate()` uses `Arc<dyn I<Port>>` — never a concrete adapter type

---

### 3.2 — `audit-layer-boundaries.md`

**Usage:** `/meta:audit-layer-boundaries`

**What it does:** Scans the source tree for hexagonal architecture violations using
grep, prints a violation report per layer rule, and (after user confirmation) fixes
each violation with minimal targeted edits.

**The command must include three phases:**

**Phase 1 — Scan (read-only). Run these exact grep commands:**

```bash
# Rule 1: src/core/ must have zero GTK/Adw/GLib/IO imports
grep -rn "use gtk4\|use adw\|use glib\|use gio\|use std::io\|use std::fs\|use std::net" src/core/

# Rule 2: src/ports/ must have zero GTK/GLib/IO imports
grep -rn "use gtk4\|use adw\|use glib\|use gio\|use std::io\|use std::fs" src/ports/

# Rule 3: src/infrastructure/ must have zero GTK/Adw imports
grep -rn "use gtk4\|use adw" src/infrastructure/

# Rule 4: spawn_driver_task must only be called from src/window/views/
grep -rn "spawn_driver_task" src/window/ | grep -v "src/window/views/"

# Rule 5: Concrete driver types only in src/infrastructure/ and src/app.rs
grep -rn "DockerDriver\|PodmanDriver\|ContainerdDriver\|MockContainerDriver" src/ \
  | grep -v "src/infrastructure/\|src/app.rs\|#\[cfg(test)\]"

# Rule 6: ContainerDriverFactory only instantiated in src/app.rs
grep -rn "ContainerDriverFactory::detect\|ContainerDriverFactory::with_runtime\|ContainerDriverFactory::new" src/ \
  | grep -v "src/app.rs\|src/infrastructure/containers/factory.rs"
```

Print the report:

```
=== LAYER BOUNDARY AUDIT ===

Rule 1 — src/core/ zero GTK/Adw/GLib/IO:          PASS / FAIL (N violations)
Rule 2 — src/ports/ zero GTK/Adw/GLib/IO:          PASS / FAIL
Rule 3 — src/infrastructure/ zero GTK/Adw:          PASS / FAIL
Rule 4 — spawn_driver_task only in views/:           PASS / FAIL
Rule 5 — Concrete drivers only in infra + app.rs:   PASS / FAIL
Rule 6 — ContainerDriverFactory only in app.rs:     PASS / FAIL

Violations:
  <path>:<line>  RULE-<N>  <the offending import or reference>

Proposed fixes:
  <path>:<line> — <specific change: remove import / move type / use re-export>

Total violations: <N>
If zero violations: report CLEAN and skip Phase 2.
Proceed with fixes? (stop — wait for user confirmation)
```

**Phase 2 — Fix each violation.** Minimal targeted edits — do not restructure files.
If fixing a violation requires moving a type between layers, stop and ask the user.
Run `cargo check` after each file edit.

**Phase 3 — Rescan.** Re-run all 6 grep commands. Every rule must show PASS.
Run `make test` and `make lint`.

**Exit criteria:**

- [ ] All 6 grep rules return zero matches
- [ ] `cargo check` passes
- [ ] `make test` passes (all existing tests continue to pass)
- [ ] `make lint` reports zero warnings
- [ ] No file moved between layers without explicit user confirmation
- [ ] Every violation found in Phase 1 is listed in the final report with its fix
- [ ] If zero violations found in Phase 1, Phase 2 is explicitly skipped and logged
- [ ] Final rescan report shows CLEAN for all 6 rules

---

### 3.3 — `audit-i18n.md`

**Usage:** `/meta:audit-i18n`

**What it does:** Scans `src/window/` for hardcoded user-visible strings not wrapped
in `gettext!`, `pgettext!`, or `ngettext!`; audits `po/POTFILES` for stale entries
and missing source files; checks for bare plural constructions; then fixes all issues.

**The command must include three phases:**

**Phase 1 — Scan (read-only). Run these exact commands:**

```bash
# 1a. Hardcoded strings in UI layer (capitalized English literals not in a macro)
grep -rn '"[A-Z][a-zA-Z ]' src/window/ \
  | grep -v 'gettext!\|pgettext!\|ngettext!\|//\|#\[' \
  | grep -v '\.css\|\.ui\|set_icon_name\|set_widget_name\|set_name\|app_id\|css_class'

# 1b. Stale non-Rust entries in po/POTFILES
grep -E '\.(vala|js|py|c|h)$' po/POTFILES

# 1c. Rust files under src/window/ not listed in po/POTFILES
for f in $(find src/window/ -name "*.rs" | sort); do
  grep -qF "$f" po/POTFILES || echo "MISSING FROM POTFILES: $f"
done

# 1d. Plural constructions not using ngettext (bare string interpolation with counts)
grep -rn 'format!.*containers\|format!.*images\|format!.*volumes\|format!.*networks' src/window/ \
  | grep -v ngettext
```

Print the report:

```
=== I18N AUDIT ===

Hardcoded strings in src/window/ (not in gettext macro):
  <path>:<line>  "<string>"

Stale entries in po/POTFILES:
  <path>  — <reason: Vala / JS / Python / not found in tree>

Rust files missing from po/POTFILES:
  <path>

Plural constructions missing ngettext:
  <path>:<line>  <format! call>

Summary: <N> hardcoded | <N> stale POTFILES | <N> missing POTFILES | <N> bad plurals
Proceed with fixes? (stop — wait for user confirmation)
```

**Phase 2 — Fix each issue:**

- Wrap each hardcoded string: `gettext!("…")` for standalone, `pgettext!("context", "…")` when
  the same English word has different meanings in different contexts, `ngettext!("1 x", "{n} xs", n)`
  for counts.
- Remove stale entries from `po/POTFILES`.
- Add missing `.rs` file paths to `po/POTFILES` (one path per line, relative to repo root).
- Replace bare `format!("… {n} containers …")` with `ngettext!("1 container", "{n} containers", n)`.
- Run `cargo check` after each file change.

**Phase 3 — Validate:**

- Re-run all 4 scan commands — each must return zero output.
- Run `make test` and `make lint`.
- If `msgfmt` is available: run `msgfmt --check po/pt_BR.po` — must pass.

**Exit criteria:**

- [ ] All 4 scan commands return zero output
- [ ] `cargo check` passes
- [ ] `make test` passes
- [ ] `make lint` reports zero warnings
- [ ] `po/POTFILES` contains every `.rs` file under `src/window/` and `src/app.rs`
- [ ] `po/POTFILES` has zero non-Rust entries (no `.vala`, `.js`, `.py`, `.c`, `.h`)
- [ ] Every count string uses `ngettext!` — no bare `format!("{n} containers")`
- [ ] `msgfmt --check po/pt_BR.po` passes (if msgfmt is installed)

---

### 3.4 — `audit-a11y.md`

**Usage:** `/meta:audit-a11y`

**What it does:** Scans `src/window/` for accessibility violations: icon-only buttons
without tooltip and accessible label, widgets without proper accessible roles,
touch targets smaller than 44 sp, and missing focus management after destructive
actions and dialog closes.

**The command must include three phases:**

**Phase 1 — Scan (read-only). Run these exact commands:**

```bash
# 1a. Icon buttons created without set_tooltip_text
grep -rn "set_icon_name\|icon_name(" src/window/ | grep -v "//\|set_tooltip\|tooltip_text"

# 1b. Buttons missing update_property(Property::Label)
grep -rn "set_tooltip_text\|set_tooltip" src/window/ \
  | grep -v "update_property\|Property::Label"

# 1c. Explicit size_request smaller than 44 (accessible minimum)
grep -rn "set_size_request" src/window/ \
  | grep -vE "set_size_request\(-?1|set_size_request\(4[4-9]|set_size_request\([5-9][0-9]"

# 1d. StatusBadge usage — check that accessible role is set
grep -rn "StatusBadge::new\|set_status" src/window/

# 1e. Destructive actions missing focus management
grep -rn "remove_container\|stop_container\|delete_volume\|remove_image\|remove_network\|prune" \
  src/window/ | grep -v "grab_focus\|//\|spawn_driver_task"

# 1f. ConfirmDialog (or AlertDialog) usage missing focus return
grep -rn "ConfirmDialog::ask\|adw::AlertDialog\|adw::MessageDialog" src/window/ \
  | grep -v "grab_focus\|//"
```

Print the report:

```
=== A11Y AUDIT ===

Icon buttons missing tooltip:
  <path>:<line>  <code excerpt>

Buttons with tooltip but missing Property::Label:
  <path>:<line>  <code excerpt>

Explicit size_request below 44 sp:
  <path>:<line>  set_size_request(<W>, <H>)

StatusBadge instances to verify role:
  <path>:<line>  <code excerpt>

Destructive actions missing grab_focus after operation:
  <path>:<line>  <code excerpt>

Dialogs missing focus return to trigger widget:
  <path>:<line>  <code excerpt>

Summary: <N> tooltip-missing | <N> label-missing | <N> size | <N> focus-action | <N> focus-dialog
Proceed with fixes? (stop — wait for user confirmation)
```

**Phase 2 — Fix each issue:**

- Icon-only buttons: add `button.set_tooltip_text(gettext!("…"))` immediately after creation,
  then `button.update_property(&[gtk4::accessible::Property::Label(gettext!("…").into())])`.
- Touch targets: `button.set_size_request(44, 44)` minimum (or remove explicit undersized request).
- Destructive actions: after the driver call completes in the callback, call
  `list.select_row(next_row.as_ref())` then `next_row.grab_focus()`, or focus the empty-state
  widget if the list is now empty.
- Dialog focus return: capture the trigger widget before `ConfirmDialog::ask`, then after
  it resolves call `trigger.grab_focus()`.
- Run `cargo check` after each file change.

**Phase 3 — Validate:**

- Re-run all 6 scan commands — each must return zero relevant output.
- Run `make test` and `make lint`.

**Exit criteria:**

- [ ] All icon-only buttons have `set_tooltip_text` + `update_property(Property::Label(…))`
- [ ] No explicit `set_size_request` below 44 sp on interactive elements
- [ ] `StatusBadge` sets `set_accessible_role(gtk4::AccessibleRole::Status)` on construction
- [ ] After every destructive action callback: focus moves to next row or empty-state widget
- [ ] After every `ConfirmDialog::ask` / `AlertDialog`: focus returns to the trigger widget
- [ ] `cargo check` passes
- [ ] `make test` passes
- [ ] `make lint` reports zero warnings
- [ ] `make fmt` shows no diff
- [ ] `tests/widget_test.rs` has at least one test per component that verifies its accessible role

---

### 3.5 — `add-ci-quality-gates.md`

**Usage:** `/meta:add-ci-quality-gates`

**What it does:** Adds security scanning (`cargo-audit`), dependency license checking
(`cargo-deny`), code coverage (`cargo-tarpaulin`), and GTK widget tests via Xvfb to the
existing GitHub Actions CI workflow. Creates `deny.toml` with a license allowlist and
a ban on `tokio`.

**The command must include three phases:**

**Phase 1 — Audit current CI (read-only).**

Read `.github/workflows/ci.yml` and run:

```bash
cargo audit --version 2>/dev/null || echo "cargo-audit: NOT INSTALLED locally"
cargo deny  --version 2>/dev/null || echo "cargo-deny: NOT INSTALLED locally"
cargo tarpaulin --version 2>/dev/null || echo "cargo-tarpaulin: NOT INSTALLED locally"
which xvfb-run 2>/dev/null || echo "xvfb-run: NOT FOUND (apt: xvfb)"
```

Print the report:

```
=== CI AUDIT ===

Current jobs in .github/workflows/ci.yml:
  <list each job name and its steps>

Missing quality gates:
  [ ] cargo-audit (CVE scanning)
  [ ] cargo-deny (license + banned crate check)
  [ ] cargo-tarpaulin (code coverage)
  [ ] Xvfb widget tests (tests/widget_test.rs with --ignored)
  [ ] macOS build matrix
  [ ] Windows build matrix

Local tool status:
  cargo-audit:    <installed version / NOT INSTALLED>
  cargo-deny:     <installed version / NOT INSTALLED>
  cargo-tarpaulin:<installed version / NOT INSTALLED>
  xvfb-run:       <path / NOT FOUND>

Files to create: deny.toml
Proceed? (stop — wait for user confirmation)
```

**Phase 2 — Update CI and create config files.**

Add these jobs to `.github/workflows/ci.yml` (after the existing lint job):

```yaml
  security:
      name: Security audit
      runs-on: ubuntu-latest
      steps:
          -   uses: actions/checkout@v4
          -   uses: rustsec/audit-check@v2
              with:
                  token: ${{ secrets.GITHUB_TOKEN }}

  deny:
      name: Dependency license check
      runs-on: ubuntu-latest
      steps:
          -   uses: actions/checkout@v4
          -   uses: dtolnay/rust-toolchain@stable
          -   uses: EmbarkStudios/cargo-deny-action@v2

  coverage:
      name: Code coverage
      runs-on: ubuntu-latest
      steps:
          -   uses: actions/checkout@v4
          -   uses: dtolnay/rust-toolchain@stable
          -   name: Install cargo-tarpaulin
              run: cargo install cargo-tarpaulin --locked
          -   name: Generate coverage
              run: cargo tarpaulin --out Lcov --output-dir coverage/ --timeout 120
          -   name: Upload coverage
              uses: coverallsapp/github-action@v2
              with:
                  github-token: ${{ secrets.GITHUB_TOKEN }}
                  path-to-lcov: coverage/lcov.info

  widget-tests:
      name: GTK widget tests (Xvfb)
      runs-on: ubuntu-latest
      steps:
          -   uses: actions/checkout@v4
          -   name: Install GTK4 and Xvfb
              run: |
                  sudo apt-get update -qq
                  sudo apt-get install -y xvfb libgtk-4-dev libadwaita-1-dev
          -   uses: dtolnay/rust-toolchain@stable
          -   name: Run widget tests
              run: xvfb-run -a cargo test --test widget_test -- --ignored --test-threads=1
```

Create `deny.toml`:

```toml
[graph]
targets = []

[licenses]
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "ISC",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "Unicode-DFS-2016",
    "LGPL-2.1-or-later",
    "GPL-3.0-or-later",
    "CC0-1.0",
    "OpenSSL",
]
confidence-threshold = 0.8

[bans]
multiple-versions = "warn"
deny = [
    { name = "tokio", reason = "conflicts with the GLib event loop — use async-channel + glib::spawn_local instead" },
    { name = "openssl-sys", reason = "prefer rustls or native-tls; openssl adds build complexity and CVE surface" },
]

[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"
notice = "warn"
```

Run `cargo check` after updating `.github/workflows/ci.yml`.
If `cargo-deny` is installed locally: run `cargo deny check` to validate `deny.toml`.

**Phase 3 — Validate.**

- Run `make test` and `make lint`.
- If `cargo-deny` is installed: run `cargo deny check` — must pass.
- If `cargo-audit` is installed: run `cargo audit` — must show zero vulnerabilities.

**Exit criteria:**

- [ ] `.github/workflows/ci.yml` has `security`, `deny`, `coverage`, and `widget-tests` jobs
- [ ] `deny.toml` exists with `[licenses]` allowlist and `tokio` in `[bans]`
- [ ] `cargo check` passes
- [ ] `make test` passes
- [ ] `make lint` reports zero warnings
- [ ] `cargo deny check` passes (if cargo-deny installed locally)
- [ ] `cargo audit` reports zero vulnerabilities (if cargo-audit installed locally)
- [ ] CI YAML is valid (no syntax errors — check with
  `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"`)
- [ ] Coverage job references the correct lcov output path
- [ ] Widget-test job installs GTK4 system dependencies before running

---

### 3.6 — `prepare-release.md`

**Usage:** `/meta:prepare-release <version>`  
**Example:** `/meta:prepare-release 0.2.0`

**What it does:** Audits repository state, generates CHANGELOG entries from Conventional
Commits, bumps version in `Cargo.toml` and AppStream metainfo, creates a release commit,
then (after explicit user confirmation) creates a signed tag.

**The command must include three phases:**

**Phase 1 — Audit current state (read-only).**

```bash
# Commits since last tag (grouped by Conventional Commits type)
git log "$(git describe --tags --abbrev=0 2>/dev/null || git rev-list --max-parents=0 HEAD)"..HEAD \
  --pretty=format:"%s" | sort

# Uncommitted changes (release requires a clean tree)
git status --short

# Current version
grep '^version' Cargo.toml

# CHANGELOG [Unreleased] section
head -30 CHANGELOG.md

# AppStream placeholder check
grep -n "example\.com\|TODO\|PLACEHOLDER" data/com.example.GtkCrossPlatform.metainfo.xml

# CVE check (if cargo-audit is installed)
cargo audit 2>/dev/null || echo "cargo-audit not installed — run manually before tagging"
```

Print the report:

```
=== RELEASE AUDIT ===

Current version: <X.Y.Z from Cargo.toml>
Target version:  $ARGUMENTS

Commits since last tag (<N> total):
  feat:     <list>
  fix:      <list>
  i18n:     <list>
  a11y:     <list>
  chore:    <list>
  (others): <list>

Working tree: <CLEAN / <N> uncommitted files — release requires clean tree>
CHANGELOG [Unreleased] section: <present with content / empty / missing>
AppStream placeholder URLs: <NONE / list of lines>
CVEs: <none / list>

Version bump plan:
  Cargo.toml version: <current> → $ARGUMENTS
  metainfo.xml <release>: <current> → $ARGUMENTS  date: <today YYYY-MM-DD>
  CHANGELOG: [Unreleased] → [$ARGUMENTS] - <today YYYY-MM-DD>

Proceed with release preparation? (stop — wait for user confirmation)
```

Stop if: working tree is not clean, target version is not valid semver, or target
version is lower than the current version.

**Phase 2 — Prepare the release.**

1. Write CHANGELOG entries into the `[Unreleased]` section (if empty) grouped by:
   `### Added`, `### Fixed`, `### Changed`, `### Security`, `### Internationalization`,
   `### Accessibility`. Use the commit messages found in Phase 1. Do not invent entries.

2. Rename `[Unreleased]` to `[$ARGUMENTS] - <today YYYY-MM-DD>` and add a fresh
   empty `[Unreleased]` section above it.

3. Update `version = "<current>"` → `version = "$ARGUMENTS"` in `Cargo.toml`.

4. Update `<release version="<current>" date="…">` → `<release version="$ARGUMENTS" date="<today>">`
   in `data/com.example.GtkCrossPlatform.metainfo.xml`.

5. Run `cargo check` — must pass.

6. Run `make test && make lint`.

7. Create commit:
   ```
   chore(release): bump version to $ARGUMENTS

   Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
   ```

8. Stop before tagging — print the commit hash and ask for explicit user confirmation
   to proceed with Phase 3.

**Phase 3 — Tag and publish (only after explicit user confirmation).**

```bash
# Prefer signed tag; fall back to annotated tag if GPG is not configured
git tag -s "v$ARGUMENTS" -m "Release v$ARGUMENTS" 2>/dev/null \
  || git tag -a "v$ARGUMENTS" -m "Release v$ARGUMENTS"
```

Then print:

```
Tag v$ARGUMENTS created locally.

To publish:
  git push origin main
  git push origin v$ARGUMENTS

After push: the Flatpak CI workflow (.github/workflows/flatpak.yml) will
automatically build and publish a new nightly bundle. For a stable release,
update the workflow trigger conditions to also fire on version tags.
```

**Exit criteria:**

- [ ] `CHANGELOG.md` has a `[$ARGUMENTS] - <today>` section with at least one entry
- [ ] `CHANGELOG.md` has a fresh empty `[Unreleased]` section above the new release
- [ ] `Cargo.toml` `version` field equals `$ARGUMENTS`
- [ ] `data/com.example.GtkCrossPlatform.metainfo.xml` `<release version>` equals `$ARGUMENTS`
- [ ] `cargo check` passes
- [ ] `make test` passes
- [ ] `make lint` reports zero warnings
- [ ] Release commit exists with message matching `chore(release): bump version to $ARGUMENTS`
- [ ] Tag `v$ARGUMENTS` exists locally
- [ ] No placeholder URLs (`example.com`) in metainfo.xml at time of tagging

---

## Phase 4 — Validate and update CLAUDE.md

After generating all 6 new command files and updating the 3 existing ones:

1. Run `cargo check` — must pass.

2. Verify every file in `.claude/commands/` has a frontmatter `description:` line.
   Print a warning for any file missing it.

3. Update the Slash Commands table in `CLAUDE.md` to include all commands (existing + new):

   | Command | When to use |
            |---------|-------------|
   | `/meta:update-readme` | README or CONTRIBUTING.md needs updating |
   | `/meta:refactor-components` | Decompose `src/window/` into components |
   | `/meta:add-runtime-driver <name>` | Add a new container runtime adapter |
   | `/meta:implement-container-ui` | Implement or overhaul the GTK4/Adwaita UI layer |
   | `/meta:scaffold-oss-docs` | Generate OSS documentation structure |
   | `/meta:update-gitignore` | Audit `.gitignore` against detected stack |
   | `/meta:redesign-setup` | Audit and regenerate the Claude command set |
   | `/meta:add-use-case <name>` | Add a domain use case with hexagonal wiring |
   | `/meta:audit-layer-boundaries` | Detect and fix hexagonal boundary violations |
   | `/meta:audit-i18n` | Audit i18n completeness in `src/window/` |
   | `/meta:audit-a11y` | Audit accessibility compliance in `src/window/` |
   | `/meta:add-ci-quality-gates` | Add security, coverage, and widget-test CI jobs |
   | `/meta:prepare-release <version>` | Prepare CHANGELOG, bump version, create tag |

4. Print the final summary:

```
=== GENERATION COMPLETE ===

New commands created (6):
  ✓ .claude/commands/add-use-case.md
  ✓ .claude/commands/audit-layer-boundaries.md
  ✓ .claude/commands/audit-i18n.md
  ✓ .claude/commands/audit-a11y.md
  ✓ .claude/commands/add-ci-quality-gates.md
  ✓ .claude/commands/prepare-release.md

Commands updated (3):
  ✓ .claude/commands/add-runtime-driver.md       (+ error-path test section)
  ✓ .claude/commands/implement-container-ui.md   (+ error-path view test requirements)
  ✓ .claude/commands/scaffold-oss-docs.md        (+ CODEOWNERS, deny.toml, clippy.toml)

CLAUDE.md Slash Commands table: UPDATED (13 commands total)

Auxiliary files to create (run /meta:add-ci-quality-gates to generate deny.toml):
  deny.toml           — cargo-deny license + bans configuration
  clippy.toml         — clippy lint overrides
  rustfmt.toml        — formatting preferences
  .github/CODEOWNERS  — automatic review routing

FLOSS lifecycle coverage:
  ✓ Build (Cargo + Flatpak)      ✓ Test (unit + integration + widget-CI)
  ✓ Lint (clippy -D warnings)    ✓ Format (rustfmt + EditorConfig)
  ✓ Security (cargo-audit)       ✓ Coverage (tarpaulin / Coveralls)
  ✓ License audit (cargo-deny)   ✓ i18n completeness audit
  ✓ A11Y compliance audit        ✓ Architecture boundary enforcement
  ✓ Commit conventions           ✓ CHANGELOG generation
  ✓ Release preparation          ✓ Distribution (Flatpak nightly + stable)
  ✓ Documentation (CLAUDE.md)    ✓ Community (GOVERNANCE/SECURITY/COC)
```

---

## Quick-reference: which command covers which lifecycle stage

| Lifecycle stage                 | Command                                                  |
|---------------------------------|----------------------------------------------------------|
| Environment setup               | `CLAUDE.md` + `CONTRIBUTING.md` (manual)                 |
| Cargo build                     | `make build` (Makefile)                                  |
| Flatpak build                   | `make flatpak-build` / CI flatpak.yml                    |
| Unit tests                      | `make test`                                              |
| Integration tests               | `make test`                                              |
| GTK widget tests in CI          | `/meta:add-ci-quality-gates`                             |
| Code coverage                   | `/meta:add-ci-quality-gates`                             |
| Lint (clippy)                   | `make lint` / CI ci.yml                                  |
| Format (rustfmt + EditorConfig) | `make fmt` / CI ci.yml / editorconfig.yml                |
| Security audit (CVEs)           | `/meta:add-ci-quality-gates`                             |
| Dependency license audit        | `/meta:add-ci-quality-gates`                             |
| i18n completeness               | `/meta:audit-i18n`                                       |
| A11Y compliance                 | `/meta:audit-a11y`                                       |
| Architecture boundaries         | `/meta:audit-layer-boundaries`                           |
| New domain use case             | `/meta:add-use-case <name>`                              |
| New runtime adapter             | `/meta:add-runtime-driver <name>`                        |
| Conventional Commits            | `CONTRIBUTING.md` (enforced by convention)               |
| CHANGELOG + version bump        | `/meta:prepare-release <version>`                        |
| Signed release tag              | `/meta:prepare-release <version>`                        |
| Flatpak distribution            | CI flatpak.yml (triggers on tag / push to main)          |
| OSS documentation               | `/meta:scaffold-oss-docs`                                |
| README / CONTRIBUTING           | `/meta:update-readme`                                    |
| Community documents             | Maintained manually (GOVERNANCE, SECURITY, COC, AUTHORS) |
