---
name: plan:add-quality-gates
version: 1.0.0
description: Implement all missing CI quality gates (AppStream, desktop, deny, typos, nextest, coverage)
---

# plan:add-quality-gates

Implement all missing quality gates in this GTK4/Rust/Flatpak project. This command is
self-contained — run it fresh without prior conversation context.

**Repository:** `gtk-cross-platform`
**Stack:** Rust 2024 edition · GTK4 0.9 · libadwaita 0.7 · glib/gio 0.20 · Flatpak/GNOME Platform 48

---

## What to read before implementing

Read the following files in full before making any changes:

- `CLAUDE.md` — architecture rules and project conventions
- `.github/workflows/ci.yml` — current pipeline (base for modifications)
- `Makefile` — existing local targets (add local equivalents for the new gates)
- `Cargo.toml` — current package version and dependency list
- `data/com.example.GtkCrossPlatform.metainfo.xml` — version declared in AppStream
- `data/com.example.GtkCrossPlatform.desktop` — desktop entry
- `po/POTFILES` — current list of files registered for i18n
- `.config/nextest.toml` — nextest profiles already configured (ci and default)
- `.editorconfig` — current checker scope

Do not modify `CLAUDE.md`, `README.md`, domain files (`src/core/`), or existing tests.

---

## Gate 1 — AppStream metainfo validation

**File:** `.github/workflows/ci.yml`
**Job:** `lint` (add step after `i18n lint`)

Add a step that installs `appstream` and validates the metainfo with the `--pedantic` flag:

```yaml
-   name: Validate AppStream metadata
    run: |
        sudo apt-get install -y appstream
        appstreamcli validate --pedantic data/com.example.GtkCrossPlatform.metainfo.xml
```

**Makefile:** add the `validate-metainfo` target:

```makefile
validate-metainfo:
	appstreamcli validate --pedantic data/com.example.GtkCrossPlatform.metainfo.xml
```

---

## Gate 2 — .desktop file validation

**File:** `.github/workflows/ci.yml`
**Job:** `lint` (add step after Gate 1)

```yaml
-   name: Validate .desktop file
    run: |
        sudo apt-get install -y desktop-file-utils
        desktop-file-validate data/com.example.GtkCrossPlatform.desktop
```

**Makefile:** add the `validate-desktop` target:

```makefile
validate-desktop:
	desktop-file-validate data/com.example.GtkCrossPlatform.desktop
```

**Note:** the two `apt-get install` calls for Gates 1 and 2 can be combined into a single
installation step at the beginning of the job to save time:

```yaml
-   name: Install validation tools
    run: sudo apt-get install -y appstream desktop-file-utils gettext
```

If the job already installs `gettext` separately, remove the duplicate step and consolidate into one.

---

## Gate 3 — Version consistency Cargo.toml ↔ metainfo.xml

**File:** `.github/workflows/ci.yml`
**Job:** `lint` (add step after Gate 2)

```yaml
-   name: Check version consistency (Cargo.toml vs metainfo.xml)
    run: |
        CARGO_VER=$(grep '^version' Cargo.toml | head -1 | grep -oP '[\d.]+')
        META_VER=$(grep -oP '(?<=version=")[^"]+' data/com.example.GtkCrossPlatform.metainfo.xml | head -1)
        echo "Cargo version: $CARGO_VER"
        echo "Metainfo version: $META_VER"
        [ "$CARGO_VER" = "$META_VER" ] || \
          { echo "ERROR: Version mismatch — Cargo.toml=$CARGO_VER metainfo.xml=$META_VER"; exit 1; }
```

**Makefile:** add the `check-version` target:

```makefile
check-version:
	@CARGO_VER=$$(grep '^version' Cargo.toml | head -1 | grep -oP '[\d.]+'); \
	 META_VER=$$(grep -oP '(?<=version=")[^"]+' data/com.example.GtkCrossPlatform.metainfo.xml | head -1); \
	 echo "Cargo: $$CARGO_VER  Metainfo: $$META_VER"; \
	 [ "$$CARGO_VER" = "$$META_VER" ] || { echo "ERROR: version mismatch"; exit 1; }
```

---

## Gate 4 — POTFILES completeness

