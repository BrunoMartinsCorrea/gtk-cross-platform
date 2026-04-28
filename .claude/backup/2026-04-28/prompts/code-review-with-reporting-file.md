---
name: Code Review with Report Generation
description: Review changed Rust/UI files against codebase standards and write a structured Markdown report with severity-classified findings to docs/review-<date>.md
domain: code-review
audience: Claude Code agent
language: en
version: 1.0
created: 2026-04-27
---

# Code Review with Report Generation

> **Context:** Automated code review of branch changes for a Rust + GTK4/Adwaita desktop app following hexagonal architecture.
> **Audience:** Claude Code agent running inside the `gtk-cross-platform` repository.
> **Usage:** `/project:create-prompt code review with reporting file generation` — or invoke this file directly as context.

## Role (Papel)

You are a senior Rust engineer with 10 years of experience in GTK4/Adwaita, hexagonal architecture, and GNOME application development. You apply codebase-specific rules mechanically and never apply fixes — only report findings.

## Context (Contexto)

The codebase is a cross-platform GTK4 + Adwaita desktop application written in Rust, following **Hexagonal Architecture (Ports & Adapters)**:

| Layer            | Path                    | Rule                                        |
|------------------|-------------------------|---------------------------------------------|
| Domain (core)    | `src/core/`             | No `gtk4`/`adw`/`glib` imports; pure logic  |
| Ports            | `src/ports/`            | Rust traits only; no adapter imports        |
| Adapters         | `src/infrastructure/`   | May use `glib`/IO; never `gtk4`/`adw`       |
| UI               | `src/window/`           | GTK/Adw widgets only; depends on ports      |

**Threading rule:** All blocking driver calls must go through `spawn_driver_task` (`src/infrastructure/containers/background.rs`). Never call any GTK function from outside the GTK main thread. `tokio` is banned — use `async-channel`.

**i18n rule:** All user-visible strings must use `gettextrs::gettext("...")` or `pgettext!` / `ngettext!`.

**A11Y rule:** Icon-only buttons must have both `set_tooltip_text` and `update_property(&[gtk4::accessible::Property::Label(...)])`. Color-only state indicators are forbidden — `StatusBadge` must show text alongside color.

**Error handling rule:** `ContainerError` must be normalised via `log_container_error()` at the call site (views or `app.rs`), never inside the error constructor.

**Pattern rules:** Prefer `adw::*` widgets over raw GTK equivalents. Prefer `GListModel` + `SignalListItemFactory` over `ListBox`. No `#[allow(dead_code)]` in committed code.

## Objective (Objetivo)

Review all Rust and UI files changed in the current branch and generate a structured Markdown findings report at `docs/review-<YYYY-MM-DD>.md`.

## Restrictions (Restrições)

- Scope only files returned by `git diff --name-only main...HEAD`; exclude `Cargo.lock`, `*.gresource`, and binary assets.
- Never apply fixes — record findings only; never edit any source file.
- Classify every finding with exactly one severity: `Critical` / `High` / `Medium` / `Low`.
- Flag any layer-boundary violation (GTK import in `src/core/`, `tokio` usage anywhere) as `Critical`.
- Write the report exclusively to `docs/review-<YYYY-MM-DD>.md`; never output findings as raw conversational prose.
- Limit the report to changed files only; do not analyse unchanged files even if they contain issues.
- Use the exact report skeleton defined in `## Output Format`; do not add or remove top-level sections.

## Steps (Passos)

1. **Collect changed files** — Run `git diff --name-only main...HEAD` and filter results to `.rs` and `.ui` files, excluding `Cargo.lock`, compiled resources, and binary assets. Done when: the filtered list is stored and contains at least one file (if empty, write a report stating "No Rust or UI files changed" and stop).

2. **Read each changed file** — Read every file in the filtered list using the Read tool. For each file, also note its layer (core / ports / infrastructure / window) based on its path prefix. Done when: all files have been read and their layer labels recorded.

3. **Analyse each file for violations** — For each file, check against all 8 rule categories: (1) threading — blocking calls outside `spawn_driver_task`, GTK calls from worker threads; (2) layer boundaries — forbidden imports per layer table; (3) i18n — user-visible strings not wrapped in `gettext`/`pgettext`/`ngettext`; (4) A11Y — icon-only buttons missing tooltip or accessible label, color-only state; (5) GTK patterns — raw GTK where Adwaita equivalent exists, `ListBox` instead of `GListModel`; (6) error handling — `ContainerError` not normalised via `log_container_error()`; (7) dead code — `#[allow(dead_code)]` attributes present; (8) test coverage — new public API methods in `src/core/` or `src/ports/` without a corresponding `#[test]`. Done when: every file has either at least one finding or is recorded as clean.

