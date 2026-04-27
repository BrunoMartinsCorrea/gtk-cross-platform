# Relatório de Melhorias Conceituais — gtk-cross-platform

**Data:** 2026-04-26  
**Base de análise:** `docs/gtk-sources.md` × estado atual da codebase

---

## Sumário executivo

O projeto implementa uma base sólida de arquitetura hexagonal com GTK4/Adwaita, mas deixa padrões idiomáticos
importantes não explorados. Três gaps concentram o maior risco técnico: listas gerenciadas manualmente em vez de
`GListModel`, ausência de `FilterListModel`/`SortListModel` para busca reativa, e ausência de GObjects de domínio
que permitam property bindings declarativos. Os demais gaps (portais, gestos avançados) têm impacto menor e são
opcionais para o escopo atual.

---

## 1. Substituição de ListBox manual por GListModel + SignalListItemFactory

**Severidade: alta**

**Estado atual:** `containers_view.rs`, `images_view.rs`, `volumes_view.rs` e `networks_view.rs` constroem suas
listas com `gtk4::ListBox` e chamadas imperativas de `append()` / `remove()` em cada ciclo de refresh. O helper
`list_factory.rs` existe mas não é usado com um `GListModel` real.

**Referências de boas práticas:**
- gtk4-rs book capítulo Todo (progressão de `ListBox` simples → `ListView` + `GListModel` + factory com recycling)
- Authenticator: `SignalListItemFactory` com `FilterListModel` + `StringFilter` para busca em tempo real
- Amberol: `GridView` + `gio::ListStore<SongObject>` para fila de reprodução com factory customizada

**Por que importa:**
- O padrão atual reconstrói todos os widgets a cada refresh, descartando e recriando rows para listas que podem
  ter centenas de containers/imagens.
- `ListView` com factory recicla widgets — apenas os rows visíveis existem no widget tree, reduzindo uso de
  memória e melhorando o tempo de scroll.
- `gio::ListStore` é o único store que permite conectar `FilterListModel` e `SortListModel` sem reescrever a
  lógica de filtragem.

**Migração sugerida:**

```
gio::ListStore<ContainerObject>         ← fonte de dados
    ↓
FilterListModel(CustomFilter)           ← busca por texto
    ↓
SortListModel(CustomSorter)             ← grouping por compose project
    ↓
NoSelection / SingleSelection
    ↓
ListView + SignalListItemFactory         ← widget recycling
```

**Impacto estimado:** refactoring de médio porte por view (~200–300 linhas por view); habilitador para os itens
2 e 3 abaixo.

---

## 2. GObject wrappers para domain models

**Severidade: alta (pré-requisito para itens 1 e 3)**

**Estado atual:** `Container`, `Image`, `Volume`, `Network` são structs Rust puras sem suporte a GObject. Isso
impede qualquer binding declarativo: `FilterListModel` exige que os itens do store implementem `glib::Object`.

**Referências de boas práticas:**
- Shortwave: `Station` como GObject com properties `name`, `url`, `favicon` para binding direto com widgets de lista
- Authenticator: `Account` como GObject com properties `issuer`, `label`, `algorithm` usadas para binding na UI
- Amberol: `SongObject` com properties para rows da fila de música

**Implementação idiomática:**

```rust
// src/window/objects/container_object.rs
mod imp {
    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::ContainerObject)]
    pub struct ContainerObject {
        #[property(get, set)] id: RefCell<String>,
        #[property(get, set)] name: RefCell<String>,
        #[property(get, set)] status: RefCell<String>,
        #[property(get, set)] image: RefCell<String>,
    }
}
glib::wrapper! {
    pub struct ContainerObject(ObjectSubclass<imp::ContainerObject>);
}
impl ContainerObject {
    pub fn from_domain(c: &Container) -> Self { … }
}
```

Com isso, `gio::ListStore::new::<ContainerObject>()` funciona nativamente e cada row pode fazer
`label.bind_property("label", &container_object, "name").sync_create().build()` sem callbacks manuais.

**Impacto estimado:** 4 novos arquivos (`container_object.rs`, `image_object.rs`, `volume_object.rs`,
`network_object.rs`); sem alteração na camada de domínio (structs puras continuam existindo).

---

## 3. FilterListModel + CustomFilter para busca reativa

**Severidade: alta**

**Estado atual:** `filter_containers()` em `container.rs` é uma função pura que retorna um `Vec` filtrado. A view
chama `repopulate()` a cada keystroke, reconstruindo toda a lista. O mesmo padrão existe em todas as views.

