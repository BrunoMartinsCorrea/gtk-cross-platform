// SPDX-License-Identifier: GPL-3.0-or-later
use std::cell::{Cell, RefCell};

use glib::Properties;
use glib::prelude::*;
use glib::subclass::prelude::*;

use gtk_cross_platform::core::domain::volume::Volume;

mod imp {
    use super::*;

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::VolumeObject)]
    pub struct VolumeObject {
        #[property(get, set)]
        pub name: RefCell<String>,
        #[property(get, set)]
        pub driver: RefCell<String>,
        #[property(get, set)]
        pub mountpoint: RefCell<String>,
        #[property(get, set)]
        pub scope: RefCell<String>,
        /// Size in bytes; -1 when unknown (None in domain)
        #[property(get, set)]
        pub size_bytes: Cell<i64>,
        #[property(get, set)]
        pub in_use: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VolumeObject {
        const NAME: &'static str = "GtkCrossPlatformVolumeObject";
        type Type = super::VolumeObject;
    }

    impl ObjectImpl for VolumeObject {
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
    pub struct VolumeObject(ObjectSubclass<imp::VolumeObject>);
}

impl VolumeObject {
    pub fn from_domain(vol: &Volume) -> Self {
        let obj: Self = glib::Object::new();
        obj.set_name(vol.name.clone());
        obj.set_driver(vol.driver.clone());
        obj.set_mountpoint(vol.mountpoint.clone());
        obj.set_scope(vol.scope.clone());
        obj.set_size_bytes(vol.size_bytes.map(|b| b as i64).unwrap_or(-1));
        obj.set_in_use(vol.in_use);
        obj
    }
}
