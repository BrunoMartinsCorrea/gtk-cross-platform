// SPDX-License-Identifier: GPL-3.0-or-later
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CreateVolumeOptions {
    pub name: String,
    pub driver: String,
    pub labels: HashMap<String, String>,
}

impl Default for CreateVolumeOptions {
    fn default() -> Self {
        Self {
            name: String::new(),
            driver: "local".to_string(),
            labels: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Volume {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
    pub created: String,
    pub labels: HashMap<String, String>,
    pub scope: String,
    pub size_bytes: Option<u64>,
    pub in_use: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_volume() -> Volume {
        Volume {
            name: "postgres-data".into(),
            driver: "local".into(),
            mountpoint: "/var/lib/docker/volumes/postgres-data/_data".into(),
            created: "2024-01-01T00:00:00Z".into(),
            labels: HashMap::new(),
            scope: "local".into(),
            size_bytes: None,
            in_use: false,
        }
    }

    #[test]
    fn volume_fields_accessible() {
        let v = make_volume();
        assert_eq!(v.name, "postgres-data");
        assert_eq!(v.driver, "local");
        assert_eq!(v.scope, "local");
        assert!(!v.mountpoint.is_empty());
    }

    #[test]
    fn volume_default_equality() {
        assert_eq!(Volume::default(), Volume::default());
    }

    #[test]
    fn volume_with_labels() {
        let mut v = make_volume();
        v.labels.insert("com.example.env".into(), "prod".into());
        assert_eq!(
            v.labels.get("com.example.env").map(String::as_str),
            Some("prod")
        );
    }
}
