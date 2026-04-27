// SPDX-License-Identifier: GPL-3.0-or-later
use std::cell::RefCell;

use glib::Properties;
use glib::prelude::*;
use glib::subclass::prelude::*;

use gtk_cross_platform::core::domain::container::Container;

mod imp {
    use super::*;

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::ContainerObject)]
    pub struct ContainerObject {
        #[property(get, set)]
        pub id: RefCell<String>,
        #[property(get, set)]
        pub name: RefCell<String>,
        /// Display label from ContainerStatus::label() — e.g. "Running", "Stopped"
        #[property(get, set)]
        pub status: RefCell<String>,
        /// CSS class from ContainerStatus::css_class() — e.g. "success", "dim-label"
        #[property(get, set)]
        pub status_css: RefCell<String>,
        #[property(get, set)]
        pub image: RefCell<String>,
        #[property(get, set)]
        pub short_id: RefCell<String>,
        /// Compose project name, empty string when None
        #[property(get, set)]
        pub compose_project: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ContainerObject {
        const NAME: &'static str = "GtkCrossPlatformContainerObject";
        type Type = super::ContainerObject;
    }

    impl ObjectImpl for ContainerObject {
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
    pub struct ContainerObject(ObjectSubclass<imp::ContainerObject>);
}

impl ContainerObject {
    pub fn from_domain(c: &Container) -> Self {
        let obj: Self = glib::Object::new();
        obj.set_id(c.id.clone());
        obj.set_name(c.name.clone());
        obj.set_status(c.status.label().to_string());
        obj.set_status_css(c.status.css_class().to_string());
        obj.set_image(c.image.clone());
        obj.set_short_id(c.short_id.clone());
        obj.set_compose_project(c.compose_project.clone().unwrap_or_default());
        obj
    }
}
