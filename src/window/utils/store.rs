// SPDX-License-Identifier: GPL-3.0-or-later
use gtk4::gio;
use gtk4::gio::prelude::*;
use gtk4::glib;

pub fn find_store_position<T, F>(store: &gio::ListStore, pred: F) -> Option<u32>
where
    T: glib::object::IsA<glib::Object>,
    F: Fn(&T) -> bool,
{
    (0..store.n_items()).find(|&i| {
        store
            .item(i)
            .and_downcast::<T>()
            .map_or(false, |o| pred(&o))
    })
}
