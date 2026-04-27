// SPDX-License-Identifier: GPL-3.0-or-later
use std::fmt;

use crate::infrastructure::logging::app_logger::AppLogger;

#[derive(Debug)]
pub enum ContainerError {
    /// Could not open / connect to the runtime socket or process.
    ConnectionFailed(String),
    /// The requested resource (container, image, etc.) was not found.
    NotFound(String),
    /// A resource with that name already exists.
    AlreadyExists(String),
    /// The container is not in a running state and the operation requires it.
    NotRunning(String),
    /// Insufficient permissions to access the socket.
    PermissionDenied,
    /// No supported runtime was detected on this host.
    RuntimeNotAvailable(String),
    /// The runtime returned a non-2xx HTTP status.
    ApiError { status: u16, message: String },
    /// JSON deserialization failed.
    ParseError(String),
    /// Underlying I/O failure.
    Io(std::io::Error),
    /// A subprocess (nerdctl, etc.) exited with a non-zero code.
    SubprocessFailed { code: Option<i32>, stderr: String },
}

impl fmt::Display for ContainerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectionFailed(s) => write!(f, "Connection failed: {s}"),
            Self::NotFound(s) => write!(f, "Not found: {s}"),
            Self::AlreadyExists(s) => write!(f, "Already exists: {s}"),
            Self::NotRunning(s) => write!(f, "Container not running: {s}"),
            Self::PermissionDenied => write!(f, "Permission denied — check socket access"),
            Self::RuntimeNotAvailable(s) => write!(f, "No container runtime available: {s}"),
            Self::ApiError { status, message } => {
                write!(f, "API error {status}: {message}")
            }
            Self::ParseError(s) => write!(f, "Parse error: {s}"),
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::SubprocessFailed { code, stderr } => {
                write!(f, "Subprocess failed (exit {code:?}): {stderr}")
            }
        }
    }
}

impl std::error::Error for ContainerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ContainerError {
    fn from(e: std::io::Error) -> Self {
        use std::io::ErrorKind;
        match e.kind() {
            ErrorKind::PermissionDenied => Self::PermissionDenied,
            ErrorKind::NotFound | ErrorKind::ConnectionRefused => {
                Self::ConnectionFailed(e.to_string())
            }
            _ => Self::Io(e),
        }
    }
}

impl From<serde_json::Error> for ContainerError {
    fn from(e: serde_json::Error) -> Self {
        Self::ParseError(e.to_string())
    }
}

/// Log `err` at the appropriate level for its variant.
///
/// # Why at the call site, not inside ContainerError
///
/// `ContainerError` is a passive data type. Embedding log calls would create hidden
/// side effects, make unit tests noisy, and mix concerns. Call sites (views, app.rs)
/// know the domain and context; this helper just normalises the level per variant so
/// that logic is not duplicated across call sites.
///
/// | Variant             | Level    | Rationale                                      |
/// |---------------------|----------|------------------------------------------------|
/// | PermissionDenied    | critical | Operator action required — likely misconfigured |
/// | ParseError          | critical | Indicates a driver/API contract violation       |
/// | NotFound            | info     | Expected in normal operation (resource gone)    |
/// | all others          | warning  | Transient or recoverable failures               |
pub fn log_container_error(logger: &AppLogger, err: &ContainerError) {
    match err {
        ContainerError::PermissionDenied | ContainerError::ParseError(_) => {
            logger.critical(&format!("{err:?}"));
        }
        ContainerError::NotFound(_) | ContainerError::AlreadyExists(_) => {
            logger.info(&format!("{err:?}"));
        }
        _ => {
            logger.warning(&format!("{err:?}"));
        }
    }
}
