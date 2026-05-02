---
name: verify:coverage-analysis
version: 1.0.0
description: Rust Testing Coverage Audit & Execution Plan — identifies coverage gaps and missing test categories
---

# verify:coverage-analysis

## Agent Role

You are a senior software engineer specialising in software quality and the Rust ecosystem. Your role in this
task is to conduct a **complete audit of the test infrastructure** of the gtk-cross-platform project, cross-reference
the findings against industry best practices, and deliver a prioritised, actionable execution plan — without depending
on follow-up.

---

## Project Context

The project is a **GTK4 + Adwaita application** written in Rust, targeting Linux, macOS, Windows, and GNOME Mobile.
It follows **Hexagonal Architecture** (Ports & Adapters) with four layers:

| Layer    | Path                  | Purity rule                                 |
|----------|-----------------------|---------------------------------------------|
| Domain   | `src/core/`           | No GTK/Adw/GLib; pure business logic        |
| Ports    | `src/ports/`          | Rust traits consumed by core and UI         |
| Adapters | `src/infrastructure/` | Implement ports; may use GLib/IO, never GTK |
| UI       | `src/window/`         | GTK/Adw widgets; depends on ports only      |

**Critical types for testing:**

- `IContainerDriver` (`src/ports/i_container_driver.rs`) — central port; implemented by Docker, Podman, containerd, and
  Mock.
- `ContainerDriverFactory` (`src/infrastructure/containers/factory.rs`) — auto-detects the available runtime.
- `spawn_driver_task` (`src/infrastructure/containers/background.rs`) — blocking ↔ GLib main loop bridge via
  `async-channel`; `tokio` is banned in the project.
- `MockContainerDriver` (`src/infrastructure/containers/mock_driver.rs`) — in-memory fake used in integration tests.
- `ContainerStatus` (`src/core/domain/container.rs`) — enum with `Exited(i32)` variant and `from_state` logic.

**Mandatory environment constraints:**

- Widget tests in `tests/widget_test.rs` require a display (GTK init). On Linux: `xvfb-run cargo test --test
  widget_test -- --test-threads=1 --ignored`. On macOS: same without xvfb. Never run these tests in headless CI without
  Xvfb.
- `tokio` is explicitly banned — conflicts with the GLib event loop. Use `async-channel` and `glib::spawn_local`.
- Tests in the `src/core/` layer must not import `gtk4` or `adw`.

**Current tooling state (starting point — verify and supplement in Phase 1):**

- Runner: `cargo nextest` — `.config/nextest.toml` configured with `default` and `ci` (fail-fast) profiles;
  `NEXTEST_PROFILE ?= default` in the Makefile.
- Coverage: `cargo llvm-cov` available via `make coverage` (manual tool; **not part of the CI pipeline**).
- Dev-dependencies: verify `Cargo.toml` — `proptest`, `insta`, `mockall` are not part of the standard project stack.
- CI: `.github/workflows/ci.yml` runs `make test-unit`, `make test-integration`, and `make test-i18n` via nextest.

Reference best practices for this audit:

**Test organisation**

- Unit tests with `#[cfg(test)]` inside the module itself for private function access
- Integration tests in `tests/` consuming public API only
- Doc tests in `///` to keep documentation examples executable

**Test tooling**

- `cargo-nextest` as the primary runner (parallelism, process isolation, native JUnit XML)
- `cargo-llvm-cov` for LLVM-based coverage (supports LCOV, Cobertura XML, HTML)
- `insta` for snapshot testing of complex outputs
- `proptest` or `quickcheck` for property-based testing on critical domains

**Metrics and thresholds**

- Minimum coverage per layer: domain ≥ 90%, infrastructure ≥ 60%, testable UI (no display) ≥ 40%
- Preferred metric: **regions** (captures branches within a line, more precise than lines)
- `cargo-mutants` for validating the effectiveness of existing tests

**Vendor-agnostic output formats**

- Coverage: LCOV (`lcov.info`) or Cobertura XML
- Test results: JUnit XML via nextest
- Terminal summary: `cargo llvm-cov --summary-only`
- GitHub Actions: `$GITHUB_STEP_SUMMARY` for inline reporting on PRs

**CI/CD**

- `cargo test --no-run --locked` as a test compilation step before execution
- `RUSTFLAGS="-D warnings"` to treat warnings as errors
- Branch protection with mandatory status checks on GitHub
- Threshold enforcement via `--fail-under-lines`, `--fail-under-functions`, `--fail-under-regions`

---

## Task

