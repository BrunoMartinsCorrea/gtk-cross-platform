# Compliance Plan — Documented Concepts vs. Implementation

This document tracks the 18 gaps found between what CLAUDE.md, README, and slash commands document
and what the codebase actually implements. Each gap has a severity, an owner category, and a
guardrail to prevent recurrence.

Run `/project:compliance-audit` at any time to re-check current status against this plan.

---

## Gap Registry

### CRITICAL — Blocks i18n correctness

#### GAP-01 · gettext not used on user-visible strings
- **Documented in:** CLAUDE.md §i18n
- **Expected:** every string in `src/window/views/*.rs` and `src/window/components/*.rs` wrapped in `gettext!()`
- **Actual:** strings hardcoded — "Stop container", "Start container", "Container stopped", etc.
- **Fix:** replace all user-visible string literals with `gettext!("…")` imports via `gettextrs`
- **Guardrail:** add `grep -rn '"[A-Z][a-z]' src/window/ | grep -v gettext` to CI lint step — fail if non-empty

#### GAP-02 · pgettext not used for context-disambiguated strings
- **Documented in:** CLAUDE.md §i18n — "Context disambiguation"
- **Expected:** `pgettext!("container action", "Remove")` vs `pgettext!("image action", "Remove")`
- **Actual:** zero uses of `pgettext!` anywhere
- **Fix:** audit every "Remove", "Start", "Stop" label — wrap with `pgettext!("resource action", "…")`
- **Guardrail:** compliance-audit Category 1 check

#### GAP-03 · ngettext not used for countable strings
- **Documented in:** CLAUDE.md §i18n — "Plurals"
- **Expected:** `ngettext!("1 container", "{n} containers", n)`
- **Actual:** format strings with hardcoded singular/plural
- **Fix:** replace `format!("{n} containers")` patterns with `ngettext!`
- **Guardrail:** compliance-audit Category 1 check

#### GAP-04 · po/POTFILES references stale .vala paths
- **Documented in:** CLAUDE.md §i18n
- **Expected:** lists actual `.rs` and `.ui` files under `src/`
- **Actual:** lists `src/app.vala`, `src/window/main_window.vala` — files that do not exist
- **Fix:** rewrite POTFILES with real paths: `src/app.rs`, `src/window/main_window.rs`, `src/window/views/*.rs`, `src/window/components/*.rs`, `data/resources/window.ui`
- **Guardrail:** add CI step: `while read f; do test -f "$f" || (echo "POTFILES stale: $f" && exit 1); done < po/POTFILES`

---

### HIGH — CLAUDE.md compliance

#### GAP-05 · Icon-only buttons missing accessible label Property
- **Documented in:** CLAUDE.md §Design Standards — "Icon-only buttons must have…"
- **Expected:** `btn.update_property(&[gtk4::accessible::Property::Label(label)])`
- **Actual:** `resource_row::icon_button()` only calls `set_tooltip_text()` — screen readers silent
- **Fix:** add `update_property` call alongside `set_tooltip_text` in `icon_button()` helper
- **Guardrail:** widget test in `tests/widget_test.rs` asserting `accessible_name()` is non-empty for each icon button

#### GAP-06 · StatusBadge missing accessible role
- **Documented in:** CLAUDE.md §Design Standards + implement-container-ui.md
- **Expected:** `set_accessible_role(gtk4::AccessibleRole::Status)` in `status_badge::new()`
- **Actual:** badge is a plain Label with CSS class — no role declared
- **Fix:** call `badge.set_accessible_role(gtk4::AccessibleRole::Status)` in constructor
- **Guardrail:** unit test: `assert_eq!(badge.accessible_role(), AccessibleRole::Status)`

#### GAP-07 · Focus not restored after destructive actions
- **Documented in:** CLAUDE.md §Design Standards — "Focus management"
- **Expected:** after remove_container completes → focus moves to next row or empty-state
- **Actual:** no `grab_focus()` or `set_focus_child()` after any remove callback
- **Fix:** in each remove handler, store next-sibling reference before removal, call `next.grab_focus()` on success
- **Guardrail:** manual test checklist in CONTRIBUTING.md §Testing Checklist

#### GAP-08 · Dialog focus not restored to trigger widget
- **Documented in:** CLAUDE.md §Design Standards — "after any dialog closes, return focus"
- **Expected:** `confirm_dialog::ask()` restores focus to the button that opened it
- **Actual:** `adw::AlertDialog::present()` does not restore focus on close
- **Fix:** capture `trigger_widget.downgrade()` before presenting; in response handler `trigger_widget.upgrade()?.grab_focus()`
- **Guardrail:** manual test checklist in CONTRIBUTING.md §Testing Checklist

