// SPDX-License-Identifier: GPL-3.0-or-later
use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum ContainerStatus {
    Running,
    Paused,
    #[default]
    Stopped,
    Exited(i32),
    Restarting,
    Dead,
    Unknown(String),
}

impl ContainerStatus {
    pub fn from_state(state: &str, exit_code: Option<i32>) -> Self {
        match state {
            "running" => Self::Running,
            "paused" => Self::Paused,
            "created" | "stopped" => Self::Stopped,
            "restarting" => Self::Restarting,
            "dead" => Self::Dead,
            "exited" => Self::Exited(exit_code.unwrap_or(0)),
            other => Self::Unknown(other.to_string()),
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Running => "Running",
            Self::Paused => "Paused",
            Self::Stopped => "Stopped",
            Self::Exited(_) => "Exited",
            Self::Restarting => "Restarting",
            Self::Dead => "Dead",
            Self::Unknown(_) => "Unknown",
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Running => "success",
            Self::Paused => "warning",
            Self::Stopped | Self::Exited(_) => "dim-label",
            Self::Dead => "error",
            Self::Restarting => "accent",
            Self::Unknown(_) => "dim-label",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Running => "media-playback-start-symbolic",
            Self::Paused => "media-playback-pause-symbolic",
            Self::Stopped | Self::Exited(_) => "media-playback-stop-symbolic",
            Self::Dead => "dialog-error-symbolic",
            Self::Restarting => "view-refresh-symbolic",
            Self::Unknown(_) => "emblem-default-symbolic",
        }
    }

