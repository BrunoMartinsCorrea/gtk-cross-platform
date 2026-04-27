# GTK in Rust — Reference Projects & Example Topics

O ecossistema Rust/GTK é centrado no crate `gtk4-rs`, que provê bindings safe gerados via GObject-Introspection para GTK
4, GLib, GIO e Pango. Desde 2023 o stack atingiu maturidade suficiente para produção: aplicações como Loupe, Fractal e
Authenticator fazem parte do GNOME Core/Circle e demonstram que padrões idiomáticos estão consolidados. Ainda assim, a
documentação de alto nível é esparsa — os projetos abaixo são as fontes primárias que a comunidade usa como referência
real.

---

## 1. gtk4-rs (Bindings Oficiais)

- **Repositório:** https://github.com/gtk-rs/gtk4-rs
- **Tipo:** official bindings
- **GTK version:** GTK4
- **Temas cobertos:**
    - GObject subclassing (`#[glib::object_subclass]`, `glib::wrapper!`): presente em `examples/` e nas listagens do
      livro como ponto de entrada canônico para widgets customizados.
    - GObject properties via derive macro (`#[derive(Properties)]`, `#[property(get, set)]`): o livro dedica um capítulo
      inteiro com progressão de `Cell<i32>` até bindings bidirecionais entre propriedades.
    - Signal handling e custom signals (`glib::subclass::Signal`, `OnceLock`): demonstrado na progressão do CustomButton
      com sinais como `max-number-reached`.
    - State management com `RefCell` / `Cell` / `OnceCell`: padrão recorrente em todos os exemplos de subclasse;
      `RefCell` para estado mutável em `imp::*`, `OnceCell` para inicialização única de `Settings`.
    - Composite templates (`.ui` + `#[derive(CompositeTemplate)]`, `#[template_child]`): exemplo `composite_template/`
      no repo; tutorial Todo usa template para `Window` e `TaskRow`.
    - `GListModel` + list views + factory pattern (`gio::ListStore`, `ListView`, `SignalListItemFactory`,
      `NoSelection`): tutorial Todo evolui de `ListBox` simples até `ListView` com factory e widget recycling.
    - `GSettings` integration (`gio::Settings` com schema compilado): integrado na versão Todo 2 para persistir a cor do
      filtro ativo entre sessões.
    - `gio::SimpleAction` + `install_action` / `install_action_async`: padrão de actions do menu e atalhos de teclado no
      tutorial Todo.
    - CSS styling (`CssProvider`, `StyleContext::add_provider_for_display`): exemplo `css/` e uso em tutorial para
      alternar temas dinamicamente.
    - Async com `glib::MainContext::spawn_local`: exemplos de operações I/O não bloqueantes dentro do event loop do GLib
      sem threads.
    - Libadwaita (`adw::Application`, `adw::StyleManager`, `adw::ActionRow`): capítulo dedicado no livro migrando o Todo
      app para Adwaita com `adw::ApplicationWindow`.
    - Drag & drop (`DragSource`, `DropTarget` com `gdk::ContentProvider`): exemplo `drag_and_drop/` no repositório.
    - Clipboard API (`gdk::Clipboard`, `widget.clipboard()`, `read_text_async`): exemplo `clipboard/` no repositório.
    - Custom drawing (`DrawingArea` + `set_draw_func`, snapshot com `gtk::Snapshot`): exemplo `custom_paintable/`.
    - OpenGL via `GlArea`: exemplo `glium_gl_area/` usando glium para renderização custom.
    - Video playback (`gtk::Video`, `gtk::MediaFile`): exemplo `video_player/`.
    - `GtkExpression` e property bindings declarativos: demonstrado em capítulo específico do livro para binding
      bidirecional sem código imperativo.

---

## 2. GUI Development with Rust and GTK 4 (Book + Listings)

