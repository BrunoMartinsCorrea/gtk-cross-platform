---
name: meta:format-makefile
version: 1.0.0
description: Audits and formats the project Makefile against the canonical style defined in .claude/rules/standards/makefile.md — runs the full audit-fix-verify cycle and reports violations found and fixed.
---

# meta:format-makefile

Invoke with `/meta:format-makefile` to run the full audit-fix-verify cycle on `Makefile`.

The style rules are defined in `.claude/rules/standards/makefile.md` (auto-loaded whenever
Claude edits `Makefile`). This skill orchestrates their systematic application.

## When to use

- After adding or renaming targets (`.PHONY` may be incomplete)
- After a section grows and misaligns variable declarations
- Before opening a PR that touches `Makefile`
- When `make help` output looks inconsistent

## Process

### Step 1 — Read and parse `Makefile`

Read the full file. For each rule in `.claude/rules/standards/makefile.md`, scan for violations:

| Rule                  | What to scan                                                            |
|-----------------------|-------------------------------------------------------------------------|
| Variable alignment    | `:=` / `?=` column positions in the top block                           |
| Section headers       | Width (must be 80 chars), separator character (`─`), blank lines around |
| Help comments         | Public targets (non-`_` prefix) without a `## …` inline comment         |
| Tab indentation       | Recipe lines starting with spaces instead of a real tab                 |
| Silent prefix         | `@echo` / `@…` presence; bare `cargo`/`flatpak` commands not silenced   |
| Continuation `\`      | Indentation consistency of continuation lines                           |
| `.PHONY` completeness | Targets defined in the file but absent from `.PHONY`                    |
| Target naming         | Non-kebab-case names; missing namespace prefix                          |
| Trailing whitespace   | Lines ending with space or tab                                          |

### Step 2 — Report violations

Produce a table before making any edits:

```
| Rule              | Line | Violation                                              |
|-------------------|------|--------------------------------------------------------|
| Variable alignment| 5    | BINARY := col 9, APP_ID := col 18 — misaligned         |
| Help comment      | 72   | Target `schema` has no ## comment                      |
| .PHONY            | —    | `icons-png`, `watch` defined but missing from .PHONY   |
```

If there are no violations, report that and stop — do not modify the file.

### Step 3 — Apply fixes

Fix each violation in place. Apply in this order to avoid cascading edits:

1. `.PHONY` block — add missing targets, sort alphabetically within each section group
2. Variable alignment — adjust `:=` column across the top block
3. Section headers — fix width and separator character
4. Help comments — add missing `## …` to public targets
5. Tab indentation — convert leading spaces to real tabs on recipe lines
6. Trailing whitespace — strip from all lines

### Step 4 — Verify

After applying all fixes:

```sh
make help          # verify all public targets appear with their description
make --dry-run ci  # verify the Makefile parses without errors
```

If `make --dry-run ci` fails, revert only the offending change and report it as
an unresolved violation.

## Output format

```
meta:format-makefile — 7 violations fixed, 0 remaining.

Fixed:
  • Variable alignment: aligned 14 declarations (col 18)
  • Help comment: added ## comment to `schema`, `icons-png`
  • .PHONY: added `schema`, `icons-png`, `watch`

Verified:
  ✓ make help — 42 targets listed
  ✓ make --dry-run ci — parses cleanly
```

No violations found:

```
meta:format-makefile — Makefile is already well-formed. No changes made.
```
