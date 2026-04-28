---
name: pipeline:risk-assessment
description: Evaluates implementation risks — breaking changes in ports, cross-platform compatibility, i18n/A11Y impact, and distribution regressions — before starting a feature.
---

# pipeline:risk-assessment

Invoke with `/pipeline:risk-assessment` to surface risks before implementation starts.

## When to use

- Before any change to `src/ports/` (port trait changes have broad blast radius)
- Before UI changes that affect accessibility or touch targets
- Before changes to `build.rs`, `Makefile`, or CI workflows
- Before adding/removing dependencies in `Cargo.toml`

## Risk categories

### Port trait changes (HIGH risk)

**Trigger:** any change to `src/ports/i_container_driver.rs` or `src/ports/use_cases/`

**Risk:** All 5 adapter implementations (Docker, Podman, containerd, Mock, Dynamic) must implement the new method. Missing implementations cause compile errors across all test targets.

**Mitigation:**
- Add the method with a default implementation if backwards compatibility is possible
- Update `MockContainerDriver` first so tests can compile before adapters are updated
- Verify with `make test` after each adapter update

### Cross-platform regressions (MEDIUM risk)

**Trigger:** changes to file paths, socket paths, process detection, or CI workflow steps

**Risk:** Works on Linux/macOS, fails on Windows or inside Flatpak sandbox.

**Mitigation:**
- Socket paths (`/var/run/docker.sock`, `/run/user/{uid}/podman/podman.sock`) must be runtime-detected, not hardcoded
- `build.rs` must pass `PROFILE` and `PKGDATADIR` correctly for all platforms
- Flatpak sandbox test: `make dist-flatpak-run` to verify sandbox permissions

### i18n regressions (LOW-MEDIUM risk)

**Trigger:** new user-visible strings added or existing strings changed

**Risk:** `make check-potfiles` fails (CI breaks) if file not listed in `po/POTFILES`.

**Mitigation:**
- Add source file to `po/POTFILES` when adding the first `gettext()` call
- Run `make validate-i18n` after string changes

### A11Y regressions (MEDIUM risk)

**Trigger:** new interactive widgets, icon-only buttons, or custom widgets

**Risk:** Screen reader users cannot navigate the new widget.

**Mitigation:**
- Icon-only buttons need `set_tooltip_text` + `update_property(&[accessible::Property::Label(…)])`
- Custom widgets need `update_property(&[accessible::Property::Role(…)])`
- Touch targets ≥ 44×44 sp

### Threading regressions (HIGH risk)

**Trigger:** new driver calls, new background operations, or changes to `spawn_driver_task`

**Risk:** GTK called from non-main thread → crash or undefined behavior.

**Mitigation:**
- All driver calls must go through `spawn_driver_task`
- Never use `tokio` (conflicts with GLib event loop)
- The callback passed to `spawn_driver_task` runs on the GTK main loop via `glib::spawn_local`

### Dependency security (MEDIUM risk)

**Trigger:** adding or updating dependencies in `Cargo.toml`

**Risk:** Introducing a dependency with a known CVE or incompatible license.

**Mitigation:**
- Run `make audit` (cargo audit) after adding any dependency
- Run `make deny` (cargo deny) to verify license compatibility
- Prefer `async-channel` over `tokio` for async — aligns with GLib event loop

## Output

A risk register:

```
| Risk | Category | Likelihood | Impact | Mitigation |
|------|----------|------------|--------|------------|
| ...  | ...      | LOW/MED/HIGH | LOW/MED/HIGH | ... |
```

Plus a one-sentence overall assessment: proceed / proceed with caution / recommend scope reduction.
