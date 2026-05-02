---
name: verify:design-sync
version: 1.1.0
description: >
  Compara o design de referência (Doca.html + JSX sources extraídos de Doca.zip) com a
  implementação GTK4/Adwaita do projeto. Produz relatório bidirecional de gaps em
  .claude/docs/reports/. Invocar sempre que Doca.zip for atualizado. Não faz alterações
  no código — apenas análise e relatório.
---

# verify:design-sync

Invoke with `/verify:design-sync` whenever `Doca.zip` is replaced with a new Claude Design export.

Esta skill é **somente leitura** — não modifica nenhum arquivo do projeto além de gravar
o relatório em `.claude/docs/reports/`.

## When to use

- Sempre que `Doca.zip` for atualizado com um novo export do Claude Design
- Antes de iniciar um ciclo de implementação, para identificar o backlog correto
- Para detectar features já implementadas no GTK que o Claude Design ainda não conhece

## Input sources

| Arquivo no zip | Papel |
|---------------|-------|
| `Doca.html` | **Fonte primária** — shell HTML que define o entry point visual |
| `doca-app.jsx` | Views de nível top (App, Dashboard, Sidebar, rows, RuntimeSwitcher) |
| `doca-features.jsx` | Detail views (ContainerDetail, ImageDetail, VolumeDetail, NetworkDetail, EmptyStates) |
| `doca-dialogs.jsx` | Dialogs modais (Confirm, PullImage, CreateContainer, CreateVolume, CreateNetwork, Preferences, About) |
| `doca-core.jsx` | Componentes atômicos (Ico, StatusBadge, PR, Spinner) e mock data |
| `tweaks-panel.jsx` | Painel de tweaks de UI (opcional, ler se presente) |
| `REFACTOR_PROMPT.md` | Análise v1→v2 estruturada — leitura de alta qualidade para bugs conhecidos |
| `Feature Gap Analysis.html` | Análise competitiva (contexto suplementar) |

> **Nota:** `Doca.html` é um shell que carrega os arquivos JSX como scripts externos.
> O código dos componentes está nos `.jsx` — ler `Doca.html` sozinho é insuficiente.
> Arquivos versionados (`Doca v1.html`, `Doca v2.html`) são rascunhos intermediários e
> devem ser **completamente ignorados**.

## Process

### Phase 1 — Verify and extract sources

1. Confirmar que `Doca.zip` existe na raiz do projeto:
   ```sh
   ls -la Doca.zip
   ```
   Se não existir: **abortar e informar o usuário** — "Doca.zip não encontrado na raiz do projeto. Por favor, exporte o design do Claude Design e salve como Doca.zip na raiz."

2. Listar conteúdo para confirmar presença dos arquivos esperados:
   ```sh
   unzip -l Doca.zip
   ```

3. Extrair e ler os arquivos-fonte (ignorar arquivos com "v1", "v2" ou sufixo de versão no nome):
   ```sh
   unzip -p Doca.zip Doca.html
   unzip -p Doca.zip doca-app.jsx
   unzip -p Doca.zip doca-features.jsx
   unzip -p Doca.zip doca-dialogs.jsx
   unzip -p Doca.zip doca-core.jsx
   # Opcionais — ler se presentes:
   unzip -p Doca.zip tweaks-panel.jsx 2>/dev/null || true
   unzip -p Doca.zip REFACTOR_PROMPT.md 2>/dev/null || true
   unzip -p Doca.zip 'Feature Gap Analysis.html' 2>/dev/null || true
   ```

4. Registrar a data de modificação do zip para cabeçalho do relatório:
   ```sh
   stat -f "%Sm" -t "%Y-%m-%d" Doca.zip   # macOS
   # ou: stat -c "%y" Doca.zip | cut -d' ' -f1  # Linux
   ```

5. Verificar se existe relatório anterior para comparação:
   ```sh
   ls -t .claude/docs/reports/design-sync-*.md 2>/dev/null | head -1
   ```

### Phase 2 — Parse design features

