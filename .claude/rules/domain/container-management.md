---
description: Container management domain context — models, port contracts, hexagonal layer rules, and spawn_driver_task pattern. Auto-loaded when editing files in src/core/domain/, src/infrastructure/containers/, or src/ports/.
globs: ["src/core/domain/**", "src/infrastructure/containers/**", "src/ports/**"]
---

# Container Management Domain Context

## Domain models (`src/core/domain/`)

### ContainerStatus variants

```rust
pub enum ContainerStatus {
    Running,
    Stopped,
    Paused,
    Unknown(String),  // runtime-specific unknown status
}
```

All four statuses must be handled in every match expression — no `_ =>` without logging.

### Container

Key fields:
- `id: String` — runtime-assigned container ID (Docker: 64-char hex prefix; Podman: similar)
- `name: String` — human-readable name (without leading `/` for Docker)
- `image: String` — image name including tag
- `status: ContainerStatus` — current lifecycle state
- `created: String` — ISO 8601 timestamp

### Image, Volume, Network

Each domain model follows the same conventions: `id`, `name`, and no GTK imports.

## Port contracts (`src/ports/`)

### IContainerDriver

Every method returns `Result<T, ContainerError>`. The trait is implemented by:
- `DockerDriver` — HTTP over `/var/run/docker.sock`
- `PodmanDriver` — HTTP over `/run/user/{uid}/podman/podman.sock` (rootless) or `/run/podman/podman.sock` (root)
- `ContainerdDriver` — via `nerdctl` CLI
- `MockContainerDriver` — in-memory, used in ALL tests
- `DynamicDriver` — wraps `Arc<dyn IContainerDriver>` for runtime switching

**Rule:** when adding a new method to `IContainerDriver`, ALL five implementations must be updated before integration tests can compile.

### ContainerError variants

```rust
pub enum ContainerError {
    NotFound,              // container/image/volume/network not found
    PermissionDenied,      // socket not accessible
    RuntimeNotAvailable,   // no runtime detected
    ParseError(String),    // JSON/response parsing failure
    Unknown(String),       // anything else
}
```

Log levels: `PermissionDenied` + `ParseError` → `AppLogger::critical`, `NotFound` → `AppLogger::info`, all others → `AppLogger::warning`. Always normalize via `log_container_error()`.

### Use case ports (`src/ports/use_cases/`)

- `IContainerUseCase` — consumed by `ContainersView`
- `IImageUseCase` — consumed by `ImagesView`
- `IVolumeUseCase` — consumed by `VolumesView`
- `INetworkUseCase` — consumed by `NetworksView`

Views depend on the use case port, not on `IContainerDriver` directly.

## Hexagonal layer rules

| Layer | Path | Imports allowed |
|-------|------|-----------------|
| Domain | `src/core/` | std only (no GTK, no GLib, no IO) |
| Ports | `src/ports/` | std + glib (error types only); no gtk4, no adw |
| Adapters | `src/infrastructure/` | std + glib + gio (no gtk4, no adw) |
| UI | `src/window/` | all — gtk4, adw, glib, domain, ports |
| Composition root | `src/app.rs` | all — wires concrete types to ports |

## spawn_driver_task pattern

All blocking driver calls MUST use this pattern:

```rust
// In a view (src/window/views/):
spawn_driver_task(
    Arc::clone(&self.driver),
    |driver| async move { driver.list_containers().await },
    clone!(@weak self as view => move |result| {
        // This closure runs on the GTK main loop via glib::spawn_local
        match result {
            Ok(containers) => view.update_list(containers),
            Err(e) => {
                log_container_error(&e, &view.logger);
                view.show_error_toast(&e);
            }
        }
        view.end_loading();
    }),
);
```

**Never** call `driver.list_containers()` directly from a GTK signal handler.
**Never** use `tokio::spawn` or `tokio::runtime` — GLib event loop handles async.
**Channel:** `async_channel::bounded(1)` — never `std::sync::mpsc`.
