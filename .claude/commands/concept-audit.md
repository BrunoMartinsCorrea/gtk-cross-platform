# /project:concept-audit

Audit this GTK4/Rust codebase for **internal conceptual inconsistencies** — places where the code
contradicts itself semantically, regardless of whether documentation is present. This command is
self-contained — run it fresh without prior conversation context.

This audit differs from `/project:compliance-audit` (documented vs implemented) and
`/project:github-audit` (repository health). Its scope is: *does the code form a coherent,
internally consistent system?*

---

## What to read before auditing

Read these files in full before emitting any diagnosis:

- `CLAUDE.md` — architecture rules, layer contracts, threading model, design standards
- `src/` — entire source tree: domain, ports, infrastructure, window
- `tests/` — integration and widget tests
- `Cargo.toml` — declared features and dependencies
- `data/resources/window.ui` — composite templates and breakpoints
- `data/resources/style.css` — CSS class contracts

---

## Category 1 — Naming vs Behaviour

A function, type, or module whose name implies one thing but does another is a conceptual
inconsistency regardless of whether it works. Check:

1. **Predicate names that mutate state.** Functions named `is_*`, `has_*`, `can_*`, or
   `should_*` must be pure (no `&mut self`, no side effects, no I/O). Report any that write,
   spawn tasks, or modify GTK widgets.

2. **`new` constructors that can fail silently.** `new()` implies guaranteed construction.
   Any `new()` that returns `Option<Self>` or `Result<Self, _>` without the suffix `try_new`
   is a naming lie — report it.

3. **`list_*` methods that do not return a collection.** Any method named `list_containers`,
   `list_images`, etc. that returns a scalar, triggers a callback, or mutates state instead
   of returning `Vec<T>` is inconsistent.

4. **Module names mismatched with contents.** Read every `mod.rs` and compare the module's
   declared name with what it exports. A module named `domain` that exports `gtk4` types
   is a conceptual violation.

5. **`View` types that contain business logic.** Types under `src/window/views/` must not
   implement filtering, sorting, parsing, or validation logic — those belong in `src/core/`.
   Report any view that manipulates domain data beyond formatting for display.

---

## Category 2 — Abstraction Boundary Leaks

Hexagonal Architecture (documented in `CLAUDE.md`) requires strict layering. Leaks happen when
concrete types bleed through port boundaries. Check:

1. **Ports (`src/ports/`) exposing infrastructure types.** Trait method signatures in
   `IContainerDriver` and `IGreetingService` must use only domain types (`src/core/domain/`)
   or primitives. Report any method that accepts or returns a type from `src/infrastructure/`
   or a library-specific type (e.g., `reqwest::Error`, `serde_json::Value`).

2. **Domain types importing adapters.** Grep `src/core/` for `use crate::infrastructure`.
   Any such import violates the dependency rule — the domain must not know about adapters.

3. **Views bypassing ports.** Views (`src/window/views/`) must call `IContainerDriver` methods
   via the port trait, never by downcasting to a concrete driver type. Grep for `as DockerDriver`,
   `as PodmanDriver`, or direct instantiation of any concrete driver inside `src/window/`.

4. **Error types crossing layer boundaries.** `ContainerError` (from `src/infrastructure/`)
   should not appear in `src/core/` method signatures. The port trait may re-export a domain
   error type, but it must not import infrastructure errors directly. Check `i_container_driver.rs`.

5. **`factory.rs` called outside `activate()`.** The composition root rule states
   `ContainerDriverFactory::detect()` is only called in `src/app.rs::activate()`. Grep all
   other files for `ContainerDriverFactory` or `detect()`.

---

## Category 3 — Trait Contract Violations

A type that implements a trait but violates the trait's semantic contract is a conceptual
inconsistency. Check:

1. **`MockContainerDriver` semantic drift.** The mock must honour all IContainerDriver contracts:
    - `list_containers()` must return the containers previously inserted via test setup
    - `start_container(id)` must fail with `ContainerError::NotFound` for unknown IDs
    - `remove_container(id)` must make the container absent from subsequent `list_containers()`
      Verify these invariants hold by reading `tests/container_driver_test.rs`.