**File:** `.github/workflows/ci.yml`
**Job:** `lint` (add step alongside the i18n block)

```yaml
-   name: i18n POTFILES completeness
    run: |
        grep -rl 'gettext!(' src/ | sort > /tmp/has_gettext.txt
        sort po/POTFILES | grep '\.rs$' > /tmp/potfiles_rs.txt
        MISSING=$(comm -23 /tmp/has_gettext.txt /tmp/potfiles_rs.txt)
        if [ -n "$MISSING" ]; then
          echo "ERROR: Files with gettext!() not registered in po/POTFILES:"
          echo "$MISSING"
          exit 1
        fi
        echo "POTFILES completeness OK"
```

**Makefile:** add the `check-potfiles` target:

```makefile
check-potfiles:
	@grep -rl 'gettext!(' src/ | sort > /tmp/has_gettext.txt; \
	 sort po/POTFILES | grep '\.rs$$' > /tmp/potfiles_rs.txt; \
	 MISSING=$$(comm -23 /tmp/has_gettext.txt /tmp/potfiles_rs.txt); \
	 if [ -n "$$MISSING" ]; then \
	   echo "Files with gettext!() not in POTFILES:"; echo "$$MISSING"; exit 1; \
	 fi
```

---

## Gate 5 — cargo deny (licenses + banned dependencies)

### 5.1 Create `deny.toml` at the repository root

```toml
[graph]
targets = []

[advisories]
ignore = []

[licenses]
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "GPL-3.0-or-later",
    "LGPL-2.1-or-later",
    "Unicode-3.0",
]
exceptions = []

[bans]
multiple-versions = "warn"
deny = [
    # tokio conflicts with the GLib event loop (see CLAUDE.md — Threading section)
    { name = "tokio" },
]

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
```

### 5.2 Add `deny` job in `.github/workflows/ci.yml`

Create an independent job (does not depend on `lint`):

```yaml
deny:
    name: cargo deny
    runs-on: ubuntu-latest
    steps:
        -   uses: actions/checkout@v4

        -   uses: EmbarkStudios/cargo-deny-action@v2
            with:
                command: check
                arguments: --all-features
```

### 5.3 Makefile

```makefile
deny:
	cargo deny check
```

---

## Gate 6 — typos (spell checking)

**File:** `.github/workflows/ci.yml`
**Job:** add independent job `typos`

```yaml
typos:
    name: Spell check (typos)
    runs-on: ubuntu-latest
    steps:
        -   uses: actions/checkout@v4

        -   uses: crate-ci/typos-action@v1
```

If there are legitimate false positives (technical terms, API names), create `.typos.toml` at
the repository root with the exceptions:

```toml
[default.extend-words]
# Add real exceptions here, for example:
# "gio" = "gio"
```

Only create `.typos.toml` if the step fails due to false positives when running locally with
`typos .` — do not create the file preemptively.

**Makefile:**

```makefile
spell-check:
	typos .
```

---

## Gate 7 — Coverage threshold

**File:** `.github/workflows/ci.yml`
**Job:** `coverage` (modify the `Run coverage` step)

Add `--fail-under-lines 60` to the existing command:

```yaml
-   name: Run coverage (summary + threshold)
    run: |
        cargo llvm-cov --lib --summary-only --fail-under-lines 60
        cargo llvm-cov --test container_driver_test --summary-only
        cargo llvm-cov --test greet_use_case_test --summary-only
```

The 60% threshold applies only to `--lib` (domain + infrastructure). Integration tests are
verified separately without a threshold because they target the public API.

**Makefile:** update the existing `coverage` target to reflect the threshold:

```makefile
coverage:
	cargo llvm-cov --lib --test container_driver_test --test greet_use_case_test \
	  --summary-only --fail-under-lines 60
```

---

## Gate 8 — Migrate to cargo nextest in CI

**File:** `.github/workflows/ci.yml`
**Job:** `lint`

`.config/nextest.toml` already defines the `ci` profile with `fail-fast = true` and
`status-level = "all"`. CI still uses `cargo test`. Replace all `cargo test` steps
in the `lint` job with `cargo nextest run --profile ci`:

