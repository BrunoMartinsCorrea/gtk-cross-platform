---
description: Tooling design — restructure the Makefile as a SOLID lifecycle instrument and consolidate GitHub Actions workflows into exactly 2 files. Covers environment-governance sub-cycle: Makefile redesign (aggregators delegate to specifics), CI/CD alignment, and workflow consolidation.
---

# /project:tooling-design

Design and implement the project's development tooling infrastructure. This command covers the
environment-governance sub-cycle:

1. **Makefile redesign** — restructure as a SOLID lifecycle instrument where aggregators delegate to specifics
2. **Workflow consolidation** — consolidate GitHub Actions into exactly 2 files aligned with the Makefile

Run it fresh without prior conversation context. Both phases are independent but Phase 2 depends on
Phase 1 targets existing.

---

## Phase 1: Makefile Redesign

> Read `CLAUDE.md` before starting — especially §Build and Run Commands and §Architecture.

Restructure the `Makefile` following separation of responsibilities:
**aggregators** have generic names and only list dependencies; **specifics** execute exactly one tool.
The result is a Makefile that serves as the complete development lifecycle map and source of truth
for CI/CD pipelines.

### Design Principle

```
AGGREGATORS (generic names — only list dependencies)
    setup      test      validate      dist      clean
       │          │           │           │          │
       ▼          ▼           ▼           ▼          ▼
SPECIFICS (qualified — execute exactly one tool/command)
 setup-rust  test-unit  validate-lint  dist-flatpak  clean-build
 setup-macos test-integration          dist-macos     clean-icons
 setup-linux test-i18n  validate-deps  dist-flatpak-arm
```

**Absolute rule**: an aggregator target executes no commands directly — it only lists dependencies
that are specific targets.

**Pipeline rule**: the `Makefile` is the source of truth. `.github/workflows/ci.yml` and
`.github/workflows/release.yml` must call Makefile targets where possible.
`make ci` must be the exact local replica of the `ci.yml` pipeline.

### Phase 0: Audit Inconsistencies

Read and register differences between current state and this spec:

```sh
cat Makefile
cat .github/workflows/ci.yml
cat .github/workflows/release.yml
cat .config/nextest.toml
```

Known inconsistencies to fix:

1. **Nextest profile**: CI uses `--profile ci` (fail-fast=true); use `NEXTEST_PROFILE ?= default`
   so the same target works locally (default) and in CI (`make test-unit NEXTEST_PROFILE=ci`).
2. **`make ci` target**: correct is `ci: validate test` (coverage is NOT part of CI pipeline).
3. **`cargo audit` missing**: add `audit` as specific target and include in `validate-deps`.
4. **`release-github` incomplete**: must include all 4 artifacts: flatpak-x86_64, flatpak-aarch64, macos-dmg, windows-zip.
5. **`run` idempotency**: `run: build schema` (not `run: setup build schema` — setup on every run is wasteful).
6. **CI inline scripts**: `ci.yml` duplicates logic of check-version, check-potfiles, validate-metainfo,
   validate-desktop inline. After redesigning Makefile, update `ci.yml` to call targets.

### Acceptance Criterion: New Developer

```sh
git clone <repo> && cd gtk-cross-platform && make
```

Expected result: environment configured automatically for detected platform, binary compiled,
application runs — without reading any documentation first.

Requirements:
- `.DEFAULT_GOAL := run`
- `run` depends on `build schema` (not `setup`)
- `setup` is **idempotent** (checks presence of each dependency before installing)
- `make` alone executes `run`, which executes `build schema` automatically

### Global Variables

Maintain all current variables. Add to the variables block:

```makefile
OS             := $(shell uname 2>/dev/null || echo Windows)
GIT_TAG        := $(shell git describe --tags --abbrev=0 2>/dev/null || echo "v$(VERSION)")
NEXTEST_PROFILE ?= default
```

### Section 0: Meta — `help`

