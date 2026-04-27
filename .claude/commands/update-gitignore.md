---
description: Audit .gitignore against the project's actual file extensions and community best practices for the detected stack, then apply the consolidated result.
---

You are a build-system and VCS expert. Your job is to produce a single, well-commented `.gitignore`
that covers every artifact the project can generate, without over-ignoring files that belong in the
repository.

---

## Phase 1 — Discovery (read-only)

Run the following analysis and print the report below before touching any file.

### 1.1 — Collect all file extensions present in the working tree

```bash
find . \
  -not -path './.git/*' \
  -not -path './target/*' \
  -not -path './.flatpak-builder/*' \
  -not -path './flatpak-build/*' \
  -not -path './repo/*' \
  -type f \
  | sed 's/.*\.//' \
  | sort -u
```

Also list every **extension-less file** (Makefile, LICENSE, AUTHORS, etc.) so nothing is missed.

### 1.2 — Read the current `.gitignore`

Print every pattern already declared.

### 1.3 — Detect the tech stack

Infer from `Cargo.toml`, manifest files, build scripts, and CI configs which tools are in use.
For each tool/runtime detected, name the canonical gitignore template from
[github.com/github/gitignore](https://github.com/github/gitignore) that should be consulted.

### 1.4 — Gap analysis

Compare current patterns against:

- The file extensions found in step 1.1
- The official gitignore templates for the detected stack
- The categories below (community minimum for any serious project):

| Category              | Patterns to check                                        |
|-----------------------|----------------------------------------------------------|
| Language build output | Compiled binaries, object files, generated source        |
| Dependency caches     | Fetched packages, lock-file-adjacent caches              |
| Test / coverage       | Coverage reports, snapshot diffs, test databases         |
| IDE / editor          | JetBrains, VS Code, Vim, Emacs, Xcode, Sublime           |
| OS artifacts          | `.DS_Store`, `Thumbs.db`, `desktop.ini`, `ehthumbs.db`   |
| Secrets / env         | `.env`, `.env.*`, `*.pem`, `*.key`, `*.p12`, `secrets.*` |
| Build tools           | Meson/Ninja, CMake, Autotools, Flatpak builder           |
| Distribution          | Flatpak bundles, AppImage, Snap, `.deb`, `.rpm`          |
| Logs                  | `*.log`, crash dumps, sanitizer output                   |
| Temp files            | `*.tmp`, `*.bak`, `*.orig`, editor swap files            |

Print the discovery report:

```
=== GITIGNORE AUDIT REPORT ===

Extensions found in working tree:
  <comma-separated list>

Extension-less tracked files:
  <list>

Stack detected:
  <tool/runtime> → template: <template name or URL>

Current .gitignore patterns (<N> entries):
  <list>

GAPS — patterns missing or incomplete:
  [MISSING]  <pattern>  — reason: <why it should be added>
  [WEAK]     <pattern>  — current: <existing>, better: <proposed>
  [REDUNDANT] <pattern> — already covered by <other pattern>

Proceed with applying changes? (waiting for confirmation)
```

**Stop and wait for user confirmation before Phase 2.**

---

## Phase 2 — Apply

After the user confirms, rewrite `.gitignore` in place using this structure.
Keep every existing pattern that is not redundant. Add all identified gaps.
Group entries under the section headers below (add sections only if relevant to this project).
Each section must open with a `# ---` separator and a title comment.

```
# --- Rust / Cargo ---
target/
Cargo.lock           # remove this line for libraries; keep for binaries and applications

# --- Build systems ---
# Meson / Ninja
build/
builddir/
_build/
.ninja_deps
.ninja_log

# CMake (if present)
CMakeCache.txt
CMakeFiles/
cmake_install.cmake

# --- GTK / GLib generated files ---
# Compiled GResource bundles (regenerated at build time)
*.gresource
# Vala-generated C/H intermediaries (never edit directly)
*.c
*.h
*.o
*.vapi

# --- Flatpak / AppStream ---
.flatpak-builder/
flatpak-build/
repo/
*.flatpak
*.oci

# --- Distribution packages ---
*.deb
*.rpm
*.AppImage

# --- Secrets and credentials ---
.env
.env.*
!.env.example
*.pem
*.key
*.p12
*.pfx
secrets.*

# --- Test / coverage artifacts ---
lcov.info
coverage/
tarpaulin-report.*
*.profraw
*.profdata

# --- Logs and crash dumps ---
*.log
crash-*.txt
sanitizer-*.txt

# --- Temporary and editor swap files ---
*.tmp
*.bak
*.orig
*.swp
*~
\#*\#
.\#*

# --- IDE and editor ---
# JetBrains
.idea/
*.iml
*.iws
out/

# VS Code
.vscode/
*.code-workspace

# Xcode
*.xcodeproj/
*.xcworkspace/
DerivedData/

# Sublime Text
*.sublime-project
*.sublime-workspace

# --- OS artifacts ---
# macOS
.DS_Store
.AppleDouble
.LSOverride
._*

# Windows
Thumbs.db
ehthumbs.db
desktop.ini
$RECYCLE.BIN/

# Linux
.directory
.Trash-*

# --- Project-local tooling (not published source) ---
.prompt/
```

Rules for the rewrite:

- Do **not** add patterns that would accidentally ignore tracked source files (`*.rs`, `*.ui`, `*.toml`, `*.md`, etc.).
- Do **not** add patterns that are already handled by a broader glob already in the file — consolidate instead.
- Add a one-line comment after any non-obvious pattern explaining why it is ignored.
- Preserve any project-specific patterns that exist today (e.g., `.prompt/`).
- If `Cargo.lock` is present and this is a binary application (not a library crate), keep it tracked; add a note.

---

## Phase 3 — Validation

After writing the file, run:

```bash
git check-ignore -v $(git ls-files) 2>/dev/null | head -20
```

If any tracked source file is matched, print a warning and remove the offending pattern.

Then print:

```
=== VALIDATION REPORT ===

Patterns added:   <N>
Patterns removed: <N> (redundant or overly broad)
Patterns kept:    <N>

Tracked files accidentally ignored: <none / list>

Sections in new .gitignore:
  ✓ Rust / Cargo
  ✓/✗ Build systems
  ✓/✗ GTK / GLib
  ✓/✗ Flatpak
  ✓/✗ Secrets
  ✓/✗ Test artifacts
  ✓/✗ Logs
  ✓/✗ Temp / swap
  ✓/✗ IDE / editor
  ✓/✗ OS artifacts
  ✓/✗ Project-local tooling

Suggested next steps:
  1. Run `git status` to verify no previously-tracked file is now ignored.
  2. Commit with: git add .gitignore && git commit -m "chore: update .gitignore for <stack> stack"
```
