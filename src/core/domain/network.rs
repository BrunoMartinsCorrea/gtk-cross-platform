// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Debug, Clone)]
pub struct CreateNetworkOptions {
    pub name: String,
    pub driver: String,
    pub subnet: Option<String>,
}

impl Default for CreateNetworkOptions {
    fn default() -> Self {
        Self {
            name: String::new(),
            driver: "bridge".to_string(),
            subnet: None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Network {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub scope: String,
    pub internal: bool,
    pub created: String,
    pub subnet: Option<String>,
    pub gateway: Option<String>,
    pub containers_count: u64,
}

#[derive(Debug, Clone)]
pub struct ContainerEvent {
    pub timestamp: String,
    pub event_type: String,
    pub action: String,
    pub actor: String,
    pub actor_id: String,
}

#[derive(Debug, Clone, Default)]
pub struct SystemUsage {
    pub containers_total: u64,
    pub containers_running: u64,
    pub containers_stopped: u64,
    pub images_total: u64,
    pub images_size: u64,
    pub volumes_total: u64,
    pub volumes_size: u64,
}

#[derive(Debug, Clone, Default)]
pub struct PruneReport {
    pub containers_deleted: Vec<String>,
    pub images_deleted: Vec<String>,
    pub volumes_deleted: Vec<String>,
    pub space_reclaimed: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_usage_fields_accessible() {
        let u = SystemUsage {
            containers_total: 5,
            containers_running: 3,
            containers_stopped: 2,
            images_total: 10,
            images_size: 1_048_576,
            volumes_total: 4,
            volumes_size: 2_097_152,
        };
        assert_eq!(u.containers_total, 5);
        assert_eq!(u.containers_running, 3);
        assert_eq!(u.containers_stopped, 2);
    }

    #[test]
    fn system_usage_default_is_zero() {
        let u = SystemUsage::default();
        assert_eq!(u.containers_total, 0);
        assert_eq!(u.images_total, 0);
        assert_eq!(u.volumes_total, 0);
    }

    #[test]
    fn prune_report_space_reclaimed() {
        let r = PruneReport {
            containers_deleted: vec!["abc123".into(), "def456".into()],
            images_deleted: vec!["img1".into()],
            volumes_deleted: vec![],
            space_reclaimed: 5_000_000,
        };
        assert_eq!(r.containers_deleted.len(), 2);
        assert_eq!(r.images_deleted.len(), 1);
        assert!(r.volumes_deleted.is_empty());
        assert_eq!(r.space_reclaimed, 5_000_000);
    }

    #[test]
    fn prune_report_default_is_empty() {
        let r = PruneReport::default();
        assert!(r.containers_deleted.is_empty());
        assert_eq!(r.space_reclaimed, 0);
    }

    #[test]
    fn network_fields_accessible() {
        let n = Network {
            id: "net1".into(),
            name: "bridge".into(),
            driver: "bridge".into(),
            scope: "local".into(),
            internal: false,
            created: "2024-01-01T00:00:00Z".into(),
            subnet: Some("172.17.0.0/16".into()),
            gateway: Some("172.17.0.1".into()),
            containers_count: 3,
        };
        assert_eq!(n.name, "bridge");
        assert!(!n.internal);
        assert!(n.subnet.is_some());
        assert_eq!(n.containers_count, 3);
    }

    #[test]
    fn network_default_equality() {
        assert_eq!(Network::default(), Network::default());
    }

    #[test]
    fn network_internal_flag() {
        let n = Network {
            id: "net2".into(),
            name: "isolated".into(),
            driver: "bridge".into(),
            scope: "local".into(),
            internal: true,
            created: "2024-01-01T00:00:00Z".into(),
            subnet: None,
            gateway: None,
            containers_count: 0,
        };
        assert!(n.internal);
        assert!(n.subnet.is_none());
        assert!(n.gateway.is_none());
    }

    #[test]
    fn container_event_fields() {
        let e = ContainerEvent {
            timestamp: "12:00:00".into(),
            event_type: "container".into(),
            action: "start".into(),
            actor: "web-server".into(),
            actor_id: "aabbccdd1122".into(),
        };
        assert_eq!(e.event_type, "container");
        assert_eq!(e.action, "start");
    }
}
