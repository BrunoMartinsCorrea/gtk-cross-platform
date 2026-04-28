---
name: craft:fault-analysis
description: Investigates defects with a structured methodology (hypothesis → evidence → root cause → fix), specialized for GTK4/GLib/async-channel/Rust concurrency bugs in this project.
---

# craft:fault-analysis

Invoke with `/craft:fault-analysis` when a bug, crash, or unexpected behavior is observed.

## When to use

- A test is failing and the root cause is unclear
- The GTK application crashes or shows unexpected behavior
- A driver adapter returns wrong data
- A UI widget doesn't update after a driver call returns

## Process

### Step 1 — Reproduce and isolate

Before forming any hypothesis:
1. Identify the minimal reproduction path (which action triggers the bug?)
2. Determine the layer where failure is visible (UI glitch? Wrong data? Crash?)
3. Check if `G_MESSAGES_DEBUG=all make run` produces relevant log output
4. Run the test suite: `make test` — is the failure in a specific test?

### Step 2 — Form hypotheses

List 2–3 candidate hypotheses, ordered by probability:

```
Hypothesis A: [most likely — why?]
Hypothesis B: [second candidate — why?]
Hypothesis C: [less likely — why?]
```

### Step 3 — Gather evidence

For each hypothesis, identify what evidence would confirm or deny it:

**Threading bugs (most common in this project):**
- GTK called from non-main thread? → Check for `spawn_driver_task` bypass
- `glib::spawn_local` closure captures data incorrectly?
- `async_channel` sender/receiver mismatch?

**Driver adapter bugs:**
- Is the Docker/Podman socket accessible? (`ls -la /var/run/docker.sock`)
- Does `MockContainerDriver` reproduce the issue? (isolates adapter vs. use case)
- Is the JSON parsing correct? (check `serde_json` deserialization)

**UI update bugs:**
- Is `begin_loading()`/`end_loading()` called symmetrically?
- Is the GObject property notification fired after data update?
- Does `GListModel` emit the right `items-changed` signal?

**i18n bugs:**
- Is the text domain bound before the first `gettext()` call in `main()`?
- Is the `.po` file compiled (`.mo` exists in the build directory)?

### Step 4 — Verify root cause

State the root cause as one sentence: "The bug occurs because [specific condition] causes [observable effect]."

Verify it predicts the reproduction path exactly.

### Step 5 — Fix and verify

1. Apply the minimal fix (no unrelated cleanup in the same commit)
2. Run `make test` — must pass
3. Run `make lint` — must pass
4. If the fix touches threading: manually verify the GTK main thread invariant holds

## Common root causes in this project

| Symptom | Root cause pattern |
|---------|-------------------|
| Crash on `assert!(gtk4::is_initialized())` | GTK operation outside main thread |
| UI never updates after driver call | `end_loading()` not called on error path |
| `async-channel` full/blocked | `bounded(1)` + uncollected receiver |
| Missing mock implementation | New `IContainerDriver` method not added to `MockContainerDriver` |
| `msgfmt` fails in CI | New source file with `gettext()` not in `po/POTFILES` |
| GResource not found | New `.ui` file not listed in `resources.gresource.xml` |

## Output

A structured fault report:
- Observed symptom
- Root cause (one sentence)
- Evidence that confirms the root cause
- Fix applied (file + line + change)
- Verification: `make test` result
