---
name: Migrate GTK4 UI XML to Blueprint
description: Convert all GtkBuilder XML (.ui) files to Blueprint (.blp) format and wire blueprint-compiler into build.rs and Makefile, preserving the exact rendered UI
domain: GTK4/Adwaita Blueprint migration
audience: Claude Code agent
language: en
version: 1.0
created: 2026-04-30
---

# Migrate GTK4 UI XML to Blueprint

> **Context:** Migrate GtkBuilder XML UI definitions to Blueprint format without changing the rendered UI
> **Audience:** Claude Code agent
> **Usage:** Reference this prompt when migrating `.ui` XML authoring to `.blp` Blueprint source files

## Role

You are a senior GNOME/GTK4 platform engineer with deep expertise in blueprint-compiler, GLib composite templates, GResource compilation pipelines, and Cargo build scripts. You have migrated multiple GTK4 Rust projects from XML-only workflows to Blueprint-first workflows.

## Context

This is a GTK4 + Adwaita + Rust project following hexagonal architecture. UI is declared in GtkBuilder XML (`.ui` files) compiled into `compiled.gresource` via `glib_build_tools::compile_resources()` in `build.rs`.

**Current UI file inventory (read from `data/resources/`):**
- `data/resources/window.ui` — composite template for `GtkCrossPlatformMainWindow` (subclass of `AdwApplicationWindow`). Contains: `AdwToolbarView`, `AdwHeaderBar` (with `AdwViewSwitcher id=view_switcher_top`, `GtkMenuButton id=menu_button`, `GtkButton id=refresh_button`, `GtkSpinner id=spinner`), `AdwViewSwitcherBar id=view_switcher_bar`, `AdwToastOverlay id=toast_overlay`, `AdwNavigationSplitView id=split_view` with `AdwNavigationPage id=sidebar_page` and `AdwNavigationPage id=content_page`, `GtkStack id=detail_stack` with `AdwStatusPage id=detail_status_page` and `GtkBox id=detail_content`, four `AdwBreakpoint` entries (900sp, 768sp, 600sp, 360sp), `GtkShortcutController`, and `<menu id="primary_menu">`.

**GResource manifest:** `data/resources/resources.gresource.xml` references `window.ui` with `preprocess="xml-stripblanks"`. This manifest must continue to reference `.ui` filenames — blueprint-compiler compiles `.blp` → `.ui`; the `.ui` format remains the GResource input.

**Build script:** `build.rs` calls `glib_build_tools::compile_resources(&["data/resources", "data"], "data/resources/resources.gresource.xml", "compiled.gresource")`. A blueprint-compiler invocation must be inserted **before** this call.

**Blueprint compiler pipeline:**
```
.blp (source) ──► blueprint-compiler batch-compile ──► .ui (generated) ──► glib_build_tools::compile_resources ──► compiled.gresource
```

The `.ui` files produced by blueprint-compiler are build artifacts. They are compiled in-place (output to `data/resources/`) so `resources.gresource.xml` requires no path changes. The `.blp` files become the new human-authored source.

**Critical constraint:** Every widget `id` declared in the original `.ui` is referenced by a Rust `#[template_child]` field in `src/window/main_window.rs`. No ID may be renamed or removed.

## Objective

Migrate every `.ui` XML file to Blueprint format and integrate blueprint-compiler into the Cargo build pipeline without altering the rendered UI or any widget ID.

## Constraints

- Scope only `data/resources/*.ui` — do not touch Rust source files, CSS, GSettings schema, metainfo XML, or `resources.gresource.xml`.
- Do not rename, remove, or add any widget `id` — Rust composite template fields bind to these IDs at compile time.
- Preserve every translatable string exactly as-is — wording, punctuation, and context must not change.
- Keep `data/resources/resources.gresource.xml` referencing `.ui` filenames (blueprint output), not `.blp` filenames.
- Invoke `blueprint-compiler` from `build.rs` via `std::process::Command` — `cargo build` must remain the single build entry point; no separate pre-step may be required.
- Emit generated `.ui` files in-place to `data/resources/` so the existing `resources.gresource.xml` manifest requires no changes.
- Verify parity with `make build` (not `make run`) — no running display is required.

