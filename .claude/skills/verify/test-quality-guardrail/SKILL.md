---
name: verify:test-quality-guardrail
version: 1.0.0
description: Audit tests against universal quality principles; identifies antipatterns and proposes abstractions
---

# verify:test-quality-guardrail

Evaluate the automated test layer of the project as an external software quality auditor. This command is
self-contained — run it in a fresh session, without context from prior conversations.

---

## Agent Role

You are a senior software quality engineer specialising in test architecture, clean code, and long-term
sustainability. Maintain the position of an **external auditor**: your goal is not to validate the effort
invested, but to identify precisely where coverage fails, where confidence is false, and where maintenance becomes
expensive over time. Be direct. Avoid praise that does not contribute to the diagnosis.

Your goal is threefold:

1. **Audit** the test layer against universal quality principles
2. **Identify** abstraction and code reuse opportunities in the tests
3. **Apply** concrete improvements using established software engineering patterns

Read `CLAUDE.md` in full before emitting any diagnosis — it defines the layered architecture, threading
constraints, and per-layer purity rules that the tests must respect.

Do not modify production code. Act exclusively on the test layer.

---

## Phase 0 — Mandatory Reading

Before starting the survey, read the following files in the indicated order. Each read forms a prerequisite
for correct diagnosis in the next phase.

### 0.1 Project architecture and rules

Read `CLAUDE.md` in full. Record:

- The four layers of the hexagonal architecture and the import rules for each
- The threading model (`spawn_driver_task`, `async-channel`, prohibition of `tokio`)
- Layer purity rules: domain layer tests must not import `gtk4`, `adw`, or `glib`
- The `IContainerDriver` contract and which errors each operation can produce

### 0.2 Test infrastructure map

Read all files in `tests/` and all inline `#[cfg(test)]` modules in `src/`. For each file, record:

- Which subsystem is being tested
- Which architectural layer the test touches
- Which local fixture/factory functions exist

### 0.3 Central test double

Read `src/infrastructure/containers/mock_driver.rs` in full. Map:

- Which operations return `Ok(...)` unconditionally (ignoring the ID or resource state)
- Which operations model realistic error behaviour
- Whether the mock faithfully reflects all failure modes defined in `ContainerError`

---

## Phase 1 — Structural Survey

Map the current state of the test layer. For each dimension, record what exists and what is absent.

### 1.1 Test Pyramid

The test pyramid requires: many unit tests → fewer integration tests → minimum end-to-end tests.
Count and classify all tests:

| Layer        | Expected location                           | How to count                                             |
|--------------|---------------------------------------------|----------------------------------------------------------|
| Unit         | `#[cfg(test)]` inline in `src/`             | `grep -rn "#\[test\]" src/ --include="*.rs" \| wc -l`    |
| Integration  | Files in `tests/` (except `widget_test.rs`) | `grep -rn "^#\[test\]" tests/ --include="*.rs" \| wc -l` |
| E2E / Widget | `tests/widget_test.rs` with `#[ignore]`     | Manual count                                             |

Evaluate:

- Is the unit:integration ratio in the expected pyramid shape (unit > integration)?
- Are unit tests collocated with the modules they test (Rust convention)?
- Is there at least one unit test for each invariant documented in the domain model?
- Does every `#[ignore]` have a documented condition and execution runbook in the comments?

### 1.2 Test Naming

Test names are executable documentation. Evaluate all names against the
**Subject / Condition / Expectation** pattern:

- The name must describe *what* is being tested, *under what condition*, and *what the expected result is*
- Recommended pattern: `<subject>_<condition>_<result>` or `<verb>_<subject>_when_<condition>_returns_<result>`
- The `test_` prefix is redundant in Rust — `#[test]` already marks the function; the prefix pollutes the name without
  adding semantics
- Names like `events_returns_list` (no condition) or `layers_have_id_cmd_and_size` (no scenario) violate the pattern

Flag every test whose name does not convey all three elements. Generic names (`test_1`, `it_works`, `test_foo`) are
a critical failure. Names with a redundant `test_` prefix are a low-severity failure but indicate a convention
inconsistency in the project.

### 1.3 Assertion Quality

Each test must verify a single observable behavioural result. Evaluate:

