# Content Audit

> Last run: 2026-04-27
> Branch: main
> Files scanned: 213 (from `git ls-files`)

## Summary

| Category | Findings | Highest severity |
|---|---|---|
| Human-First Readability | 8 | MEDIUM |
| AI-Free Codebase | 2 | HIGH |
| Visualization Quality | 2 | HIGH |
| Code Comment Quality | 8 | MEDIUM |
| Terminology Consistency | 1 | MEDIUM |
| Placeholder and Draft Content | 4 | CRITICAL |
| Link and Reference Integrity | 1 | HIGH |
| Version Consistency | 0 | — |
| **Total** | **26** | **CRITICAL** |

## Files Checked

- **Documentation (*.md):** README.md, CONTRIBUTING.md, CHANGELOG.md, CLAUDE.md, SECURITY.md, GOVERNANCE.md, CODE_OF_CONDUCT.md, docs/compliance-plan.md, docs/conceptual-improvements.md, docs/gtk-sources.md, docs/test-quality-audit.md, .github/PULL_REQUEST_TEMPLATE.md
- **Code comments (*.rs):** src/app.rs, src/config.rs, src/main.rs, src/core/domain/container.rs, src/infrastructure/containers/error.rs, src/infrastructure/logging/app_logger.rs, src/window/components/status_badge.rs, src/window/views/containers_view.rs (via grep)
- **UI / data files:** data/com.example.GtkCrossPlatform.metainfo.xml, data/com.example.GtkCrossPlatform.desktop, data/com.example.GtkCrossPlatform.gschema.xml, data/resources/resources.gresource.xml
- **Shell scripts:** Makefile (via grep), .github/workflows/ci.yml, .github/workflows/release.yml, .github/workflows/audit.yml (via grep)
- **Translation:** po/POTFILES, po/LINGUAS, po/pt_BR.po (via grep)
- **Additional (code):** build.rs, Cargo.toml

---

## Findings

### CRITICAL

#### [C6.4-001] Placeholder — metainfo.xml contains template identity in shipped fields
- **File:** `data/com.example.GtkCrossPlatform.metainfo.xml:7-14`
- **Finding:** The `<name>`, `<summary>`, and `<description>` fields describe the app as "A cross-platform GTK4 + Adwaita template application" — directly exposing the placeholder identity in GNOME Software and Flathub store listings. Any actual user or store bot reading the metainfo sees a template description, not a product description.
- **Suggested fix:** Replace `<name>`, `<summary>`, and `<description>` with the real app name ("Container Manager" or the final product name) and a description of its actual purpose as a GNOME container manager.

#### [C6.4-002] Placeholder — .desktop entry contains template identity
- **File:** `data/com.example.GtkCrossPlatform.desktop:1-3`
- **Finding:** `Name=GTK Cross Platform` and `Comment=A cross-platform GTK4 + Adwaita template application` are placeholder values that appear in application launchers, taskbars, and Wayland window decorations. Any user who installs this app sees the template name.
- **Suggested fix:** Update `Name` to the real application name and `Comment` to a one-sentence description of the container manager.

#### [C6.4-003] Placeholder — About dialog hardcodes `your-org` placeholder GitHub URLs
- **File:** `src/app.rs:247-265`
- **Finding:** The About dialog (shown to all users via F1 or menu) links to `https://github.com/your-org/gtk-cross-platform` for both the homepage and issue tracker. The real repository URL `https://github.com/BrunoMartinsCorrea/gtk-cross-platform` is used in README.md and SECURITY.md but not wired into the About dialog. Users clicking "Report an Issue" will reach a 404.
- **Suggested fix:** Replace `your-org` with `BrunoMartinsCorrea` in all four URL strings in `src/app.rs` lines 247, 248, 261, 265.

---

### HIGH