**Referências de boas práticas:**
- gtk4-rs book: `FilterListModel` + `CustomFilter` para listas filtradas reativamente sem recriar widgets
- Authenticator: `FilterListModel` + `StringFilter` com busca em tempo real

**Implementação idiomática (após item 2):**

```rust
let filter = gtk4::CustomFilter::new(glib::clone!(
    #[weak] search_entry,
    move |obj| {
        let query = search_entry.text();
        if query.is_empty() { return true; }
        let c = obj.downcast_ref::<ContainerObject>().unwrap();
        c.name().to_lowercase().contains(&query.to_lowercase())
            || c.image().to_lowercase().contains(&query.to_lowercase())
    }
));
let filter_model = gtk4::FilterListModel::new(Some(store.clone()), Some(filter.clone()));
search_entry.connect_changed(move |_| filter.changed(gtk4::FilterChange::Different));
```

**Benefício:** a `filter_containers()` no domínio permanece como lógica testável e pura; o `CustomFilter` a chama
sem recriar widgets. O `EmptyState` é exibido automaticamente quando `filter_model.n_items() == 0`.

---

## 4. Property bindings reativos para estado de UI

**Severidade: média**

**Estado atual:** estados de botão (enabled/disabled), visibilidade de painéis e indicadores de loading são
controlados por chamadas imperativas (`set_sensitive(false)`, `set_visible(true)`) espalhadas pelas views.

**Referências de boas práticas:**
- Loupe: zoom level, rotation e flip como GObject properties; botões vinculados via `bind_property()`
- Fractal: estado de UI (botão de envio desabilitado quando input vazio) via `bind_property().sync_create()`
- gtk4-rs book: `GtkExpression` e property bindings declarativos

**Melhorias de alto valor:**

```rust
// Botão "start" desabilitado quando container já está Running
container_object
    .bind_property("status", &start_btn, "sensitive")
    .transform_to(|_, status: String| Some(status != "running"))
    .sync_create()
    .build();

// Label de status sincronizado automaticamente
container_object
    .bind_property("status", &status_label, "label")
    .sync_create()
    .build();
```

Isso elimina a necessidade de atualizar manualmente cada widget quando o estado muda — o binding mantém a
sincronização automaticamente.

---

## 5. adw::NavigationView para navegação multi-nível dentro de detalhes

**Severidade: média**

**Estado atual:** detalhes de containers usam `gtk4::Stack` com troca manual de páginas para as abas
info/stats/inspect/logs/terminal. Não há hierarquia de navegação com histórico de back/forward.

**Referências de boas práticas:**
- Fractal: `adw::NavigationView` + `adw::NavigationPage` para navegação lista → detalhe → sub-detalhe (sala → thread)

**Aplicação no projeto:**

O painel de detalhes do container poderia usar `adw::NavigationView` para navegar de:
- Lista de containers → Detalhe → Inspect JSON → Editor de variáveis de ambiente

Isso daria navegação com back button automático e suporte a gestos de swipe-back no mobile, alinhando
com o HIG do GNOME.

**Quando aplicar:** somente se o painel de detalhes crescer para mais de 2 níveis de profundidade. Para o
estado atual (abas flat), `AdwTabView` ou `gtk4::Stack` são suficientes.

---

## 6. ashpd (XDG Portals) para operações de filesystem no Flatpak

**Severidade: média**

**Estado atual:** não há integração com portais XDG. Operações de filesystem (importar `docker-compose.yml`,
exportar logs) dentro do sandbox Flatpak falhariam silenciosamente ou exigiriam permissões amplas de filesystem.

**Referências de boas práticas:**
- Fractal: `FileChooserPortal` para seleção de arquivos dentro do sandbox Flatpak
- Loupe: portal de abertura de arquivos

**Implementação:**

```toml
# Cargo.toml
ashpd = "0.9"
```

```rust
use ashpd::desktop::file_chooser::{FileChooserProxy, OpenFileRequest};

async fn pick_compose_file() -> Option<PathBuf> {
    let proxy = FileChooserProxy::new().await.ok()?;
    let files = proxy.open_file(
        OpenFileRequest::default()
            .title("Open docker-compose.yml")
            .filter(FileFilter::new("YAML").mimetype("application/yaml"))
    ).await.ok()?;
    files.uris().first().map(|u| u.to_file_path().ok()).flatten()
}
```