- **Missing assertions** — a test that never fails is useless as a guardrail. Recognise the two vacuity patterns:
    1. Tautological assertion: `assert!(x.is_empty() || !x.is_empty())` — logically always true
    2. Dead assignment: `let _ = value;` — reads the value but asserts nothing about it
- **Multiple unrelated assertions** — suggests the test covers too many behaviours; split it
- **Assertions on internal state** — verifying private fields couples the test to the implementation
- **Overly broad assertions** — full object equality when only one field is relevant

### 1.4 Test Independence

Tests must be runnable in any order, individually or in parallel. Evaluate:

- **Shared mutable state** — `MockContainerDriver` uses `Mutex<Vec<_>>` internally; does each test create its
  own instance via a factory function? Or does it share state between calls?
- **Order dependency** — a test that passes in the full suite but fails when run in isolation
- **Residual side effects** — do state-mutation operations (start, stop, create) leak into other tests?

---

## Phase 2 — Quality Antipatterns

For each antipattern found, report: **category**, **description of the problem**, **identification of the affected test
**
(by scenario name, not line number), and **impact**.

### 2.1 False Confidence Antipatterns

**Vacuous Test** — always passes regardless of implementation changes. This is the most
dangerous antipattern: it gives false confidence and lets regressions pass silently. Two occurrence patterns in this
verify:

*Pattern 1 — Logical tautology:*

```rust
// ANTIPATTERN: always true regardless of what `s` contains
assert!(s.is_empty() || !s.is_empty(), "no panic is the contract")
```

The expression `x || !x` is De Morgan's tautology — true for any value of `x`. Any production change that
alters the returned value will not be detected.

*Pattern 2 — Dead assignment:*

```rust
// ANTIPATTERN: reads fields but asserts nothing
let _ = report.containers_deleted.len();
let _ = report.images_deleted.len();
let _ = report.space_reclaimed;
```

`let _ = expr` discards the value without checking anything. The only contract exercised is "the call did not panic",
which the preceding `.expect()` already covers. These lines are noise that masks the absence of real assertions.

**Happy Path Only** — a test file that covers only the success scenario, with no tests for
error conditions, boundary values, or invalid inputs. Every `IContainerDriver` operation with more than one failure
mode must have at least one test per `ContainerError` variant — otherwise the error mapping is never exercised.

**Non-Failing Fake** — a test double that returns `Ok(...)` unconditionally for
operations that should fail under certain conditions. If the double does not faithfully represent the failure modes of
the contract, integration tests exercise only the happy paths of production code.

The `MockContainerDriver` in this project contains operations that completely ignore their input parameters:

```rust
// ANTIPATTERN in mock — operations that never fail, regardless of resource state
fn remove_volume(&self, _name: &str, _force: bool) -> Result<(), ContainerError> { Ok(()) }
fn remove_network(&self, _id: &str) -> Result<(), ContainerError> { Ok(()) }
fn remove_image(&self, _id: &str, _force: bool) -> Result<(), ContainerError> { Ok(()) }
fn restart_container(&self, _id: &str, _timeout_secs: Option<u32>) -> Result<(), ContainerError> { Ok(()) }
fn pause_container(&self, _id: &str) -> Result<(), ContainerError> { Ok(()) }
fn unpause_container(&self, _id: &str) -> Result<(), ContainerError> { Ok(()) }
```

None of these operations checks whether the ID exists, whether the resource is in the correct state, or whether the
operation is valid in that context. Any test that calls `remove_image("nonexistent-id")` expecting `NotFound` will
pass silently — and more importantly, the absence of such tests means the error paths in production code are never
exercised.

### 2.2 Maintainability Antipatterns

**Fixture Duplication** — identical setup code repeated at the start of multiple tests.

In the project, `fn container_uc()` is copy-pasted in 4 distinct files with identical body:

```rust
// Declared identically in:
// tests/container_driver_test.rs
// tests/container_stats_test.rs
// tests/inspect_test.rs
// tests/create_container_test.rs
fn container_uc() -> ContainerUseCase {
    ContainerUseCase::new(Arc::new(MockContainerDriver::new()))
}
```

Similarly, `fn driver() -> Arc<MockContainerDriver>` is copy-pasted in:

