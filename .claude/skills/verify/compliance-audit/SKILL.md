---
name: verify:compliance-audit
version: 1.0.0
description: Audit documented concepts vs. implementation; outputs gap report with severity
---

# verify:compliance-audit

Audit this GTK4/Rust codebase for documented concepts that are not yet implemented or are incorrectly implemented. This
command is self-contained — run it fresh without prior conversation context.

## What to audit

Read every source file under `src/`, `data/`, `po/`, `tests/`, `Cargo.toml`, `Makefile`, and `CLAUDE.md`, then check
each category below. Report every gap with: **concept → documented in → expected location → actual status → severity**.

---

## Category 1 — Internationalisation (i18n)

1. **Every user-visible string** in `src/window/` must use `gettext!("…")`. Search for string literals inside
   `gtk4::Label`, `adw::ActionRow`, `adw::Toast`, button labels, dialog titles, and tooltip text that are NOT wrapped in
   `gettext!()`, `pgettext!()`, or `ngettext!()`.
2. **pgettext!** — when the same English word maps to different concepts (e.g. "Remove" for containers vs images),
   `pgettext!("context", "string")` must be used. Verify at least one use per resource type.
3. **ngettext!** — any string containing a count (e.g. "1 container", "3 images") must use `ngettext!()`. Never
   `format!("{n} containers")`.
4. **po/POTFILES** — must list actual `.rs` and `.ui` files, not stale `.vala` paths. Cross-reference with real file
   tree.
5. **RTL directional icons** — icons like `media-playback-start-symbolic`, `go-previous-symbolic`, `go-next-symbolic`
   must call `widget.set_direction(gtk4::TextDirection::Ltr)` to prevent incorrect mirroring in Arabic/Hebrew/Persian
   locales.

---

## Category 2 — Accessibility (A11Y)

1. **Icon-only buttons** — every button created by `resource_row::icon_button()` or similar helpers must call both
   `set_tooltip_text(label)` AND `update_property(&[gtk4::accessible::Property::Label(label)])`. Tooltip alone is
   insufficient for screen readers (WCAG 2.4.6).
2. **StatusBadge accessible role** — `status_badge::new()` must call `set_accessible_role(gtk4::AccessibleRole::Status)`
   or equivalent. Color alone must never convey state without a text label.
3. **Focus after destructive action** — after `remove_container` / `remove_image` / `remove_volume` / `remove_network`
   completes, focus must move to the next list row or the empty-state widget. Verify with `grab_focus()` or
   `set_focus_child()` calls.
4. **Dialog focus restoration** — after any `adw::AlertDialog` (confirm_dialog::ask) closes, focus must return to the
   widget that triggered it. Verify the response handler restores focus.
5. **Touch targets** — `.action-button` CSS class must be applied to all interactive buttons (including
   `refresh_button`, `menu_button` in window.ui). Check `data/resources/style.css` sets
   `min-width: 44px; min-height: 44px` AND that widget templates apply the class.

---

## Category 3 — Responsive Breakpoints

CLAUDE.md declares four breakpoints. Verify `data/resources/window.ui` contains all four
`<object class="AdwBreakpoint">`:

| Breakpoint      | Condition        | Expected behaviour                                |
|-----------------|------------------|---------------------------------------------------|
| GNOME Mobile    | max-width: 360sp | Collapse split view, margins 16sp, bottom tab bar |
| Tablet          | max-width: 600sp | margins 24sp                                      |
| Desktop compact | max-width: 768sp | margins 32sp                                      |
| Desktop normal  | > 768sp          | margins 48sp (default, no breakpoint needed)      |

Report missing breakpoints. Report breakpoints present in `.ui` that are not in CLAUDE.md.

---

## Category 4 — Threading Rules

1. `src/window/views/` must be the **only** layer calling `spawn_driver_task`. Grep all other `src/` files for
   `spawn_driver_task` — report any non-view call.
2. No `tokio` in `Cargo.toml` or any `use tokio` in `src/`. Report if found.
3. All channels must be `async_channel::bounded(1)`. Report any `std::sync::mpsc`, `std::sync::channel`, or unbounded
   channels.
4. No GTK calls outside the GTK main thread — no `gtk4::` / `adw::` usage in `src/infrastructure/` or `src/core/`.

---

## Category 5 — Architecture (Hexagonal)

1. `src/core/` — must not import `gtk4`, `adw`, `gio`, `glib`. Grep and report violations.
2. `src/ports/` — same constraint. Grep and report violations.
3. `src/infrastructure/` — must not import `gtk4` or `adw`. `glib`/`gio` is allowed for async-channel integration.
4. `src/app.rs::activate()` — must be the **only** place concrete types are wired to ports. Report any
   `ContainerDriverFactory::detect()` calls outside `activate()`.

---

## Category 6 — Logging

All log output must go through `AppLogger` (`src/infrastructure/logging/app_logger.rs`) using `g_debug!` / `g_info!` /
`g_warning!` / `g_critical!`. Grep entire `src/` for:

- `println!` — report each occurrence with file + line
- `eprintln!` — report each occurrence
- `dbg!` — report each occurrence
- `print!` — report each occurrence

---

## Category 7 — GestureLongPress (Mobile context menu)

CLAUDE.md requires `gtk4::GestureLongPress` as the touchscreen alternative to right-click. Check:

1. Is `GestureLongPress` used in any view or component?
2. If not: list every row widget (`adw::ActionRow`, `gtk4::ListBoxRow`) that has a context action — these are the
   missing sites.

---

## Category 8 — Version Consistency

Cross-reference:

- `CLAUDE.md` dependency versions (gtk4, libadwaita, glib, gio)
- `Cargo.toml` actual versions and features
- `README.md` stated requirements

Report any mismatch. The source of truth is `Cargo.toml`.

---

## Category 9 — POTFILES & Translation Coverage

1. `po/POTFILES` — list every path. For each path, verify the file exists. Report stale paths.
2. `po/LINGUAS` — list locales. For each locale code, verify a `.po` file exists.
3. Count translated vs untranslated entries in each `.po` file (lines matching `^msgstr ""` after a `^msgid`).
4. `po/meson.build` — does it reference real files? Is it necessary in a Cargo project?

---

## Category 10 — Feature Completeness vs. README

Read `README.md` features section. For each claimed feature, verify it is callable from the UI:

- Container: list, start, stop, restart, pause, unpause, remove, inspect
- Image: list, remove, inspect, pull, tag
- Volume: list, remove, create
- Network: list, remove

Report features claimed in README with no corresponding UI trigger or driver method.

---

## Output format

For each gap found, emit one block:

```
## [SEVERITY] CATEGORY — Concept name

- **Documented in:** <file>:<line or section>
- **Expected location:** <file or module>
- **Status:** ABSENT | PARTIAL | STALE | MISMATCH
- **Detail:** <one sentence — what is missing and why it matters>
- **Fix:** <one sentence — what needs to change>
```

Severity levels: CRITICAL (blocks release), HIGH (CLAUDE.md compliance), MEDIUM (polish), LOW (nice-to-have).

End with a summary table: category → gap count → highest severity.

Do not report items that are correctly implemented. Focus only on gaps and mismatches.