- **Repositório:** https://github.com/gtk-rs/gtk4-rs (diretório `book/`)
- **Documentação:** https://gtk-rs.org/gtk4-rs/stable/latest/book/
- **Tipo:** tutorial oficial (livro + listagens executáveis)
- **GTK version:** GTK4
- **Temas cobertos:**
    - Widget lifecycle completo: do `Application::connect_activate` até `ApplicationWindow::present`, cobrindo
      realize/map/show.
    - GObject memory model: `glib::clone!` macro para weak references em closures de signal, evitando ciclos de
      referência.
    - Interior mutability pattern: progressão de `Cell` (Copy types) → `RefCell` (heap types) → `OnceCell` (init-once),
      com justificativa de cada escolha.
    - Composite templates com Blueprint (alternativa ao XML `.ui`): referenciado no livro como opção moderna onde o
      compilador `blueprint-compiler` emite `.ui`.
    - Saving/restoring window state via `GSettings`: o Todo app salva tarefas em JSON + preferências de filtro em
      `GSettings`.
    - `FilterListModel` + `CustomFilter` para listas filtradas reativamente sem recriar widgets.
    - Async actions com `install_action_async`: padrão para operações de I/O disparadas por menu sem bloquear o event
      loop.
    - Libadwaita `adw::ToastOverlay`, `adw::SwitchRow`, `adw::PreferencesWindow`: capítulo mostrando como substituir
      widgets GTK básicos pelos equivalentes Adwaita.
    - Persistência com `serde_json` + `gio::File`: serialização de estado da aplicação para XDG data dir.

---

## 3. Relm4

- **Repositório:** https://github.com/Relm4/Relm4
- **Documentação:** https://relm4.org/book/stable/
- **Tipo:** framework / library sobre gtk4-rs
- **GTK version:** GTK4
- **Temas cobertos:**
    - Elm-like Model-Update-View: cada componente implementa `SimpleComponent` ou `Component` com `type Input`,
      `type Output`, e `update()` puro.
    - Component hierarchy com typed messages: `ComponentSender` para comunicação pai → filho e filho → pai via `Output`,
      eliminando closures compartilhadas.
    - `AsyncComponent` para operações assíncronas: `update_async()` roda em tokio runtime separado; o resultado é
      enviado de volta via `sender.input()` para o event loop do GLib.
    - Factory pattern para listas (`FactoryVecDeque`, `FactoryHashMap`): alternativa mais ergonômica ao `GListModel` +
      `ListItemFactory` do GTK puro.
    - `RelmWorker` / background threads: padrão para CPU-bound tasks sem bloquear a UI.
    - Widget macros declarativas (`view!`, `#[relm4::component]`): reduz drasticamente boilerplate de criação e conexão
      de widgets.
    - `relm4-components`: componentes reutilizáveis prontos (file chooser dialog, alert dialog, spinning spinner)
      demonstrando o padrão de biblioteca de componentes.
    - Typed `gio::SimpleAction` wrappers: `relm4::actions!` macro que tipifica actions evitando strings mágicas.
    - Libadwaita integration: `RelmApp` com `adw::Application`, suporte a `adw::*` widgets diretamente no `view!`.

---

## 4. Fractal (GNOME Matrix Client)

- **Repositório:** https://gitlab.gnome.org/GNOME/fractal
- **Tipo:** application (GNOME Circle)
- **GTK version:** GTK4 (reescrita completa em GTK4 + matrix-rust-sdk desde v5)
- **Temas cobertos:**
    - Async com `tokio` + bridge para `glib::MainContext`: a matrix-rust-sdk roda em runtime tokio; resultados são
      enviados ao thread GTK via `glib::MainContext::channel()` ou `spawn_from_within`.
    - `adw::NavigationView` + `adw::NavigationPage`: padrão de navegação multi-nível (lista de salas → sala → thread),
      referência para apps com hierarquia de telas.
    - Custom widgets complexos via subclassing: `MessageRow`, `RoomRow`, `MediaViewer` são GObjects com template,
      signals e properties próprios.
    - `GListModel` + factory para listas de alto volume: lista de mensagens e membros da sala usam `ListView` +
      `SignalListItemFactory` com recycling.
    - Secret Service via `oo7` crate: armazenamento seguro de tokens de sessão Matrix, desacoplado do GNOME
      Keyring/KWallet via abstração da Secret Service API.
    - i18n com `gettext-rs`: todas as strings UI passam por `gettext()` / `i18n()`, com arquivos `.po` gerenciados pelo
      GNOME Translation Platform.
    - Flatpak packaging: manifesto completo com Rust SDK extension, GNOME runtime, e
      `--talk-name=org.freedesktop.secrets` no sandbox.
    - DBus portal (`ashpd` crate): `FileChooserPortal` para seleção de arquivos dentro do sandbox Flatpak,
      `NotificationPortal` para notificações do sistema.
    - Property bindings reativos: estado de UI (ex: botão de envio desabilitado quando input vazio) via
      `bind_property().sync_create().build()`.
    - Drag & drop para upload de arquivos: `DropTarget` aceita `gdk::FileList` na área de composição de mensagem.