- `tests/pull_image_streaming_test.rs`
- `tests/container_logs_test.rs`
- `tests/terminal_test.rs`

And `fn make_container(...)` has two implementations with incompatible signatures:

- `tests/search_filter_test.rs`: 4 parameters `(name, image, short_id, compose)`
- `tests/compose_grouping_test.rs`: 2 parameters `(name, compose)`

The copy produces drift: an update to the signature of `ContainerUseCase::new` must be propagated manually to
each file.

**Magic Values** — unnamed literals in assertions that require reading production code to be understood.

In the project, container IDs are repeated as literals in 7+ files without named constants:

```rust
// "aabbccdd1122334455667788" appears in container_stats_test, inspect_test,
// container_logs_test, terminal_test, container_driver_test — always representing
// the same "web-server" running container
uc.stats("aabbccdd1122334455667788").expect("stats");

// The reader does not know what this ID represents without consulting mock_driver.rs
```

Numeric values derived from the mock also appear hardcoded:

```rust
assert_eq!(usage.containers_total, 2);  // why 2? what is the contract?
assert_eq!(report.containers_deleted.len(), 1);  // depends on mock internal state
assert!((stats.memory_usage_mb() - 50.0).abs() < 1.0);  // 50 MiB hardcoded in mock
```

**God Test** — a single test that covers multiple chained operations. When it fails, it is impossible to
identify which step failed without debugging the entire path.

```rust
// ANTIPATTERN: covers stop + verify + start + verify in a single test
fn start_container_makes_it_running() {
    let uc = container_uc();
    uc.stop("aabbccdd1122", None).expect("stop");    // operation 1
    let before = uc.list(false).expect("list").len(); // intermediate assertion
    assert_eq!(before, 0);
    uc.start("aabbccdd1122").expect("start");         // operation 2
    let after = uc.list(false).expect("list").len();  // final assertion
    assert_eq!(after, 1);
}
```

Should be split into two focused tests: `stop_running_container_removes_from_running_list` and
`start_stopped_container_adds_to_running_list`.

**Redundant Test Coverage** — 7 inline unit tests duplicate exactly the scenarios covered by integration tests in
`tests/`:

| Inline test in `src/`                              | Duplicate in `tests/`                                       |
|----------------------------------------------------|-------------------------------------------------------------|
| `container_use_case.rs::list_all_returns_all`      | `container_driver_test.rs::list_containers_all_returns_all` |
| `container_use_case.rs::list_running_only`         | `container_driver_test.rs::list_containers_running_only`    |
| `greet_use_case.rs::returns_greeting`              | `greet_use_case_test.rs::returns_greeting`                  |
| `image_use_case.rs::list_returns_images`           | `container_driver_test.rs::list_images_returns_images`      |
| `network_use_case.rs::list_networks_returns_two`   | `container_driver_test.rs::list_networks_returns_two`       |
| `network_use_case.rs::prune_system_returns_report` | `container_driver_test.rs::prune_system_returns_report`     |
| `volume_use_case.rs::list_returns_volumes`         | `container_driver_test.rs::list_volumes_returns_volumes`    |

Duplicate tests double the maintenance cost without increasing coverage.

### 2.3 Reliability Antipatterns

**Missing state transition coverage** — the domain models `ContainerStatus` with 7 variants and an implicit
lifecycle. Tests cover only the happy transitions (running → stopped → running). There are no tests for:

- Trying to start a container already in `Running` state (idempotent or error?)
- Trying to pause a container in `Stopped` state (must return `NotRunning`)
- Trying to remove a container in `Running` state without `force=true` (must return error or force?)
- The `ContainerStatus::Paused` variant is tested in the domain model but never exercised via driver

**Layer purity violation** — `src/window/components/status_badge.rs` resides in the UI layer but its tests
exercise exclusively domain logic (`ContainerStatus::css_class()`, `.label()`):

```rust
// In src/window/components/status_badge.rs — UI layer
#[cfg(test)]
mod tests {
    use gtk_cross_platform::core::domain::container::ContainerStatus; // imports domain ✓
    // But tests only ContainerStatus methods — not testing any UI component behaviour
    fn css_class_matches_domain() { ... }
    fn label_is_non_empty_for_all_variants() { ... }
}
```