Ler os arquivos JSX (não apenas `Doca.html`) e catalogar features por tela.
Se `REFACTOR_PROMPT.md` estiver presente, usá-lo como fonte de alta qualidade para bugs conhecidos e diferenças v1→v2.

**Por tela — o que catalogar:**

| Tela | Features a identificar |
|------|------------------------|
| Dashboard | Cards de status (running/paused/stopped/errors), cards de recursos (images/volumes/networks), host resources (CPU/mem/disk), recent containers, recent events, comportamento de navegação ao clicar nos cards |
| Containers | Sidebar list + status badge + ações (start/stop/pause/unpause/restart/remove), compose grouping, search/filter entry, detail pane com abas (Info/Stats/Logs/Terminal/Inspect), sparklines, env vars masking |
| Images | Sidebar list + ações (remove, pull, run, push), detail pane (Info/Layers), pull dialog com progresso por layer |
| Volumes | Sidebar list + ações (create/remove), detalhe (driver, mountpoint, in-use indicator) |
| Networks | Sidebar list + ações (create/remove, guard para built-ins), detalhe (driver/subnet/gateway/containers) |
| Events | Stream de eventos (actor/action/timestamp), se presente como aba separada |

**Componentes globais:**
- Header bar: menu button, view switcher/tab bar, runtime switcher, refresh button, search
- RuntimeSwitcher: runtimes mostrados, comportamento de troca
- Preferences dialog: seções (General, Container Defaults, Danger Zone) e campos de cada seção
- About dialog
- Dialogs de criação: CreateContainer (steps), PullImage, CreateVolume, CreateNetwork
- ConfirmDialog para ações destrutivas
- Tweaks panel (se presente)
- AdwBanner para estados de erro/aviso

**Classificação de features:** para cada feature, marcar como:
- **Presente** — claramente visível na UI do design
- **Parcial no design** — esboçado mas sem comportamento completo definido
- **Stub no design** — placeholder sem implementação (ex: botão que não faz nada)

### Phase 3 — Catalog GTK codebase features

Percorrer o codebase GTK Rust e catalogar features implementadas.

**Views** (`src/window/views/`):
- Para cada `*_view.rs`: identificar operações via métodos públicos, GAction names, chamadas `spawn_driver_task`
- `containers_view.rs`: verificar logs follow-toggle, terminal/VTE, inspect JSON, env vars masking, sparklines, compose grouping, search/filter, restart action
- `dashboard_view.rs`: stat cards, host resources, recent containers/events, navegação ao clicar em cards
- `images_view.rs`: pull com progresso por layer, run, push (verificar se é stub)
- `volumes_view.rs`, `networks_view.rs`: create/remove, guards para recursos built-in

**Components** (`src/window/components/`): listar cada componente e função

**Dialogs**: buscar `adw::AlertDialog`, `adw::Dialog`, `confirm_dialog`, wizards de criação

**Preferences / Settings**:
- Ler `data/com.example.GtkCrossPlatform.gschema.xml` — chaves existentes
- Identificar `PreferencesWindow` e suas páginas/grupos

**Features transversais**:
- RuntimeSwitcher: `dynamic_driver.rs` + UI em `main_window.rs`
- AdwBanner: buscar por `adw::Banner` ou `adw_banner` no codebase
- Auto-refresh: buscar `glib::timeout_add` ou similar

**Classificação de features:** para cada feature do codebase, marcar como:
- **Implementada** — funciona completamente
- **Parcial** — estrutura existe (tab/botão/dialog) mas comportamento incompleto ou é stub
- **Ausente** — não existe

### Phase 4 — Bidirectional gap analysis

Cruzar as duas tabelas. Classificar cada gap:

**Gap A — Code → Design (design context gaps):**
Features **Implementadas** no GTK que **não aparecem** ou **aparecem incorretamente** em `Doca.html`.
Cada item é algo que o Claude Design deveria conhecer para não produzir designs conflitantes.

**Gap B — Design → Code (implementation backlog):**
Features **Presentes** no design que **Ausentes** no código GTK.
Priorizar: P1 (bloqueante para uso diário) > P2 (importante) > P3 (diferencial/nice-to-have).

