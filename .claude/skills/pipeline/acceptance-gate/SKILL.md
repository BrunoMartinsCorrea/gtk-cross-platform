---
name: pipeline:acceptance-gate
description: Pre-PR gate — verifies definition of done, acceptance criteria, and readiness checklist before a branch is ready for review. Specialized for this GTK4/Rust/Flatpak project.
---

# pipeline:acceptance-gate

Invoke with `/pipeline:acceptance-gate` before opening a pull request.

## When to use

- Before creating a PR with `/project:sync-pull-request`
- When unsure if the feature is complete per the project's standards
- As a structured pre-flight check after `make ci` passes

## Checklist

### Quality gates (automated)

Run and verify all pass:

```sh
make ci        # validate + test — must pass with zero failures
```

If `make ci` fails, stop here. Fix the failures before proceeding.

### Architecture compliance

- [ ] `src/core/` has no imports from `gtk4`, `adw`, `glib`, or `gio` (pure domain logic)
- [ ] `src/ports/` has no imports from `gtk4`, `adw` (traits only; `glib` allowed for error types)
- [ ] All blocking driver calls go through `spawn_driver_task` (never call driver directly from GTK main thread)
- [ ] New `IContainerDriver` methods are implemented in ALL adapters: Docker, Podman, containerd, Mock
- [ ] Async channels use `async_channel::bounded(1)` — no `std::sync::mpsc`, no `tokio`

### UI/UX compliance

- [ ] All new user-visible strings use `gettext("…")` / `pgettext!("ctx", "…")` / `ngettext!("1 item", "{n} items", n)`
- [ ] New files with `gettext()` calls are listed in `po/POTFILES`
- [ ] New interactive widgets have `set_tooltip_text` AND `update_property(&[accessible::Property::Label(…)])`
- [ ] Touch targets on new interactive elements are ≥ 44×44 sp
- [ ] After destructive actions (remove), focus moves to next row or empty-state widget
- [ ] Color alone does not convey state — `StatusBadge` must have text label alongside color

### Documentation

- [ ] CLAUDE.md §Project Structure updated if new files were added
- [ ] CLAUDE.md §Key types updated if new key types were added
- [ ] CLAUDE.md §Slash Commands table updated if commands were added/renamed
- [ ] `make validate-metainfo` passes (AppStream metainfo is valid)

### Cross-platform

- [ ] Tested on Linux (native or Flatpak sandbox)
- [ ] No Linux-only syscalls or hardcoded Linux paths outside `#[cfg(target_os = "linux")]`
- [ ] Flatpak manifest still builds: `make dist-flatpak` (or CI will catch it)

### Definition of done

Feature is **done** when:
1. `make ci` passes
2. All checklist items above are checked
3. PR description explains the _why_, not just the _what_
4. Screenshots attached if UI changed

## Output

Report which items passed, which failed, and a one-sentence recommendation: ready to open PR / blocked by [issue].
