---
description: Align the project's hexagonal architecture to the idiomatic Rust port/adapter/use-case pattern. Introduces driver ports (inbound traits), container use cases, and proper wiring — without touching the GTK layer or the existing driven port (IContainerDriver).
---

Refactor the codebase to implement the full hexagonal architecture as described below. The
levantamento (reference survey) uses an Axum/Kafka/PostgreSQL backend as reference. This command
maps every concept to the GTK4 + GLib event-loop context of this project.

> Architecture rules and key types are in `CLAUDE.md`. Read it before starting.

---

## Concept Mapping: Backend → GTK Desktop

| Backend (Axum/Kafka/Postgres)         | GTK Desktop (this project)                                   |
|---------------------------------------|--------------------------------------------------------------|
| Cargo workspace with separate crates  | Single crate; strict module layering enforced by `CLAUDE.md` |
| Driver port (inbound) — HTTP handler calls use case  | `src/ports/use_cases/` — trait that a View calls via `spawn_driver_task` |
| Driven port (outbound) — use case calls DB/Kafka  | `src/ports/i_container_driver.rs` — already exists           |
| `@Singleton class FooUseCase`         | `struct FooUseCase { driver: Arc<dyn IContainerDriver> }` + `Arc::new(...)` |
| `Arc<dyn Trait>` for shared adapters  | Already used: `Arc<dyn IContainerDriver>`                    |
| `@Controller` Axum handler            | `ContainersView` / `ImagesView` / … in `src/window/views/`  |
| `axum::State<T>` dependency injection  | `OnceCell<Arc<dyn ContainerUseCase>>` field in each view     |
| `tokio::spawn` + async I/O            | `spawn_driver_task` + `async-channel` + `glib::spawn_local`  |
| `IntoResponse for AppError`           | `log_container_error()` + `adw::Toast` in views             |
| `impl From<DbRow> for DomainModel`    | `impl From<ContainerJson> for Container` inside each driver  |
| Manual wiring in `main.rs`            | `activate()` in `src/app.rs` — already the composition root  |

---

## Current Gaps

The project already implements **driven ports** (`IContainerDriver`) and a composition root
(`activate()`). What is missing:

1. **Driver ports (inbound)** — no use-case traits in `src/ports/`. Views call
   `spawn_driver_task` directly with raw driver methods, bypassing the domain layer.

2. **Container use cases** — `src/core/use_cases/` only has `GreetUseCase`. There are no use
   cases for container lifecycle, image management, volume, or network operations.

3. **Use case wiring** — `activate()` wires `Arc<dyn IContainerDriver>` to views. It should
   wire `Arc<dyn IContainerUseCase>` (use case ports) to views, keeping the driver hidden.

4. **From mappings inside drivers** — JSON deserialization in `docker_driver.rs` /
   `podman_driver.rs` should use `impl From<ApiDto> for DomainModel` instead of inline mapping.

---

## Target Structure

```
src/
  ports/
    mod.rs
    i_container_driver.rs          ← driven port (outbound) — keep as-is
    i_greeting_service.rs          ← keep as-is
    use_cases/                     ← NEW — driver ports (inbound)
      mod.rs
      i_container_use_case.rs      ← container lifecycle operations
      i_image_use_case.rs          ← image operations
      i_volume_use_case.rs         ← volume operations
      i_network_use_case.rs        ← network + system operations
  core/
    use_cases/
      greet_use_case.rs            ← keep as-is
      container_use_case.rs        ← NEW — implements IContainerUseCase
      image_use_case.rs            ← NEW — implements IImageUseCase
      volume_use_case.rs           ← NEW — implements IVolumeUseCase
      network_use_case.rs          ← NEW — implements INetworkUseCase
```

---

## Step 1 — Define Driver Ports (Inbound Traits)

Create `src/ports/use_cases/mod.rs` and one trait per resource group. Traits must be
`Send + Sync` because `spawn_driver_task` crosses thread boundaries.

