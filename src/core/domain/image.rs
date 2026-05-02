// SPDX-License-Identifier: GPL-3.0-or-later
use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Image {
    pub id: String,
    pub short_id: String,
    pub tags: Vec<String>,
    pub size: u64,
    pub created: i64,
    pub digest: Option<String>,
    pub labels: HashMap<String, String>,
    pub in_use: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ImageLayer {
    pub id: String,
    pub cmd: String,
    pub size: u64,
}

impl Image {
    pub fn size_mb(&self) -> f64 {
        self.size as f64 / 1_048_576.0
    }

    pub fn primary_tag(&self) -> &str {
        self.tags
            .first()
            .map(String::as_str)
            .unwrap_or("<none>:<none>")
    }

    pub fn is_dangling(&self) -> bool {
        !self.in_use && (self.tags.is_empty() || self.tags.iter().all(|t| t == "<none>:<none>"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_image(tags: Vec<&str>, size: u64) -> Image {
        Image {
            id: "sha256:abc".into(),
            short_id: "abc123".into(),
            tags: tags.into_iter().map(str::to_owned).collect(),
            size,
            created: 0,
            digest: None,
            labels: HashMap::new(),
            in_use: false,
        }
    }

    #[test]
    fn primary_tag_returns_first() {
        let img = make_image(vec!["nginx:latest", "nginx:1.25"], 0);
        assert_eq!(img.primary_tag(), "nginx:latest");
    }

    #[test]
    fn primary_tag_no_tags_returns_fallback() {
        let img = make_image(vec![], 0);
        assert_eq!(img.primary_tag(), "<none>:<none>");
    }

    #[test]
    fn size_mb_converts_correctly() {
        let img = make_image(vec![], 1_048_576);
        assert!((img.size_mb() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn size_mb_zero() {
        let img = make_image(vec![], 0);
        assert_eq!(img.size_mb(), 0.0);
    }

    #[test]
    fn is_dangling_no_tags_not_in_use() {
        let img = make_image(vec![], 0);
        assert!(img.is_dangling());
    }

    #[test]
    fn is_dangling_none_tag_not_in_use() {
        let img = make_image(vec!["<none>:<none>"], 0);
        assert!(img.is_dangling());
    }

    #[test]
    fn not_dangling_when_in_use() {
        let mut img = make_image(vec![], 0);
        img.in_use = true;
        assert!(!img.is_dangling());
    }

    #[test]
    fn not_dangling_when_has_tag() {
        let img = make_image(vec!["nginx:latest"], 0);
        assert!(!img.is_dangling());
    }

    #[test]
    fn image_default_equality() {
        assert_eq!(Image::default(), Image::default());
    }

    #[test]
    fn image_layer_fields() {
        let layer = ImageLayer {
            id: "abc123def456".into(),
            cmd: "RUN apt-get install -y curl".into(),
            size: 5_000_000,
        };
        assert_eq!(layer.id, "abc123def456");
        assert_eq!(layer.size, 5_000_000);
    }
}
