---
name: plan:dependency-mapping
version: 1.0.0
description: Maps dependencies between tasks, modules, and ports to identify the safe implementation order for this hexagonal GTK4/Rust project.
---

# plan:dependency-mapping

Invoke with `/plan:dependency-mapping` after `/plan:work-planning` to verify sequencing.

## When to use

- After breaking a feature into tasks to order them safely
- When a change affects a port trait (propagates to all adapters + tests)
- Before starting parallel work to identify what can be parallelized

## Dependency analysis patterns

### Port trait change cascade

When a new method is added to `IContainerDriver` or any port in `src/ports/`:

```
IContainerDriver (src/ports/i_container_driver.rs)
    ├── DockerDriver (src/infrastructure/containers/docker_driver.rs) — must implement
    ├── PodmanDriver (src/infrastructure/containers/podman_driver.rs) — must implement
    ├── ContainerdDriver (src/infrastructure/containers/containerd_driver.rs) — must implement
    ├── MockContainerDriver (src/infrastructure/containers/mock_driver.rs) — must implement
    └── DynamicDriver (src/infrastructure/containers/dynamic_driver.rs) — delegates, may need update
```

All 5 must be updated before integration tests can compile.

### Use case change cascade

When a use case in `src/core/use_cases/` changes:

```
IContainerUseCase (src/ports/use_cases/)
    ├── ContainerUseCase (src/core/use_cases/) — implementation
    └── ContainersView (src/window/views/) — consumer
```

### GResource change cascade

When a new `.ui` file or CSS is added:

```
data/resources/<file>.ui
    ├── resources.gresource.xml — must list the file
    └── build.rs — recompiles GResource (automatic)
```

### i18n change cascade

When a new `gettext("…")` call is added in a `.rs` or `.ui` file:

```
src/**/*.rs or data/resources/*.ui
    └── po/POTFILES — must list the file (or make check-potfiles fails)
```

### GSettings change cascade

When a new key is added to `data/com.example.GtkCrossPlatform.gschema.xml`:

```
gschema.xml
    ├── make schema — recompiles (run after change)
    └── src/**/*.rs — consumers must bind to the new key
```

## Output format

```
Task dependency graph:
  Task 1: port trait update (IContainerDriver) — no dependencies
    → Task 2: MockContainerDriver (parallel with adapters)
    → Task 3: DockerDriver / PodmanDriver / ContainerdDriver (parallel)
  Task 4: use case update — depends on Task 1
  Task 5: view wiring — depends on Task 4
  Task 6: integration tests — depends on Task 2 + Task 4
```

Identify tasks that can run in parallel vs. must be sequential.
Flag any circular dependencies or missing mock implementations.