```makefile
.DEFAULT_GOAL := run

help: ## Show all available targets
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
	  awk 'BEGIN {FS = ":.*?## "}; {printf "  %-26s %s\n", $$1, $$2}'
```

Each target must have a `## description` comment on the same line to appear in `make help`.

### Section 1: SETUP

| Target             | Type          | Implementation                                                       |
|--------------------|---------------|----------------------------------------------------------------------|
| `setup`            | **aggregator**| `setup-rust setup-platform setup-cargo-deps`                         |
| `setup-rust`       | specific      | Check `which cargo`; if absent, install via rustup                   |
| `setup-platform`   | specific      | Detect OS; delegate to `setup-macos/linux/windows`                   |
| `setup-macos`      | specific      | Idempotent Homebrew install of gtk4 libadwaita adwaita-icon-theme dylibbundler create-dmg |
| `setup-linux`      | specific      | Detect apt/dnf; idempotent install of libgtk-4-dev libadwaita-1-dev  |
| `setup-windows`    | specific      | Print MSYS2/MINGW64 instructions, exit 1                             |
| `setup-cargo-deps` | specific      | `cargo fetch`                                                        |

### Section 2: BUILD

| Target          | Type          | Implementation                                      |
|-----------------|---------------|-----------------------------------------------------|
| `build`         | **aggregator**| `build-debug`                                       |
| `build-debug`   | specific      | `cargo build`                                       |
| `build-release` | specific      | `cargo build --release`                             |
| `schema`        | specific      | `glib-compile-schemas $(SCHEMA_DIR)`                |
| `run`           | **aggregator**| depends `build schema`; executes `GSETTINGS_SCHEMA_DIR=$(SCHEMA_DIR) cargo run` |
| `run-mobile`    | **aggregator**| depends `build schema`; executes with `GTK_DEBUG=interactive` |

### Section 3: FORMAT & LINT

| Target      | Type          | Implementation                                    |
|-------------|---------------|---------------------------------------------------|
| `format`    | **aggregator**| `fmt-fix lint lint-i18n`                          |
| `fmt`       | specific      | `cargo fmt --check`                               |
| `fmt-fix`   | specific      | `cargo fmt`                                       |
| `lint`      | specific      | `cargo clippy -- -D warnings`                     |
| `lint-i18n` | specific      | Loop `msgfmt --check --check-format` on `po/*.po` |

### Section 4: TEST

Use `NEXTEST_PROFILE ?= default` in all test targets.

| Target             | Type          | Implementation                                                                                    |
|--------------------|---------------|---------------------------------------------------------------------------------------------------|
| `test`             | **aggregator**| `test-unit test-integration test-i18n`                                                            |
| `test-unit`        | specific      | `cargo nextest run --profile $(NEXTEST_PROFILE) --lib`                                            |
| `test-integration` | specific      | `cargo nextest run --profile $(NEXTEST_PROFILE) --test container_driver_test --test greet_use_case_test` |
| `test-i18n`        | specific      | `cargo nextest run --profile $(NEXTEST_PROFILE) --test i18n_test`                                 |
| `coverage`         | specific      | `cargo llvm-cov --lib --test container_driver_test --summary-only` (manual only; not in `ci`)    |

### Section 5: VALIDATE (Quality Gates)

| Target              | Type          | Implementation                                                                    |
|---------------------|---------------|-----------------------------------------------------------------------------------|
| `validate`          | **aggregator**| `validate-format validate-lint validate-metadata validate-i18n validate-deps`     |
| `validate-format`   | **aggregator**| `fmt`                                                                             |
| `validate-lint`     | **aggregator**| `lint`                                                                            |
| `validate-metadata` | **aggregator**| `validate-metainfo validate-desktop check-version`                                |
| `validate-i18n`     | **aggregator**| `lint-i18n check-potfiles`                                                        |
| `validate-deps`     | **aggregator**| `audit deny spell-check check-unused-deps`                                        |
| `validate-metainfo` | specific      | `appstreamcli validate --pedantic data/com.example.GtkCrossPlatform.metainfo.xml` |
| `validate-desktop`  | specific      | `desktop-file-validate data/com.example.GtkCrossPlatform.desktop`                 |
| `check-version`     | specific      | Cross-check Cargo.toml version vs metainfo.xml                                    |
| `check-potfiles`    | specific      | Cross-check source files with gettext() vs po/POTFILES                            |
| `audit`             | specific      | `cargo audit`                                                                     |
| `deny`              | specific      | `cargo deny check`                                                                |
| `spell-check`       | specific      | `typos .`                                                                         |
| `check-unused-deps` | specific      | `cargo machete`                                                                   |
| `ci`                | **aggregator**| `validate test`                                                                   |

