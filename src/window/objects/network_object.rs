// SPDX-License-Identifier: GPL-3.0-or-later
use std::cell::{Cell, RefCell};

use glib::Properties;
use glib::prelude::*;
use glib::subclass::prelude::*;

use gtk_cross_platform::core::domain::network::Network;

mod imp {
    use super::*;

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::NetworkObject)]
    pub struct NetworkObject {
        #[property(get, set)]
        pub id: RefCell<String>,
        #[property(get, set)]
        pub name: RefCell<String>,
        #[property(get, set)]
        pub driver: RefCell<String>,
        #[property(get, set)]
        pub scope: RefCell<String>,
        #[property(get, set)]
        pub containers_count: Cell<i64>,
        #[property(get, set)]
        pub internal: Cell<bool>,
        /// Subnet CIDR, empty when None
        #[property(get, set)]
        pub subnet: RefCell<String>,
        /// Gateway address, empty when None
        #[property(get, set)]
        pub gateway: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NetworkObject {
        const NAME: &'static str = "GtkCrossPlatformNetworkObject";
        type Type = super::NetworkObject;
    }

    impl ObjectImpl for NetworkObject {
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
    pub struct NetworkObject(ObjectSubclass<imp::NetworkObject>);
}

impl NetworkObject {
    pub fn from_domain(net: &Network) -> Self {
        let obj: Self = glib::Object::new();
        obj.set_id(net.id.clone());
        obj.set_name(net.name.clone());
        obj.set_driver(net.driver.clone());
        obj.set_scope(net.scope.clone());
        obj.set_containers_count(net.containers_count as i64);
        obj.set_internal(net.internal);
        obj.set_subnet(net.subnet.clone().unwrap_or_default());
        obj.set_gateway(net.gateway.clone().unwrap_or_default());
        obj
    }
}