These tests belong in the `src/core/domain/container.rs` module where `ContainerStatus` is defined — and in fact
the same logic is already covered by `status_css_classes` and `status_labels` in that module. This is duplication
with a location violation.

---

## Phase 3 — Abstraction and Reuse Opportunities

### 3.1 Object Mother — Mock ID Constants

**Problem:** The ID `"aabbccdd1122334455667788"` appears as a literal in at least 7 test files, always representing
the same "web-server" container in `MockContainerDriver`. The same applies to the stopped "db" container and the
non-existent ID used in `NotFound` tests.

**Pattern to apply:** A `tests/support/fixtures.rs` module (or `mod fixtures` inside each file) with named
constants that document the semantics of each ID:

```rust
// Concept — expected structure
pub mod fixtures {
    // MockContainerDriver IDs (match mock_driver.rs)
    pub const RUNNING_CONTAINER_ID: &str = "aabbccdd1122334455667788"; // web-server, nginx:latest
    pub const STOPPED_CONTAINER_ID: &str = "112233445566778899aabbcc"; // db, postgres:15, Exited
    pub const STANDALONE_CONTAINER_ID: &str = "223344556677889900aabbcc"; // standalone, redis
    pub const UNKNOWN_CONTAINER_ID: &str = "nonexistentid0000000000";

    // Numeric values derived from the fixed mock state
    pub const MOCK_CONTAINERS_TOTAL: usize = 3;
    pub const MOCK_RUNNING_CONTAINERS: usize = 1;
    pub const MOCK_IMAGES_TOTAL: usize = 2;
    pub const MOCK_WEB_SERVER_MEMORY_MIB: f64 = 50.0;
}
```

**Duplication eliminated:** ~30 ID literals distributed across 7 files collapsed into 4 named constants.

### 3.2 Shared Fixture — Use Case Factory Functions

**Problem:** `fn container_uc()`, `fn driver()`, and variants are declared locally in each test file with
identical body. Any change to the `ContainerUseCase::new` or `MockContainerDriver::new` signature requires
updating 4–5 places.

**Pattern to apply:** A shared support module that each integration file imports:

```rust
// Concept — tests/support/mod.rs
pub fn container_uc() -> ContainerUseCase {
    ContainerUseCase::new(Arc::new(MockContainerDriver::new()))
}

pub fn image_uc() -> ImageUseCase {
    ImageUseCase::new(Arc::new(MockContainerDriver::new()))
}

pub fn mock_driver() -> Arc<MockContainerDriver> {
    Arc::new(MockContainerDriver::new())
}
```

**Duplication eliminated:** 4 declarations of `container_uc()`, 3 of `fn driver()`, 2 of `fn use_case()` for image.

### 3.3 Test Data Builder — Container Builder

**Problem:** `fn make_container(...)` has two incompatible implementations in `search_filter_test.rs` (4 parameters)
and `compose_grouping_test.rs` (2 parameters), both constructing structurally the same type with different signatures.
The divergence will grow as the `Container` domain evolves.

**Pattern to apply:** A fluent builder that makes the intent of each field explicit:

```rust
// Concept — eliminates the two incompatible implementations
ContainerBuilder::default()
    .name("nginx-proxy")
    .image("nginx:latest")
    .short_id("aabbccdd1122")
    .compose_project("web-stack")
    .status(ContainerStatus::Running)
    .build()
```

**Duplication eliminated:** 2 incompatible implementations of `make_container` collapsed into 1 builder with
explicit per-field semantics.

### 3.4 Custom Assertion — Error Variant Verification

**Problem:** The error variant verification pattern is repeated in multiple tests with a common antipattern:
using `format!("{}", err)` and `contains("string")` instead of checking the variant directly. This couples the test
to the error text message, not the error type.

```rust
// ANTIPATTERN — fragile against renaming error strings
let msg = format!("{}", result.unwrap_err());
assert!(msg.contains("Not found") || msg.contains("not found"), "...");
```

**Pattern to apply:** A custom assertion macro that checks the variant via pattern matching:

```rust
// Concept — expressive and robust assertion macro
assert_error_variant!(result, ContainerError::NotFound(_));
assert_error_variant!(result, ContainerError::NotRunning(_));
assert_error_variant!(result, ContainerError::RuntimeNotAvailable(_));
```

