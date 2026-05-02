# GTK in Rust — Reference Projects & Example Topics

The Rust/GTK ecosystem is centred on the `gtk4-rs` crate, which provides safe bindings generated via
GObject-Introspection for GTK
4, GLib, GIO and Pango. Since 2023 the stack has reached production-ready maturity: applications like Loupe, Fractal and
Authenticator are part of GNOME Core/Circle and demonstrate that idiomatic patterns are well established. Even so,
high-level
documentation is sparse — the projects below are the primary sources the community uses as real-world references.

---

## 1. gtk4-rs (Bindings Oficiais)

- **Repository:** https://github.com/gtk-rs/gtk4-rs
- **Type:** official bindings
- **GTK version:** GTK4
- **Topics covered:**
    - GObject subclassing (`#[glib::object_subclass]`, `glib::wrapper!`): present in `examples/` and book listings
      as the canonical entry point for custom widgets.
    - GObject properties via derive macro (`#[derive(Properties)]`, `#[property(get, set)]`): the book dedicates a full
      chapter with a progression from `Cell<i32>` to bidirectional property bindings.
    - Signal handling and custom signals (`glib::subclass::Signal`, `OnceLock`): demonstrated in the CustomButton
      progression with signals such as `max-number-reached`.
    - State management with `RefCell` / `Cell` / `OnceCell`: recurring pattern across all subclassing examples;
      `RefCell` for mutable state in `imp::*`, `OnceCell` for one-time initialisation of `Settings`.
    - Composite templates (`.ui` + `#[derive(CompositeTemplate)]`, `#[template_child]`): `composite_template/` example
      in the repo; Todo tutorial uses templates for `Window` and `TaskRow`.
    - `GListModel` + list views + factory pattern (`gio::ListStore`, `ListView`, `SignalListItemFactory`,
      `NoSelection`): Todo tutorial evolves from a simple `ListBox` to `ListView` with factory and widget recycling.
    - `GSettings` integration (`gio::Settings` with compiled schema): integrated in the Todo 2 version to persist the
      active filter colour between sessions.
    - `gio::SimpleAction` + `install_action` / `install_action_async`: pattern for menu actions and keyboard shortcuts
      in the Todo tutorial.
    - CSS styling (`CssProvider`, `StyleContext::add_provider_for_display`): `css/` example and tutorial usage for
      dynamically switching themes.
    - Async with `glib::MainContext::spawn_local`: examples of non-blocking I/O operations inside the GLib event loop
      without threads.
    - Libadwaita (`adw::Application`, `adw::StyleManager`, `adw::ActionRow`): dedicated chapter in the book migrating
      the Todo app to Adwaita with `adw::ApplicationWindow`.
    - Drag & drop (`DragSource`, `DropTarget` with `gdk::ContentProvider`): `drag_and_drop/` example in the repository.
    - Clipboard API (`gdk::Clipboard`, `widget.clipboard()`, `read_text_async`): `clipboard/` example in the repository.
    - Custom drawing (`DrawingArea` + `set_draw_func`, snapshot with `gtk::Snapshot`): `custom_paintable/` example.
    - OpenGL via `GlArea`: `glium_gl_area/` example using glium for custom rendering.
    - Video playback (`gtk::Video`, `gtk::MediaFile`): `video_player/` example.
    - `GtkExpression` and declarative property bindings: demonstrated in a dedicated book chapter for bidirectional
      binding without imperative code.

---

## 2. GUI Development with Rust and GTK 4 (Book + Listings)

