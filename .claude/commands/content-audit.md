# /project:content-audit

Audit the **content quality** of this repository against the project's editorial guidelines.
This command is self-contained — run it fresh without prior conversation context.

Output all findings to `.claude/docs/content-audit.md`, creating the file if it does not exist.
Preserve the `## Resolved Findings` table from any previous run.

This audit is complementary to, and does not duplicate, the scope of:

- `/project:docs-audit` — documentation staleness, path accuracy, cross-document inconsistencies
- `/project:compliance-audit` — documented concepts vs. implementation
- `/project:concept-audit` — internal code inconsistencies

Focus here is: *do the content, language, and visual style of every tracked file follow the
project's human-first, AI-agnostic, and diagram quality standards?*

---

## Execution Strategy

Execute in this order — do not shortcut any step:

1. **Build the file list**: run `git ls-files` and partition results into the scope groups defined in §What to scan. Store the list.
2. **Read every in-scope file**: use the `Read` tool on each file explicitly. Do not rely on memory, grep output alone, or inferred content. Reading is mandatory — a file that is not read is not checked.
3. **Read files in parallel batches** where the tool allows: group `*.md` files into batches of 4–6 per call to avoid serial bottlenecks.
4. **Run all category checks against the read content** and accumulate findings as you go.
5. **Assign a finding ID** to every finding in the format `[Cx.y-NNN]` (e.g., `[C1.1-001]`), where `x.y` is the subcategory and `NNN` is a zero-padded sequence within the run.
6. **Count findings per category** and populate the Summary table.
7. **Write the complete output file** in a single Write operation.
8. **Print a terminal summary** of the most urgent findings.

---

## What to scan

Run `git ls-files` to get the canonical list of tracked files. Restrict checks to:

| Scope | Files |
|---|---|
| Documentation | `*.md` at any level, including `.github/PULL_REQUEST_TEMPLATE.md` |
| Code comments | `src/**/*.rs`, `build.rs` |
| UI / data files | `data/**/*.ui`, `data/**/*.xml`, `data/**/*.desktop` |
| Shell scripts | `Makefile`, `.github/workflows/*.yml` |
| Translation | `po/*.po`, `po/POTFILES`, `po/LINGUAS` |

**Agent-structure exemption:** skip `.claude/` for all checks unless a specific dimension states otherwise.
That directory is the agent's own workspace and is not subject to the AI-mention or terminology rules.

---

## Category 1 — Human-First Readability

Content must be navigable and understandable for humans without requiring prior context or
specialized tooling knowledge. Check in all `*.md` files:

### 1.1 — Unbroken prose blocks

Flag any paragraph block that:
- Runs longer than **6 continuous lines** without a bullet list, numbered list, header, or code block
- Combines more than **two distinct instructions** into a single sentence
- Contains a shell command inline (e.g., `` run `cargo build` ``) instead of in a fenced code block

### 1.2 — Missing section headers

Flag any document where:
- The file is longer than **80 lines** and has no `##` or `###` subheader
- The file is longer than **200 lines** and has no table of contents (a list of anchor links at the top)
- A section header is immediately followed by another header with no content between them (empty section)

### 1.3 — Illogical reading order

Documents should read: what it is → why use it → how to use it → how to contribute.
Flag documents where:
- Installation or build commands appear **before** a description of what the project does
- "How to contribute" content appears before "How to use"

### 1.4 — Undefined first-use acronyms

Flag acronyms used in `README.md` or `CONTRIBUTING.md` without a definition or link on first
appearance. Known acronyms that do **not** need definition in this project: GTK, GNOME, HIG, UI, API,
CI, PR, CD, SDK, ARM, DMG, ZIP, URL, OSS, FLOSS, RFC, PDF, SVG, PNG, XML, JSON, CSS, YAML.

Acronyms that **do** require a definition or parenthetical on first use:
- `GResource` — parenthetical e.g. "(compiled binary resource bundle)"
- `POTFILES` — parenthetical e.g. "(list of files containing translatable strings)"
- `LINGUAS` — parenthetical e.g. "(active locale list)"
- `sp` — must appear as "sp (scale pixels)" or "scale pixels (sp)" on first use in each document
- `Phosh` — parenthetical e.g. "(GNOME Mobile phone shell)"
- `postmarketOS` — parenthetical e.g. "(Linux-based mobile OS)"
- `AppStream` — parenthetical e.g. "(software metadata standard)"
- `metainfo` — parenthetical e.g. "(AppStream metadata file)"
- `Flatpak bundle` — acceptable without definition only if "Flatpak" itself is already defined

