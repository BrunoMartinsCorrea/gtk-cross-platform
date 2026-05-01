---
description: Testing standards — nextest configuration, MockContainerDriver usage, test layer rules, and naming conventions. Auto-loaded when editing test files.
globs: ["tests/**", "src/**/tests/**"]
---

# Testing Standards

## Test runner

This project uses `cargo nextest` (not `cargo test`). Run tests with:

```sh
make test             # all tests (unit + integration + i18n)
make test-unit        # unit tests only: cargo nextest --lib
make test-integration # integration tests: container_driver_test + greet_use_case_test
make test-i18n        # i18n structural tests: i18n_test
```

For CI (fail-fast mode): `make test-unit NEXTEST_PROFILE=ci`

## Test layer rules

| Layer       | Location                             | Rule                                                   |
|-------------|--------------------------------------|--------------------------------------------------------|
| Unit        | `#[cfg(test)]` inline in `src/core/` | No gtk4/adw/glib imports; access private internals     |
| Integration | `tests/*.rs`                         | Public API only; use MockContainerDriver               |
| Widget      | `tests/widget_test.rs`               | Marked `#[ignore]`; require display; not in default CI |

**Rule: no `tests/unit/` directory** — domain unit tests live inline (Rust convention for private access).

## MockContainerDriver is mandatory

```rust
// CORRECT — use MockContainerDriver in all integration tests
let driver = Arc::new(MockContainerDriver::new());
let use_case = ContainerUseCase::new(Arc::clone(&driver));

// FORBIDDEN — never use real Docker/Podman sockets in tests
let driver = DockerDriver::new();  // breaks CI (no Docker socket in GitHub Actions)
```

## Test naming convention

```rust
// Pattern: test_{function}_{condition}_{expected_outcome}
fn test_list_containers_when_empty_returns_ok_with_empty_vec()
fn test_start_container_when_not_found_returns_not_found_error()
fn test_remove_container_when_permission_denied_returns_permission_error()
```

## Test structure (AAA)

```rust
#[test]
fn test_container_use_case_lists_running_containers() {
    // Arrange — set up state
    let driver = Arc::new(MockContainerDriver::with_containers(vec![
        Container { status: ContainerStatus::Running, ..Default::default() },
    ]));
    let use_case = ContainerUseCase::new(driver);

    // Act — call the function under test
    let result = use_case.list_containers();

    // Assert — verify the outcome
    assert!(result.is_ok());
    let containers = result.unwrap();
    assert_eq!(containers.len(), 1);
    assert_eq!(containers[0].status, ContainerStatus::Running);
}
```

## Error path testing

Every `Result`-returning function must have at least one error test:

```rust
#[test]
fn test_start_container_propagates_runtime_not_available() {
    let driver = Arc::new(MockContainerDriver::with_error(
        ContainerError::RuntimeNotAvailable("no runtime".into())
    ));
    let use_case = ContainerUseCase::new(driver);
    let result = use_case.start_container("abc123");
    assert!(matches!(result, Err(ContainerError::RuntimeNotAvailable(_))));
}
```

## Widget test requirements

Widget tests in `tests/widget_test.rs` require a display and are excluded from default CI:

```sh
# Run widget tests manually:
xvfb-run cargo test --test widget_test -- --test-threads=1 --ignored
```

Widget tests must:

- Initialize GTK with `gtk4::init()`
- Not call GObject methods from multiple threads
- Not assume a specific screen size
