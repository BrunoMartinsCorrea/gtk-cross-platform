---
description: Analyze a repository and scaffold complete OSS documentation (CLAUDE.md, VISION.md, CONTRIBUTING.md, CHANGELOG.md, docs/architecture/, docs/specs/). Run from the repo root.
---

You are an expert open source software architect and technical writer. Analyze this repository
and produce a complete OSS documentation structure following FLOSS best practices.

> **How to read this document:** Every section has a **Template** (the skeleton to fill in) and
> an **Example** (a filled-in instance). Use the Example to calibrate verbosity and specificity.
> Never produce output more generic than the Example shows.

---

## Phase 1 — Discovery (read-only, no writes yet)

Scan the repository and produce a Discovery Report before creating any file.

1. **File tree** — language, framework, entry points, test structure, build system, existing docs
2. **Existing documentation** — README, docs/, comments in key files, package manifest
3. **Architecture** — trace call graph from entry point; identify layers and separation of concerns
4. **Tech stack** — all direct dependencies and their purpose
5. **Conventions** — linters, formatters, CI configs, commit patterns, test structure
6. **Gaps** — what is missing or undocumented

Print this report before doing anything else:

```
=== DISCOVERY REPORT ===

Project name: <name>
Primary language: <lang + version>
Architecture pattern: <detected pattern or "unclear">
Entry point: <file>
Test structure: <location and pattern>
Build system: <make/npm/gradle/etc>
License: <detected or "MISSING">
Existing docs: <list files>

Gaps found:
- <gap 1>
- <gap 2>

Files to create:
- <file> — <reason>

Files to update:
- <file> — <what changes>

Proceed? (waiting for confirmation)
```

**Stop and wait for user confirmation before Phase 2.**

---

## Phase 2 — File generation

After user confirms, create or update the files below. Adapt all content to what was discovered —
never use placeholders like "TODO" or "describe your project here". Every field must contain
real, inferred content.

### 2.1 — CLAUDE.md (highest priority)

The Claude Code agent reads this automatically. It must be dense, precise, and written for an
AI agent — not for humans. Include:

- Project overview (2–4 lines: what it is, who it's for, current status)
- Goals and Non-goals (Non-goals are mandatory — infer from what the project clearly does NOT do)
- Architecture (layers, directory tree with inline comments)
- Tech stack: Core + Dev tooling + **Deliberately avoided** (name at least one with a technical reason)
- Development setup (prerequisites, quickstart commands, env vars)
- Code style (formatter, naming, error handling, commit convention)
- Testing (unit vs integration, how to run, coverage expectations)
- Key files (8–12 entries: real paths, what each does, when to touch it)
- "To add a new X" recipe (infer X from project type)
- Constraints (platform, dependencies, security)

**Rules:** no placeholder text; non-goals are explicit and specific; key files reference real paths.

### 2.2 — VISION.md

```markdown
# Vision

## Problem statement

<what pain this project addresses>

## Solution approach

<philosophy, not features>

## Target users

<specific, not "developers" — e.g., "backend engineers who run containers locally">

## Goals (prioritized)

1. <most important long-term goal>
2. <second>
3. <third>

## Non-goals

- <what this project will never try to be>

## Success metrics

- <how we know the project is succeeding>
```

### 2.3 — CONTRIBUTING.md

Update if shallow; create if missing. The **commit scope table is mandatory** — it prevents
commits that don't map to the module structure.

Required sections:

- Development workflow (fork → branch → test → commit → PR)
- Commit convention (Conventional Commits format)
- **Scope table** derived from top-level module structure — every `src/` directory gets a scope
- PR checklist (tests, lint, fmt, CHANGELOG, UI at narrow widths if applicable)

### 2.4 — docs/architecture/README.md

Include:

- 2–3 paragraph overview at C4 Container level
- C4 Level 1 ASCII or Mermaid context diagram
- Layer responsibilities table (Layer | Path | Responsibility)
- Data flow description (numbered steps)
- Extension points (where and how to extend)

### 2.5 — docs/architecture/decisions/ADR-001-\<topic\>.md

One ADR for the most significant architectural decision. Format:

```markdown
# ADR-001: <title>

## Status: Accepted

## Date: <infer from git log or use today>

## Context

## Decision

## Consequences

### Positive

### Negative / trade-offs

## Alternatives considered
```

### 2.6 — docs/specs/RFC-\<NNN\>-\<feature\>.md

One RFC per major domain. The **Error cases section is mandatory** — if the system makes I/O
calls, there are failure modes. "None known" is not acceptable.

```markdown
# RFC-NNN: <feature>

## Status: Accepted

## Context

## Specification

### Inputs

### Processing

### Outputs

### Error cases ← mandatory, non-empty table

## Implementation notes ← real file paths

## Open questions
```

### 2.7 — CHANGELOG.md

Follow Keep a Changelog format. The `[Unreleased]` section must list every gap from Phase 1.

```markdown
# Changelog

Format: Keep a Changelog · Versioning: Semantic Versioning

## [Unreleased]

### Added

### Changed

### Fixed

## [<latest version from git tags>] — <date>

### Added

- Initial release
```

### 2.8 — LICENSE

If no LICENSE file exists, ask the user which license to apply before creating it. Provide
a recommendation with a one-sentence rationale based on the project type. Do not create the
file until the user explicitly selects an option.

---

## Phase 3 — Validation

After all files are created, perform a self-check:

```
=== VALIDATION REPORT ===

Files created:
✓/✗ CLAUDE.md
✓/✗ VISION.md
✓/✗ CONTRIBUTING.md
✓/✗ docs/architecture/README.md
✓/✗ docs/architecture/decisions/ADR-001-*.md
✓/✗ docs/specs/*.md (N files)
✓/✗ CHANGELOG.md
✓/✗ LICENSE

Content checks:
✓/✗ CLAUDE.md has zero placeholder text
✓/✗ Non-goals are explicit and specific
✓/✗ Key files map references real paths (verified against file tree)
✓/✗ Tech stack entries have one-line justifications
✓/✗ "Deliberately avoided" has at least one entry with a technical reason
✓/✗ Commit scope table covers all top-level src/ directories
✓/✗ ADR references a real file or decision visible in the codebase
✓/✗ RFC error cases table is non-empty
✓/✗ All cross-references point to paths that were created

Suggested next steps:
1. <specific action>
2. Run <build command> to verify no compilation errors
```

---

## Behavioral rules

- **Never hallucinate paths.** Only reference files that exist in the repository.
- **Never use placeholders.** Every field must contain real, inferred content.
- **Preserve existing content.** When updating files, append or refine — never delete content that was already there.
- **Infer, don't ask (except license).** Use the codebase to fill all fields.
- **Scope table is mandatory.** CONTRIBUTING.md must always contain a commit scope table.
- **Error cases are mandatory.** Every RFC must have a non-empty Error cases section.
- **Match example density.** If the Example names a file path, your output must also name a real file path.
