---
description: Context feedback loop — how the agent must handle errors caused by missing project context. Defines the virtual incident/postmortem process and criteria for persisting new context into the artifact structure.
---

# Context Feedback Loop

## Why this exists

Errors caused by missing project context are **normal and recurring**. The correct response is
not to apologize but to treat each gap as an incident, run a virtual postmortem, and persist the
new context so the same gap cannot recur.

The only artifact produced by the process is the updated rule, memory entry, or documentation —
not intermediate analysis files.

## Virtual postmortem process

When the agent detects — through user correction, contradiction, or unexpected outcome — that an
error was caused by missing project context:

```
1. UNDERSTAND  — What exactly was missing? What assumption was wrong?
2. QUALIFY     — Does this pass the 3-criteria test? (see below)
3. PLAN        — Which artifact receives the new context?
4. EXECUTE     — Create or update the artifact. This is the ONLY phase that produces files.
```

Do not announce the postmortem. Do not create intermediate analysis files. The only visible
result is the updated artifact.

## Qualification criteria (3-question test)

A context gap must be persisted if ALL THREE are true:

1. **Will this gap affect future sessions in this project?**
2. **Does the gap reveal something about how THIS project works (not general tech knowledge)?**
3. **Would a future agent repeat the same error without this context?**

If any answer is "no" → do not persist.

## Persist vs. do not persist

| Persist | Do not persist |
|---------|----------------|
| New aspect of a pre-existing project topic | Generic research error (file not found) |
| Project philosophy or working principle | Isolated language ambiguity (one-off) |
| Undocumented execution obligation | General programming knowledge |
| Project-specific naming convention | Temporary code state (in-progress change) |
| Systematic agent behavior feedback | Information from another project's session |

## Target artifact by context type

| Context type | Target artifact | Reason |
|-------------|-----------------|--------|
| Execution obligation in `.rs` or `.ui` | `rules/standards/language.md` | Auto-loaded via `globs: ["**/*.rs"]` |
| Project philosophy or working principle | This file (`context-feedback.md`) | Always available as meta-rule |
| Expected agent behavior (systematic) | `memory/feedback_*.md` | Loaded at session start |
| New architecture pattern | `CLAUDE.md` | Source of truth for humans and agents |
| Project naming or workspace convention | `memory/project_*.md` | Persists across sessions |

## Example: i18n execution gap (this session, 2026-05-01)

- **Gap**: `rules/standards/language.md` described what strings need gettext() but not what
  to run after adding them. An agent could add `gettext("New String")` and consider the task
  done without updating the `.pot` / `.po` files.
- **Qualified as**: project-specific, recurrent, future agents would repeat the same mistake.
- **Persisted to**: `rules/standards/language.md` — execution obligations section.
- **Secondary artifact**: Makefile targets `validate-pot-sync`, `pot-update`, `po-sync`.