#### [C2.1-001] AI-Free — AI tool explicitly attributed as document author
- **File:** `docs/test-quality-audit.md:4`
- **Finding:** Line 4 reads `**Auditor:** Claude Code (externo)` — naming Claude Code as the external auditor of this document. Per the project's "AI-free codebase" principle, distributed source files must not carry explicit AI authorship claims. A contributor reading this file is told the analysis was performed by an AI tool, not a human.
- **Suggested fix:** Remove the `**Auditor:**` line or replace with a neutral process description (e.g., `**Method:** automated audit against the test quality principles in this file`).

#### [C2.1-002] AI-Free — Claude Code explicitly named in root-level tracked file
- **File:** `CLAUDE.md:3`
- **Finding:** Line 3 reads: `This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.` CLAUDE.md is checked into the repository (not in `.claude/`) and publicly visible. The file names the specific AI tool and includes a URL (`claude.ai/code`) identifying the AI platform, coupling the project identity to a vendor.
- **Suggested fix:** Rephrase the first paragraph to describe the file's purpose without naming the AI vendor — e.g., `This file documents project conventions and is structured for use with AI coding assistants.`

#### [C3.1-001] Visualization — ASCII architecture diagram in README.md replaceable by Mermaid
- **File:** `README.md:70-86`
- **Finding:** The architecture section uses a box-drawing ASCII diagram (`┌─`, `│`, `├─`, `└─`) to show the four-layer hexagonal architecture. This cannot be copy-searched, reflowed on mobile, or parsed by accessibility tooling. A Mermaid `flowchart TB` or `classDiagram` would render natively in GitHub, GitLab, and most Markdown viewers.
- **Suggested fix:** Replace the ASCII block with a Mermaid `flowchart TB` diagram. Example approach:
  ````
  ```mermaid
  flowchart TB
    UI["UI (src/window/) — GTK4 · LibAdwaita"]
    Ports["Ports (src/ports/) — Rust traits"]
    Infra["Infrastructure (src/infrastructure/) — GLib · GIO"]
    Domain["Domain (src/core/) — zero external deps"]
    UI --> Ports --> Infra --> Domain
  ```
  ````

#### [C6.3-001] Placeholder — Screenshot section exists but `docs/screenshots/` does not
- **File:** `README.md:32-34`
- **Finding:** The README contains a "Screenshots" section with the prose `> Add screenshots to \`docs/screenshots/\` and link them here.` The directory `docs/screenshots/` does not exist in the repository. This placeholder has been in the README since the initial commit and makes the project appear unfinished to first-time visitors.
- **Suggested fix:** Either add at least one screenshot to `docs/screenshots/` and link it, or remove the Screenshots section entirely until screenshots are available.

#### [C7.2-001] Link integrity — `example.com` used as real homepage URL in AppStream metadata
- **File:** `data/com.example.GtkCrossPlatform.metainfo.xml:22`
- **Finding:** `<url type="homepage">https://example.com</url>` is an IANA reserved domain used in place of the real project URL. AppStream validators (and Flathub review bots) will reject or warn on this. GNOME Software will display `https://example.com` as the project homepage.
- **Suggested fix:** Replace with the real project URL: `https://github.com/BrunoMartinsCorrea/gtk-cross-platform`.

---

### MEDIUM

#### [C1.2-001] Readability — README.md is 236 lines with no table of contents
- **File:** `README.md`
- **Finding:** At 236 lines, the README exceeds the 200-line threshold that warrants a ToC. The 11 top-level sections (What is this? → License) are not linked from any navigation anchor at the top, requiring readers to scroll to discover the structure.
- **Suggested fix:** Add a ToC immediately after the badges block, listing anchor links to each `##` section.

#### [C1.4-001] Readability — "Phosh" used without definition in README.md
- **File:** `README.md:25`
- **Finding:** "GNOME Mobile (Phosh / postmarketOS)" uses "Phosh" without explanation. New contributors unfamiliar with the GNOME Mobile ecosystem will not know that Phosh is the GNOME Mobile phone shell.
- **Suggested fix:** Add a parenthetical: `GNOME Mobile (Phosh — the GNOME phone shell — / postmarketOS)` or link to the Phosh project.