---

## 5. Shortwave (Internet Radio)

- **Repositório:** https://gitlab.gnome.org/World/Shortwave
- **Tipo:** application (GNOME Circle)
- **GTK version:** GTK4
- **Temas cobertos:**
    - GStreamer pipeline em Rust (`gstreamer-rs`): pipeline `playbin` para streaming de áudio HTTP, com manipulação de
      `gst::Bus` messages via `add_watch` no event loop do GLib.
    - `GSettings` para preferências persistentes: volume, codec preferido, histórico de estações — schema declarado em
      `.gschema.xml` compilado via Meson.
    - Async REST com `reqwest`: chamadas à RadioBrowser API para busca de estações; executadas via
      `glib::MainContext::spawn_local` com `reqwest` compilado com feature `rustls`.
    - MPRIS DBus interface (`mpris2-zbus` ou custom via `zbus`): expõe controles de playback ao sistema (media keys,
      applets de painel).
    - Adaptive UI com `libadwaita`: `adw::Breakpoint` + `adw::OverlaySplitView` para adaptar layout de desktop para
      mobile/narrow.
    - Custom GObject para modelo de estação: `Station` como GObject com properties (`name`, `url`, `favicon`) para
      binding direto com widgets de lista.
    - Composite templates: janela principal, dialog de configurações e row de estação definidos em `.ui` com
      `#[template_child]`.
    - Flatpak + Meson: manifesto `.json` com dependências de GStreamer plugins; build system Meson compilando recursos
      GLib, schemas e gettext.
    - i18n com gettext + GNOME Translation Platform: uso de `gettextrs` e `cargo-i18n`.

---

## 6. Amberol (Music Player)

- **Repositório:** https://gitlab.gnome.org/World/amberol
- **Tipo:** application (GNOME Circle)
- **GTK version:** GTK4
- **Temas cobertos:**
    - GStreamer pipeline avançado: `playbin3` com `GstPlay` de alto nível; gestão de posição/duração via
      `glib::timeout_add_local` polling o pipeline.
    - CSS theming dinâmico baseado em artwork: extração de cor dominante da capa de album via `glycin` / `gdk_pixbuf`;
      aplicação via `CssProvider` substituído dinamicamente para criar gradientes que combinam com o álbum atual.
    - `gdk::Clipboard` para copiar metadados: demonstra `clipboard().set_text()` e leitura assíncrona.
    - Drag & drop para enfileirar músicas: `DropTarget` com `gdk::FileList` na área principal, adicionando arquivos à
      `gio::ListStore` da fila.
    - `GListModel` + `GridView` para fila de reprodução: playlist como `gio::ListStore<SongObject>` com `GridView` e
      factory customizada.
    - MPRIS via DBus (`zbus`): integração com controles de mídia do sistema operacional.
    - `glib::spawn_local` para I/O async: leitura de metadados (ID3/Vorbis) de forma não bloqueante antes de adicionar à
      fila.
    - Libadwaita puro: `adw::Application`, `adw::Window` sem `HeaderBar` padrão, customização do `WindowHandle`.
    - Composite templates com binding de propriedades: rows da fila de música com labels vinculados às properties do
      `SongObject`.

---

## 7. GNOME Authenticator (2FA Generator)

- **Repositório:** https://gitlab.gnome.org/World/Authenticator
- **Tipo:** application (GNOME Circle)
- **GTK version:** GTK4
- **Temas cobertos:**
    - Secret Service API via `oo7` crate: armazenamento de segredos TOTP no GNOME Keyring / KWallet de forma
      transparente ao backend; padrão de referência para apps que precisam de credenciais seguras.
    - DBus com `zbus`: comunicação com o daemon `org.freedesktop.secrets` e integração com
      `org.freedesktop.portal.Camera` para scan de QR code.
    - GStreamer para câmera (QR scan): pipeline de câmera com `gstreamer-rs`, análise de frames para detecção de QR code
      via `zbar` ou `rqrr`.
    - `GSettings` para preferências: lock on idle, tema, algoritmo padrão de OTP.
    - Custom GObject para conta OTP: `Account` como GObject com properties (`issuer`, `label`, `algorithm`, `period`,
      `digits`) usadas para binding na UI e serialização.
    - `GListModel` + `ListView`: lista de contas com `SignalListItemFactory`, suporte a busca via `FilterListModel` +
      `StringFilter`.
    - Composite templates com `adw::PreferencesWindow`: pattern de janela de preferências dividida em grupos com
      `adw::PreferencesGroup` + `adw::ActionRow`.
    - i18n gettext completo + Flatpak: manifesto com `--talk-name=org.freedesktop.secrets`, `--device=all` para câmera.
    - TOTP/HOTP logic em Rust puro: geração dos códigos via `totp-rs` crate, desacoplada da UI como domínio testável
      isolado.

