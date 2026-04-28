# /project:docs-audit

Audit the **documentation layer** of this GTK4/Rust/Flatpak project for staleness, cross-document
inconsistencies, coverage gaps, and readability failures. This command is self-contained — run it
fresh without prior conversation context.

This audit is complementary to, and does not duplicate, the scope of:

- `/project:compliance-audit` — documented concepts vs. implementation (i18n, A11Y, threading, architecture)
- `/project:concept-audit` — internal code inconsistencies
- `/project:github-audit` — repository health (CI/CD, security, distribution)

Focus here is: *do the human-facing documents form a correct, consistent, and navigable system?*

---

## What to read before auditing

Read these files in full before emitting any diagnosis:

- `README.md`
- `CLAUDE.md`
- `CONTRIBUTING.md`
- `CHANGELOG.md`
- `SECURITY.md`
- `GOVERNANCE.md`
- `CODE_OF_CONDUCT.md`
- `AUTHORS`
- `Makefile`
- `Cargo.toml` — source of truth for versions
- `src/` — verify type names, file paths, and module structure are reflected correctly in docs
- `tests/` — verify test file names and locations are documented correctly
- `data/` — verify resource paths, metainfo, and desktop file referenced in docs exist
- `.github/workflows/` — verify CI step names referenced in docs exist
- `.claude/commands/` — verify slash command list in `CLAUDE.md` is complete and accurate

---

## Category 1 — Stale File Paths and Type Names

Documentation that references files, modules, or types by path or name becomes stale when code
is moved, renamed, or deleted. Check:

1. **`CLAUDE.md` Project Structure section** — every path listed in the `src/`, `tests/`, `data/`,
   and `.claude/` trees must exist on disk. Verify each path with the actual file tree.

2. **`CLAUDE.md` Key types section** — every named type (`GtkCrossPlatformApp`, `MainWindow`,
   `IContainerDriver`, `ContainerDriverFactory`, `spawn_driver_task`, `MockContainerDriver`,
   `AppLogger`, `log_container_error`, `config`) must exist at the stated path. Grep `src/` to
   confirm each name and its file location.

3. **`README.md` Project layout** — all paths listed must exist. Compare against actual file tree.

4. **`CONTRIBUTING.md`** — any file paths cited (e.g. `tests/container_driver_test.rs`,
   `po/POTFILES`, `src/core/`) must exist.

5. **`CLAUDE.md` Slash Commands table** — each `commands/<name>.md` listed must exist under
   `.claude/commands/`. Report commands listed in the table but missing on disk, and commands
   on disk that are not listed in the table.

6. **`Makefile` targets referenced in docs** — every `make <target>` mentioned in `README.md`,
   `CONTRIBUTING.md`, or `CLAUDE.md` must exist as a `.PHONY` target in the `Makefile`. Report
   any documented target absent from the Makefile.

---

## Category 2 — Cross-Document Inconsistencies

When the same fact appears in multiple documents, they must agree. Check:

1. **Dependency versions** — compare the versions stated in `CLAUDE.md` (§Dependencies) against
   `Cargo.toml` `[dependencies]` section. The `Cargo.toml` is the source of truth. Report any
   mismatch in major or minor version numbers.

2. **GTK/Adwaita minimum runtime** — `README.md` Requirements table vs. `CLAUDE.md` §Dependencies
   vs. `com.example.GtkCrossPlatform.json` runtime declaration. All three must agree on
   `GTK4 ≥ 4.12` and `LibAdwaita ≥ 1.4`.

3. **Breakpoint values** — `README.md` mentions four breakpoints; `CLAUDE.md` declares them with
   exact `sp` values. Compare both against `data/resources/window.ui` `<object class="AdwBreakpoint">`
   declarations. Report any value that differs across the three sources.

4. **Runtime detection order** — `CLAUDE.md` table (Docker → Podman rootless → Podman root →
   containerd/nerdctl) vs. actual code in `src/infrastructure/containers/factory.rs`. All
   documentation must reflect the same order.

5. **App ID** — `CLAUDE.md`, `README.md`, `Makefile`, `Cargo.toml` (`[package].name`),
   `com.example.GtkCrossPlatform.json`, `data/com.example.GtkCrossPlatform.desktop`,
   `data/com.example.GtkCrossPlatform.metainfo.xml` must all reference the same App ID string.
   Report any document that uses a different or inconsistent identifier.

6. **Test suite names** — `CONTRIBUTING.md` lists three test suites; `CLAUDE.md` §Testing lists
   the same suites. Compare the two tables for discrepancies in test file names, suite descriptions,
   and display requirements.

7. **`make test` vs. `make test-nextest`** — `CLAUDE.md` §Testing says `make test` runs via
   `cargo test`; `make test-nextest` via `cargo nextest`. `CONTRIBUTING.md` says `make test` runs
   all tests. Verify both documents agree on which command to run for CI and which for local
   development.