#### [C1.4-002] Readability — "postmarketOS" used without definition in README.md
- **File:** `README.md:26`
- **Finding:** "postmarketOS" is named without a parenthetical explaining it is a Linux-based mobile OS. Combined with the missing Phosh definition above, the sentence expects too much prior knowledge from a first-time reader.
- **Suggested fix:** Add a parenthetical: `postmarketOS (a Linux-based mobile OS)`.

#### [C1.4-003] Readability — "sp" (scale pixels) used without definition in README.md
- **File:** `README.md:49`
- **Finding:** "four breakpoints (> 768 sp desktop → 360 sp GNOME Mobile)" uses "sp" as a unit without defining it. First use of a unit must include a definition per the audit rules.
- **Suggested fix:** On first use: "four breakpoints (> 768 sp (scale pixels) desktop → 360 sp GNOME Mobile)". Subsequent uses may abbreviate.

#### [C1.4-004] Readability — "GResource" used without definition in README.md
- **File:** `README.md:162`
- **Finding:** The project layout code block contains the comment `# entry point: GResource, i18n, app launch` without defining what GResource is. While this is inside a code-layout block, it is the first occurrence of the term in README.md.
- **Suggested fix:** Add a note in the "What is this?" or "Architecture" section: "GResource (compiled binary resource bundle for UI files)".

#### [C1.4-005] Readability — "sp" (scale pixels) undefined in CONTRIBUTING.md
- **File:** `CONTRIBUTING.md:107`
- **Finding:** "All interactive elements must be ≥ 44 × 44 sp" is the first use of "sp" in this file and it is not defined. Contributors writing UI code may not know this unit.
- **Suggested fix:** First use: "≥ 44 × 44 sp (scale pixels, a GTK resolution-independent unit)".

#### [C3.1-002] Visualization — ASCII arrow flow in docs/conceptual-improvements.md replaceable by Mermaid
- **File:** `docs/conceptual-improvements.md:41-51`
- **Finding:** A vertical arrow diagram (`↓` between stack layers) is used to show the GListModel pipeline. This ASCII flow could be rendered as a more readable Mermaid `flowchart TB`.
- **Suggested fix:** Replace with:
  ````
  ```mermaid
  flowchart TB
    LS["gio::ListStore&lt;ContainerObject&gt;"]
    FL["FilterListModel(CustomFilter)"]
    SL["SortListModel(CustomSorter)"]
    SEL["NoSelection / SingleSelection"]
    LV["ListView + SignalListItemFactory"]
    LS --> FL --> SL --> SEL --> LV
  ```
  ````

#### [C4.3-001] Comment quality — 15-line comment block on `log_container_error`
- **File:** `src/infrastructure/containers/error.rs:79-93`
- **Finding:** The `log_container_error` function has a 15-line doc comment that includes a rationale section and a table. The length exceeds the 3-line threshold. The content is legitimate (explains why logging belongs at the call site, not inside `ContainerError`), but most of it could live in the CONTRIBUTING.md or a decision-doc rather than inline source code.
- **Suggested fix:** Keep at most one paragraph: `/// Log \`err\` at the level appropriate for its variant. Call at the call site, not inside ContainerError, to avoid hidden side effects.` Move the table to CONTRIBUTING.md if needed.

#### [C4.3-002] Comment quality — 6-line comment on `spawn_driver_task`
- **File:** `src/infrastructure/containers/background.rs:11-16`
- **Finding:** The function's doc comment is 6 lines — above the 3-line threshold. The threading model rationale is valuable but could be condensed.
- **Suggested fix:** Condense to: `/// Bridge a blocking driver call to the GTK main thread. The task runs on a worker thread; the callback fires on the GLib main loop.`