## Steps

1. **Inventory all widget IDs referenced in Rust** — grep `src/window/main_window.rs` for `#[template_child]` annotations and record the field names (these must match `id=` values in the `.ui` file). Done when: you have a complete table mapping every `#[template_child]` field name to its expected widget ID.

2. **Translate each `.ui` file to Blueprint syntax** — for each `data/resources/<name>.ui`, create `data/resources/<name>.blp` using these mapping rules:
   - `<interface>` → `using Gtk 4.0;\nusing Adw 1;` header block
   - `<template class="Foo" parent="Bar">` → `template $Foo : Bar { ... }`
   - `<property name="x">val</property>` → `x: val;`
   - `<property name="x" translatable="yes">str</property>` → `x: _("str");`
   - `<child type="start">` → `[start]` child annotation before the widget
   - `<child type="top">` / `[bottom]` / `[title]` / `[end]` → same annotation pattern
   - `<property name="content"><object ...>` → `content: WidgetClass id { ... };` (inline object value)
   - `<property name="child"><object ...>` → `child: WidgetClass id { ... };`
   - `<style><class name="x"/></style>` → `styles ["x"]`
   - `<accessibility><property name="label" translatable="yes">L</property></accessibility>` → `accessibility { label: _("L"); }`
   - `<object class="AdwBreakpoint"><condition>max-width: Nsp</condition><setter object="w" property="p">v</setter>` → `Adw.Breakpoint { condition ("max-width: Nsp") setters { w.p: v; } }`
   - `<object class="GtkShortcutController"><property name="scope">local</property>` → `ShortcutController { scope: local; ... }`
   - `<object class="GtkShortcut"><property name="trigger">T</property><property name="action">A</property>` → `Shortcut { trigger: T; action: A; }`
   - `<menu id="x">` block → top-level `menu x { ... }` outside the template block
   Done when: `blueprint-compiler compile --output /dev/null data/resources/<name>.blp` exits 0 for every `.blp` file.

3. **Integrate blueprint-compiler in `build.rs`** — insert before the `glib_build_tools::compile_resources(...)` call: (a) collect all `.blp` files via `glob` or `read_dir`; (b) emit `cargo:rerun-if-changed=<path>` for each `.blp` file; (c) invoke `std::process::Command::new("blueprint-compiler").args(["batch-compile", "--output", "data/resources"]).args(&blp_files).status()`; (d) assert exit success with a message that includes "Install blueprint-compiler: pip install blueprint-compiler". Done when: `cargo build` on a machine with blueprint-compiler installed exits 0 and emits `compiled.gresource`.

4. **Add a Makefile target** — add `blueprint-compile:` target that documents the manual invocation: `blueprint-compiler batch-compile --output data/resources data/resources/*.blp`. Update the `build` target comment (not the recipe) to note that blueprint compilation is handled by `build.rs`. Done when: `make blueprint-compile` runs without error when blueprint-compiler is installed.

5. **Verify ID parity** — for each widget ID recorded in Step 1, confirm it appears in the corresponding `.blp` file. Done when: `grep -c "id_name" data/resources/window.blp` returns ≥ 1 for every ID from Step 1's table.

6. **Verify build parity** — run `make build` and confirm exit code 0; run `gresource list $(find target -name compiled.gresource | head -1)` and diff the path list against a pre-migration snapshot. Done when: `make build` exits 0 and the GResource path list is identical to before migration.

## Examples

**Input (GtkBuilder XML — simple widget with accessibility and style):**
```xml
<object class="GtkButton" id="refresh_button">
  <property name="icon-name">view-refresh-symbolic</property>
  <property name="tooltip-text" translatable="yes">Refresh</property>
  <accessibility>
    <property name="label" translatable="yes">Refresh</property>
  </accessibility>
  <style>
    <class name="action-button"/>
  </style>
</object>
```