The macro is declared once in `tests/support/mod.rs` inside `#[cfg(test)]`.

**Duplication eliminated:** The `format!() + contains()` pattern used in 8+ tests replaced by structural type
verification.

### 3.5 Parameterised Test — `is_secret_env_key`

**Problem:** `env_masking_test.rs` contains 7 test functions that differ only in input values and the expected
boolean assertion. Each function tests one variant of the masking behaviour.

```rust
// 7 nearly identical tests — differ only in values
fn mask_password_suffix() {
    assert!(is_secret_env_key("POSTGRES_PASSWORD"));
    ...
}
fn mask_password_lowercase() {
    assert!(is_secret_env_key("password"));
    ...
}
fn mask_secret_substring() {
    assert!(is_secret_env_key("API_SECRET"));
    ...
}
fn mask_token_substring() { ... }
fn mask_key_substring() { ... }
fn safe_key_not_masked() { ... }
fn empty_key_not_masked() { ... }
```

**Pattern to apply:** A table-driven test that eliminates 7 functions and centralises the data:

```rust
// Concept — (env_key, expected_secret)
let cases = [
    ("POSTGRES_PASSWORD", true),
    ("DB_PASSWORD",       true),
    ("password", true),
    ("API_SECRET", true),
    ("GITHUB_TOKEN", true),
    ("AWS_ACCESS_KEY_ID", true),
    ("NGINX_HOST",        false),
    ("TZ", false),
    ("PORT", false),
    ("", false),
];
for (key, expected) in cases {
    assert_eq!(is_secret_env_key(key), expected, "key={key:?}");
}
```

**Duplication eliminated:** 7 test functions collapsed into 1, with equal or greater coverage and trivial addition of
new cases.

### 3.6 Parameterised Test — `ContainerStatus` Parsing

**Problem:** The inline tests in `container.rs` cover each `ContainerStatus::from_state` variant with a separate
function (`status_from_running_state`, `status_from_paused_state`, `status_from_exited_with_code`,
`status_from_unknown_state`). These are direct candidates for a table.

**Pattern to apply:** A `(state_str, exit_code, expected_variant)` table that covers all cases in a single loop,
making it simple to add new states:

```rust
// Concept — (state_string, exit_code, expected_variant) table
let cases: &[(&str, Option<i32>, ContainerStatus)] = &[
    ("running", None,    ContainerStatus::Running),
    ("paused",  None,    ContainerStatus::Paused),
    ("exited",  Some(0), ContainerStatus::Exited(0)),
    ("exited",  Some(1), ContainerStatus::Exited(1)),
    ("restarting", None, ContainerStatus::Restarting),
    ("dead",    None,    ContainerStatus::Dead),
    ("fancy-new-state", None, /* matches Unknown(_) */),
];
```

---

## Phase 4 — Technical Report (Mandatory Delivery)

This phase produces the output document. Run it **after** Phases 1–3. The report is the primary deliverable of
this command — it is not optional and cannot be replaced by a verbal summary.

Generate the report in the file `docs/test-quality-audit.md`. The mandatory structure is:

```markdown
# Quality Report — Test Layer

**Date:** YYYY-MM-DD
**Auditor:** Claude Code (external)
**Scope:** All `#[cfg(test)]` modules in `src/` and all files in `tests/`
**Audited version:** (result of `git rev-parse --short HEAD`)

## Executive Summary

| Dimension | State | Max Severity |
|-----------|-------|--------------|
| Test pyramid | ... | ... |
| Naming | ... | ... |
| Assertion quality | ... | ... |
| Mock fidelity | ... | ... |
| Fixture duplication | ... | ... |
| Layer purity | ... | ... |

## Statistics

- Inline unit tests: N
- Integration tests: N
- E2E/widget tests (ignored): N
- Total: N
- Unit:integration ratio: N:N

## Findings by Severity

### CRITICAL

[ ] AC-01 — <antipattern name> — <affected file(s)>
Description: ...
Impact: ...
Fix: ...

### HIGH

[ ] AA-01 — ...

### MEDIUM

[ ] AM-01 — ...