- **Repository:** https://github.com/gtk-rs/gtk4-rs (`book/` directory)
- **Documentation:** https://gtk-rs.org/gtk4-rs/stable/latest/book/
- **Type:** official tutorial (book + executable listings)
- **GTK version:** GTK4
- **Topics covered:**
    - Complete widget lifecycle: from `Application::connect_activate` to `ApplicationWindow::present`, covering
      realize/map/show.
    - GObject memory model: `glib::clone!` macro for weak references in signal closures, avoiding reference cycles.
    - Interior mutability pattern: progression from `Cell` (Copy types) → `RefCell` (heap types) → `OnceCell`
      (init-once), with rationale for each choice.
    - Composite templates with Blueprint (alternative to XML `.ui`): referenced in the book as a modern option where
      the `blueprint-compiler` emits `.ui`.
    - Saving/restoring window state via `GSettings`: the Todo app saves tasks in JSON + filter preferences in
      `GSettings`.
    - `FilterListModel` + `CustomFilter` for reactively filtered lists without recreating widgets.
    - Async actions with `install_action_async`: pattern for I/O operations triggered from a menu without blocking the
      event loop.
    - Libadwaita `adw::ToastOverlay`, `adw::SwitchRow`, `adw::PreferencesWindow`: chapter showing how to replace
      basic GTK widgets with their Adwaita equivalents.
    - Persistence with `serde_json` + `gio::File`: serialising application state to the XDG data dir.

---

## 3. Relm4

- **Repository:** https://github.com/Relm4/Relm4
- **Documentation:** https://relm4.org/book/stable/
- **Type:** framework / library on top of gtk4-rs
- **GTK version:** GTK4
- **Topics covered:**
    - Elm-like Model-Update-View: each component implements `SimpleComponent` or `Component` with `type Input`,
      `type Output`, and a pure `update()`.
    - Component hierarchy with typed messages: `ComponentSender` for parent → child and child → parent communication
      via `Output`, eliminating shared closures.
    - `AsyncComponent` for asynchronous operations: `update_async()` runs in a separate tokio runtime; the result is
      sent back via `sender.input()` to the GLib event loop.
    - Factory pattern for lists (`FactoryVecDeque`, `FactoryHashMap`): a more ergonomic alternative to GTK's
      `GListModel` + `ListItemFactory`.
    - `RelmWorker` / background threads: pattern for CPU-bound tasks without blocking the UI.
    - Declarative widget macros (`view!`, `#[relm4::component]`): drastically reduces widget creation and connection
      boilerplate.
    - `relm4-components`: ready-made reusable components (file chooser dialog, alert dialog, spinner) demonstrating
      the component library pattern.
    - Typed `gio::SimpleAction` wrappers: `relm4::actions!` macro that types actions, avoiding magic strings.
    - Libadwaita integration: `RelmApp` with `adw::Application`, support for `adw::*` widgets directly in `view!`.

---

## 4. Fractal (GNOME Matrix Client)

- **Repository:** https://gitlab.gnome.org/GNOME/fractal
- **Type:** application (GNOME Circle)
- **GTK version:** GTK4 (complete rewrite in GTK4 + matrix-rust-sdk since v5)
- **Topics covered:**
    - Async with `tokio` + bridge to `glib::MainContext`: matrix-rust-sdk runs in a tokio runtime; results are sent
      to the GTK thread via `glib::MainContext::channel()` or `spawn_from_within`.
    - `adw::NavigationView` + `adw::NavigationPage`: multi-level navigation pattern (room list → room → thread),
      reference for apps with a screen hierarchy.
    - Complex custom widgets via subclassing: `MessageRow`, `RoomRow`, `MediaViewer` are GObjects with their own
      template, signals, and properties.
    - `GListModel` + factory for high-volume lists: message and room-member lists use `ListView` +
      `SignalListItemFactory` with recycling.
    - Secret Service via `oo7` crate: secure storage of Matrix session tokens, decoupled from GNOME
      Keyring/KWallet via the Secret Service API abstraction.
    - i18n with `gettext-rs`: all UI strings go through `gettext()` / `i18n()`, with `.po` files managed by the
      GNOME Translation Platform.
    - Flatpak packaging: complete manifest with Rust SDK extension, GNOME runtime, and
      `--talk-name=org.freedesktop.secrets` in the sandbox.
    - DBus portal (`ashpd` crate): `FileChooserPortal` for file selection inside the Flatpak sandbox,
      `NotificationPortal` for system notifications.
    - Reactive property bindings: UI state (e.g. send button disabled when input is empty) via
      `bind_property().sync_create().build()`.
    - Drag & drop for file upload: `DropTarget` accepts `gdk::FileList` in the message composition area.

