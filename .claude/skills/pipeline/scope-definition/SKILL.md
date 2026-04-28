---
name: pipeline:scope-definition
description: Defines feature scope — problem statement, objectives, acceptance criteria, and explicit negative space — before any implementation starts.
---

# pipeline:scope-definition

Invoke with `/pipeline:scope-definition` to frame a feature before decomposing it.

## When to use

- When a user request is ambiguous ("add container filtering", "improve performance")
- Before `/pipeline:work-planning` to ensure the scope is unambiguous
- When stakeholder intent and implementation details need to be separated

## Process

### Step 1 — Problem statement

One sentence: what user problem does this solve, and why does it matter now?

Example: "Users cannot filter containers by status, forcing them to scroll through long lists to find running containers."

### Step 2 — Objectives (what success looks like)

2–5 bullet points, each measurable:

- User can filter the container list by status (running / stopped / paused / all)
- Filter state persists across app restarts (GSettings)
- Filter applies within 100ms of user interaction
- Empty state shows when no containers match the filter

### Step 3 — Acceptance criteria (verifiable)

Concrete, testable criteria — each should map to a test or manual check:

- `ContainersView` displays only containers matching the selected status
- Selecting "All" restores the full list
- `FilterListModel` with `CustomFilter` is used (not manual `ListBox` rebuilds)
- Setting `filter-status` in GSettings schema persists the value

### Step 4 — Negative space (explicit non-goals)

What is OUT of scope for this feature:

- Free-text search (separate feature)
- Sorting by other columns (separate feature)
- Filter on Images/Volumes/Networks views (follow-up)

### Step 5 — Constraints

Non-negotiable constraints that apply:

- Must work on all 4 container runtimes (Docker, Podman, containerd, Mock)
- Must not call driver from GTK main thread
- Must use `spawn_driver_task` for any driver calls
- GTK minimum: 4.12 (no API above this version)

## Output

A structured scope document with all 5 sections filled. This becomes the reference for `/pipeline:work-planning` and the PR description.