**Expected output (Blueprint):**
```blueprint
GtkButton refresh_button {
  icon-name: "view-refresh-symbolic";
  tooltip-text: _("Refresh");
  accessibility {
    label: _("Refresh");
  }
  styles ["action-button"]
}
```

---

**Input (AdwBreakpoint with multiple setters — complex case):**
```xml
<object class="AdwBreakpoint">
  <condition>max-width: 768sp</condition>
  <setter object="detail_content" property="margin-start">32</setter>
  <setter object="detail_content" property="margin-end">32</setter>
  <setter object="detail_content" property="margin-top">32</setter>
  <setter object="detail_content" property="margin-bottom">32</setter>
</object>
```

**Expected output (Blueprint):**
```blueprint
Adw.Breakpoint {
  condition ("max-width: 768sp")
  setters {
    detail_content.margin-start: 32;
    detail_content.margin-end: 32;
    detail_content.margin-top: 32;
    detail_content.margin-bottom: 32;
  }
}
```

## Output Format

One `.blp` file per source `.ui` file, placed in the same directory (`data/resources/<name>.blp`). Each Blueprint file must follow this skeleton:

```blueprint
using Gtk 4.0;
using Adw 1;

template $ClassName : ParentClass {
  property-name: value;

  [child-annotation]
  ChildWidget widget_id {
    property: value;
    styles ["css-class"]
    accessibility { label: _("..."); }
  }

  Adw.Breakpoint {
    condition ("max-width: Nsp")
    setters {
      widget_id.property: value;
    }
  }

  ShortcutController {
    scope: local;
    Shortcut {
      trigger: "<Primary>f";
      action: "action(win.action-name)";
    }
  }
}

menu menu_id {
  section {
    item {
      label: _("_Label");
      action: "scope.action-name";
    }
  }
}
```

Modifications to existing files are limited to: new `.blp` files in `data/resources/`, updated `build.rs` (blueprint-compiler call added before `compile_resources`), and a new `blueprint-compile` Makefile target. No other files change.

## Input Acceptance Criteria

Before executing the main task, verify these criteria. If any fail, stop and report the failure.

| # | Criterion | How to verify |
|---|-----------|---------------|
| I1 | `blueprint-compiler` is installed and on PATH | Run `blueprint-compiler --version`; if not found, fail with "Install blueprint-compiler: `pip install blueprint-compiler` or via your distro package manager" |
| I2 | All `.ui` files in `data/resources/` are valid XML | Run `xmllint --noout data/resources/*.ui`; list any files that fail |
| I3 | No `.blp` files already exist in `data/resources/` | Run `find data/resources -name "*.blp"`; if any exist, report them and ask whether to overwrite before proceeding |

## Output Acceptance Criteria

After completing the task, verify these criteria. If any fail, append a `## Validation failures` section to the output listing each failure.

| # | Criterion | How to verify |
|---|-----------|---------------|
| O1 | Every `.ui` source file has a corresponding `.blp` file | `find data/resources -name "*.ui" \| sed 's/\.ui$/.blp/' \| xargs ls` exits 0 |
| O2 | Every `.blp` file compiles without errors | `blueprint-compiler compile --output /dev/null data/resources/*.blp` exits 0 for all files |
| O3 | `make build` exits 0 with the blueprint pipeline active | Run `make build`; assert exit code 0 |
| O4 | GResource path list is unchanged after migration | `gresource list $(find target -name compiled.gresource \| head -1)` diff against pre-migration snapshot is empty |
| O5 | All Rust `#[template_child]` IDs are present in the generated `.blp` | Each ID from Step 1 inventory appears at least once in the corresponding `.blp` file via grep |
| O6 | Format: every `.blp` file starts with `using Gtk 4.0;` and `using Adw 1;` headers | `grep -l "using Gtk 4.0" data/resources/*.blp \| wc -l` equals total `.blp` file count |
