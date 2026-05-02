---
name: plan:work-planning
version: 1.0.0
description: Decomposes a feature scope into prioritized tasks with effort per session, maps inter-task dependencies, and outputs a structured execution plan for this GTK4/Rust/Flatpak project.
---

# plan:work-planning

Invoke with `/plan:work-planning` at the start of any non-trivial feature.

## When to use

- Before starting a feature that spans multiple files or layers
- When scope is unclear and needs decomposition before coding starts
- When multiple parallel implementation paths exist and sequencing matters

## Process

### Step 1 — Understand the scope

Ask or infer:

- What is the user-visible outcome? (one sentence)
- Which layers are touched? (core, ports, infrastructure, window — per CLAUDE.md §Architecture)
- Are new ports (traits) needed?
- Are there cross-platform implications (Linux, macOS, Windows, Flatpak)?
- Does this affect i18n (new user-visible strings)?
- Does this affect A11Y (new interactive widgets)?

### Step 2 — Decompose into tasks

Break the scope into concrete, verifiable tasks. Each task must:

- Map to a single layer (core / ports / infrastructure / window)
- Have a clear done-state (can be verified with `make test` or visual inspection)
- Be estimable in session-length units (S = ≤1h, M = 1-3h, L = 3-6h, XL = multi-session)

### Step 3 — Map dependencies

Identify which tasks must complete before others can start. Common dependency chains in this project:

```
Domain model changes → Port trait changes → Adapter implementation → Use case update → View update
Port additions → MockContainerDriver update → Integration tests → UI wiring
i18n string additions → po/POTFILES update → Translation placeholders
```

### Step 4 — Order and estimate

Produce an ordered task list:

```
| # | Task | Layer | Effort | Depends on | Done state |
|---|------|-------|--------|------------|------------|
| 1 | ... | core  | S      | —          | unit tests pass |
| 2 | ... | ports | S      | #1         | trait compiles |
```

### Step 5 — Identify risks

Flag tasks that:

- Change a port trait (breaking change for all adapters + mock)
- Touch cross-platform conditionals in `build.rs` or CI
- Add new POTFILES entries (i18n pipeline must be updated)
- Require new GLib types (GObject subclassing has boilerplate)

### Step 6 — Surface blockers

Confirm before proceeding:

- Are all affected port traits understood?
- Is `MockContainerDriver` updated if a new method is added to `IContainerDriver`?
- Does `make ci` pass on the current state before starting?

## Output

A task list suitable for `TaskCreate` entries, plus a one-paragraph summary of the feature scope, the main risk, and the
estimated total effort.
