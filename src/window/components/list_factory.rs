// SPDX-License-Identifier: GPL-3.0-or-later

#[allow(dead_code)]
/// Build a `SignalListItemFactory` from separate setup and bind closures.
///
/// - `setup`: called once per visible slot to create the widget skeleton.
/// - `bind`: called every time a slot is bound to a data item.
///
/// Callers receive a `&glib::Object` that must be downcast to `gtk4::ListItem`:
/// ```no_run
/// make_factory(
///     |_, obj| {
///         let item = obj.downcast_ref::<gtk4::ListItem>().unwrap();
///         item.set_child(Some(&gtk4::Label::new(None)));
///     },
///     |_, obj| {
///         let item = obj.downcast_ref::<gtk4::ListItem>().unwrap();
///         // bind data to item.child()
///     },
/// );
/// ```
pub fn make_factory<S, B>(setup: S, bind: B) -> gtk4::SignalListItemFactory
where
    S: Fn(&gtk4::SignalListItemFactory, &glib::Object) + 'static,
    B: Fn(&gtk4::SignalListItemFactory, &glib::Object) + 'static,
{
    let factory = gtk4::SignalListItemFactory::new();
    factory.connect_setup(setup);
    factory.connect_bind(bind);
    factory
}