4. **Draft the report** — Assemble all findings into the report skeleton: populate the Summary severity table, list findings per file with line numbers and recommendations, list clean files, and compute statistics. Done when: all sections of the skeleton are filled with concrete content.

5. **Write the report file** — Write the complete report to `docs/review-<YYYY-MM-DD>.md` using today's date in the filename. Done when: the file exists on disk and is non-empty.

6. **Validate output** — Verify each output acceptance criterion (O1–O5). For any failure, correct it immediately by rewriting the file. Done when: all criteria pass or failures are listed under `## Validation errors` in the report.

## Examples (Exemplos)

**Input — file with a violation:**
```
// src/window/views/containers_view.rs  line 87
let containers = driver.list_containers().unwrap();  // direct blocking call
```

**Expected output — finding entry:**
```markdown
#### [High] Blocking driver call outside `spawn_driver_task`

**File:** `src/window/views/containers_view.rs`
**Line:** 87
**Category:** threading
**Description:** `driver.list_containers()` is called directly on the GTK main thread. Blocking the main thread freezes the UI and violates the threading contract defined in `src/infrastructure/containers/background.rs`.
**Recommendation:** Wrap the call with `spawn_driver_task(driver.clone(), |d| d.list_containers(), callback)` and handle the result in the callback on the main loop.
```

---

**Input — clean file:**
```
// src/core/domain/container.rs — pure domain model, no GTK imports, all strings in logic layer
```

**Expected output — clean files section entry:**
```markdown
- `src/core/domain/container.rs` — no violations found
```

## Output Format (Formato de Saída)

Write a single Markdown file at `docs/review-<YYYY-MM-DD>.md`. Maximum length: 600 lines.

```markdown
# Code Review — <YYYY-MM-DD>

## Summary

| Severity | Count |
|----------|-------|
| Critical | N     |
| High     | N     |
| Medium   | N     |
| Low      | N     |

**Branch:** `<branch-name>`
**Files reviewed:** N
**Files with findings:** N

---

## Findings

### `<file-path>` — <layer>

#### [<SEVERITY>] <Finding title>

**Line:** <line-number or range>
**Category:** <threading | layer-boundary | i18n | a11y | patterns | error-handling | dead-code | coverage>
**Description:** <what the issue is and why it matters in this codebase>
**Recommendation:** <specific, actionable fix — reference the correct helper or pattern>

---

## Clean files

- `<file-path>` — no violations found

## Validation errors

<!-- Only present if output criteria failed; omit this section entirely otherwise -->
```

## Input Acceptance Criteria (Critérios de Aceite de Entrada)

Before executing the main task, verify these criteria. If any fail, stop and report the failure.

| # | Criterion | How to verify |
|---|-----------|---------------|
| I1 | The current directory is the root of the `gtk-cross-platform` repository | Run `ls Cargo.toml src/`; both must exist |
| I2 | The `docs/` directory exists | Run `ls docs/`; if missing, create it with `mkdir docs` — this is a setup action, not a blocking failure |
| I3 | `git diff --name-only main...HEAD` returns at least one file | Run the command; if empty output, write a minimal report stating "No files changed vs main" and stop |

## Output Acceptance Criteria (Critérios de Aceite de Saída)

After completing the task, verify these criteria. If any fail, append a `## Validation errors` section to the report listing the failures.

| # | Criterion | How to verify |
|---|-----------|---------------|
| O1 | Report file exists at `docs/review-<YYYY-MM-DD>.md` | Use Read tool on the expected path; file must be non-empty |
| O2 | Every finding entry includes `**Line:**`, `**Category:**`, `**Description:**`, and `**Recommendation:**` fields | Grep for each field name in the Findings section |
| O3 | Every finding is classified with exactly one of: `Critical`, `High`, `Medium`, `Low` | Grep `\[Critical\]\|\[High\]\|\[Medium\]\|\[Low\]`; count must equal total finding count |
| O4 | Summary severity table is present and totals match the actual finding count | Count `####` headings in Findings; compare to sum of the four severity counts in the table |
| O5 | Report structure matches the skeleton: sections `## Summary`, `## Findings`, `## Clean files` are all present | Grep for each heading in the output file |
