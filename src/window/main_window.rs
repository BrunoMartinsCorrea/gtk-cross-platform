// SPDX-License-Identifier: GPL-3.0-or-later
use std::cell::OnceCell;
use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk4::gio;
use gtk4::CompositeTemplate;

use gtk_cross_platform::config;
use gtk_cross_platform::ports::i_greeting_service::IGreetingService;

mod imp {
    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/example/GtkCrossPlatform/window.ui")]
    pub struct MainWindow {
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub button: TemplateChild<gtk4::Button>,
        pub greeting_service: OnceCell<Rc<dyn IGreetingService>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MainWindow {
        const NAME: &'static str = "GtkCrossPlatformMainWindow";
        type Type = super::MainWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MainWindow {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_icon_name(Some(config::APP_ID));
            self.setup_long_press();
        }
    }

    impl WidgetImpl for MainWindow {}
    impl WindowImpl for MainWindow {}
    impl ApplicationWindowImpl for MainWindow {}
    impl AdwApplicationWindowImpl for MainWindow {}

    impl MainWindow {
        fn setup_long_press(&self) {
            let long_press = gtk4::GestureLongPress::new();
            let toast_overlay = (*self.toast_overlay).clone();
            long_press.connect_pressed(move |_, _, _| {
                let toast = adw::Toast::new(&gettextrs::gettext("Long press detected!"));
                toast.set_timeout(2);
                toast_overlay.add_toast(toast);
            });
            self.button.add_controller(long_press);
        }
    }
}

glib::wrapper! {
    pub struct MainWindow(ObjectSubclass<imp::MainWindow>)
        @extends adw::ApplicationWindow, gtk4::ApplicationWindow, gtk4::Window, gtk4::Widget,
        @implements gio::ActionGroup, gio::ActionMap,
                    gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget,
                    gtk4::Native, gtk4::Root, gtk4::ShortcutManager;
}

impl MainWindow {
    pub fn new(app: &impl IsA<adw::Application>, greeting_service: Rc<dyn IGreetingService>) -> Self {
        let win: Self = glib::Object::builder()
            .property("application", app)
            .build();
        win.imp()
            .greeting_service
            .set(greeting_service)
            .ok()
            .expect("greeting_service already set");
        win.connect_button_signal();
        win
    }

    fn connect_button_signal(&self) {
        let imp = self.imp();
        let button = (*imp.button).clone();
        let toast_overlay = (*imp.toast_overlay).clone();
        let greeting_service = imp.greeting_service.get().unwrap().clone();

        button.connect_clicked(move |_| {
            let message = greeting_service.greet();
            let toast = adw::Toast::new(&gettextrs::gettext(message.as_str()));
            toast.set_timeout(3);
            toast_overlay.add_toast(toast);
        });
    }
}
