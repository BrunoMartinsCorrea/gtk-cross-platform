---
description: Rust project standards — threading model, async patterns, prohibited dependencies, logging conventions, and code quality rules. Auto-loaded for all .rs files.
globs: ["**/*.rs"]
---

# Rust Project Standards

## Threading model

- **GTK main thread invariant**: all GTK functions must be called from the GTK main thread only
- **Blocking driver calls**: must go through `spawn_driver_task` (never call driver directly from signal handlers)
- **Async channels**: always use `async_channel::bounded(1)` — never `std::sync::mpsc`, never `tokio::channel`
- **GLib async**: use `glib::spawn_local` for posting work back to the GTK main loop

## Prohibited patterns

```rust
// FORBIDDEN — tokio conflicts with GLib event loop
tokio::spawn(async { ... });
tokio::runtime::Builder::new_multi_thread().build();

// FORBIDDEN — direct driver call from GTK signal handler
let containers = self.driver.list_containers().await;  // if called on main thread

// FORBIDDEN — dead code suppression
#[allow(dead_code)]
fn unused_function() { ... }

// FORBIDDEN — unsafe without documented justification
unsafe { ... }  // only allowed with a comment explaining the safety invariant
```

## Required patterns

```rust
// REQUIRED — async channel for cross-thread results
let (tx, rx) = async_channel::bounded::<Result<Vec<Container>, ContainerError>>(1);

// REQUIRED — posting to GTK main loop
glib::spawn_local(clone!(@weak widget => async move {
    match rx.recv().await { ... }
}));

// REQUIRED — logging via AppLogger, not println! or eprintln!
logger.info("Container list refreshed");
log_container_error(&logger, &error);  // logger is first, error is second
```

## Logging conventions

```rust
// Log level mapping (from CLAUDE.md):
// AppLogger::critical → ContainerError::PermissionDenied, ContainerError::ParseError(_)
// AppLogger::info     → ContainerError::NotFound(_), ContainerError::AlreadyExists(_)
// AppLogger::warning  → all other ContainerError variants
// Signature: log_container_error(&logger, &err)  — logger first, error second

// G_LOG_DOMAIN hierarchy:
// "com.example.GtkCrossPlatform" — app-level
// "…GtkCrossPlatform.containers" — infrastructure
// "…GtkCrossPlatform.background" — spawn_driver_task
// "…GtkCrossPlatform.view.containers" — ContainersView
// etc.
```

## Code quality rules

- No `#[allow(dead_code)]` — remove dead code instead
- No `#[allow(clippy::…)]` without a comment explaining why the lint is wrong for this case
- `impl Display` on domain types used in UI (ContainerStatus, ContainerError)
- `derive(Default, PartialEq, Clone, Debug)` on all domain models in `src/core/domain/`
- Use `&str` for input parameters, `String` for owned return values
- `Result<T, ContainerError>` — never `unwrap()` in production code paths

## i18n

- All user-visible strings: `gettext("…")` — no hardcoded English in user-facing code
- Context disambiguation: `pgettext!("context", "string")`
- Plurals: `ngettext!("1 item", "{n} items", n)`
- New source files with `gettext()` must be listed in `po/POTFILES`

## Module organization

```
src/core/       — domain models and use cases (no external I/O)
src/ports/      — trait definitions (no implementations)
src/infrastructure/ — adapter implementations
src/window/     — GTK widgets and views
src/app.rs      — composition root (wires concrete types)
src/config.rs   — compile-time constants from build.rs
```
