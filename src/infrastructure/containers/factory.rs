// SPDX-License-Identifier: GPL-3.0-or-later
use std::sync::Arc;

use crate::infrastructure::containers::{
    containerd_driver::ContainerdDriver, error::ContainerError,
};
use crate::ports::i_container_driver::IContainerDriver;

#[cfg(unix)]
use crate::infrastructure::containers::{docker_driver::DockerDriver, podman_driver::PodmanDriver};
#[cfg(unix)]
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeKind {
    Docker,
    Podman,
    Containerd,
}

pub struct ContainerDriverFactory;

#[cfg(unix)]
fn find_docker_socket() -> Option<String> {
    const CANDIDATES: &[&str] = &[
        "/var/run/docker.sock",
        "~/.rd/docker.sock", // Rancher Desktop on macOS
    ];
    for raw in CANDIDATES {
        let path = if let Some(rest) = raw.strip_prefix("~/") {
            let home = std::env::var("HOME").ok()?;
            format!("{home}/{rest}")
        } else {
            raw.to_string()
        };
        if Path::new(&path).exists() {
            return Some(path);
        }
    }
    None
}

impl ContainerDriverFactory {
    pub fn detect() -> Result<Arc<dyn IContainerDriver>, ContainerError> {
        #[cfg(unix)]
        {
            if let Some(sock) = find_docker_socket() {
                let driver = DockerDriver::new(sock);
                if driver.ping().is_ok() {
                    return Ok(Arc::new(driver));
                }
            }

            if let Some(podman) = PodmanDriver::detect()
                && podman.ping().is_ok()
            {
                return Ok(Arc::new(podman));
            }
        }

        if let Some(ctr) = ContainerdDriver::detect() {
            return Ok(Arc::new(ctr));
        }

        Err(ContainerError::RuntimeNotAvailable(
            "No container runtime found. Install Docker, Podman, or nerdctl.".into(),
        ))
    }

    pub fn with_runtime(kind: RuntimeKind) -> Result<Arc<dyn IContainerDriver>, ContainerError> {
        match kind {
            #[cfg(unix)]
            RuntimeKind::Docker => {
                let d = DockerDriver::default_socket();
                d.ping()?;
                Ok(Arc::new(d))
            }
            #[cfg(not(unix))]
            RuntimeKind::Docker => Err(ContainerError::RuntimeNotAvailable(
                "Docker Unix socket not available on this platform".into(),
            )),

            #[cfg(unix)]
            RuntimeKind::Podman => {
                let p = PodmanDriver::detect().ok_or_else(|| {
                    ContainerError::RuntimeNotAvailable("Podman socket not found".into())
                })?;
                p.ping()?;
                Ok(Arc::new(p))
            }
            #[cfg(not(unix))]
            RuntimeKind::Podman => Err(ContainerError::RuntimeNotAvailable(
                "Podman Unix socket not available on this platform".into(),
            )),

            RuntimeKind::Containerd => {
                let c = ContainerdDriver::detect().ok_or_else(|| {
                    ContainerError::RuntimeNotAvailable("nerdctl not found in PATH".into())
                })?;
                c.ping()?;
                Ok(Arc::new(c))
            }
        }
    }

    /// Return a driver for a specific runtime by name ("docker", "podman", "containerd").
    /// Returns `ContainerError::RuntimeNotAvailable` for unrecognised names or unavailable runtimes.
    pub fn detect_specific(name: &str) -> Result<Arc<dyn IContainerDriver>, ContainerError> {
        match name.to_ascii_lowercase().as_str() {
            "docker" => Self::with_runtime(RuntimeKind::Docker),
            "podman" => Self::with_runtime(RuntimeKind::Podman),
            "containerd" | "nerdctl" => Self::with_runtime(RuntimeKind::Containerd),
            other => Err(ContainerError::RuntimeNotAvailable(format!(
                "unknown runtime: {other}"
            ))),
        }
    }

    #[cfg(unix)]
    pub fn from_socket(socket_path: impl Into<String>) -> Arc<dyn IContainerDriver> {
        Arc::new(DockerDriver::new(socket_path))
    }

    pub fn available_runtimes() -> Vec<(RuntimeKind, String)> {
        let mut available = Vec::new();

        #[cfg(unix)]
        {
            if let Some(sock) = find_docker_socket() {
                let d = DockerDriver::new(sock);
                if let Ok(v) = d.version() {
                    available.push((RuntimeKind::Docker, v));
                }
            }

            if let Some(podman) = PodmanDriver::detect()
                && let Ok(v) = podman.version()
            {
                available.push((RuntimeKind::Podman, v));
            }
        }

        if let Some(ctr) = ContainerdDriver::detect()
            && let Ok(v) = ctr.version()
        {
            available.push((RuntimeKind::Containerd, v));
        }

        available
    }
}
