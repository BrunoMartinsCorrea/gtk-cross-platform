// SPDX-License-Identifier: GPL-3.0-or-later
pub mod background;
pub mod containerd_driver; // CLI-based — cross-platform
pub mod dynamic_driver;
pub mod error;
pub mod factory;
pub mod host_stats;
pub mod mock_driver;

// Unix-socket drivers (Linux + macOS). Docker Desktop on Windows uses named pipes
// (//./pipe/docker_engine) — a Windows-specific adapter can be added later.
#[cfg(unix)]
pub mod docker_driver;
#[cfg(unix)]
pub mod http_over_unix;
#[cfg(unix)]
pub mod podman_driver;
