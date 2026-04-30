---
name: Rancher Desktop Runtime Detection Mismatch
description: Diagnose why the GTK4 container management app reports RuntimeNotAvailable while containers are visible in Rancher Desktop — covers socket paths, nerdctl binary location, namespace, and ContainerDriverFactory logic
domain: container runtime detection
audience: Claude Code agent
language: en
version: 1.0
created: 2026-04-29
---

# Rancher Desktop Runtime Detection Mismatch

> **Context:** The app returns `RuntimeNotAvailable` at startup even though Rancher Desktop is running and containers are visible in its UI.
> **Audience:** Claude Code agent working in the `gtk-cross-platform` Rust/GTK4 repository.
> **Usage:** `/project:create-prompt Diagnose Rancher Desktop RuntimeNotAvailable mismatch`

## Role (Persona)

You are a senior Rust/GTK4 systems engineer who has debugged macOS container runtime integration issues across Docker Desktop, Rancher Desktop, Podman, and containerd. You understand both the GTK application's hexagonal architecture and the specifics of how Rancher Desktop exposes its runtime on macOS (non-standard socket paths, `~/.rd/bin` binaries, and the `k8s.io` vs `default` nerdctl namespace split).

## Context

The application detects runtimes in `src/infrastructure/containers/factory.rs` via `ContainerDriverFactory::detect()`. The detection order is:

1. Check `/var/run/docker.sock` (Docker)
2. Check Podman sockets (`/run/user/{uid}/podman/podman.sock`, `/run/podman/podman.sock`)
3. Run `nerdctl version` and expect exit 0 (containerd via nerdctl)
4. Return `ContainerError::RuntimeNotAvailable` if none match

**Known Rancher Desktop quirks on macOS:**

| Aspect | Standard path checked | Rancher Desktop actual path |
|--------|----------------------|------------------------------|
| Docker-compatible socket | `/var/run/docker.sock` | `~/.rd/docker.sock` |
| nerdctl binary | `nerdctl` (PATH) | `~/.rd/bin/nerdctl` (often not in GUI app PATH) |
| containerd socket | — | `~/.lima/rancher-desktop/sock/containerd.sock` |
| Container namespace | `default` | `k8s.io` (for Kubernetes workloads) |

GUI apps launched from the macOS dock or Spotlight do **not** inherit the user's interactive shell `PATH`, so binaries added to `PATH` in `~/.zshrc` or `~/.bashrc` are invisible to the app even though they work in the terminal.

The `ContainerdDriver` is initialised with `namespace: "default"` — containers managed by Rancher Desktop Kubernetes run in the `k8s.io` namespace and will not appear under `default`.

Files to read before analysing:
- `src/infrastructure/containers/factory.rs` — current detection logic
- `src/infrastructure/containers/containerd_driver.rs` — `detect()` and `new()` constructors, `namespace` field

## Objective

Identify every path in `ContainerDriverFactory::detect()` that silently fails when Rancher Desktop is the active runtime, then propose concrete, minimal code changes that fix each failure.

## Constraints

- Scope to `src/infrastructure/containers/factory.rs` and `src/infrastructure/containers/containerd_driver.rs`; do not touch GTK layer or port traits.
- Do not introduce new external crates; use only `std::path::Path`, `std::env`, and `std::process::Command` already in scope.
- Do not change the detection order documented in CLAUDE.md (Docker → Podman → containerd → error).
- Extend the detection logic only by adding fallback paths and a namespace probe; do not remove existing checks.
- Output each finding as a numbered diagnosis entry followed immediately by its proposed fix; do not write a general narrative.

## Steps

1. **Read the factory** — Open `src/infrastructure/containers/factory.rs` and trace the exact conditions under which each runtime branch returns `None` or `Err`. Done when: every branch that can silently fail on macOS is identified and listed.

2. **Read the containerd driver** — Open `src/infrastructure/containers/containerd_driver.rs` and note the `detect()` method's exact `Command::new("nerdctl")` call and the hardcoded `"default"` namespace. Done when: both the binary lookup strategy and the namespace value are recorded.

3. **Map each failure mode to Rancher Desktop** — For each failing branch identified in steps 1–2, state the specific Rancher Desktop artefact that is absent from the current check (wrong socket path, wrong binary path, wrong namespace). Done when: a 1-to-1 table of `[Failure mode] → [Rancher Desktop artefact]` is produced.

4. **Propose concrete code changes** — For each failure mode, write the minimal Rust code change (a new fallback path, a candidate binary list, a namespace probe) that would make detection succeed. Present each change as a `// BEFORE` / `// AFTER` diff block tied to a specific file and function. Done when: every failure mode from step 3 has a matching code fix.

