---
description: Port and adapter contract standards — IContainerDriver, use case ports, adapter implementation rules, and runtime detection order. Auto-loaded when editing ports or container infrastructure.
globs: ["src/ports/**", "src/infrastructure/containers/**"]
---

# Port and Adapter Standards

## Port trait design rules

All port traits in `src/ports/` must:

- Use `async fn` returning `Result<T, ContainerError>`
- Accept `&str` for ID parameters (not `String`)
- Be object-safe (compatible with `Arc<dyn Trait>`)
- Have no default implementations that hide missing adapter methods

## IContainerDriver methods

When implementing `IContainerDriver` in a new adapter, every method must be implemented:

- `list_containers() -> Result<Vec<Container>, ContainerError>`
- `list_images() -> Result<Vec<Image>, ContainerError>`
- `list_volumes() -> Result<Vec<Volume>, ContainerError>`
- `list_networks() -> Result<Vec<Network>, ContainerError>`
- `start_container(id: &str) -> Result<(), ContainerError>`
- `stop_container(id: &str) -> Result<(), ContainerError>`
- `remove_container(id: &str, force: bool) -> Result<(), ContainerError>`
- `inspect_container(id: &str) -> Result<Container, ContainerError>`
- `inspect_container_json(id: &str) -> Result<String, ContainerError>`
- (see `src/ports/i_container_driver.rs` for the complete current list)

## Adapter implementation rules

Each adapter in `src/infrastructure/containers/` must:

1. Parse the runtime's JSON/CLI output into domain models (no GTK imports)
2. Map all runtime error codes to the appropriate `ContainerError` variant
3. Handle socket open/connection failures as `ContainerError::ConnectionFailed(String)`; handle "no runtime detected" as
   `ContainerError::RuntimeNotAvailable(String)`
4. Use `serde_json` for JSON parsing (not manual string parsing)
5. Log the raw error with `AppLogger` before converting to `ContainerError`

## MockContainerDriver contract

`MockContainerDriver` (`src/infrastructure/containers/mock_driver.rs`) is the canonical test double:

- Must implement all methods on `IContainerDriver` (compile error if missing)
- Must return deterministic results (no randomness)
- Must support pre-seeding with test data (e.g., `set_containers(vec![...])`)
- Used in ALL integration tests — no real Docker/Podman sockets in CI
- Must simulate `ContainerError` variants for negative path testing

## Runtime detection order

`ContainerDriverFactory::detect()` in `src/infrastructure/containers/factory.rs`:

| Order | Check                                                          | Runtime                               |
|-------|----------------------------------------------------------------|---------------------------------------|
| 1     | `/var/run/docker.sock` or `~/.rd/docker.sock` accessible       | Docker (or Rancher Desktop on macOS)  |
| 2     | `CONTAINER_HOST` env var socket path accessible                | Podman (explicit override)            |
| 3     | `/run/user/{uid}/podman/podman.sock` accessible                | Podman (rootless, Linux)              |
| 4     | `/run/podman/podman.sock` accessible                           | Podman (root, Linux)                  |
| 5     | `~/.local/share/containers/podman/machine/default/podman.sock` | Podman 5.x (macOS)                    |
| 6     | `~/.local/share/containers/podman/machine/qemu/podman.sock`    | Podman 4.x (macOS)                    |
| 7     | `nerdctl version` exits 0                                      | containerd/nerdctl                    |
| —     | None found                                                     | `ContainerError::RuntimeNotAvailable` |

When adding a new runtime: register it after order 7.

## DynamicDriver

`DynamicDriver` wraps `Arc<dyn IContainerDriver>` and delegates all methods. It supports
runtime switching: when the user selects a different runtime in the UI, `DynamicDriver::swap()`
atomically replaces the inner driver. This is the only adapter that accesses `Arc<RwLock<…>>`.

## Use case ports

`src/ports/use_cases/` contains the inbound ports (driver ports) consumed by views:

- `IContainerUseCase` — used by `ContainersView`
- `IImageUseCase` — used by `ImagesView`
- `IVolumeUseCase` — used by `VolumesView`
- `INetworkUseCase` — used by `NetworksView`

Views must not import `IContainerDriver` directly — they depend on the use case port.
`ContainerUseCase` implements `IContainerUseCase` and depends on `Arc<dyn IContainerDriver>`.