Isso habilita importação de arquivos compose sem `--filesystem=home` no manifesto Flatpak.

---

## 7. GesturePinch + GestureSwipe para navegação mobile

**Severidade: baixa**

**Estado atual:** apenas `GestureLongPress` é implementado (para context menu em touch). Sem suporte a swipe
de navegação ou pinch-to-zoom.

**Referências de boas práticas:**
- Loupe: `GesturePinch` (zoom), `GestureDrag` (pan), `GestureSwipe` (navegação entre imagens)

**Aplicação no projeto:**

- `GestureSwipe` no `AdwNavigationSplitView` para deslizar do detalhe de volta à sidebar em mobile
- `GestureSwipe` na lista de containers para reveal de ações rápidas (swipe-to-reveal: start/stop)

**Nota:** O HIG do GNOME desencoraja swipe-to-delete em listas — preferir botões visíveis ou
`adw::AlertDialog`. O swipe-to-reveal de ações (não destructivas) é aceitável.

---

## 8. CustomSorter + SortListModel para ordenação interativa

**Severidade: baixa**

**Estado atual:** agrupamento por compose project é feito manualmente em `group_by_compose()`. Não há
ordenação por nome, status ou data na UI.

**Referências de boas práticas:**
- gtk4-rs book: `SortListModel` com `CustomSorter` para listas ordenadas reativamente

**Implementação sugerida (após item 1):**

```rust
let sorter = gtk4::CustomSorter::new(|a, b| {
    let a = a.downcast_ref::<ContainerObject>().unwrap();
    let b = b.downcast_ref::<ContainerObject>().unwrap();
    // Running primeiro, depois alphabético
    match (a.status().as_str(), b.status().as_str()) {
        ("running", "running") => a.name().cmp(&b.name()).into(),
        ("running", _) => gtk4::Ordering::Smaller,
        (_, "running") => gtk4::Ordering::Larger,
        _ => a.name().cmp(&b.name()).into(),
    }
});
let sort_model = gtk4::SortListModel::new(Some(filter_model), Some(sorter));
```

---

## Padrões não aplicáveis a este projeto

Os padrões a seguir são documentados em `gtk-sources.md` mas fora do escopo de um gerenciador de containers:

| Padrão | Motivo de exclusão |
|--------|-------------------|
| GStreamer pipeline | Sem playback de mídia no domínio |
| glycin (sandboxed image decoding) | Sem visualização de imagens no domínio |
| MPRIS DBus interface | Sem controles de mídia |
| oo7 / Secret Service | Containers não armazenam credenciais na keychain do usuário |
| GLArea / OpenGL | Sem renderização 3D necessária |
| adw::NavigationView (multi-level) | Hierarquia atual (split-view + tabs) é suficiente |

---

## Roadmap de prioridades

| # | Melhoria | Dependência | Esforço estimado |
|---|----------|-------------|-----------------|
| 1 | GObject wrappers para domain models | — | 1–2 dias |
| 2 | GListModel + SignalListItemFactory | Item 1 | 3–4 dias |
| 3 | FilterListModel + CustomFilter | Item 2 | 1 dia |
| 4 | Property bindings reativos | Item 1 | 2 dias |
| 5 | CustomSorter + SortListModel | Item 2 | 1 dia |
| 6 | ashpd para file chooser portal | — | 1 dia |
| 7 | adw::NavigationView em detalhes | — | 2 dias (opcional) |
| 8 | GestureSwipe para mobile | — | 1 dia (opcional) |

**Sequência recomendada:** 1 → 2 → 3 → 4 → 5 (pipeline coeso de 8–10 dias de trabalho focado).
Os itens 6–8 são independentes e podem ser feitos a qualquer momento.

---

## Impacto arquitetural consolidado

Migrar para `GListModel` + GObjects de domínio não é apenas uma otimização de performance — é uma mudança de
paradigma que elimina uma categoria inteira de bugs: toda vez que um refresh atualiza dados mas esquece de
atualizar um widget específico. Com bindings reativos, esse tipo de desincronização se torna impossível por
construção.

A camada de domínio (`src/core/`) permanece inalterada: `Container`, `Image`, `Volume`, `Network` continuam
como structs Rust puras e testáveis. Os GObject wrappers ficam exclusivamente em `src/window/objects/` — uma
camada de tradução entre domínio e UI, alinhada com o princípio hexagonal existente.