---

## 5. Shortwave (Internet Radio)

- **Repository:** https://gitlab.gnome.org/World/Shortwave
- **Type:** application (GNOME Circle)
- **GTK version:** GTK4
- **Topics covered:**
    - GStreamer pipeline in Rust (`gstreamer-rs`): `playbin` pipeline for HTTP audio streaming, with `gst::Bus`
      message handling via `add_watch` on the GLib event loop.
    - `GSettings` for persistent preferences: volume, preferred codec, station history — schema declared in
      `.gschema.xml` compiled via Meson.
    - Async REST with `reqwest`: calls to the RadioBrowser API for station search; executed via
      `glib::MainContext::spawn_local` with `reqwest` compiled with the `rustls` feature.
    - MPRIS DBus interface (`mpris2-zbus` or custom via `zbus`): exposes playback controls to the system (media keys,
      panel applets).
    - Adaptive UI with `libadwaita`: `adw::Breakpoint` + `adw::OverlaySplitView` to adapt the desktop layout for
      mobile/narrow screens.
    - Custom GObject for the station model: `Station` as a GObject with properties (`name`, `url`, `favicon`) for
      direct binding with list widgets.
    - Composite templates: main window, settings dialog, and station row defined in `.ui` with `#[template_child]`.
    - Flatpak + Meson: `.json` manifest with GStreamer plugin dependencies; Meson build system compiling GLib
      resources, schemas, and gettext.
    - i18n with gettext + GNOME Translation Platform: usage of `gettextrs` and `cargo-i18n`.

---

## 6. Amberol (Music Player)

- **Repository:** https://gitlab.gnome.org/World/amberol
- **Type:** application (GNOME Circle)
- **GTK version:** GTK4
- **Topics covered:**
    - Advanced GStreamer pipeline: `playbin3` with high-level `GstPlay`; position/duration management via
      `glib::timeout_add_local` polling the pipeline.
    - Dynamic CSS theming based on artwork: extraction of the dominant colour from album art via `glycin` /
      `gdk_pixbuf`; applied via a dynamically replaced `CssProvider` to create gradients that match the current album.
    - `gdk::Clipboard` for copying metadata: demonstrates `clipboard().set_text()` and asynchronous reading.
    - Drag & drop for queuing songs: `DropTarget` with `gdk::FileList` on the main area, adding files to the
      queue's `gio::ListStore`.
    - `GListModel` + `GridView` for the playback queue: playlist as `gio::ListStore<SongObject>` with `GridView` and
      a custom factory.
    - MPRIS via DBus (`zbus`): integration with the operating system's media controls.
    - `glib::spawn_local` for async I/O: non-blocking reading of metadata (ID3/Vorbis) before adding to the queue.
    - Pure Libadwaita: `adw::Application`, `adw::Window` without a default `HeaderBar`, customisation of
      `WindowHandle`.
    - Composite templates with property bindings: queue rows with labels bound to `SongObject` properties.

---

## 7. GNOME Authenticator (2FA Generator)