---

## 8. Loupe / Image Viewer (GNOME Core)

- **Repositório:** https://gitlab.gnome.org/GNOME/loupe
- **Tipo:** application (GNOME Core, substituiu Eye of GNOME no GNOME 45)
- **GTK version:** GTK4
- **Temas cobertos:**
    - Sandboxed image decoding via `glycin` crate: cada formato de imagem roda em subprocesso isolado; padrão de
      referência para integração de subprocess IPC dentro de Flatpak com `--talk-name` mínimo.
    - Custom rendering widget com `gtk::GLArea` + GPU tiling: renderização acelerada de imagens grandes via tiles, com
      lógica de viewport e zoom; subclasse de `gtk::Widget` com override de `snapshot()`.
    - Gesture handling completo: `GesturePinch` (zoom), `GestureDrag` (pan), `GestureSwipe` (navegação entre imagens),
      `GestureClick` (toggle UI) — demonstra a API de event controllers do GTK4 em profundidade.
    - Async com `glib::spawn_local` + `glycin::Loader`: carregamento de imagem é totalmente assíncrono; progressos
      parciais (thumbnails) são exibidos antes do full decode.
    - Property bindings para estado de zoom: zoom level, rotation e flip expostos como GObject properties; UI (botões,
      labels) vinculados via `bind_property()`.
    - Drag & drop para abrir arquivos: `DropTarget` na janela principal aceita `gdk::FileList`; também suporta abrir via
      portal file chooser.
    - Clipboard (`gdk::Clipboard`): cópia da imagem atual para área de transferência como `gdk::Texture`.
    - Metadata display via `glycin`: exibição de EXIF/XMP como `adw::PreferencesGroup` gerado dinamicamente a partir dos
      metadados retornados pelo loader.
    - i18n + Flatpak + Meson: manifesto com portal de câmera/tela, build com `cargo-build` via Meson.
    - `adw::ToolbarView` + gestos para UI imersiva: barra de ferramentas que some automaticamente (auto-hide) com
      reintegração ao retornar o cursor.

---

## Theme Coverage Matrix

| Tema                                              | gtk4-rs bindings | gtk4-rs book | Relm4 | Fractal | Shortwave | Amberol | Authenticator | Loupe |
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
| CSS theming dinâmico                              |        ✓         |      ✓       |   —   |    —    |     —     |    ✓    |       —       |   —   |
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
| Property bindings reativos                        |        ✓         |      ✓       |   —   |    ✓    |     —     |    ✓    |       —       |   ✓   |
| Sandboxed IPC (`glycin`)                          |        —         |      —       |   —   |    —    |     —     |    —    |       —       |   ✓   |

---

## Ordem de Leitura Recomendada

Para um developer iniciando um projeto GTK4/Rust, a progressão mais eficiente é:

1. **gtk4-rs book** (capítulos 1–6): GObject model, subclassing, properties, signals, composite templates, GSettings.
2. **gtk4-rs book** (capítulos Todo 1–6): progressão completa de uma app real com GListModel, factory, actions,
   libadwaita.
3. **gtk4-rs examples/**: buscar exemplos pontuais para temas específicos (drag-drop, clipboard, GLArea).
4. **Relm4**: adotar se a complexidade de estado do projeto justificar a camada extra de abstração; avaliar
   `AsyncComponent` antes de implementar bridges manuais.
5. **Amberol**: referência para integração GStreamer simples + CSS dinâmico.
6. **Authenticator**: referência para Secret Service e composição de `adw::PreferencesWindow`.
7. **Fractal**: referência para apps com estado complexo, async pesado, DBus portals e múltiplas telas de navegação.
8. **Loupe**: referência para rendering customizado, gestures e sandboxing avançado.