### LOW

[ ] AB-01 — ...

## Corrective Action Checklist

> Mark each item with [x] after applying the fix. Run `make test` at the end.

### Immediate Actions (Critical / High)

- [ ] Replace tautological assertion in `test_exec_empty_command_handled_gracefully` with a real assertion
- [ ] Replace dead assignments in `test_prune_system_returns_report` with `assert_eq!` on specific fields
- [ ] Add resource existence check in `remove_volume`, `remove_network`, `remove_image` in the mock
- [ ] Add state check in `restart_container`, `pause_container`, `unpause_container` in the mock
- [ ] Move `ContainerStatus` tests from `status_badge.rs` to `container.rs` (layer violation)
- [ ] Add invalid state transition tests: pause(Stopped), start(Running), remove(Running without force)

### Maintainability Actions (Medium)

- [ ] Create `tests/support/fixtures.rs` with `RUNNING_CONTAINER_ID`, `STOPPED_CONTAINER_ID`, `UNKNOWN_CONTAINER_ID`
- [ ] Create `tests/support/factories.rs` with shared `container_uc()`, `image_uc()`, `mock_driver()`
- [ ] Eliminate local duplicate declarations of `fn container_uc()` in 4 test files
- [ ] Eliminate local duplicate declarations of `fn driver()` in 3 test files
- [ ] Consolidate `make_container` into a single shared builder or factory
- [ ] Parameterise 7 `is_secret_env_key` tests into a single table
- [ ] Parameterise `ContainerStatus::from_state` tests into a single table
- [ ] Split `start_container_makes_it_running` into two focused tests
- [ ] Remove duplicate inline tests (the 7 listed in the Redundant Test Coverage table)
- [ ] Replace `format!() + contains()` with `assert_error_variant!` macro in 8+ affected tests

### Naming Improvements (Low)

- [ ] Remove `test_` prefix from tests in `dashboard_test.rs`
- [ ] Remove `test_` prefix from tests in `pull_image_streaming_test.rs`
- [ ] Remove `test_` prefix from tests in `runtime_switcher_test.rs`
- [ ] Remove `test_` prefix from tests in `container_logs_test.rs`
- [ ] Remove `test_` prefix from tests in `terminal_test.rs`
- [ ] Add condition to names: `events_returns_list` → `events_with_no_filter_returns_all_events`
- [ ] Add condition: `layers_have_id_cmd_and_size` → `layers_for_known_image_have_populated_fields`
- [ ] Add `//!` doc comment in `compose_lifecycle_test.rs` and `system_events_test.rs`

## Result of `make test`

```

(full output here)

```

**Final status:** PASSED / FAILED
```

---

## Intermediate Phase Delivery Format

### Phase 1 — Structural Survey

| Dimension         | Current State | Recommended Practice | Gap |
|-------------------|---------------|----------------------|-----|
| Pyramid shape     | ...           | ...                  | ... |
| Naming            | ...           | ...                  | ... |
| Assertion quality | ...           | ...                  | ... |
| Test independence | ...           | ...                  | ... |

### Phase 2 — Antipatterns

For each antipattern found:

```
## [SEVERITY] ANTIPATTERN — <name>

- **Affected tests:** <scenario names, not code line numbers>
- **Problem:** <why it is dangerous or misleading>
- **Impact:** <what fails or passes undetected>
- **Recommended fix:** <pattern or technique to apply>
```

Severity levels:

- **CRITICAL** — the test will never detect the regression it should detect
- **HIGH** — violates an architecture rule or produces systemic false confidence
- **MEDIUM** — hinders maintenance or failure diagnosis
- **LOW** — cosmetic or minor improvement opportunity

### Phase 3 — Abstraction Opportunities

For each opportunity:

```
## PATTERN — <pattern name>

- **Applies to:** <test scenarios, not file paths>
- **Duplication eliminated:** <how many occurrences collapsed into one>
- **Expected structure:** <pseudocode showing the intent, not the implementation>
```

---

## Command Guardrails

This command must never:

- Modify any project file (production code or tests)
- Report as a gap something that is already correctly implemented — focus only on real problems
- Omit Phase 4 — the report in `docs/test-quality-audit.md` is the mandatory deliverable of this command