| Current command                           | Replace with                                                  |
|-------------------------------------------|---------------------------------------------------------------|
| `cargo test --lib`                        | `cargo nextest run --profile ci --lib`                        |
| `cargo test --test container_driver_test` | `cargo nextest run --profile ci --test container_driver_test` |
| `cargo test --test greet_use_case_test`   | `cargo nextest run --profile ci --test greet_use_case_test`   |
| `cargo test --test i18n_test`             | `cargo nextest run --profile ci --test i18n_test`             |

Add nextest installation before the test steps:

```yaml
-   name: Install cargo-nextest
    run: cargo install cargo-nextest --locked
```

Or use the official action which is faster:

```yaml
-   uses: taiki-e/install-action@cargo-nextest
```

**Makefile:** the `test-nextest` target already exists — do not change it.

---

## Gate 9 — cargo machete (unused dependencies)

**File:** `.github/workflows/ci.yml`
**Job:** add independent job `unused-deps`

```yaml
unused-deps:
    name: Unused dependencies
    runs-on: ubuntu-latest
    steps:
        -   uses: actions/checkout@v4

        -   uses: dtolnay/rust-toolchain@stable

        -   name: Install cargo-machete
            run: cargo install cargo-machete --locked

        -   name: Check for unused dependencies
            run: cargo machete
```

**Makefile:**

```makefile
check-unused-deps:
	cargo machete
```

---

## Gate 10 — Expanded EditorConfig scope

**File:** `.github/workflows/ci.yml`
**Job:** `editorconfig`

The current step validates only `src/ data/ po/ tests/`. Expand to include `Cargo.toml`,
`.github/`, and `build.rs`:

```yaml
-   name: Run editorconfig-checker
    run: npx editorconfig-checker src/ data/ po/ tests/ Cargo.toml build.rs .github/
```

---

## Makefile — aggregator target

After implementing all gates, add a `validate` target that groups the checks
runnable locally without CI:

```makefile
validate: check-version check-potfiles validate-metainfo validate-desktop lint lint-i18n fmt
	@echo "All local validations passed."
```

Add `validate` to the existing `.PHONY` line.

---

## Implementation order

Execute in this sequence to keep CI green at each step:

1. **Gates 1 and 2** — install tools, validate metainfo and .desktop (read-only; do not
   fail if the file is already valid)
2. **Gate 3** — check version (confirm manually that `Cargo.toml` and `metainfo.xml`
   are in sync before activating)
3. **Gate 4** — POTFILES completeness (verify locally with `make check-potfiles` before
   committing)
4. **Gate 5** — create `deny.toml` and `deny` job (run `cargo deny check` locally first;
   adjust license `allow` list if necessary)
5. **Gate 6** — `typos` job (run `typos .` locally; create `.typos.toml` only if there are
   false positives)
6. **Gate 8** — migrate to nextest (low risk; `ci` profile already exists)
7. **Gate 7** — coverage threshold (may fail if current coverage < 60%; adjust the threshold
   to the actual current value and then raise it gradually)
8. **Gate 9** — `cargo machete` (may identify unused dependencies that need removal or justification)
9. **Gate 10** — EditorConfig scope (verify locally that new files pass)
10. **Makefile `validate`** — aggregator target after all others are stable

---

## Final verification

After implementing all gates:

1. Run `make validate` locally — must pass without errors.
2. Run `cargo deny check` — must pass without license or CVE errors.
3. Run `typos .` — must return without errors or only with exceptions documented in `.typos.toml`.
4. Open a PR and confirm that all new jobs appear in CI and pass.
5. Do not merge if any new job is in a `skipped` state — investigate the activation condition.

---

## Constraints

- Do not remove existing gates (`cargo fmt`, `cargo clippy`, `cargo audit`, `editorconfig`).
- Do not modify tests in `tests/` or code in `src/`.
- Do not add new dependencies to `Cargo.toml` — all tools are installed via CI action
  or `cargo install`.
- New Makefile targets must follow the existing convention: kebab-case name, no unnecessary comments,
  listed in `.PHONY`.
- If a gate fails during implementation due to the current repository state (e.g. coverage < 60%,
  a typo in a legacy comment), fix the problem or adjust the threshold before activating the gate —
  never mark the gate as `continue-on-error: true`.