8. **Rust edition** — `Cargo.toml` `[package].edition` vs. any edition mention in `README.md` or
   `CLAUDE.md`. Must agree on `2024` (or whatever the actual value is).

---

## Category 3 — Coverage Gaps (Undocumented Additions)

New files, targets, or features added to the codebase but not reflected in documentation. Check:

1. **Makefile targets not documented anywhere** — enumerate all `.PHONY` targets in `Makefile`
   and verify each appears in at least one of: `README.md` Build reference, `CLAUDE.md` Build
   and Run Commands, or `CONTRIBUTING.md`. Report targets that exist only in the Makefile.

2. **Source files not in `CLAUDE.md` Project Structure** — list every `.rs` file under `src/`.
   For each, verify it appears in the CLAUDE.md tree (exact path or a covering directory
   description). Report files present in `src/` but absent from the documented tree.

3. **Test files not documented** — list every file under `tests/`. Verify each is mentioned in
   `CLAUDE.md` §Testing or `CONTRIBUTING.md`. Report undocumented test files.

4. **`.claude/commands/` files not in `CLAUDE.md` Slash Commands table** — any `.md` file under
   `.claude/commands/` with no entry in the CLAUDE.md table is an undocumented command.

5. **GitHub Actions workflows not mentioned in any doc** — list `.github/workflows/*.yml`. Verify
   each workflow (CI, Flatpak, EditorConfig, audit, release) is mentioned somewhere in
   `README.md`, `CONTRIBUTING.md`, or `CLAUDE.md`. Report workflows with no documentation.

6. **`data/` resources not referenced in docs** — list every file under `data/` and `po/`.
   Verify schema files, icon sets, metainfo, and desktop file are documented in `CLAUDE.md`
   Project Structure or `README.md` Project layout. Report undocumented data files.

---

## Category 4 — README Quality

1. **Features section completeness** — `README.md` claims the app connects to Docker, Podman,
   and containerd for containers, images, volumes, and networks. Verify the IContainerDriver port
   (`src/ports/i_container_driver.rs`) actually declares methods covering all claimed operations.
   Report claimed features with no corresponding trait method.

2. **Architecture diagram accuracy** — the ASCII diagram in `README.md` must match `CLAUDE.md`
   layer table. Compare layer names, paths, and descriptions in both. Report any divergence.

3. **Requirements table completeness** — `README.md` requirements table lists GTK4, LibAdwaita,
   Rust edition, and container runtime. Verify no build dependency (e.g., `glib-build-tools`,
   `gettext`) is missing that a new contributor would need.

4. **Screenshots section** — `README.md` has a placeholder `> Add screenshots to docs/screenshots/`.
   Check whether `docs/screenshots/` exists and has any content. If screenshots exist, report
   that the placeholder should be replaced with actual images.

5. **macOS and Windows sections** — `README.md` documents `make setup-macos` and
   `make setup-windows`. Verify these targets exist in the Makefile and that the described
   output still matches what the targets actually print.

6. **Getting started section** — verify the three commands (`git clone`, `make setup`, `make run`)
   actually work as described: that `make setup` calls `cargo fetch` and `make run` calls
   `cargo run`. Cross-reference with Makefile.

---

## Category 5 — CLAUDE.md Accuracy

`CLAUDE.md` is the primary reference for AI agents and contributors. Any inaccuracy degrades
both human and AI collaboration.

1. **`G_LOG_DOMAIN` table** — every sub-domain listed (`…containers`, `…background`,
   `…view.containers`, etc.) must be produced by an actual `AppLogger::subdomain()` call or
   hardcoded domain string in `src/`. Grep for each domain suffix; report any listed in CLAUDE.md
   that has no corresponding call.

2. **Composition root rule** — CLAUDE.md states `src/app.rs::activate()` is the only place
   concrete types are wired to ports. Verify this still holds by grepping for
   `ContainerDriverFactory::detect()` outside `app.rs`.

3. **`spawn_driver_task` contract** — CLAUDE.md describes the exact async-channel pattern with
   `async_channel::bounded(1)`, `glib::spawn_local`, `begin_loading/end_loading`. Verify the
   description matches the actual implementation in `src/infrastructure/containers/background.rs`.

4. **Threading rules completeness** — CLAUDE.md says "views are the primary layer that calls
   `spawn_driver_task`; `MainWindow` may also call it for window-scoped actions." Verify this
   is reflected in the code — grep for `spawn_driver_task` outside `src/window/` and report
   any CLAUDE.md omission.

5. **`config` constants list** — CLAUDE.md lists the constants from `build.rs`
   (`APP_ID`, `VERSION`, `PROFILE`, `LOCALEDIR`, `PKGDATADIR`, `GETTEXT_PACKAGE`). Read
   `src/config.rs` and `build.rs`; report any constant present in code but absent from CLAUDE.md,
   and any listed in CLAUDE.md but absent from code.