2. **Infallible operations in a fallible trait.** If `IContainerDriver` declares a method
   returning `Result<_, ContainerError>`, every implementation must propagate real errors.
   Report any implementation that returns `Ok(...)` unconditionally (permanently swallowing
   errors, not just the mock).

3. **`Clone`/`Default` on types that own non-cloneable resources.** If a struct derives
   `Clone` or `Default` but contains `Arc<dyn Trait>` fields, verify the derive is intentional
   and not inherited from a copy-paste. Non-resource structs deriving `Clone` are fine.

4. **Async/sync mismatch.** If `IContainerDriver` declares async methods, all implementations
   must be async. If it declares sync methods, none may block the GTK thread directly (they
   must go through `spawn_driver_task`). Report any implementation that calls a blocking
   operation from a sync trait method without `spawn_driver_task`.

---

## Category 4 — Domain Model Coherence

The domain (`src/core/domain/`) should represent a unified, self-consistent model of container
management. Check:

1. **Duplicate or overlapping types.** Are `ContainerStatus`, `ContainerStats`, and
   `Container` self-consistent? Does `ContainerStats` reference fields that `Container` also
   carries, creating ambiguity about which is the authoritative source?

2. **Missing inverse operations.** If the domain model includes `start_container`, it should
   also include `stop_container`. If it includes `create_volume`, it should include
   `remove_volume`. List any resource type with asymmetric CRUD coverage in `IContainerDriver`.

3. **Status enum coverage.** `ContainerStatus` variants must cover every status string the
   Docker/Podman API can return (`created`, `running`, `paused`, `restarting`, `removing`,
   `exited`, `dead`). Report missing variants that would silently fall to an `Unknown` arm.

4. **`PruneReport` and `SystemUsage` field coherence.** Check `network.rs` — `PruneReport`
   contains `containers_deleted`, `images_deleted`, `volumes_deleted`. Verify these field names
   match the corresponding Docker API JSON keys; a mismatch would cause silent deserialization
   failures.

---

## Category 5 — Threading Model Coherence

The threading contract (`spawn_driver_task` + `async_channel`) must be applied consistently.
Check:

1. **Inconsistent loading state management.** Every view that calls `spawn_driver_task` must
   call `begin_loading()` before the call and `end_loading()` in the callback. Grep for
   `spawn_driver_task` across views; for each call site, verify the surrounding loading-state
   symmetry. An unmatched `begin_loading` without `end_loading` leaves the spinner running
   forever.

2. **Channel capacity inconsistency.** All channels must be `async_channel::bounded(1)`. A
   `bounded(N)` with N > 1 allows stale results to queue — the view may process an old response
   after the user has already triggered a new reload. Report any `bounded(N)` where N ≠ 1.

3. **Callback runs driver logic.** The `cb` callback passed to `spawn_driver_task` must only
   update GTK widgets, not call another `spawn_driver_task`. Nested driver calls bypass the
   loading-state guard. Report any callback that spawns a second driver task.

4. **Error handling symmetry.** Every `spawn_driver_task` call site must handle both the `Ok`
   and `Err` arms of the result. Report call sites where the `Err` arm is `_ => {}` (silently
   discarded) or where only the happy path is handled.

---

## Category 6 — UI Semantic Consistency

GTK/Adwaita patterns must be applied uniformly. Check:

1. **Inconsistent empty-state handling.** If one view shows an `adw::StatusPage` when the
   list is empty, all views must. Report views that show an empty `gtk4::ListBox` without an
   empty-state widget.

2. **Inconsistent loading indicators.** If one view uses `adw::Spinner` during `reload()`,
   all views must. Report views that update the list without any loading indicator.

3. **Inconsistent action availability.** If the containers view disables action buttons when
   no item is selected, all views must do the same. Report views where action buttons (remove,
   start, stop) are always enabled regardless of selection state.