```rust
// src/ports/use_cases/i_container_use_case.rs
use crate::core::domain::container::{Container, ContainerStats, CreateContainerOptions};
use crate::infrastructure::containers::error::ContainerError;
use std::collections::HashMap;

pub trait IContainerUseCase: Send + Sync {
    fn list(&self, all: bool) -> Result<Vec<Container>, ContainerError>;
    fn inspect(&self, id: &str) -> Result<Container, ContainerError>;
    fn start(&self, id: &str) -> Result<(), ContainerError>;
    fn stop(&self, id: &str, timeout_secs: Option<u32>) -> Result<(), ContainerError>;
    fn restart(&self, id: &str, timeout_secs: Option<u32>) -> Result<(), ContainerError>;
    fn pause(&self, id: &str) -> Result<(), ContainerError>;
    fn unpause(&self, id: &str) -> Result<(), ContainerError>;
    fn remove(&self, id: &str, force: bool, remove_volumes: bool) -> Result<(), ContainerError>;
    fn create(&self, opts: &CreateContainerOptions) -> Result<String, ContainerError>;
    fn rename(&self, id: &str, new_name: &str) -> Result<(), ContainerError>;
    fn logs(&self, id: &str, tail: Option<u32>, timestamps: bool) -> Result<String, ContainerError>;
    fn stats(&self, id: &str) -> Result<ContainerStats, ContainerError>;
}

// src/ports/use_cases/i_image_use_case.rs
pub trait IImageUseCase: Send + Sync {
    fn list(&self) -> Result<Vec<Image>, ContainerError>;
    fn pull(&self, reference: &str) -> Result<(), ContainerError>;
    fn remove(&self, id: &str, force: bool) -> Result<(), ContainerError>;
    fn tag(&self, source: &str, target: &str) -> Result<(), ContainerError>;
    fn inspect(&self, id: &str) -> Result<Image, ContainerError>;
}

// src/ports/use_cases/i_volume_use_case.rs
pub trait IVolumeUseCase: Send + Sync {
    fn list(&self) -> Result<Vec<Volume>, ContainerError>;
    fn create(&self, name: &str, labels: HashMap<String, String>) -> Result<Volume, ContainerError>;
    fn remove(&self, name: &str, force: bool) -> Result<(), ContainerError>;
}

// src/ports/use_cases/i_network_use_case.rs
pub trait INetworkUseCase: Send + Sync {
    fn list(&self) -> Result<Vec<Network>, ContainerError>;
    fn create(&self, name: &str) -> Result<Network, ContainerError>;
    fn remove(&self, id: &str) -> Result<(), ContainerError>;
    fn system_df(&self) -> Result<SystemUsage, ContainerError>;
    fn prune(&self, volumes: bool) -> Result<PruneReport, ContainerError>;
}
```

---

## Step 2 — Implement Use Cases

Each use case struct receives an `Arc<dyn IContainerDriver>` and delegates to it. Business
rules (validation, enrichment, cross-cutting logic) go here — not in views or drivers.

```rust
// src/core/use_cases/container_use_case.rs
use std::sync::Arc;
use crate::ports::i_container_driver::IContainerDriver;
use crate::ports::use_cases::i_container_use_case::IContainerUseCase;

pub struct ContainerUseCase {
    driver: Arc<dyn IContainerDriver>,
}

impl ContainerUseCase {
    pub fn new(driver: Arc<dyn IContainerDriver>) -> Self {
        Self { driver }
    }
}

impl IContainerUseCase for ContainerUseCase {
    fn list(&self, all: bool) -> Result<Vec<Container>, ContainerError> {
        self.driver.list_containers(all)
    }

    fn start(&self, id: &str) -> Result<(), ContainerError> {
        // Future: emit audit log, validate preconditions, etc.
        self.driver.start_container(id)
    }

    // … delegate remaining methods to self.driver
}
```

Apply the same pattern for `ImageUseCase`, `VolumeUseCase`, `NetworkUseCase`.

---

## Step 3 — Update Composition Root (`src/app.rs`)

`activate()` must wire use cases, not raw drivers, to the window. The driver itself becomes
an implementation detail invisible to the UI layer.

```rust
fn activate(&self) {
    match ContainerDriverFactory::detect() {
        Ok(driver) => {
            let container_uc: Arc<dyn IContainerUseCase> =
                Arc::new(ContainerUseCase::new(driver.clone()));
            let image_uc: Arc<dyn IImageUseCase> =
                Arc::new(ImageUseCase::new(driver.clone()));
            let volume_uc: Arc<dyn IVolumeUseCase> =
                Arc::new(VolumeUseCase::new(driver.clone()));
            let network_uc: Arc<dyn INetworkUseCase> =
                Arc::new(NetworkUseCase::new(driver));

            let win = MainWindow::new(&*self.obj(), container_uc, image_uc, volume_uc, network_uc);
            win.present();
        }
        Err(e) => { … }
    }
}
```

---

## Step 4 — Update Views to Call Use Case Ports

Replace `Arc<dyn IContainerDriver>` with `Arc<dyn IContainerUseCase>` (and equivalents) in
each view. Calls to `spawn_driver_task` switch from driver methods to use case methods.