### Sections 6–9: ICONS, PACKAGE, PUBLISH, CLEAN

Preserve all existing logic for icons (PNG/macOS/Windows generation), dist-flatpak/arm/run/install,
dist-macos (bundle + DMG), release/release-tag/release-github (all 4 artifacts), clean/clean-all,
cache-info/cache-prune. Apply aggregator/specific pattern consistently.

### Backwards-Compatibility Aliases

```makefile
flatpak-build:      dist-flatpak
flatpak-run:        dist-flatpak-run
flatpak-install:    dist-flatpak-install
flatpak-build-arm:  dist-flatpak-arm
dmg:                dist-macos
test-nextest:       test
```

### Verification

1. `make help` — complete table with all targets
2. `make setup` — idempotent; second run skips already-installed tools
3. `make build` → `make lint` → `make test` → `make validate` each pass individually
4. `make ci` — full chain; fails on first broken gate
5. `make test-unit NEXTEST_PROFILE=ci` — uses ci profile (fail-fast=true)
6. CLAUDE.md updated with new target names

---

## Phase 2: Workflow Consolidation

> **Prerequisite**: Phase 1 (Makefile redesign) must be complete. Makefile targets must exist
> before `ci.yml` can delegate to them.

Consolidate GitHub Actions workflows into exactly **2 files**:

1. **`.github/workflows/ci.yml`** — complete quality gate; triggers only on pull_request
2. **`.github/workflows/release.yml`** — build and publish artifacts; triggers only on version tag

### Step 1 — Audit existing workflows

Read all files in `.github/workflows/`. For each file, record:
- The `on:` trigger
- Each job name and what it does
- `if:` conditions that limit jobs to specific events

### Step 2 — Rewrite `ci.yml`

Triggers: **only** `pull_request` with `types: [opened, synchronize, reopened]`.
No `push:`, `schedule:`, or `workflow_dispatch:`.

Jobs:
- `lint` — delegates to Makefile targets (fmt, lint, test-unit, test-integration, validate-metainfo, validate-desktop, check-version, lint-i18n, check-potfiles, test-i18n)
- `editorconfig` — `npx editorconfig-checker`
- `audit` — `cargo audit` (parallel job)
- `deny` — cargo-deny action
- `typos` — typos-action
- `unused-deps` — `cargo machete` (parallel job)

Remove: `flatpak` job (nightly publishing is release concern), `coverage` job (not a CI gate).

### Step 3 — Leave `release.yml` untouched

`.github/workflows/release.yml` already has the correct trigger (`push: tags: ['v[0-9]*.[0-9]*.[0-9]*']`)
and the correct jobs (`flatpak-x86_64`, `flatpak-aarch64`, `macos`, `windows`, `publish`). **Do not modify.**

### Step 4 — Delete redundant workflow files

If they exist, delete:
- `.github/workflows/audit.yml` (consolidated into ci.yml)
- `.github/workflows/editorconfig.yml` (consolidated into ci.yml)

### Step 5 — Verify

```sh
ls .github/workflows/          # Must show only ci.yml and release.yml
grep -A8 "^on:" .github/workflows/ci.yml      # Must show only pull_request trigger
grep -A8 "^on:" .github/workflows/release.yml # Must show only push tags trigger
grep "run: make" .github/workflows/ci.yml      # Steps must call make targets
```