5. **Validate the fix set** — Verify that the proposed changes (a) do not break the detection order from CLAUDE.md, (b) compile without new dependencies, and (c) fall through correctly to `RuntimeNotAvailable` when Rancher Desktop is not installed. Done when: each proposed change is confirmed safe against the three criteria.

## Examples

**Input scenario:**
```
User sees containers in Rancher Desktop UI.
App logs: ContainerError::RuntimeNotAvailable("No container runtime found. Install Docker, Podman, or nerdctl.")
$ which nerdctl → /Users/me/.rd/bin/nerdctl
$ ls /var/run/docker.sock → No such file or directory
$ ls ~/.rd/docker.sock → ~/.rd/docker.sock (exists)
```

**Expected output:**
```
## Diagnosis

### Failure 1 — Docker socket path mismatch
- Current check: Path::new("/var/run/docker.sock").exists()
- Rancher Desktop actual path: ~/.rd/docker.sock (resolved: /Users/me/.rd/docker.sock)
- Fix: add a second candidate in DockerDriver detection:

// BEFORE (factory.rs, line ~27)
if Path::new("/var/run/docker.sock").exists() { ... }

// AFTER
for candidate in ["/var/run/docker.sock", "~/.rd/docker.sock"] {
    let path = expand_tilde(candidate);
    if Path::new(&path).exists() { ... break }
}

### Failure 2 — nerdctl not in GUI app PATH
- Current check: Command::new("nerdctl").arg("version").output().ok()?
- Rancher Desktop binary: ~/.rd/bin/nerdctl (absent from GUI PATH)
- Fix: probe a static candidate list before falling back to PATH:

// BEFORE (containerd_driver.rs, ContainerdDriver::detect)
Command::new("nerdctl").arg("version").output().ok()?

// AFTER
const CANDIDATES: &[&str] = &["nerdctl", "~/.rd/bin/nerdctl", "/usr/local/bin/nerdctl"];
let bin = CANDIDATES.iter()
    .map(|c| expand_tilde(c))
    .find(|p| Command::new(p).arg("version").output().map(|o| o.status.success()).unwrap_or(false))?;
Some(Self::new(bin, "default"))
```

---

**Input scenario (namespace mismatch only):**
```
nerdctl IS in PATH and detection succeeds.
App shows 0 containers; Rancher Desktop shows 5.
$ nerdctl --namespace k8s.io ps → 5 containers listed
$ nerdctl --namespace default ps → 0 containers
```

**Expected output:**
```
### Failure 3 — containerd namespace mismatch
- Current namespace: "default" (hardcoded in ContainerdDriver::new)
- Rancher Desktop Kubernetes namespace: "k8s.io"
- Fix: probe both namespaces and pick the one with containers, or expose it as a runtime switcher option.
```

## Output Format

Produce a Markdown document with the following structure (max 600 words):

```
## Diagnosis

### Failure N — <short title>
- **Current check:** <exact code reference>
- **Rancher Desktop actual value:** <path / binary / namespace>
- **Fix:**

// BEFORE (<file>:<function>)
<minimal before snippet>

// AFTER
<minimal after snippet>

---
```

Repeat one `### Failure N` section per distinct failure mode found. End with:

```
## Validation summary

| Fix | Preserves detection order | Compiles without new deps | Falls through to RuntimeNotAvailable |
|-----|--------------------------|--------------------------|--------------------------------------|
| Fix 1 | ✓ / ✗ | ✓ / ✗ | ✓ / ✗ |
```

## Input Acceptance Criteria

Verify these before starting. If any fail, stop and report the failure.

| # | Criterion | How to verify |
|---|-----------|---------------|
| I1 | `src/infrastructure/containers/factory.rs` is readable | `Read` the file; confirm it contains `ContainerDriverFactory` |
| I2 | `src/infrastructure/containers/containerd_driver.rs` is readable | `Read` the file; confirm it contains `ContainerdDriver::detect` |

## Output Acceptance Criteria

After completing the task, verify these criteria. If any fail, append a `## Validation failures` section.

| # | Criterion | How to verify |
|---|-----------|---------------|
| O1 | At least 2 `### Failure N` sections present | Count headings matching `### Failure` |
| O2 | Every failure section contains a `// BEFORE` / `// AFTER` diff block | Grep for `// BEFORE` and `// AFTER`; count must match the failure section count |
| O3 | Validation summary table is present and has the correct columns | Grep for `Preserves detection order`; confirm table has 4 columns |
| O4 | No new external crate names appear in the proposed code changes | Confirm no `extern crate` or `use <new_crate>` outside of `std`, `serde_json`, or crates already in `Cargo.toml` |
| O5 | Each fix references a specific file and function | Every `// BEFORE` block includes a `(<file>:<function>)` annotation |