- **Repository:** https://gitlab.gnome.org/World/Authenticator
- **Type:** application (GNOME Circle)
- **GTK version:** GTK4
- **Topics covered:**
    - Secret Service API via `oo7` crate: TOTP secret storage in GNOME Keyring / KWallet transparently across
      backends; reference pattern for apps that need secure credentials.
    - DBus with `zbus`: communication with the `org.freedesktop.secrets` daemon and integration with
      `org.freedesktop.portal.Camera` for QR code scanning.
    - GStreamer for camera (QR scan): camera pipeline with `gstreamer-rs`, frame analysis for QR code detection
      via `zbar` or `rqrr`.
    - `GSettings` for preferences: lock on idle, theme, default OTP algorithm.
    - Custom GObject for OTP account: `Account` as a GObject with properties (`issuer`, `label`, `algorithm`,
      `period`, `digits`) used for UI binding and serialisation.
    - `GListModel` + `ListView`: account list with `SignalListItemFactory`, search support via `FilterListModel` +
      `StringFilter`.
    - Composite templates with `adw::PreferencesWindow`: preferences window pattern divided into groups with
      `adw::PreferencesGroup` + `adw::ActionRow`.
    - Full gettext i18n + Flatpak: manifest with `--talk-name=org.freedesktop.secrets`, `--device=all` for camera.
    - TOTP/HOTP logic in pure Rust: code generation via the `totp-rs` crate, decoupled from UI as an isolated
      testable domain.

---

## 8. Loupe / Image Viewer (GNOME Core)

- **Repository:** https://gitlab.gnome.org/GNOME/loupe
- **Type:** application (GNOME Core, replaced Eye of GNOME in GNOME 45)
- **GTK version:** GTK4
- **Topics covered:**
    - Sandboxed image decoding via `glycin` crate: each image format runs in an isolated subprocess; reference pattern
      for subprocess IPC integration inside Flatpak with minimal `--talk-name`.
    - Custom rendering widget with `gtk::GLArea` + GPU tiling: accelerated rendering of large images via tiles, with
      viewport and zoom logic; `gtk::Widget` subclass overriding `snapshot()`.
    - Complete gesture handling: `GesturePinch` (zoom), `GestureDrag` (pan), `GestureSwipe` (navigate between images),
      `GestureClick` (toggle UI) — demonstrates the GTK4 event controller API in depth.
    - Async with `glib::spawn_local` + `glycin::Loader`: image loading is fully asynchronous; partial progress
      (thumbnails) is shown before the full decode completes.
    - Property bindings for zoom state: zoom level, rotation, and flip exposed as GObject properties; UI (buttons,
      labels) bound via `bind_property()`.
    - Drag & drop for opening files: `DropTarget` on the main window accepts `gdk::FileList`; also supports opening
      via the portal file chooser.
    - Clipboard (`gdk::Clipboard`): copying the current image to the clipboard as a `gdk::Texture`.
    - Metadata display via `glycin`: EXIF/XMP shown as `adw::PreferencesGroup` generated dynamically from the
      metadata returned by the loader.
    - i18n + Flatpak + Meson: manifest with camera/screen portal, build with `cargo-build` via Meson.
    - `adw::ToolbarView` + gestures for immersive UI: toolbar that auto-hides and reappears when the cursor returns.

---

## Theme Coverage Matrix

