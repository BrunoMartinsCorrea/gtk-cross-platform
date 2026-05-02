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
    Paused,
    Stopped,
    Exited(i32),      // container exited with a specific exit code
    Restarting,
    Dead,
    Unknown(String),  // runtime-specific unknown status
}
```

All seven statuses must be handled in every match expression — no `_ =>` without logging.

### Container

Key fields:

- `id: String` — runtime-assigned container ID (Docker: 64-char hex prefix; Podman: similar)
- `name: String` — human-readable name (without leading `/` for Docker)
- `image: String` — image name including tag
- `status: ContainerStatus` — current lifecycle state
- `created: i64` — Unix timestamp (seconds since epoch)

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

**Rule:** when adding a new method to `IContainerDriver`, ALL five implementations must be updated before integration
tests can compile.

### ContainerError variants

```rust
pub enum ContainerError {
    ConnectionFailed(String),                         // socket open/connect failed
    NotFound(String),                                 // resource not found
    AlreadyExists(String),                            // resource with that name already exists
    NotRunning(String),                               // operation requires a running container
    PermissionDenied,                                 // socket not accessible
    RuntimeNotAvailable(String),                      // no runtime detected
    ApiError { status: u16, message: String },        // non-2xx HTTP status from runtime
    ParseError(String),                               // JSON/response parsing failure
    Io(std::io::Error),                               // underlying I/O failure
    SubprocessFailed { code: Option<i32>, stderr: String }, // nerdctl/CLI non-zero exit
}
```

See `rules/standards/observability.md` for log level mapping per variant.

### Use case ports (`src/ports/use_cases/`)

- `IContainerUseCase` — consumed by `ContainersView`
- `IImageUseCase` — consumed by `ImagesView`
- `IVolumeUseCase` — consumed by `VolumesView`
- `INetworkUseCase` — consumed by `NetworksView`

Views depend on the use case port, not on `IContainerDriver` directly.

## Hexagonal layer rules

| Layer            | Path                  | Imports allowed                                |
|------------------|-----------------------|------------------------------------------------|
| Domain           | `src/core/`           | std only (no GTK, no GLib, no IO)              |
| Ports            | `src/ports/`          | std + glib (error types only); no gtk4, no adw |
| Adapters         | `src/infrastructure/` | std + glib + gio (no gtk4, no adw)             |
| UI               | `src/window/`         | all — gtk4, adw, glib, domain, ports           |
| Composition root | `src/app.rs`          | all — wires concrete types to ports            |

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
                log_container_error(&view.logger, &e);
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