#### GAP-09 · Only 1 of 4 responsive breakpoints implemented
- **Documented in:** CLAUDE.md §Responsive breakpoints (table with 4 entries)
- **Expected:** four `<object class="AdwBreakpoint">` in `data/resources/window.ui` at 360/600/768 sp
- **Actual:** one breakpoint at 720 sp (collapses split view)
- **Fix:** add breakpoints for 360 sp (bottom nav bar, margins 16 sp), 600 sp (margins 24 sp), 768 sp (margins 32 sp)
- **Guardrail:** compliance-audit Category 3; visual regression via `make run-mobile`

#### GAP-10 · GestureLongPress absent (mobile context menu)
- **Documented in:** CLAUDE.md §Design Standards — "Avoid menus activated only by right-click"
- **Expected:** every row with a context action has a `gtk4::GestureLongPress` gesture attached
- **Actual:** zero uses of `GestureLongPress` in src/
- **Fix:** in `resource_row::new()`, attach a `GestureLongPress` that triggers the same menu as right-click
- **Guardrail:** compliance-audit Category 7

#### GAP-11 · RTL directional icons not protected
- **Documented in:** CLAUDE.md §i18n — "RTL layout"
- **Expected:** `media-playback-start-symbolic` and arrow icons call `widget.set_direction(TextDirection::Ltr)`
- **Actual:** no `set_direction` calls anywhere in src/
- **Fix:** wrap every directional icon in views with the `Ltr` direction guard
- **Guardrail:** compliance-audit Category 1 (RTL check)

#### GAP-12 · Version mismatch between CLAUDE.md and Cargo.toml
- **Documented in:** CLAUDE.md §Dependencies — "gtk4 = 0.11 (v4_14), libadwaita = 0.9 (v1_6)"
- **Actual:** Cargo.toml has gtk4 = 0.9 (v4_12), adw = 0.7 (v1_4)
- **Fix:** update CLAUDE.md §Dependencies to reflect actual Cargo.toml versions; keep Cargo.toml as source of truth
- **Guardrail:** CI step: `grep 'gtk4' CLAUDE.md | grep -q "$(cargo metadata … | jq …)"` — or simply: update CLAUDE.md when bumping deps

---

### MEDIUM — Polish and completeness

#### GAP-13 · Touch target CSS class not applied to all buttons
- **Documented in:** CLAUDE.md §Design Standards + style.css
- **Expected:** `refresh_button`, `menu_button` in window.ui have `.action-button` class
- **Actual:** CSS rule exists but not applied to those specific widgets
- **Fix:** add `<style><class name="action-button"/></style>` to `refresh_button` and `menu_button` in window.ui
- **Guardrail:** compliance-audit Category 2 (Touch targets)

#### GAP-14 · Restart, pause, unpause not exposed in UI
- **Documented in:** README §Features — full container lifecycle
- **Expected:** containers_view.rs exposes restart, pause, unpause actions
- **Actual:** only start, stop, remove implemented in UI (driver methods exist)
- **Fix:** add three `icon_button` calls in containers_view for restart/pause/unpause
- **Guardrail:** compliance-audit Category 10 (Feature completeness)

#### GAP-15 · Image inspect detail pane incomplete; pull/tag absent
- **Documented in:** README §Features
- **Expected:** images_view shows full metadata, pull and tag actions
- **Actual:** only list + remove; detail pane minimal
- **Fix:** expand images_view detail groups; add pull/tag icon buttons
- **Guardrail:** compliance-audit Category 10

#### GAP-16 · Volume create and network create absent in UI
- **Documented in:** README §Features
- **Expected:** volumes_view and networks_view have create actions
- **Actual:** only list + remove
- **Fix:** add create dialogs in volumes_view and networks_view
- **Guardrail:** compliance-audit Category 10

---

### LOW — Nice-to-have

#### GAP-17 · GreetUseCase orphaned (never called)
- **Documented in:** CLAUDE.md §project structure
- **Actual:** exists in `src/core/use_cases/greet_use_case.rs`, never used by any view
- **Fix:** either wire it to a welcome message in the empty state or remove it; do not leave dead code
- **Guardrail:** `cargo clippy -- -D warnings` will catch unused items if `pub(crate)` is set correctly

#### GAP-18 · pt_BR translation has only 4 strings
- **Documented in:** CLAUDE.md §i18n + README §Localization
- **Actual:** `po/pt_BR.po` has 4 translated entries; most UI strings untranslatable due to missing gettext wrapping
- **Fix:** blocked by GAP-01 through GAP-03; after those are fixed, run `make pot-update` and translate
- **Guardrail:** CI step: `msgfmt --statistics po/pt_BR.po 2>&1 | grep -q "0 translated" && exit 1 || true`