A link to an authoritative reference counts as a definition.

---

## Category 2 — AI-Free Codebase

This project uses AI as a development instrument, but the codebase itself must not carry any
explicit reference to AI tools or AI authorship. Such references are:

- **Not neutral** — they imply authorship claims or workflow dependencies that may change
- **Not human-first** — a reader wanting to understand the code must not need to know which tool
  wrote it; the code speaks for itself

### 2.1 — AI tool mentions in tracked files

Scan all files in the audit scope (§What to scan) for the following terms (case-insensitive).
Read each file and check line by line — do not rely solely on grep output:

```
claude
chatgpt
openai
gpt-4
gpt-3
copilot
gemini
codeium
tabnine
cursor (as an AI code editor reference, not a UI cursor)
llm
large language model
ai-generated
ai-assisted
generated by ai
written by ai
anthropic
co-authored-by.*claude
co-authored-by.*openai
co-authored-by.*anthropic
```

Flag every match with its file path, line number, and the matched line.

**Exemption:** `.claude/` directory tree. Git commit messages are also exempt from this check
(they are not part of the distributed codebase source).

### 2.2 — AI attribution in commit messages (informational only)

Run `git log --oneline | head -100` and note (do not flag as a violation) any commit messages
containing `Co-Authored-By: Claude`. Report the count in the audit document under an "Informational"
subsection — this is expected from AI-assisted development and is acceptable in git history.

### 2.3 — Bot emoji and markers

Flag occurrences of `🤖`, `[AI]`, `[AUTO]`, `[GENERATED]` in any `*.md` or `*.rs` file outside `.claude/`.

---

## Category 3 — Visualization Quality

Diagrams must prioritize clarity and semantic expressiveness. The preference order is:

1. **Mermaid** (`flowchart`, `sequenceDiagram`, `classDiagram`, `stateDiagram`, `erDiagram`,
   `gitGraph`) — first choice for architecture, flows, and state machines
2. **Other markdown-native diagrams** — tables, definition lists for taxonomies
3. **SVG references** — for visual assets (icons, screenshots)
4. **ASCII art** — only acceptable for terminal session output examples (e.g., showing CLI output)
   and threading time-sequence diagrams where vertical alignment is critical

### 3.1 — ASCII art replaceable by Mermaid

Flag any block in `*.md` files that uses ASCII art for **structural or conceptual diagrams** —
identified by patterns like `+--+`, repeated `─`, `│`, `▶`, `◀` used as box-drawing borders,
or `[Box] --> [Box]` text flows. For each match, suggest the Mermaid diagram type that would
replace it (e.g., `flowchart LR`, `sequenceDiagram`, `classDiagram`).

Terminal output and threading ASCII timelines (e.g., `GTK Main Thread │ Worker Thread`) are
exempt if they use vertical column alignment to show concurrency — this structure cannot be
expressed in Mermaid without losing the parallel-column semantics.

### 3.2 — Mermaid syntax validity

For every `mermaid` fenced code block in `*.md` files, check for common structural errors:
- Missing graph type declaration (`flowchart`, `graph`, `sequenceDiagram`, etc.) as the first line
- Unclosed subgraph blocks
- Node IDs containing spaces without quotes
- Arrow syntax mixing (`-->` vs `->` vs `—>`)

Do not run an external validator; flag red flags found by pattern inspection.

### 3.3 — Undocumented architecture without any diagram

Flag any `*.md` document that contains **three or more** of the following trigger phrases without
any diagram (Mermaid or otherwise) in the same section:
- "depends on", "is called by", "calls into"
- "wires", "is wired to", "wires up"
- "inherits from", "extends", "wraps"
- "delegates to", "proxies", "routes through"
- "is the composition root", "instantiates", "constructs"

Severity: LOW. Suggest the Mermaid diagram type that would capture the relationship.

---

## Category 4 — Code Comment Quality

Comments in `src/**/*.rs` must explain *why*, not *what*.

### 4.1 — What-comments

Flag single-line or multi-line comments that:
- Restate the function or variable name in prose (e.g., `// Get the container list` above
  `fn get_containers()`)
- Describe the mechanics of the next line without adding context (e.g., `// Loop over items`)
- Begin with "This function", "This method", "This struct", "This module"

### 4.2 — Task-reference comments

Flag comments that tie the code to a specific issue, PR, or task:
- `// added for issue #N`, `// from PR #N`, `// TODO(username)`, `// see ticket`
- These belong in git commit messages, not in source code

