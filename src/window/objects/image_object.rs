// SPDX-License-Identifier: GPL-3.0-or-later
use std::cell::{Cell, RefCell};

use glib::Properties;
use glib::prelude::*;
use glib::subclass::prelude::*;

use gtk_cross_platform::core::domain::image::Image;

mod imp {
    use super::*;

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::ImageObject)]
    pub struct ImageObject {
        #[property(get, set)]
        pub id: RefCell<String>,
        #[property(get, set)]
        pub short_id: RefCell<String>,
        /// Primary tag (first tag, or "<none>:<none>")
        #[property(get, set)]
        pub tags: RefCell<String>,
        /// Size in bytes as i64 for GLib compatibility
        #[property(get, set)]
        pub size: Cell<i64>,
        /// Seconds since epoch (raw i64 from domain)
        #[property(get, set)]
        pub created: Cell<i64>,
        /// Digest string, empty when None
        #[property(get, set)]
        pub digest: RefCell<String>,
        #[property(get, set)]
        pub in_use: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ImageObject {
        const NAME: &'static str = "GtkCrossPlatformImageObject";
        type Type = super::ImageObject;
    }

    impl ObjectImpl for ImageObject {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }
        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }
        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }
}

glib::wrapper! {
    pub struct ImageObject(ObjectSubclass<imp::ImageObject>);
}

impl ImageObject {
    pub fn from_domain(img: &Image) -> Self {
        let obj: Self = glib::Object::new();
        obj.set_id(img.id.clone());
        obj.set_short_id(img.short_id.clone());
        obj.set_tags(img.primary_tag().to_string());
        obj.set_size(img.size as i64);
        obj.set_created(img.created);
        obj.set_digest(img.digest.clone().unwrap_or_default());
        obj.set_in_use(img.in_use);
        obj
    }

    /// True when the image has no meaningful tag and is not in use.
    pub fn is_dangling(&self) -> bool {
        !self.in_use() && (self.tags().is_empty() || self.tags() == "<none>:<none>")
    }
}