---

## Guardrails Summary

These checks prevent future regressions. Add them to `.github/workflows/ci.yml`.

### CI lint steps to add

```yaml
- name: Check for unwrapped user strings
  run: |
    # Fail if any quoted Title-case string in window/ is not inside gettext/pgettext/ngettext
    # This is a heuristic; tune the regex as needed
    ! grep -rn '"[A-Z][a-z][a-z]' src/window/ | grep -v 'gettext\|pgettext\|ngettext\|#\[' | grep -q .

- name: Verify POTFILES paths exist
  run: |
    while IFS= read -r f; do
      [[ "$f" == \#* ]] && continue
      [[ -z "$f" ]] && continue
      test -f "$f" || { echo "POTFILES stale: $f"; exit 1; }
    done < po/POTFILES

- name: Check for println/eprintln
  run: |
    ! grep -rn 'println!\|eprintln!\|dbg!' src/ | grep -v '#\[cfg(test)\]' | grep -q .

- name: Check for tokio usage
  run: |
    ! grep -rn 'use tokio\|extern crate tokio' src/ | grep -q .

- name: Check for GTK in core/ports
  run: |
    ! grep -rn 'gtk4\|adw::' src/core/ src/ports/ | grep -q .
```

### CLAUDE.md update rule (process guardrail)

Add to CLAUDE.md §Build and Run Commands:

> When bumping a dependency in `Cargo.toml`, update the version table in CLAUDE.md §Dependencies in the same commit. The two must stay in sync.

### Pull request checklist (add to CONTRIBUTING.md)

- [ ] New user-visible strings use `gettext!()` / `pgettext!()` / `ngettext!()`
- [ ] New icon-only buttons call both `set_tooltip_text` AND `update_property(Property::Label)`
- [ ] New list rows with actions have a `GestureLongPress` attached
- [ ] New layout changes have a corresponding `AdwBreakpoint` for each threshold they affect
- [ ] `po/POTFILES` updated if new `.rs` or `.ui` files added
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `make fmt-fix && make lint` passes

---

## Progress Tracking

| Gap | Severity | Status | Fixed in |
|-----|----------|--------|----------|
| GAP-01 gettext missing | CRITICAL | ✅ fixed 2026-04-25 | all views/components |
| GAP-02 pgettext missing | CRITICAL | ✅ fixed 2026-04-25 | per-resource context labels |
| GAP-03 ngettext missing | CRITICAL | ✅ fixed 2026-04-25 | main_window prune message |
| GAP-04 POTFILES stale | CRITICAL | ✅ fixed 2026-04-25 | real .rs/.ui paths |
| GAP-05 A11Y icon label | HIGH | ✅ fixed 2026-04-25 | resource_row::icon_button |
| GAP-06 StatusBadge role | HIGH | ✅ fixed 2026-04-25 | AccessibleRole::Status via builder |
| GAP-07 Focus after remove | HIGH | ✅ fixed 2026-04-25 | all 4 views |
| GAP-08 Dialog focus restore | HIGH | ✅ fixed 2026-04-25 | confirm_dialog::ask |
| GAP-09 Missing breakpoints | HIGH | ✅ fixed 2026-04-25 | 3 new AdwBreakpoint in window.ui |
| GAP-10 GestureLongPress | HIGH | ✅ fixed 2026-04-25 | all 4 views |
| GAP-11 RTL direction guard | HIGH | ✅ fixed 2026-04-25 | containers_view directional icons |
| GAP-12 Version mismatch docs | HIGH | ✅ fixed 2026-04-25 | CLAUDE.md updated to 0.9/0.7 |
| GAP-13 Touch target CSS class | MEDIUM | ✅ fixed 2026-04-25 | action-button class in window.ui |
| GAP-14 Restart/pause/unpause UI | MEDIUM | ✅ fixed 2026-04-25 | pause/unpause buttons added |
| GAP-15 Image inspect expanded | MEDIUM | ✅ fixed 2026-04-25 | Size/Created/Digest in detail pane |
| GAP-16 Volume/network create | MEDIUM | ✅ fixed 2026-04-25 | create_network port + dialogs in both views |
| GAP-17 GreetUseCase orphaned | LOW | won't fix — has tests in greet_use_case_test.rs |
| GAP-18 pt_BR coverage | LOW | ✅ fixed 2026-04-25 | 100+ entries with correct .rs paths |