```rust
// src/window/views/containers_view.rs — before
spawn_driver_task(self.driver.clone(), |d| d.list_containers(true), |result| { … });

// after
spawn_driver_task(self.use_case.clone(), |uc| uc.list(true), |result| { … });
```

`spawn_driver_task` accepts `Arc<dyn IContainerUseCase>` without change because the
signature is generic over `Arc<dyn T: Send + Sync>` — verify and adjust the type parameter
if needed.

---

## Step 5 — Add From Mappings Inside Drivers (Optional, Low Priority)

Where drivers currently map API JSON inline, extract:

```rust
// src/infrastructure/containers/docker_driver.rs
struct ContainerDto { /* serde fields */ }

impl From<ContainerDto> for Container {
    fn from(dto: ContainerDto) -> Self {
        Container {
            id: dto.id,
            name: dto.names.first().cloned().unwrap_or_default(),
            status: dto.status.parse().unwrap_or(ContainerStatus::Unknown),
            // …
        }
    }
}
```

This is equivalent to `impl From<DbRow> for DomainModel` from the levantamento. It isolates
the API wire format from the domain model.

---

## Layer Rules (enforced throughout)

| Layer              | May import                              | Must never import          |
|--------------------|-----------------------------------------|----------------------------|
| `src/core/domain/` | `std`, `uuid`, `chrono`                 | `gtk4`, `adw`, `glib`, `gio`, `ports::*`, `infrastructure::*` |
| `src/core/use_cases/` | `core::domain::*`, `ports::*`        | `gtk4`, `adw`, `glib`, `infrastructure::*` |
| `src/ports/`       | `core::domain::*`, `infrastructure::containers::error::ContainerError` | `gtk4`, `adw`, `glib` |
| `src/infrastructure/` | `ports::*`, `core::domain::*`, `glib` | `gtk4`, `adw`           |
| `src/window/`      | `ports::use_cases::*`, `core::domain::*`, `glib`, `gtk4`, `adw` | `infrastructure::*` directly |
| `src/app.rs`       | All layers (composition root only)      | —                          |

---

## Threading Contract (unchanged)

Views call use case methods via `spawn_driver_task` exactly as they called driver methods.
The thread boundary, `async-channel`, and `glib::spawn_local` pattern in
`src/infrastructure/containers/background.rs` remain unchanged. Only the generic type passed
changes from `Arc<dyn IContainerDriver>` to `Arc<dyn IXxxUseCase>`.

```
GTK Main Thread                            Worker Thread
─────────────                              ─────────────
spawn_driver_task(use_case, task, cb) ───▶ std::thread::spawn { task(use_case) → tx.send }
                                      ◀─── async_channel::bounded(1)
glib::spawn_local { rx.recv() → cb }
```

---

## Testing Rules

- Use cases in `src/core/use_cases/` must have inline `#[cfg(test)]` unit tests.
- Integration tests in `tests/container_driver_test.rs` test `MockContainerDriver` via
  `ContainerUseCase` — not by calling `IContainerDriver` directly.
- `MockContainerDriver` implements `IContainerDriver` (driven port). Tests that previously
  passed `Arc<dyn IContainerDriver>` directly to views must be updated to wrap it in a use case.

---

## Implementation Order

Run `make test` after each step.

1. Create `src/ports/use_cases/mod.rs` + four trait files (no implementations yet)
2. Implement `ContainerUseCase` in `src/core/use_cases/container_use_case.rs` with unit tests
3. Implement `ImageUseCase`, `VolumeUseCase`, `NetworkUseCase` with unit tests
4. Update `activate()` in `src/app.rs` to wire use cases
5. Update `MainWindow::new` signature; update `ContainersView`, `ImagesView`, `VolumesView`, `NetworksView`
6. Add `From<ApiDto> for DomainModel` in at least one driver (optional proof-of-concept)
7. Update integration tests to drive `MockContainerDriver` through use cases

## Exit Criteria

- [ ] `make test` passes (all unit + integration tests)
- [ ] `make lint` reports zero warnings (`cargo clippy -- -D warnings`)
- [ ] `make fmt` shows no diff
- [ ] `src/window/views/` has no `use crate::infrastructure::containers` import
- [ ] `src/core/use_cases/` has no `gtk4` / `adw` / `glib` import
- [ ] `src/core/domain/` has no `ports::*` or `infrastructure::*` import
- [ ] Each use case has at least two inline unit tests
- [ ] Integration tests in `tests/` use `MockContainerDriver` through use cases (not raw driver)
- [ ] `src/app.rs` is the only file that imports both `infrastructure::*` and `ports::use_cases::*`