### 4.3 — Overlong comment blocks

Flag consecutive comment blocks longer than **3 lines** in `src/**/*.rs`. These are high-probability
candidates for what-comments. Report file and line range; do not automatically remove them —
let the human judge whether the why is genuinely complex enough to warrant the length.

### 4.4 — Dead markers

Flag in `src/**/*.rs`:
- `TODO`, `FIXME`, `HACK`, `XXX`, `NOCOMMIT` tags in committed code
- Commented-out code blocks (≥ 3 consecutive `//`-prefixed lines that parse as code, not prose)

---

## Category 5 — Terminology Consistency

Canonical names for concepts must be used consistently across all `*.md` and `*.rs` comment text.

### 5.1 — Forbidden variants

| Canonical term | Forbidden variants to flag |
|---|---|
| `container runtime` | `container engine`, `docker engine` (in prose) |
| `IContainerDriver` | `ContainerDriver`, `DriverInterface`, `driver interface` |
| `spawn_driver_task` | `spawnTask`, `spawn_task`, `driver_task`, `spawned task` |
| `AdwNavigationSplitView` | `split view` (unqualified), `navigation split`, `SplitView` |
| Hexagonal Architecture | `hex arch`, `Ports-and-Adapters`, `ports & adapters` (non-canonical capitalisation) |
| `GResource` | `gresource`, `g-resource` (in prose; lowercase fine in code string literals) |
| `Composition root` | `wire-up`, `main wiring`, `dependency wiring` (in documentation prose) |
| `GNOME Platform` | `GNOME SDK` (as the runtime name; SDK is the build tool, Platform is the runtime) |
| `containerd/nerdctl` | `nerdctl` alone (when referring to the runtime; nerdctl is the CLI, not the runtime) |
| `Flatpak-first` | `flatpak-first`, `Flatpak first` (unhyphenated in prose) |

Check in: all `*.md` files and `// comments` in `src/**/*.rs`.

### 5.2 — Mixed language in documentation

The project's primary documentation language is **English**. Flag any sentence-level prose in
non-English languages appearing in `README.md`, `CONTRIBUTING.md`, `CHANGELOG.md`, `CLAUDE.md`,
`SECURITY.md`, or `GOVERNANCE.md`. (Translation files `po/*.po` are explicitly exempt.)

---

## Category 6 — Placeholder and Draft Content

### 6.1 — Unexpanded template markers

Flag in any tracked file:
- `[PLACEHOLDER]`, `[TODO]`, `[TBD]`, `[INSERT]`, `[FIXME]`, `[DRAFT]`
- Double-brace variables: `{{variable}}`, `{YOUR_VALUE}`
- Angle-bracket placeholders: `<your-name>`, `<your-email>`, `<maintainer>`, `<license-holder>` (outside
  code examples where they are intentional instructional placeholders)
- `YOUR_`, `CHANGE_ME`, `EDIT_ME` strings

### 6.2 — Lorem ipsum and filler text

Flag any occurrence of `lorem ipsum`, `dolor sit amet`, `foo`, `bar`, `baz` (in prose, not in
code examples or test fixtures where they are standard identifiers).

### 6.3 — Screenshots placeholder

If `README.md` contains a prose invitation to add screenshots (e.g., `> Add screenshots`) and
`docs/screenshots/` either does not exist or is empty, flag this as MEDIUM. If screenshots exist,
flag the placeholder prose as stale.

### 6.4 — App identity placeholders

The project App ID `com.example.GtkCrossPlatform` and application name `GtkCrossPlatform` are
explicitly marked as placeholder values in `CLAUDE.md`. Flag every occurrence of these strings in
prose documentation (`*.md`), data files (`*.xml`, `*.desktop`), and `Cargo.toml` as **CRITICAL**
when they appear in a context where a real identity would be expected (name fields, store listings,
about dialogs, metainfo). Code and configuration paths that are purely technical identifiers (e.g.,
GResource paths, D-Bus service names) are LOW severity.

---

## Category 7 — Link and Reference Integrity

### 7.1 — Internal markdown links

For every `[text](path)` link in `*.md` files where the path is a relative file path (not a URL),
verify the target file exists using `git ls-files` or a file existence check. Flag broken paths.

### 7.2 — External URL health (informational)

Do **not** attempt to fetch external URLs — this command is read-only and offline-safe.
Instead, flag any URL in documentation that:
- Points to `example.com` used as a real link (not a placeholder code example)
- Uses a bare IP address as a domain in a non-example context
- References a GitHub resource via `/blob/master/` or `/tree/master/` when the repository's
  default branch is `main` (run `git symbolic-ref refs/remotes/origin/HEAD` or check the
  remote to confirm the default branch before flagging)