| Topic                                             | gtk4-rs bindings | gtk4-rs book | Relm4 | Fractal | Shortwave | Amberol | Authenticator | Loupe |
|---------------------------------------------------|:----------------:|:------------:|:-----:|:-------:|:---------:|:-------:|:-------------:|:-----:|
| Widget lifecycle & signal handling                |        ✓         |      ✓       |   ✓   |    ✓    |     ✓     |    ✓    |       ✓       |   ✓   |
| GObject subclassing                               |        ✓         |      ✓       |   —   |    ✓    |     ✓     |    ✓    |       ✓       |   ✓   |
| GObject properties (`#[derive(Properties)]`)      |        ✓         |      ✓       |   —   |    ✓    |     ✓     |    ✓    |       ✓       |   ✓   |
| Custom signals                                    |        ✓         |      ✓       |   —   |    ✓    |     —     |    —    |       —       |   —   |
| State mgmt (`RefCell`/`Cell`/`OnceCell`)          |        ✓         |      ✓       |   —   |    ✓    |     ✓     |    ✓    |       ✓       |   ✓   |
| Composite templates (`.ui` + `#[template_child]`) |        ✓         |      ✓       |   —   |    ✓    |     ✓     |    ✓    |       ✓       |   ✓   |
| `GListModel` + list views + factory               |        ✓         |      ✓       |   ✓   |    ✓    |     ✓     |    ✓    |       ✓       |   —   |
| `FilterListModel` / `SortListModel`               |        —         |      ✓       |   —   |    ✓    |     —     |    —    |       ✓       |   —   |
| `GSettings` integration                           |        —         |      ✓       |   —   |    ✓    |     ✓     |    —    |       ✓       |   —   |
| `gio::SimpleAction` / typed actions               |        ✓         |      ✓       |   ✓   |    ✓    |     ✓     |    —    |       ✓       |   ✓   |
| Dynamic CSS theming                               |        ✓         |      ✓       |   —   |    —    |     —     |    ✓    |       —       |   —   |
| Async (`glib::MainContext::spawn_local`)          |        ✓         |      ✓       |   ✓   |    ✓    |     ✓     |    ✓    |       ✓       |   ✓   |
| tokio ↔ glib bridge                               |        —         |      —       |   ✓   |    ✓    |     —     |    —    |       —       |   —   |
| GStreamer integration                             |        —         |      —       |   —   |    —    |     ✓     |    ✓    |       ✓       |   —   |
| DBus / `zbus`                                     |        —         |      —       |   —   |    ✓    |     ✓     |    ✓    |       ✓       |   —   |
| Secret Service (`oo7`)                            |        —         |      —       |   —   |    ✓    |     —     |    —    |       ✓       |   —   |
| Drag & drop                                       |        ✓         |      —       |   —   |    ✓    |     —     |    ✓    |       —       |   ✓   |
| Clipboard API                                     |        ✓         |      —       |   —   |    —    |     —     |    ✓    |       —       |   ✓   |
| Custom drawing / `GLArea`                         |        ✓         |      —       |   —   |    —    |     —     |    —    |       —       |   ✓   |
| Gesture controllers                               |        —         |      —       |   —   |    —    |     —     |    —    |       —       |   ✓   |
| Libadwaita (`adw::*`)                             |        —         |      ✓       |   ✓   |    ✓    |     ✓     |    ✓    |       ✓       |   ✓   |
| Adaptive UI (breakpoints)                         |        —         |      —       |   —   |    ✓    |     ✓     |    —    |       —       |   —   |
| i18n / gettext                                    |        —         |      —       |   —   |    ✓    |     ✓     |    ✓    |       ✓       |   ✓   |
| Flatpak packaging + Meson                         |        —         |      —       |   —   |    ✓    |     ✓     |    ✓    |       ✓       |   ✓   |
| Subprocess / portal IPC (`ashpd`)                 |        —         |      —       |   —   |    ✓    |     —     |    —    |       ✓       |   ✓   |
| Elm-like component model                          |        —         |      —       |   ✓   |    —    |     —     |    —    |       —       |   —   |
| Reactive property bindings                        |        ✓         |      ✓       |   —   |    ✓    |     —     |    ✓    |       —       |   ✓   |
| Sandboxed IPC (`glycin`)                          |        —         |      —       |   —   |    —    |     —     |    —    |       —       |   ✓   |

---

## Recommended Reading Order

For a developer starting a GTK4/Rust project, the most efficient progression is:

1. **gtk4-rs book** (chapters 1–6): GObject model, subclassing, properties, signals, composite templates, GSettings.
2. **gtk4-rs book** (Todo chapters 1–6): complete progression of a real app with GListModel, factory, actions,
   libadwaita.
3. **gtk4-rs examples/**: look up targeted examples for specific topics (drag-drop, clipboard, GLArea).
4. **Relm4**: adopt if the project's state complexity justifies the extra abstraction layer; evaluate
   `AsyncComponent` before implementing manual bridges.
5. **Amberol**: reference for simple GStreamer integration + dynamic CSS.
6. **Authenticator**: reference for Secret Service and `adw::PreferencesWindow` composition.
7. **Fractal**: reference for apps with complex state, heavy async, DBus portals, and multiple navigation screens.
8. **Loupe**: reference for custom rendering, gestures, and advanced sandboxing.