    pub fn display_label(&self) -> &str {
        self.label()
    }

    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running | Self::Paused)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_from_state_table() {
        assert_eq!(
            ContainerStatus::from_state("running", None),
            ContainerStatus::Running
        );
        assert_eq!(
            ContainerStatus::from_state("paused", None),
            ContainerStatus::Paused
        );
        assert_eq!(
            ContainerStatus::from_state("created", None),
            ContainerStatus::Stopped
        );
        assert_eq!(
            ContainerStatus::from_state("stopped", None),
            ContainerStatus::Stopped
        );
        assert_eq!(
            ContainerStatus::from_state("restarting", None),
            ContainerStatus::Restarting
        );
        assert_eq!(
            ContainerStatus::from_state("dead", None),
            ContainerStatus::Dead
        );
        assert_eq!(
            ContainerStatus::from_state("exited", Some(1)),
            ContainerStatus::Exited(1)
        );
        assert_eq!(
            ContainerStatus::from_state("exited", None),
            ContainerStatus::Exited(0)
        );
        assert!(matches!(
            ContainerStatus::from_state("fancy-new-state", None),
            ContainerStatus::Unknown(_)
        ));
    }

    #[test]
    fn status_is_running() {
        assert!(ContainerStatus::Running.is_running());
        assert!(ContainerStatus::Paused.is_running());
        assert!(!ContainerStatus::Stopped.is_running());
        assert!(!ContainerStatus::Exited(0).is_running());
        assert!(!ContainerStatus::Dead.is_running());
    }

    #[test]
    fn status_labels() {
        assert_eq!(ContainerStatus::Running.label(), "Running");
        assert_eq!(ContainerStatus::Paused.label(), "Paused");
        assert_eq!(ContainerStatus::Stopped.label(), "Stopped");
        assert_eq!(ContainerStatus::Exited(0).label(), "Exited");
        assert_eq!(ContainerStatus::Restarting.label(), "Restarting");
        assert_eq!(ContainerStatus::Dead.label(), "Dead");
    }

    #[test]
    fn status_css_classes() {
        assert_eq!(ContainerStatus::Running.css_class(), "success");
        assert_eq!(ContainerStatus::Restarting.css_class(), "accent");
        assert_eq!(ContainerStatus::Paused.css_class(), "warning");
        assert_eq!(ContainerStatus::Stopped.css_class(), "dim-label");
        assert_eq!(ContainerStatus::Exited(1).css_class(), "dim-label");
        assert_eq!(ContainerStatus::Dead.css_class(), "error");
        assert_eq!(
            ContainerStatus::Unknown("x".into()).css_class(),
            "dim-label"
        );
    }

    #[test]
    fn status_icon_names() {
        assert_eq!(
            ContainerStatus::Running.icon_name(),
            "media-playback-start-symbolic"
        );
        assert_eq!(
            ContainerStatus::Unknown("x".into()).icon_name(),
            "emblem-default-symbolic"
        );
    }

    #[test]
    fn status_display_label() {
        assert_eq!(ContainerStatus::Running.display_label(), "Running");
        assert_eq!(
            ContainerStatus::Unknown("x".into()).display_label(),
            "Unknown"
        );
    }

    #[test]
    fn container_default_equality() {
        assert_eq!(Container::default(), Container::default());
    }

    #[test]
    fn port_display_full() {
        let port = ContainerPort {
            host_ip: Some("0.0.0.0".to_string()),
            host_port: Some(8080),
            container_port: 80,
            protocol: "tcp".to_string(),
        };
        assert_eq!(port.display(), "0.0.0.0:8080:80/tcp");
    }

    #[test]
    fn port_display_no_ip() {
        let port = ContainerPort {
            host_ip: None,
            host_port: Some(8080),
            container_port: 80,
            protocol: "tcp".to_string(),
        };
        assert_eq!(port.display(), "8080:80/tcp");
    }

    #[test]
    fn port_display_no_host() {
        let port = ContainerPort {
            host_ip: None,
            host_port: None,
            container_port: 443,
            protocol: "tcp".to_string(),
        };
        assert_eq!(port.display(), "443/tcp");
    }

    #[test]
    fn stats_memory_mb_conversion() {
        let stats = ContainerStats {
            memory_usage: 10_485_760,    // 10 MiB
            memory_limit: 1_073_741_824, // 1 GiB
            ..Default::default()
        };
        assert!((stats.memory_usage_mb() - 10.0).abs() < 0.001);
        assert!((stats.memory_limit_mb() - 1024.0).abs() < 0.001);
    }

    #[test]
    fn restart_policy_strings() {
        assert_eq!(RestartPolicy::No.as_str(), "no");
        assert_eq!(RestartPolicy::Always.as_str(), "always");
        assert_eq!(RestartPolicy::OnFailure(3).as_str(), "on-failure");
        assert_eq!(RestartPolicy::UnlessStopped.as_str(), "unless-stopped");
    }

    // ── is_secret_env_key — table-driven ──────────────────────────────────────

    #[test]
    fn secret_key_classification() {
        let cases: &[(&str, bool)] = &[
            ("POSTGRES_PASSWORD", true),
            ("password", true),
            ("GITHUB_TOKEN", true),
            ("api_token", true),
            ("API_SECRET", true),
            ("my_secret", true),
            ("AWS_ACCESS_KEY_ID", true),
            ("PRIVATE_KEY_PATH", true),
            ("NGINX_HOST", false),
            ("TZ", false),
            ("PORT", false),
            ("", false),
        ];
        for (key, expected) in cases {
            assert_eq!(
                is_secret_env_key(key),
                *expected,
                "is_secret_env_key({key:?}) should be {expected}"
            );
        }
    }

    // ── filter_containers ──────────────────────────────────────────────────────

    fn make_test_container(
        name: &str,
        image: &str,
        short_id: &str,
        compose: Option<&str>,
    ) -> Container {
        Container {
            id: format!("{short_id}aabbccdd1122"),
            short_id: short_id.to_string(),
            name: name.to_string(),
            image: image.to_string(),
            command: "/start.sh".to_string(),
            created: 0,
            status: ContainerStatus::Running,
            status_text: "Running".to_string(),
            ports: vec![],
            labels: HashMap::new(),
            mounts: vec![],
            env: vec![],
            compose_project: compose.map(str::to_string),
            networks: vec![],
        }
    }

    #[test]
    fn filter_by_name_matches() {
        let containers = vec![
            make_test_container("nginx-proxy", "nginx:latest", "aabbccdd1122", None),
            make_test_container("postgres-db", "postgres:15", "112233445566", None),
        ];
        let result = filter_containers(&containers, "nginx");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "nginx-proxy");
    }

    #[test]
    fn filter_by_image_matches() {
        let containers = vec![
            make_test_container("nginx-proxy", "nginx:latest", "aabbccdd1122", None),
            make_test_container("postgres-db", "postgres:15", "112233445566", None),
        ];
        let result = filter_containers(&containers, "postgres");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "postgres-db");
    }

    #[test]
    fn filter_empty_query_returns_all() {
        let containers = vec![
            make_test_container("nginx-proxy", "nginx:latest", "aabbccdd1122", None),
            make_test_container("postgres-db", "postgres:15", "112233445566", None),
        ];
        let result = filter_containers(&containers, "");
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filter_no_match_returns_empty() {
        let containers = vec![make_test_container(
            "nginx-proxy",
            "nginx:latest",
            "aabbccdd1122",
            None,
        )];
        let result = filter_containers(&containers, "xyzzy");
        assert!(result.is_empty());
    }

    #[test]
    fn filter_case_insensitive() {
        let containers = vec![make_test_container(
            "nginx-proxy",
            "nginx:latest",
            "aabbccdd1122",
            None,
        )];
        let result = filter_containers(&containers, "NGINX");
        assert_eq!(result.len(), 1);
    }

    // ── group_by_compose ───────────────────────────────────────────────────────

    #[test]
    fn group_by_compose_three_same_project() {
        let containers = vec![
            make_test_container("web", "nginx:latest", "aabbccdd1122", Some("my-stack")),
            make_test_container("db", "postgres:15", "112233445566", Some("my-stack")),
            make_test_container("svc", "redis:alpine", "223344556677", Some("my-stack")),
        ];
        let groups = group_by_compose(&containers);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].0.as_deref(), Some("my-stack"));
        assert_eq!(groups[0].1.len(), 3);
    }

    #[test]
    fn group_by_compose_ungrouped_last() {
        let containers = vec![
            make_test_container("web", "nginx:latest", "aabbccdd1122", Some("stack-a")),
            make_test_container("solo", "alpine:latest", "bbccddee1122", None),
        ];
        let groups = group_by_compose(&containers);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].0.as_deref(), Some("stack-a"));
        assert!(groups[1].0.is_none());
    }

    #[test]
    fn group_by_compose_all_ungrouped() {
        let containers = vec![
            make_test_container("a", "nginx:latest", "aabbccdd1122", None),
            make_test_container("b", "redis:alpine", "112233445566", None),
        ];
        let groups = group_by_compose(&containers);
        assert_eq!(groups.len(), 1);
        assert!(groups[0].0.is_none());
        assert_eq!(groups[0].1.len(), 2);
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ContainerPort {
    pub host_ip: Option<String>,
    pub host_port: Option<u16>,
    pub container_port: u16,
    pub protocol: String,
}

impl ContainerPort {
    pub fn display(&self) -> String {
        match (self.host_port, &self.host_ip) {
            (Some(hp), Some(ip)) if !ip.is_empty() => {
                format!("{}:{}:{}/{}", ip, hp, self.container_port, self.protocol)
            }
            (Some(hp), _) => format!("{}:{}/{}", hp, self.container_port, self.protocol),
            _ => format!("{}/{}", self.container_port, self.protocol),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Container {
    pub id: String,
    pub short_id: String,
    pub name: String,
    pub image: String,
    pub command: String,
    pub created: i64,
    pub status: ContainerStatus,
    pub status_text: String,
    pub ports: Vec<ContainerPort>,
    pub labels: HashMap<String, String>,
    pub mounts: Vec<String>,
    /// Environment variables in KEY=VALUE format. Populated from inspect data.
    pub env: Vec<String>,
    /// Compose project name extracted from the `com.docker.compose.project` label.
    pub compose_project: Option<String>,
    /// Names of Docker networks this container is connected to.
    pub networks: Vec<String>,
}

impl Container {
    pub fn short_id(&self) -> &str {
        &self.short_id
    }
}

/// Returns `true` if an env var key name suggests it holds a secret.
///
/// Matches keys that contain PASS, SECRET, KEY, or TOKEN (case-insensitive).
/// Used to mask values in the UI before displaying them.
pub fn is_secret_env_key(key: &str) -> bool {
    let upper = key.to_ascii_uppercase();
    upper.contains("PASS")
        || upper.contains("SECRET")
        || upper.contains("KEY")
        || upper.contains("TOKEN")
}

/// Filters `containers` by a case-insensitive query string.
///
/// Matches against name, image, short ID, and compose project.
pub fn filter_containers<'a>(containers: &'a [Container], query: &str) -> Vec<&'a Container> {
    if query.is_empty() {
        return containers.iter().collect();
    }
    let q = query.to_ascii_lowercase();
    containers
        .iter()
        .filter(|c| {
            c.name.to_ascii_lowercase().contains(&q)
                || c.image.to_ascii_lowercase().contains(&q)
                || c.short_id.to_ascii_lowercase().contains(&q)
                || c.compose_project
                    .as_deref()
                    .map(|p| p.to_ascii_lowercase().contains(&q))
                    .unwrap_or(false)
        })
        .collect()
}

/// Groups containers by their compose project.
///
/// Returns a `Vec` of `(project_name, containers)` pairs sorted alphabetically,
/// with ungrouped containers (no compose project) last under `None`.
pub fn group_by_compose(containers: &[Container]) -> Vec<(Option<String>, Vec<&Container>)> {
    let mut groups: std::collections::BTreeMap<Option<String>, Vec<&Container>> =
        std::collections::BTreeMap::new();
    for c in containers {
        groups.entry(c.compose_project.clone()).or_default().push(c);
    }
    // Sort: named groups first (alphabetically), ungrouped last.
    let mut named: Vec<_> = groups
        .iter()
        .filter(|(k, _)| k.is_some())
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    named.sort_by(|(a, _), (b, _)| a.cmp(b));
    let ungrouped = groups.get(&None).cloned().unwrap_or_default();
    if !ungrouped.is_empty() {
        named.push((None, ungrouped));
    }
    named
}

#[derive(Debug, Clone, Default)]
pub struct ContainerStats {
    pub id: String,
    pub name: String,
    pub cpu_percent: f64,
    pub memory_usage: u64,
    pub memory_limit: u64,
    pub memory_percent: f64,
    pub net_rx_bytes: u64,
    pub net_tx_bytes: u64,
    pub block_read: u64,
    pub block_write: u64,
    pub pids: u64,
}

impl ContainerStats {
    pub fn memory_usage_mb(&self) -> f64 {
        self.memory_usage as f64 / 1_048_576.0
    }

    pub fn memory_limit_mb(&self) -> f64 {
        self.memory_limit as f64 / 1_048_576.0
    }
}

#[derive(Debug, Clone, Default)]
pub enum RestartPolicy {
    #[default]
    No,
    Always,
    OnFailure(u32),
    UnlessStopped,
}

impl RestartPolicy {
    pub fn as_str(&self) -> &str {
        match self {
            Self::No => "no",
            Self::Always => "always",
            Self::OnFailure(_) => "on-failure",
            Self::UnlessStopped => "unless-stopped",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CreateContainerOptions {
    pub image: String,
    pub name: Option<String>,
    pub command: Vec<String>,
    pub env: Vec<String>,
    /// (host_port, container_port)
    pub port_bindings: Vec<(u16, u16)>,
    /// (host_path, container_path)
    pub volume_bindings: Vec<(String, String)>,
    pub restart_policy: RestartPolicy,
    pub auto_remove: bool,
    pub network: Option<String>,
}

/// Per-layer progress event streamed during `pull_image_streaming`.
#[derive(Debug, Clone)]
pub struct PullProgress {
    pub layer_id: String,
    pub status: PullStatus,
    /// Download completion 0–100, or `None` until the size is known.
    pub percent: Option<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PullStatus {
    Waiting,
    Pulling,
    Downloading(u8),
    Verifying,
    Done,
}