Execute the three phases below in sequence. Do not stop to ask for confirmation between them.

### Phase 1 — Current Structure Survey

Inspect the project and map the current state. For each item below, record what exists and what is absent.
Treat the known state described in the context above as a starting point, but verify and supplement it by reading
the relevant files:

1. **File structure**: presence and count of tests in `tests/`, inline `#[cfg(test)]` modules per layer,
   doc tests in use. Pay special attention to the `#[ignore]` status in `tests/widget_test.rs`.
2. **Cargo.toml**: declared dev-dependencies (or total absence); `cfg(test)` flags; test features.
3. **Runner**: standard `cargo test` or `cargo-nextest`; existence of `.config/nextest.toml`.
4. **Coverage**: `cargo-llvm-cov`, `cargo-tarpaulin`, or absent.
5. **CI/CD**: inspect `.github/workflows/ci.yml` — identify exactly which flags are passed to `cargo test`
   and whether tests in `tests/` are executed. Also evaluate `.github/workflows/flatpak.yml`.
6. **Quality of existing tests**: by layer — verify whether they cover only the happy path, test edge cases
   (`ContainerStatus::from_state` with unknown state, `Exited(i32)` with different codes), whether
   `MockContainerDriver::unavailable()` is sufficiently exercised, and whether `spawn_driver_task` has any coverage.
7. **Reporting**: any coverage summary in stdout or PR? What export format?

Present this survey as a table with columns: `Aspect | Current State | Observation`.

---

### Phase 2 — Gap Analysis

Based on the survey, cross-reference each aspect against best practices. Prioritise the following known gaps and
identify additional gaps:

- **CI runs only `--lib`**: the 11 tests in `tests/container_driver_test.rs` and other integration tests have never
  run in CI.
- **Widget tests permanently excluded**: `tests/widget_test.rs` has 5 tests all `#[ignore]` with no CI path
  to execute them (not even with Xvfb).
- **Zero dev-dependencies**: no `proptest` for `ContainerStatus::from_state`, no `insta` for snapshots of
  `IContainerDriver` outputs.
- **No coverage configured**: impossible to measure maturity per layer.
- **`spawn_driver_task` with no coverage**: critical threading bridge with no tests at all.

For each gap, classify:

- **Impact**: High / Medium / Low — how critical is the gap for quality and maintainability
- **Effort**: High / Medium / Low — implementation complexity estimate
- **Risk of not fixing**: what happens if this gap remains

Present as a structured list, one item per gap.

---

### Phase 3 — Execution Plan

Produce a prioritised execution plan, grouped into phases of incremental delivery. Each item must contain:

- **What to do**: objective description
- **How to do it**: commands, configurations, or minimal code — specific enough to execute without further research.
  Reference concrete project files (e.g. `Cargo.toml`, `.github/workflows/ci.yml`,
  `.config/nextest.toml`).
- **Completion criterion**: how to know it is done
- **Dependencies**: which items must be completed first

Organise by Impact × Effort matrix:

1. **Quick wins** (High impact, Low effort) — must include the CI fix to run `tests/`
2. **Big bets** (High impact, High effort) — includes coverage + thresholds per layer
3. **Fill-ins** (Low impact, Low effort)
4. **Reconsider** (Low impact, High effort) — document the reason for deferral

---

## Constraints

- Do not propose changes to business logic — exclusive scope: test infrastructure and quality.
- Do not use proprietary or paid tools as the primary solution; prefer the Cargo ecosystem and open formats.
- Do not remove existing tests even if weak — the plan must evolve on top of what exists.
- Respect project constraints: no `tokio`, widget tests require a display, `core/` layer without GTK imports.
- Initial thresholds must be conservative (achievable in the current state + growth margin), not ideal.
- The project **is not a workspace** — it is a single crate with `[lib]` + `[[bin]]`.

---

## Delivery Format

Deliver a single structured Markdown document:

```
# Rust Testing Audit — gtk-cross-platform

## Executive Summary
(3-5 lines: current state in one sentence, the 3 most critical gaps, estimated horizon to reach maturity)

## Phase 1: Current Structure
(survey table)

## Phase 2: Gap Analysis
(structured list with Impact, Effort, and Risk)

## Phase 3: Execution Plan
(grouped by priority, with what to do / how to do it / completion criterion / dependencies)

## Configuration References
(ready-to-use snippets: corrected ci.yml, nextest.toml, llvm-cov step, Cargo.toml additions)
```

The document must be self-sufficient: an engineer without prior context must be able to execute the plan from start
to finish reading only this file.
