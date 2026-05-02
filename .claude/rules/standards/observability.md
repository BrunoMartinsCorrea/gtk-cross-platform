---
description: AppLogger conventions — G_LOG_DOMAIN hierarchy, log level per ContainerError variant, structured fields, and debug environment setup. Auto-loaded when editing logging infrastructure or any Rust file.
globs: ["src/infrastructure/logging/**", "src/**/*.rs"]
---

# Observability Standards

## AppLogger API

`AppLogger` wraps GLib structured logging. Located in `src/infrastructure/logging/app_logger.rs`.

```rust
// Basic logging
logger.trace("entering function");     // maps to g_debug! (GLib has no TRACE)
logger.debug("processing container");  // maps to g_debug!
logger.info("container started");      // maps to g_info!
logger.warning("container not found"); // maps to g_warning!
logger.critical("permission denied");  // maps to g_critical!

// Subdomain logger
let containers_logger = logger.subdomain("containers");
// → G_LOG_DOMAIN becomes "…GtkCrossPlatform.containers"

// Structured fields
logger.log_with_fields("container stopped", &[
    ("container_id", id),
    ("exit_code", &exit_code.to_string()),
]);
```

## G_LOG_DOMAIN hierarchy

```
com.example.GtkCrossPlatform               ← app/startup
com.example.GtkCrossPlatform.containers    ← infrastructure (reserved)
com.example.GtkCrossPlatform.background    ← spawn_driver_task bridge
com.example.GtkCrossPlatform.view.containers
com.example.GtkCrossPlatform.view.images
com.example.GtkCrossPlatform.view.volumes
com.example.GtkCrossPlatform.view.networks
```

GLib prefix-matches `G_MESSAGES_DEBUG` against the log domain. To enable all messages:

```sh
G_MESSAGES_DEBUG=com.example.GtkCrossPlatform make run
# or
G_MESSAGES_DEBUG=all make run  # enables ALL GLib messages (very verbose)
```

In development profile (`PROFILE=development`), `G_MESSAGES_DEBUG=all` is set automatically.

## ContainerError log level mapping

```rust
// In src/infrastructure/containers/error.rs
// Signature: logger first, error second.
pub fn log_container_error(logger: &AppLogger, err: &ContainerError) {
    match err {
        ContainerError::PermissionDenied | ContainerError::ParseError(_) => {
            logger.critical(&format!("{err:?}"));
        }
        ContainerError::NotFound(_) | ContainerError::AlreadyExists(_) => {
            logger.info(&format!("{err:?}"));
        }
        _ => {
            logger.warning(&format!("{err:?}"));
        }
    }
}
```

| Variant            | Level    | Rationale                                       |
|--------------------|----------|-------------------------------------------------|
| `PermissionDenied` | critical | Operator action required — likely misconfigured |
| `ParseError(_)`    | critical | Driver/API contract violation                   |
| `NotFound(_)`      | info     | Expected in normal operation (resource gone)    |
| `AlreadyExists(_)` | info     | Expected on idempotent create calls             |
| all others         | warning  | Transient or recoverable failures               |

**Rule:** always call `log_container_error(&logger, &err)` at the call site (views, app.rs), not
inside the error constructor. Never call `AppLogger` directly for `ContainerError` variants.

## Debug environment

```sh
# Enable app-level debug logs
G_MESSAGES_DEBUG=com.example.GtkCrossPlatform ./target/debug/gtk-cross-platform

# Enable all GLib messages (includes GTK internals)
G_MESSAGES_DEBUG=all ./target/debug/gtk-cross-platform

# Interactive layout inspection (requires GTK_DEBUG)
GTK_DEBUG=interactive make run-mobile

# Development profile (auto-enables G_MESSAGES_DEBUG=all)
PROFILE=development make run
```

## Rules for new logging calls

1. Use `logger.info()` for normal operational events (container listed, view refreshed)
2. Use `logger.warning()` for recoverable errors (driver timeout, stale data)
3. Use `logger.critical()` for security-relevant or data-loss events only
4. Never use `println!` or `eprintln!` in production code (invisible in GLib log pipeline)
5. Never log sensitive data (environment variables, authentication tokens)
6. Always create a subdomain logger in each module: `let logger = app_logger.subdomain("view.containers")`