6. **Dependencies table rule** — CLAUDE.md says "When bumping a dependency in `Cargo.toml`,
   update the version table in §Dependencies in the same commit. `Cargo.toml` is the source of
   truth — CLAUDE.md must never diverge from it." Run the cross-check: compare the versions in
   the §Dependencies paragraph with `Cargo.toml` and report any drift.

---

## Category 6 — CONTRIBUTING.md Accuracy

1. **Linux setup commands** — `sudo apt install libgtk-4-dev libadwaita-1-dev gettext rustup`.
   Verify these package names exist in Ubuntu/Debian repos. Also verify `gettext` provides
   `msgfmt` and `msginit` used elsewhere in the file.

2. **`make lint && make lint-i18n && make fmt`** — verify all three targets exist in the Makefile
   and are described consistently. If `make fmt` checks (not fixes), the PR guide must say
   "check" not "auto-format".

3. **Widget test run instructions** — `xvfb-run cargo test --test widget_test -- --test-threads=1
   --ignored` must match the actual test file name and flags used. Verify `widget_test.rs` exists
   and the `--ignored` flag is needed (i.e., tests are marked `#[ignore]`).

4. **Translation workflow** — step 2 says `msginit -l <locale> -i gtk-cross-platform.pot -o <locale>.po`.
   Verify the `.pot` filename matches what `gettext` generates for this project's text domain
   (`GETTEXT_PACKAGE` in `build.rs`).

5. **PR checklist completeness** — the PR section says: fork, write tests, run lint + i18n + fmt,
   keep `src/core/` and `src/ports/` free of GTK/IO. Verify no new mandatory step (e.g., running
   `make check-potfiles`, `make validate-metainfo`) is missing from the checklist.

---

## Category 7 — CHANGELOG Discipline

1. **Format compliance** — verify `CHANGELOG.md` follows [Keep a Changelog](https://keepachangelog.com/):
   sections are `## [Unreleased]`, `## [x.y.z] – YYYY-MM-DD`; change types are `Added`, `Changed`,
   `Deprecated`, `Removed`, `Fixed`, `Security`. Report any section or entry that doesn't follow
   this structure.

2. **Version alignment** — the most recent versioned entry in `CHANGELOG.md` must match
   `Cargo.toml [package].version`. Report any mismatch.

3. **Unreleased section** — if there are commits since the last release tag (check `git log`),
   there should be an `## [Unreleased]` section. Report its absence when unreleased commits exist.

4. **Entry completeness** — for each versioned entry, verify the release date is present and
   formatted as `YYYY-MM-DD`. Report entries with missing or malformed dates.

---

## Category 8 — Supporting Documents Quality

1. **SECURITY.md** — must contain: a description of supported versions, a reporting channel
   (email or GitHub Security Advisory URL), an expected response time, and a disclosure policy.
   Report any missing element.

2. **GOVERNANCE.md** — must document: maintainership criteria, decision-making process, and
   release policy. Report any of these three sections that is missing or has placeholder content.

3. **CODE_OF_CONDUCT.md** — verify it references an enforcement contact (email or link).
   A CoC with no enforcement contact is unenforceable. Report if missing.

4. **AUTHORS** — verify the file lists at least one author. Cross-reference with `git log
   --format="%an <%ae>" | sort -u` to verify authorship attribution is consistent.

5. **`.github/PULL_REQUEST_TEMPLATE.md`** — if present, verify its checklist aligns with the
   PR requirements in `CONTRIBUTING.md`. Report any checklist item in one but not the other.

---

## Output format

For each gap found, emit one block:

```
## [SEVERITY] CATEGORY — Gap title

- **Document:** <file>:<section or line>
- **Status:** STALE | MISSING | INCONSISTENT | INCOMPLETE
- **Detail:** <one sentence — what is wrong and why it matters for contributors or AI agents>
- **Fix:** <one sentence — what needs to change>
```

Severity levels:

- **CRITICAL** — a contributor or AI agent following this document will do the wrong thing
- **HIGH** — an important fact is wrong, stale, or missing; likely to cause confusion
- **MEDIUM** — minor inaccuracy or omission; unlikely to block work but degrades trust in docs
- **LOW** — cosmetic or style issue

End with a summary table:

| Category                        | Gaps found | Highest severity |
|---------------------------------|------------|------------------|
| Stale file paths and type names | N          | …                |
| Cross-document inconsistencies  | N          | …                |
| Coverage gaps (undocumented)    | N          | …                |
| README quality                  | N          | …                |
| CLAUDE.md accuracy              | N          | …                |
| CONTRIBUTING.md accuracy        | N          | …                |
| CHANGELOG discipline            | N          | …                |
| Supporting documents quality    | N          | …                |

Do not report items that are correct. Focus only on gaps, staleness, and inconsistencies.
