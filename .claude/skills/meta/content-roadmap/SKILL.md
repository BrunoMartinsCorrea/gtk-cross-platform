---
name: meta:content-roadmap
version: 1.0.0
description: Build or refresh a prioritised improvement plan from the latest content audit
---

# meta:content-roadmap

Build or refresh the **content improvement plan** from the latest content audit.
This command is self-contained — run it fresh without prior conversation context.

Read `.claude/docs/content-audit.md` (produced by `/meta:content-audit`), translate every
open finding into an actionable task, prioritise them, estimate effort, and write the result to
`.claude/docs/content-improvement-plan.md`.

The plan document is **persistent**: re-running this command merges new findings with the existing
plan — it preserves `Completed` and `In Progress` rows and only adds or updates `Pending` rows.

---

## Input Acceptance Criteria

Before doing any work, verify the input is valid. If any criterion fails, stop immediately and
report which criterion failed — do not attempt to produce a plan.

| #  | Criterion                                                                                                                           | How to verify                                                                                                       |
|----|-------------------------------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------|
| I1 | `.claude/docs/content-audit.md` exists                                                                                              | Use the Read tool; if the file does not exist, fail with "Run `/meta:content-audit` first."                         |
| I2 | The file contains a `## Summary` section with a populated findings table                                                            | Check that the table has at least one numeric cell that is not `0` or `—` in the "Findings" column                  |
| I3 | The file contains a `## Findings` section with at least one severity block (`### CRITICAL`, `### HIGH`, `### MEDIUM`, or `### LOW`) | Check for the presence of at least one such header                                                                  |
| I4 | Every finding block has a finding ID matching `[Cx.y-NNN]`                                                                          | Scan for the pattern; if the format is absent, warn that the audit may be from an incompatible version but continue |
| I5 | The audit has a `> Last run:` date within the last **30 days** (relative to today)                                                  | Parse the date line; if older, warn "Audit may be stale — consider re-running `/meta:content-audit`" and continue   |

---

## Execution Strategy

Execute in this exact order:

1. **Verify input** against all I1–I5 criteria. Stop on I1–I3 failure; warn on I4–I5 and continue.
2. **Read the existing plan** (if `.claude/docs/content-improvement-plan.md` exists) and extract all rows with status
   `Completed` or `In Progress`. Store them for preservation.
3. **Parse the audit**: extract every finding from `## Findings` — its ID, severity, category, file, and suggested fix.
4. **Cross-reference** with the existing plan: any finding ID already present in the plan retains its current status and
   assignee; new IDs get status `Pending`.
5. **Compute effort** for each finding using the Effort Estimation rules (§Effort Estimation).
6. **Group and sort** findings using the Priority Rules (§Priority Rules).
7. **Compute the Quick Wins list** (§Quick Wins).
8. **Write the complete plan** to `.claude/docs/content-improvement-plan.md` in one Write operation using the Output
   Template (§Output Template).
9. **Print a terminal summary**: total open tasks, CRITICAL count, HIGH count, quick win count, and the single most
   impactful task to tackle first.

---

## Effort Estimation

Assign an effort label to each finding based on its scope. Use the following rules in priority order
(first matching rule wins):

| Label | Criteria                                                                                                           |
|-------|--------------------------------------------------------------------------------------------------------------------|
| `XS`  | Single-line change: fix one word, one URL, one acronym definition, one version number                              |
| `S`   | Single-file change, ≤ 10 lines: add a parenthetical, fix one paragraph, add a section header                       |
| `M`   | Single-file change, > 10 lines, or two related files: rewrite a section, replace ASCII art with a Mermaid diagram  |
| `L`   | Multi-file change (3–6 files): terminology sweep, placeholder replacement across files, structural reorder         |
| `XL`  | Systemic change (> 6 files or requires code + docs coordination): full placeholder replacement, new diagram system |

**Override rules:**

- Any finding in Category 6.4 (App identity placeholders) is always `XL` — touching App ID requires Cargo.toml,
  metainfo, desktop file, and all documentation simultaneously.
- Any finding in Category 3.1 that involves replacing ASCII art with a Mermaid diagram is at least `M`.
- Any finding in Category 2.1 (AI mentions) is at most `S` — it is always a targeted removal.

---

## Priority Rules

Sort the plan tasks in this order:

1. **CRITICAL** findings first, sorted within severity by effort ascending (fix cheap CRITICAL things first).
2. **HIGH** findings, sorted by effort ascending.
3. **MEDIUM** findings, sorted by effort ascending.
4. **LOW** findings, sorted by effort ascending.

Within the same severity + effort bucket, sort by finding ID ascending.

**Deprioritisation exceptions** (move to bottom of their severity group regardless of effort):

- Category 6.4 (App identity) findings: these require external decisions (real app name, reverse domain) and cannot be
  resolved without product input. Mark with a `⚠ blocked: needs product decision` note.
- Category 7.2 (External URL health) findings flagged as informational only.

---

## Quick Wins

A task is a **Quick Win** if it meets all three conditions:

1. Effort is `XS` or `S`
2. Severity is `HIGH` or `CRITICAL`
3. Status is `Pending`