4. **Toast vs dialog inconsistency.** `CLAUDE.md` says use `adw::ToastOverlay` for transient
   feedback. Report any view that shows a non-destructive confirmation (e.g., "Removed image X")
   via `adw::AlertDialog` instead of a toast.

5. **`DetailPane` contract misuse.** `DetailPane::set_groups` accepts translated labels.
   Report any call site that passes raw, untranslated strings (string literals without
   `gettext!()`) as group titles or row labels.

---

## Category 7 — Test–Implementation Contract Gaps

Tests define a contract. Inconsistencies between tests and implementation indicate either the
test is wrong or the implementation diverged. Check:

1. **Tests that assert the wrong thing.** Read `tests/container_driver_test.rs`. For each
   test, verify the assertion matches the documented `IContainerDriver` contract, not an
   implementation detail of `MockContainerDriver`. Tests should assert observable behaviour,
   not internal mock state.

2. **Tests missing for documented failure modes.** `ContainerError` has multiple variants
   (`NotFound`, `PermissionDenied`, `ParseError`, `RuntimeNotAvailable`, etc.). Verify at
   least one test exercises each error variant to confirm the log-level mapping in `error.rs`
   is actually exercised by the test suite.

3. **`#[ignore]` tests with no issue reference.** Widget tests in `tests/widget_test.rs` are
   marked `#[ignore]` with a runtime requirement. Verify every `#[ignore]` test has a comment
   explaining the condition under which it should be run and what it covers.

4. **Test names that don't describe the scenario.** Test functions named `test_1`, `test_foo`,
   or `it_works` are conceptual inconsistencies — they describe nothing. Report tests without
   a descriptive name following the pattern `<verb>_<subject>_<condition>_<expected>`.

---

## Category 8 — Logging Semantic Consistency

`AppLogger` maps `ContainerError` variants to log levels via `log_container_error()`. Check:

1. **Level assignment matches severity.** `CLAUDE.md` says: `critical` for `PermissionDenied`
   and `ParseError`; `info` for `NotFound`; `warning` for all others. Read `error.rs` and
   verify the match arms match these assignments exactly.

2. **G_LOG_DOMAIN hierarchy coherence.** Every sub-domain logger created by `AppLogger::subdomain()`
   must follow the prefix convention `com.example.GtkCrossPlatform.<layer>.<resource>`. Report
   any `subdomain()` call that produces a domain string not matching this pattern.

3. **Logging at call site vs inside error constructor.** `CLAUDE.md` states logging must happen
   at the call site (views/app.rs), not inside the error constructor. Grep `ContainerError`
   variants for any `AppLogger` calls inside their constructors or `impl From<_> for ContainerError`
   blocks.

---

## Output format

For each inconsistency found, emit one block:

```
## [SEVERITY] CATEGORY — Inconsistency title

- **Location:** <file>:<line or function>
- **Inconsistency:** <what claim is made> vs <what the code actually does>
- **Why it matters:** <one sentence — the runtime or maintenance risk>
- **Fix:** <one sentence — the minimal change that restores consistency>
```

Severity levels:

- **CRITICAL** — could produce silent data corruption, panic, or runtime deadlock
- **HIGH** — contradicts an explicit architectural invariant in `CLAUDE.md`
- **MEDIUM** — confuses the reader, risks wrong future changes
- **LOW** — cosmetic or minor naming issue

End with a summary table:

| Category                   | Inconsistencies found | Highest severity         |
|----------------------------|-----------------------|--------------------------|
| Naming vs Behaviour        | N                     | CRITICAL/HIGH/MEDIUM/LOW |
| Abstraction Boundary Leaks | N                     | …                        |
| Trait Contract Violations  | N                     | …                        |
| Domain Model Coherence     | N                     | …                        |
| Threading Model            | N                     | …                        |
| UI Semantic Consistency    | N                     | …                        |
| Test–Implementation        | N                     | …                        |
| Logging Semantics          | N                     | …                        |

Do not report items that are internally consistent. Focus only on places where the code
contradicts itself.