**Gap C — Parciais em ambos (attention items):**
Features marcadas como **Parcial** em ambos os lados — precisam de verificação manual se estão
alinhadas no estado parcial ou se uma está mais avançada que a outra.

**Aligned:**
Features corretamente representadas em ambos os lados.

**Delta desde último relatório** (se existir relatório anterior):
Comparar as listas de gaps com o relatório anterior (`design-sync-*.md` mais recente) e registrar:
- Gaps resolvidos desde o último sync
- Novos gaps introduzidos desde o último sync

### Phase 5 — Write report

1. Garantir que o diretório existe:
   ```sh
   mkdir -p .claude/docs/reports/
   ```

2. Definir nome do arquivo com data de hoje. Se já existir um arquivo com o mesmo nome, adicionar sufixo `-2`, `-3`, etc.:
   ```
   .claude/docs/reports/design-sync-YYYY-MM-DD.md
   # ou .claude/docs/reports/design-sync-YYYY-MM-DD-2.md se o primeiro já existir
   ```

3. Gravar o relatório com a estrutura abaixo.

**Estrutura obrigatória:**

```markdown
# Design Sync Report — YYYY-MM-DD

## Summary

| Category | Count |
|---|---|
| Design features catalogued (Doca.html + JSX) | N |
| GTK code features catalogued | N |
| Gap A — Code features absent from design | N |
| Gap B — Design features absent from code | N |
| Gap C — Partial on both sides (attention) | N |
| Aligned features | N |

Source files analysed:
- Design: `Doca.zip` (modified YYYY-MM-DD) — `Doca.html`, `doca-app.jsx`, `doca-features.jsx`, `doca-dialogs.jsx`, `doca-core.jsx`[, `REFACTOR_PROMPT.md`]
- Code: list of .rs files read

## Delta since last sync
*(Omit esta seção se não houver relatório anterior)*

**Resolved since YYYY-MM-DD:** [list or "none"]
**New gaps introduced:** [list or "none"]

---

## Gap A — Code → Design (Design Context Gaps)

Features no GTK app que o Claude Design não reflete.
Cada item indica algo que deveria ser modelado no próximo export do Doca.zip.

| Feature | File | Description for the Design |
|---|---|---|

---

## Gap B — Design → Code (Implementation Backlog)

Features no design que precisam ser implementadas no GTK app.

| Feature | Screen in Design | Priority | Notes |
|---|---|---|---|

---

## Gap C — Partial on Both Sides (Attention Items)

Features parcialmente implementadas em ambos — verificar se o estado parcial está alinhado.

| Feature | Code Status | Design Status | Verdict |
|---|---|---|---|

---

## Aligned Features

| Feature | Code Location | Design Location |
|---|---|---|

---

## Recommendations

Top 3–5 ações ordenadas por impacto:
1. ...
2. ...
3. ...
```

4. Informar o path completo do relatório ao usuário ao finalizar.

## Output

- **Relatório gerado:** `.claude/docs/reports/design-sync-YYYY-MM-DD.md`
- **Nenhum arquivo de código modificado** — esta skill é somente leitura
- **Nenhum arquivo intermediário criado** — as tabelas internas das Phases 2 e 3 são mantidas em contexto, não gravadas em disco

## Completion criteria

- [ ] `Doca.zip` encontrado e conteúdo listado
- [ ] Arquivos JSX extraídos e lidos (não apenas `Doca.html`)
- [ ] Arquivos versionados (`v1`, `v2`) ignorados completamente
- [ ] Design catalogado: ≥ 30 features (zip completo esperado)
- [ ] Código catalogado: todas as views + components + dialogs cobertos
- [ ] Gap A não vazio (features avançadas do GTK tipicamente ausentes do design)
- [ ] Gap B com prioridade P1/P2/P3 atribuída
- [ ] Gap C presente quando existem features parciais em ambos
- [ ] Delta incluído se relatório anterior existir
- [ ] Relatório salvo com nome correto (sem sobrescrever arquivo existente)
- [ ] Path do relatório informado ao usuário