---

## Category 8 — Version Consistency

Version numbers must be consistent across the four authoritative sources.

### 8.1 — Cross-file version alignment

Read `Cargo.toml` and extract the `version = "X.Y.Z"` field. Then verify:

| File | Check |
|---|---|
| `Cargo.toml` | Source of truth; read this first |
| `CHANGELOG.md` | Must have a `## [X.Y.Z]` release header for the current version; if the version is a prerelease, an `## [Unreleased]` section must exist |
| `data/com.example.GtkCrossPlatform.metainfo.xml` | `<release version="X.Y.Z" ...>` must match Cargo.toml |
| `README.md` | If the README hard-codes a version badge or version number, it must match Cargo.toml |

Flag mismatches as HIGH. Flag a missing `[Unreleased]` section during active development as MEDIUM.

### 8.2 — CHANGELOG completeness

Flag in `CHANGELOG.md`:
- An `[Unreleased]` section that is empty (no bullet points under any sub-header)
- A `[Unreleased]` section that exists but has no release date or comparison link at the bottom
- Version headers using a format inconsistent with `Keep a Changelog` (`## [X.Y.Z] - YYYY-MM-DD`)

---

## Severity Classification

| Severity | Meaning |
|---|---|
| **CRITICAL** | Violates an explicit project rule; blocks release or misleads contributors |
| **HIGH** | Degrades human readability or introduces factual inaccuracy |
| **MEDIUM** | Reduces maintainability or consistency; unlikely to block work |
| **LOW** | Style debt or improvement suggestion; fix opportunistically |

---

## Output: `.claude/docs/content-audit.md`

Create `.claude/docs/` if it does not exist, then create or overwrite `.claude/docs/content-audit.md`
using the structure below. Preserve the `## Resolved Findings` table verbatim from any prior run.

````markdown
# Content Audit

> Last run: YYYY-MM-DD
> Branch: <branch name>
> Files scanned: N (from `git ls-files`)

## Summary

| Category | Findings | Highest severity |
|---|---|---|
| Human-First Readability | N | — |
| AI-Free Codebase | N | — |
| Visualization Quality | N | — |
| Code Comment Quality | N | — |
| Terminology Consistency | N | — |
| Placeholder and Draft Content | N | — |
| Link and Reference Integrity | N | — |
| Version Consistency | N | — |
| **Total** | **N** | |

## Files Checked

List every file that was explicitly read (not just grepped) per scope group:

- **Documentation (*.md):** file1.md, file2.md, ...
- **Code comments (*.rs):** src/app.rs, src/core/..., ...
- **UI / data files:** data/...
- **Shell scripts:** Makefile, .github/workflows/...
- **Translation:** po/...

## Findings

### CRITICAL

#### [Cx.y-NNN] [Category] — [Short title]
- **File:** `path/to/file.md:42`
- **Finding:** one sentence describing the violation and why it matters
- **Suggested fix:** one sentence — concrete action to resolve it

### HIGH
...

### MEDIUM
...

### LOW
...

## Informational

### AI attribution in commit history

> N commits in the last 100 contain `Co-Authored-By: Claude` — expected from AI-assisted
> development workflow; not a violation.

## Resolved Findings

<!-- Append resolved entries here; do not delete rows -->

| Date | Finding ID | Severity | Category | File | Resolution |
|---|---|---|---|---|---|
````

---

## Execution notes

- Run `git ls-files` at the start to get the canonical file list; do not scan untracked or gitignored files.
- **Read each file explicitly with the Read tool** before checking it. Do not trust memory or inferred content.
- Do not auto-fix findings. This command is read-only — it discovers and reports only.
- For Category 2 (AI-Free), err on the side of flagging: a false positive is better than a missed
  explicit AI reference in a distributed document.
- For Category 3 (Visualization), only flag ASCII art in documentation files (`*.md`), not in
  code (`*.rs`) where terminal-style diagrams in comments are acceptable.
- For Category 6.4 (App identity), CRITICAL applies to name/identity fields; LOW applies to technical
  path identifiers (D-Bus names, GResource prefixes, schema IDs) — use judgement.
- Count findings per category and populate the Summary table before writing individual finding blocks.
- After writing the report, print a one-paragraph terminal summary: CRITICAL count, HIGH count, the
  most impactful file, and the single most urgent fix.
