---
description:
    Add a new container runtime adapter implementing IContainerDriver. Usage: /project:add-runtime-driver <runtime-name>
---

Implement a new container runtime adapter for `$ARGUMENTS`.

**Usage:** `/project:add-runtime-driver <runtime-name>`
**Example:** `/project:add-runtime-driver nerdctl`

> Layer rules are in `CLAUDE.md`. The port (`IContainerDriver`), factory, `HttpOverUnix`
> transport, and `MockContainerDriver` are already implemented — do not rebuild them.

## Files to create

```
src/infrastructure/containers/<runtime>_driver.rs   ← new adapter
```

## Files to modify

```
src/infrastructure/containers/mod.rs                ← pub mod <runtime>_driver
src/infrastructure/containers/factory.rs            ← new RuntimeKind variant + detection
tests/container_driver_test.rs                      ← integration tests for the new adapter
```

Modify `src/ports/i_container_driver.rs` only if the new runtime exposes a capability that
the existing trait doesn't cover. Every change to the trait requires updating all adapters.

## Step 1 — Implement the adapter

Create `src/infrastructure/containers/<runtime>_driver.rs` as a struct implementing
`IContainerDriver` from `src/ports/i_container_driver.rs`.

**All trait methods must be implemented.** Return `Err(ContainerError::NotSupported)` for
operations the runtime genuinely does not support — never panic or leave them unimplemented.

Required methods:

| Category   | Methods                                                                                                                                                                                                                                                        |
|------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Containers | `list_containers`, `inspect_container`, `start_container`, `stop_container`, `restart_container`, `pause_container`, `unpause_container`, `remove_container`, `create_container`, `rename_container`, `container_logs`, `container_stats`, `exec_in_container` |
| Images     | `list_images`, `pull_image`, `remove_image`, `tag_image`, `inspect_image`                                                                                                                                                                                      |
| Volumes    | `list_volumes`, `create_volume`, `remove_volume`                                                                                                                                                                                                               |
| Networks   | `list_networks`, `remove_network`                                                                                                                                                                                                                              |
| System     | `ping`, `version`, `system_df`, `prune_system`                                                                                                                                                                                                                 |

**Transport options:**

- HTTP/1.1 over Unix socket → use `HttpOverUnix` from `src/infrastructure/containers/http_over_unix.rs`
- CLI-based runtime → use `std::process::Command` (blocking; runs inside `spawn_driver_task`)

**DTO pattern:** deserialize API responses into private module-internal structs, then map to
domain types from `src/core/domain/`. Never let API response shapes leak into domain types.

**No GTK imports** anywhere in `src/infrastructure/`. All I/O is blocking; the async bridge is
`spawn_driver_task` in `background.rs` — the adapter itself is sync.

## Step 2 — Register in the factory

In `src/infrastructure/containers/factory.rs`:

1. Add a new `RuntimeKind` variant (e.g., `RuntimeKind::Nerdctl`)
2. Add detection logic in `ContainerDriverFactory::detect()` at the correct slot:

| Order | Check                                           | Runtime            |
|-------|-------------------------------------------------|--------------------|
| 1     | `/var/run/docker.sock` accessible               | Docker             |
| 2     | `/run/user/{uid}/podman/podman.sock` accessible | Podman (rootless)  |
| 3     | `/run/podman/podman.sock` accessible            | Podman (root)      |
| 4     | `nerdctl version` exits 0                       | containerd/nerdctl |
| 5+    | New runtime check                               | `$ARGUMENTS`       |

3. Instantiate: return `Arc::new(<Runtime>Driver::new(socket_path_or_config))`

## Step 3 — Write integration tests

Add tests in `tests/container_driver_test.rs`. Use `MockContainerDriver` as the reference
implementation — your adapter's behavior for the same seeded data should match.

Minimum required tests:

- [ ] `test_<runtime>_list_containers` — returns the expected container list
- [ ] `test_<runtime>_start_stop` — state transitions visible in next `list_containers` call
- [ ] `test_<runtime>_list_images`
- [ ] `test_<runtime>_list_volumes`
- [ ] `test_<runtime>_list_networks`
- [ ] `test_<runtime>_ping` — returns `Ok`

## Exit criteria

- [ ] `cargo check` passes without errors
- [ ] `make test` passes (all tests, not just the new ones)
- [ ] `make lint` reports zero warnings
- [ ] New adapter is registered in `factory.rs` with detection in the correct priority slot
- [ ] All `IContainerDriver` methods implemented — `cargo check` will catch missing items
- [ ] No GTK, Adw, or GLib-gtk imports in `src/infrastructure/`
- [ ] DTO types are private (`pub(super)` or `pub(crate)` at most); domain types are public
- [ ] Integration tests added and passing