List quick wins in a dedicated `## Quick Wins` section at the top of the plan body (before the
full task table). Limit to 5 items. If there are more than 5, pick the 5 with the highest severity
and smallest effort. Quick wins are also included in the main task table — the section is a copy,
not a replacement.

---

## Output Template

Write `.claude/docs/content-improvement-plan.md` with this exact structure:

````markdown
# Content Improvement Plan

> Last updated: YYYY-MM-DD
> Audit date: YYYY-MM-DD (from `> Last run:` in content-audit.md)
> Open tasks: N (CRITICAL: n, HIGH: n, MEDIUM: n, LOW: n)
> Completed tasks: N

## Input status

| Criterion | Result |
|-----------|--------|
| I1 — Audit file exists | ✓ Pass / ✗ Fail |
| I2 — Summary table populated | ✓ Pass / ✗ Fail |
| I3 — Findings sections present | ✓ Pass / ✗ Fail |
| I4 — Finding IDs conform to [Cx.y-NNN] | ✓ Pass / ⚠ Warn |
| I5 — Audit date within 30 days | ✓ Pass / ⚠ Warn (age: N days) |

## Quick Wins

Tasks that are `HIGH` or `CRITICAL`, effort `XS` or `S`, and status `Pending`.
Resolve these first — high impact, low cost.

| ID | Severity | File | Fix | Effort |
|----|----------|------|-----|--------|
| [Cx.y-NNN] | CRITICAL | `path/to/file.md:42` | One sentence from "Suggested fix" | XS |
| ... | | | | |

## Full Task List

| ID | Severity | Category | File | Fix summary | Effort | Status | Notes |
|----|----------|----------|------|-------------|--------|--------|-------|
| [Cx.y-NNN] | CRITICAL | Human-First Readability | `path/to/file.md:42` | One sentence | XS | Pending | |
| [Cx.y-NNN] | HIGH | AI-Free Codebase | `path/to/file.rs:10` | One sentence | S | In Progress | |
| [Cx.y-NNN] | MEDIUM | Visualization Quality | `CLAUDE.md:100` | One sentence | M | Completed | Fixed in commit abc1234 |
| ... | | | | | | | |

> Status values: `Pending` · `In Progress` · `Completed` · `Blocked`
> Effort labels: `XS` · `S` · `M` · `L` · `XL`

## Completed

<!-- Rows migrated here from the Full Task List when status = Completed -->
<!-- Format: same columns as Full Task List, plus a Resolution column -->

| ID | Severity | Category | File | Resolution | Completed date |
|----|----------|----------|------|------------|----------------|

## Blocked

Tasks that cannot be resolved without external decisions or information.

| ID | Severity | Category | File | Blocker |
|----|----------|----------|------|---------|
| [C6.4-NNN] | CRITICAL | Placeholder — App identity | `Cargo.toml` | ⚠ Needs product decision: real app name + reverse domain |

## Plan statistics

| Metric | Value |
|--------|-------|
| Total findings in audit | N |
| Tasks in this plan | N |
| Quick wins available | N |
| Estimated XS tasks | N |
| Estimated S tasks | N |
| Estimated M tasks | N |
| Estimated L tasks | N |
| Estimated XL tasks | N |
| Blocked (needs decision) | N |
````

---

## Output Acceptance Criteria

After writing the file, verify these conditions. If any fails, append a `## Plan validation errors`
section to the output file listing the failures — do not silently produce a broken plan.

| #  | Criterion                                                                                           | How to verify                                                         |
|----|-----------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------|
| O1 | Every open finding from the audit has a corresponding row in the Full Task List                     | Count finding IDs in the audit vs rows in the plan; counts must match |
| O2 | Every row in the Full Task List has a non-empty ID, Severity, File, Fix summary, Effort, and Status | Scan each row                                                         |
| O3 | Effort labels are one of `XS`, `S`, `M`, `L`, `XL` exclusively                                      | Validate each Effort cell                                             |
| O4 | Status values are one of `Pending`, `In Progress`, `Completed`, `Blocked` exclusively               | Validate each Status cell                                             |
| O5 | The Quick Wins section contains only tasks that appear in the Full Task List                        | Cross-reference IDs                                                   |
| O6 | The Quick Wins section contains at most 5 rows                                                      | Count rows                                                            |
| O7 | Completed and In Progress rows from a prior run are preserved with their original status            | Compare preserved rows against the plan; IDs must still be present    |
| O8 | The `> Open tasks:` header count matches the count of non-Completed rows in the Full Task List      | Recount                                                               |
| O9 | No row has both status `Completed` and a blank Resolution in the Completed table                    | Scan Completed table                                                  |

---

## Execution notes

- This command is **documentation-only** — do not modify any file outside `.claude/docs/`.
- Do not auto-fix findings. This command plans; `/meta:apply-docs-fixes` and similar commands execute.
- When merging with an existing plan, trust the existing `Status` and `Notes` columns — do not reset them.
- If the audit contains zero findings, write the plan with an empty Full Task List and note "No open findings — content
  audit passed cleanly."
- The `Fix summary` column must be derived from the audit's `Suggested fix` field, condensed to one sentence. Do not
  invent fixes.
- For findings without a `Suggested fix` in the audit, write `See audit finding [ID] for context`.
- Print the terminal summary after writing the file, not before — only confirm output after the write succeeds.
