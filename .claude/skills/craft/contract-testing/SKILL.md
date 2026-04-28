---
name: craft:contract-testing
description: Validates the IContainerDriver contract between all adapters (Docker, Podman, containerd, Mock) and use cases. Generates contract tests that verify behavioral equivalence across adapters.
---

# craft:contract-testing

Invoke with `/craft:contract-testing` when adding a new `IContainerDriver` method or verifying
that all adapters implement the contract consistently.

## When to use

- When a new method is added to `IContainerDriver`
- When a driver adapter behavior needs to be verified against the port contract
- When `MockContainerDriver` might diverge from production adapters
- When writing integration tests that must pass for all adapters

## Contract definition

The contract of `IContainerDriver` is defined in `src/ports/i_container_driver.rs`. Every method
on the trait defines a behavioral contract:

- **Input**: what parameters are accepted and their valid ranges
- **Output**: what return type is guaranteed (`Result<T, ContainerError>`)
- **Error conditions**: which `ContainerError` variants are returned for which conditions
- **Side effects**: what the runtime state changes after the call

## Process

### Step 1 — Identify the contract under test

Read `src/ports/i_container_driver.rs` to identify:
- Which method is being tested?
- What does success look like? (return value, side effect)
- What does failure look like? (which `ContainerError` variant, when?)

### Step 2 — Write contract tests using MockContainerDriver

All contract tests use `MockContainerDriver` (in-memory, no socket required). Location: `tests/container_driver_test.rs`.

Pattern for a contract test:

```rust
#[tokio::test]  // Note: use #[test] for sync; integration tests use nextest
async fn test_list_containers_returns_running_containers() {
    let driver = MockContainerDriver::new();
    // Arrange: seed the mock with known state
    // Act: call the method under test
    let result = driver.list_containers().await;
    // Assert: verify the contract
    assert!(result.is_ok());
    let containers = result.unwrap();
    assert!(!containers.is_empty());
}
```

### Step 3 — Test error conditions

Every `ContainerError` variant that a method can return must have a test:

```rust
#[test]
fn test_list_containers_returns_not_found_when_runtime_unavailable() {
    // Arrange: driver in a state where runtime is unavailable
    // Act + Assert
    assert!(matches!(result, Err(ContainerError::RuntimeNotAvailable)));
}
```

### Step 4 — Verify MockContainerDriver parity

After adding new contract tests, verify that `MockContainerDriver` correctly simulates the behavior:
1. Read `src/infrastructure/containers/mock_driver.rs`
2. Does the mock return the correct error types?
3. Does the mock respect the same invariants as production adapters?

### Step 5 — Document divergences

If a production adapter has documented divergent behavior from the contract (e.g., Podman returns
different error details than Docker for the same condition), document it as a comment in the adapter:

```rust
// Podman returns ContainerError::ParseError when the container JSON is truncated,
// while Docker returns ContainerError::NotFound. Both are acceptable per the contract.
```

## Coverage targets

- Every public method on `IContainerDriver` must have at least one success test and one error test
- `MockContainerDriver` must pass all contract tests
- New methods: tests must be added before the adapter implementations

## Output

A list of contract tests added to `tests/container_driver_test.rs`, with:
- Test names following `test_{method}_{condition}` convention
- `make test-integration` passes
- No new `#[allow(dead_code)]` annotations introduced