#### [C4.3-003] Comment quality — 5-line comment on `resolve_initial_driver`
- **File:** `src/app.rs:128-132`
- **Finding:** Private method has a 5-line doc comment explaining the fallback strategy. Well-named function and clear code body make most of the comment redundant.
- **Suggested fix:** Reduce to one line: `/// Prefer the saved runtime preference; fall back to auto-detect on failure.`

#### [C4.3-004] Comment quality — 4-line comment on `is_secret_env_key`
- **File:** `src/core/domain/container.rs:393-396`
- **Finding:** The "Used to mask values in the UI before displaying them" line is a caller-context note that belongs in the call site or CONTRIBUTING.md, not the function docstring. The function name already implies secret detection.
- **Suggested fix:** Keep: `/// Returns true if \`key\` name suggests it holds a secret (contains PASS, SECRET, KEY, or TOKEN).` Remove the "Used to mask" sentence.

#### [C5.1-001] Terminology — "split view" used unqualified
- **File:** `docs/compliance-plan.md:78`
- **Finding:** GAP-09 description reads "one breakpoint at 720 sp (collapses split view)". The canonical term is `AdwNavigationSplitView`; "split view" unqualified is a forbidden variant per the terminology table.
- **Suggested fix:** Replace "split view" with `AdwNavigationSplitView`.

---

### LOW

#### [C1.2-002] Readability — CLAUDE.md is 491 lines with no table of contents
- **File:** `CLAUDE.md`
- **Finding:** At 491 lines and 11 top-level sections, CLAUDE.md benefits from a ToC even as an AI configuration file, since human developers also read it to understand project conventions.
- **Suggested fix:** Add a ToC after the opening paragraph.

#### [C1.2-003] Readability — docs/test-quality-audit.md is 555 lines with no table of contents
- **File:** `docs/test-quality-audit.md`
- **Finding:** The test quality audit report is 555 lines with multiple severity sections and no navigation anchor list at the top.
- **Suggested fix:** Add a ToC listing the main severity sections and the Checklist section.

#### [C4.1-001] Comment quality — what-comment on filter phase in containers_view.rs
- **File:** `src/window/views/containers_view.rs:136`
- **Finding:** `// Filter: name, image, short_id, compose_project` describes the fields being filtered but not why those specific fields or what invariant the filter maintains. The field list is already visible in the code below.
- **Suggested fix:** Remove this comment; the code is self-documenting.

#### [C4.3-005] Comment quality — 6-line comment on `log_with_fields`
- **File:** `src/infrastructure/logging/app_logger.rs:41-46`
- **Finding:** 6-line doc comment that describes both the format and a future migration note. The migration note `// A future migration to \`glib::log_structured\`...` is a forward-looking comment that may become stale.
- **Suggested fix:** Keep to one sentence describing the format: `/// Log \`message\` with \`fields\` appended as \`key=value\` pairs for structured output.` Remove the migration note.

#### [C4.3-006] Comment quality — 4-line comment on `group_by_compose`
- **File:** `src/core/domain/container.rs:429-432`
- **Finding:** 4-line comment describes both what the function returns and the sort order. The return-type doc is redundant with the type signature; only the sort order is non-obvious.
- **Suggested fix:** Condense to: `/// Groups containers by compose project; named groups sorted alphabetically, ungrouped last.`

#### [C4.3-007] Comment quality — 3-line what-comment on `filter_containers`
- **File:** `src/core/domain/container.rs:406-408`
- **Finding:** `/// Matches against name, image, short ID, and compose project.` describes the mechanics (what fields are searched) without explaining why this specific combination. The fields are visible in the code body.
- **Suggested fix:** Either remove the second line or replace with a note about case-insensitivity being intentional for UX consistency.

---

## Informational

### AI attribution in commit history

> 8 commits in the last 100 contain `Co-Authored-By: Claude` — expected from AI-assisted
> development workflow; not a violation.

---

## Resolved Findings

<!-- Append resolved entries here; do not delete rows -->

| Date | Finding ID | Severity | Category | File | Resolution |
|---|---|---|---|---|---|
