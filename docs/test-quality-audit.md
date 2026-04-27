# Laudo de Qualidade — Camada de Testes

**Data:** 2026-04-26
**Auditor:** Claude Code (externo)
**Escopo:** Todos os módulos `#[cfg(test)]` em `src/` e todos os arquivos em `tests/`
**Versão auditada:** `dce5d84`

---

## Sumário Executivo

| Dimensão | Estado | Severidade Máxima |
|----------|--------|-------------------|
| Pirâmide de testes | Invertida — mais integração que unitário | MÉDIO |
| Nomenclatura | `test_` prefix em 5 arquivos; 2 nomes sem condição | BAIXO |
| Qualidade das asserções | 1 tautologia + 4 dead assignments + 4 pattern `format!+contains` | CRÍTICO |
| Fidelidade do mock | 6 operações ignoram ID/estado; system_df inconsistente | ALTO |
| Duplicação de fixtures | 3 factory functions copy-paste em 7 arquivos | MÉDIO |
| Pureza de camada | 2 testes de domínio colocados na camada UI | ALTO |

---

## Estatísticas

- Testes unitários inline (`src/`): **74** (grep `#[test]`); **68** executam no runner lib
- Testes de integração (`tests/`, sem widget): **108** executáveis
- Testes E2E/widget (`#[ignore]`): **5**
- **Total: 187**
- Razão unit:integration: **74:108 ≈ 0,7:1** (pirâmide invertida — esperado > 1:1)

> Os 6 testes ausentes no runner lib (`status_badge.rs` × 2 e `window/utils/format.rs` × 4)
> residem na camada UI e não são compilados separadamente neste perfil de testes.

---

## Achados por Severidade

### CRÍTICO

#### AC-01 — Tautologia lógica — `test_exec_empty_command_handled_gracefully`

**Arquivo:** `tests/terminal_test.rs`

**Problema:** O ramo `Ok(s)` contém a asserção:

```rust
assert!(s.is_empty() || !s.is_empty(), "no panic is the contract")
```

A expressão `x || !x` é identicamente verdadeira para qualquer valor de `x` — nenhuma
alteração no código de produção pode fazer essa asserção falhar. O único contrato exercitado
é "a chamada não entrou em pânico", mas a linha `.expect()` na chamada exterior já cobre
isso; o bloco `Ok(s)` não acrescenta nada.

**Impacto:** Qualquer regressão no tratamento de `exec_in_container` com comando vazio
(retornar um valor não-vazio, retornar um erro diferente) passa silenciosamente.

**Correção:** Substituir por uma asserção sobre o conteúdo real:

```rust
Ok(s) => assert!(s.is_empty(), "empty command should yield empty output, got: {s:?}")
```

---

#### AC-02 — Dead assignments — `test_prune_system_returns_report`

**Arquivo:** `tests/dashboard_test.rs`

**Problema:** Todas as "asserções" são atribuições mortas com `let _ = expr`:

```rust
let _ = report.containers_deleted.len();
let _ = report.images_deleted.len();
let _ = report.volumes_deleted.len();
let _ = report.space_reclaimed;
```

`let _ = expr` avalia a expressão (útil para suprimir warnings de não-uso) mas não verifica
nada sobre o valor. O único contrato real é "a chamada não retornou `Err`", coberto pelo
`.expect("prune")` anterior.

**Impacto:** Se `prune_system` passasse a retornar um `PruneReport` com campo `containers_deleted`
vazio quando deveria conter containers, o teste continuaria verde.

**Correção:** Substituir por asserções reais sobre o estado esperado do mock:

```rust
// O mock declara explicitamente que "112233445566" é excluído no prune
assert_eq!(report.containers_deleted.len(), 1);
assert!(report.containers_deleted.contains(&"112233445566".to_string()));
assert!(report.images_deleted.is_empty());
assert_eq!(report.space_reclaimed, 0);
```

---

### ALTO

#### AA-01 — Mock não modela falhas — 6 operações sempre retornam `Ok(())`

**Arquivo:** `src/infrastructure/containers/mock_driver.rs`

**Operações afetadas:**

| Operação | Comportamento real que o mock ignora |
|----------|--------------------------------------|
| `remove_volume(_name, _force)` | Volume inexistente → `NotFound` |
| `remove_network(_id)` | Rede inexistente → `NotFound` |
| `remove_image(_id, _force)` | Imagem inexistente → `NotFound` |
| `restart_container(_id, _timeout)` | Container inexistente → `NotFound` |
| `pause_container(_id)` | Container parado → `NotRunning` |
| `unpause_container(_id)` | Container não pausado → `NotRunning` |

**Problema:** Qualquer teste que chame `remove_image("id-que-nao-existe")` esperando
`ContainerError::NotFound` passa silenciosamente porque o mock retorna `Ok(())`. Mais
importante: nenhum teste de caminho de erro existe para essas operações porque o mock
nunca falha, tornando os testes de mapeamento de erro nas views completamente sem valor.

**Impacto:** Todo o tratamento de erro das seis operações nas camadas de use case e view
permanece sem cobertura. Regressões nesse código não são detectadas.

**Correção:** Adicionar verificação de existência/estado no mock antes de retornar `Ok`:

```rust
fn remove_image(&self, id: &str, _force: bool) -> Result<(), ContainerError> {
    self.inspect_image(id)?; // retorna NotFound se não existe
    Ok(())
}

fn pause_container(&self, id: &str) -> Result<(), ContainerError> {
    let short = id.chars().take(12).collect::<String>();
    if !self.running.lock().unwrap().contains(&short) {
        return Err(ContainerError::NotRunning(id.to_string()));
    }
    Ok(())
}
```

---

#### AA-02 — Cobertura ausente de transições de estado inválidas

**Arquivos afetados:** todos os arquivos de teste de ciclo de vida de container

**Problema:** O domínio modela `ContainerStatus` com 7 variantes e um ciclo de vida implícito.
Os testes cobrem somente as transições felizes. As seguintes transições nunca são exercitadas:

| Transição | Resultado esperado | Teste existente |
|-----------|-------------------|-----------------|
| `pause(id_stopped)` | `ContainerError::NotRunning` | Não |
| `unpause(id_not_paused)` | `ContainerError::NotRunning` | Não |
| `start(id_already_running)` | idempotente ou `AlreadyExists` | Não |
| `remove(id_running, force=false)` | erro ou sucesso por design? | Não |
| `restart(id_nonexistent)` | `ContainerError::NotFound` | Não |
| `pause/unpause` round-trip | estado volta a Running | Não |

A variante `ContainerStatus::Paused` existe no domínio, está coberta por testes de labels e
CSS, mas **nunca é exercitada via driver** — nenhum teste chama `pause_container` e depois
verifica o estado.

**Impacto:** A semântica de estados inválidos é completamente sem cobertura. Regressões
(ex: pausar um container já parado deixa-o em estado inconsistente) não são detectadas.

---

#### AA-03 — Violação de pureza de camada — testes de domínio em `status_badge.rs`

**Arquivo:** `src/window/components/status_badge.rs`

**Problema:** O módulo `#[cfg(test)]` em `status_badge.rs` importa e testa exclusivamente
`ContainerStatus` (camada de domínio), sem tocar nenhum widget GTK:

```rust
// Em src/window/components/status_badge.rs — camada UI
#[cfg(test)]
mod tests {
    use gtk_cross_platform::core::domain::container::ContainerStatus;

    #[test]
    fn css_class_matches_domain() { ... }   // testa ContainerStatus::css_class()
    #[test]
    fn label_is_non_empty_for_all_variants() { ... }  // testa ContainerStatus::label()
}
```

Esses dois testes são duplicatas exatas de `status_css_classes` e `status_labels` em
`src/core/domain/container.rs`. A localização viola a regra de pureza: testes de domínio
pertencem ao módulo de domínio, não à camada UI.

**Impacto:** Custo de manutenção dobrado; risco de divergência silenciosa se `container.rs`
atualizar os valores sem atualizar `status_badge.rs`.

**Correção:** Remover os dois testes de `status_badge.rs`. Os cenários já estão cobertos em
`container.rs` com maior detalhe.

---

### MÉDIO

#### AM-01 — Duplicação de fixture — factory functions copy-paste em 7 arquivos

**Problema:** Três factory functions com corpos idênticos ou equivalentes são copiadas
em múltiplos arquivos de teste:

| Função | Arquivos | Corpo |
|--------|----------|-------|
| `fn container_uc()` | `container_stats_test.rs`, `inspect_test.rs`, `create_container_test.rs` | `ContainerUseCase::new(Arc::new(MockContainerDriver::new()))` |
| `fn container_uc()` (retorno `impl`) | `container_driver_test.rs` | corpo idêntico, tipo de retorno diferente |
| `fn driver()` | `pull_image_streaming_test.rs`, `container_logs_test.rs`, `terminal_test.rs` | `Arc::new(MockContainerDriver::new())` |
| `fn use_case()` (NetworkUseCase) | `system_events_test.rs`, `compose_lifecycle_test.rs` | corpo idêntico |

Qualquer mudança na assinatura de `ContainerUseCase::new` ou `MockContainerDriver::new`
exige atualização manual em 4–5 locais.

**Correção:** Criar módulo `tests/support/mod.rs` (ou `tests/support/factories.rs`) com
as factories compartilhadas e importar em cada arquivo de teste.

---

#### AM-02 — `make_container` com duas implementações incompatíveis

**Arquivos:**
- `tests/search_filter_test.rs`: `fn make_container(name, image, short_id, compose)` — 4 parâmetros
- `tests/compose_grouping_test.rs`: `fn make_container(name, compose)` — 2 parâmetros

Ambas constroem o mesmo tipo `Container` com assinaturas diferentes. À medida que o domínio
`Container` evolui (ex: novos campos obrigatórios), as duas implementações divergirão
silenciosamente.

A terceira implementação em `src/infrastructure/containers/mock_driver.rs` usa 6 parâmetros,
e `src/core/domain/container.rs` tem `fn make_test_container` com 4 parâmetros compatíveis
com `search_filter_test.rs`.

**Correção:** Consolidar em um único `ContainerBuilder` fluente compartilhado em
`tests/support/builders.rs`, eliminando as quatro implementações divergentes.

---

#### AM-03 — Redundância inline vs. integração — 7+ cenários duplicados

Os testes inline em `src/` duplicam exatamente cenários já cobertos em `tests/`:

| Teste inline (`src/`) | Duplicado em (`tests/`) |
|-----------------------|-------------------------|
| `container_use_case.rs::list_all_returns_all` | `container_driver_test.rs::list_containers_all_returns_all` |
| `container_use_case.rs::list_running_only` | `container_driver_test.rs::list_containers_running_only` |
| `greet_use_case.rs::returns_greeting` | `greet_use_case_test.rs::returns_greeting` (nome idêntico) |
| `image_use_case.rs::list_returns_images` | `container_driver_test.rs::list_images_returns_images` |
| `network_use_case.rs::list_networks_returns_two` | `container_driver_test.rs::list_networks_returns_two` |
| `network_use_case.rs::prune_system_returns_report` | `container_driver_test.rs::prune_system_returns_report` |
| `volume_use_case.rs::list_returns_volumes` | `container_driver_test.rs::list_volumes_returns_volumes` |

Adicionalmente, os 5 testes de `filter_containers` em `container.rs` duplicam o núcleo de
`search_filter_test.rs`, e os 3 testes de `group_by_compose` em `container.rs` duplicam
parte de `compose_grouping_test.rs`.

**Impacto:** Dobro do custo de manutenção sem aumento de cobertura.

**Correção:** Remover os 7 testes inline duplicados. Os testes de integração exercitam a
API pública de forma mais rigorosa.

---

#### AM-04 — God test — `start_container_makes_it_running`

**Arquivo:** `tests/container_driver_test.rs`

**Problema:** Um único teste encadeia 4 operações distintas:

```rust
fn start_container_makes_it_running() {
    let uc = container_uc();
    uc.stop("aabbccdd1122", None).expect("stop");    // operação 1
    let before = uc.list(false).expect("list").len(); // asserção 1
    assert_eq!(before, 0);
    uc.start("aabbccdd1122").expect("start");         // operação 2
    let after = uc.list(false).expect("list").len();  // asserção 2
    assert_eq!(after, 1);
}
```

Quando falha, não é possível saber imediatamente se o problema está em `stop`, `list`,
`start`, ou no estado inicial do mock.

O mesmo padrão aparece em `container_use_case.rs::start_stopped_container_makes_it_running`.

**Correção:** Fragmentar em dois testes focados: `stop_running_container_removes_from_running_list`
e `start_stopped_container_adds_to_running_list`.

---

#### AM-05 — Pattern frágil `format!() + contains()` para asserções de erro

**Arquivos afetados e testes:**

| Arquivo | Teste |
|---------|-------|
| `tests/container_stats_test.rs` | `stats_for_stopped_container_returns_not_running_error` |
| `tests/inspect_test.rs` | `inspect_json_unknown_id_returns_not_found` |
| `tests/create_container_test.rs` | `create_container_unknown_image_returns_not_found` |
| `tests/create_container_test.rs` | `create_container_name_conflict_returns_already_exists` |

Exemplo do antipadrão:

```rust
let msg = format!("{}", result.unwrap_err());
assert!(msg.contains("Not found") || msg.contains("not found"), "...");
```

Verificar a string formatada acopla o teste à mensagem de texto do erro, não à variante
do tipo. Renomear a mensagem de exibição quebra o teste sem que o contrato tenha mudado.

**Correção:** Usar `matches!` diretamente:

```rust
assert!(matches!(result, Err(ContainerError::NotFound(_))), "...");
```

---

#### AM-06 — Asserção inútil com comparação sempre verdadeira — `stats_for_running_container_returns_values`

**Arquivo:** `tests/container_stats_test.rs` (linhas 23–24)

**Problema:** O compilador emite dois warnings:

```
warning: comparison is useless due to type limits
  --> tests/container_stats_test.rs:23:13
23 |     assert!(stats.net_rx_bytes >= 0);
24 |     assert!(stats.net_tx_bytes >= 0);
```

`net_rx_bytes` e `net_tx_bytes` são `u64` — sempre `>= 0` por definição de tipo. O teste
nunca pode falhar independentemente da implementação.

**Correção:** Substituir por uma asserção sobre um valor real, ex:

```rust
assert!(stats.net_rx_bytes <= 10_000_000, "mock net_rx unexpected: {}", stats.net_rx_bytes);
```

Ou verificar o valor exato do mock (`net_rx_bytes == 1024`).

---

#### AM-07 — `system_df()` retorna valores inconsistentes com o estado do mock

**Arquivo:** `src/infrastructure/containers/mock_driver.rs`, método `system_df`

**Problema:** O mock possui 3 containers (`web-server`, `db`, `standalone`) mas `system_df`
retorna `containers_total: 2`. O teste em `container_driver_test.rs::system_df_returns_usage`
asserta `containers_total == 2`, que passa porque está ancorado no valor hardcoded, não no
estado real. Um colaborador que adicionar um 4° container ao mock não saberá que `system_df`
precisa de atualização correspondente.

O mock de `system_df` também reporta `images_total: 2` quando `list_images` retorna 3
imagens (incluindo a dangling `sha256:cccc`).

**Impacto:** Testes que dependem de `system_df` não detectam drift entre o estado do mock
e os valores reportados, criando confiança falsa na consistência do dashboard.

---

### BAIXO

#### AB-01 — Prefixo `test_` redundante em 5 arquivos

O atributo `#[test]` já distingue funções de teste das demais. O prefixo `test_` no nome
da função é ruído que polui o output do test runner sem adicionar semântica.

| Arquivo | Quantidade de funções com prefixo `test_` |
|---------|------------------------------------------|
| `tests/terminal_test.rs` | 4 |
| `tests/pull_image_streaming_test.rs` | 4 |
| `tests/dashboard_test.rs` | 3 |
| `tests/runtime_switcher_test.rs` | 3 |
| `tests/container_logs_test.rs` | 4 |

**Total:** 18 funções com prefixo desnecessário.

---

#### AB-02 — Nomes de teste sem condição explícita

| Teste | Problema | Sugestão |
|-------|----------|----------|
| `events_returns_list` | Sem condição — quando? com que filtro? | `events_with_no_filter_returns_all_events` |
| `layers_have_id_cmd_and_size` | Sem sujeito nem cenário | `layers_for_known_image_have_populated_fields` |
| `list_containers_all_returns_all` | Redundante ("all returns all") | `list_with_all_flag_includes_stopped_containers` |

---

#### AB-03 — Oportunidade de parametrização — 12 testes reduzíveis a 2 tabelas

**`is_secret_env_key`:** 7 funções em `env_masking_test.rs` + 5 funções duplicadas em
`container.rs` = 12 funções testando a mesma lógica com valores diferentes. Colapsáveis
em uma tabela `(key, expected_bool)` com 10 casos.

**`ContainerStatus::from_state`:** 4 funções em `container.rs` que diferem apenas nos
argumentos. Colapsáveis em uma tabela `(state_str, exit_code, expected_variant)`.

---

#### AB-04 — Ausência de doc comment em 2 arquivos de teste

`tests/compose_lifecycle_test.rs` e `tests/system_events_test.rs` não possuem comentário
`//!` descrevendo a feature coberta, ao contrário dos demais arquivos de integração que
seguem o padrão estabelecido (ex: `terminal_test.rs`, `dashboard_test.rs`).

---

## Oportunidades de Abstração

### Object Mother — constantes de IDs do mock

O ID `"aabbccdd1122334455667788"` aparece como literal em pelo menos 7 arquivos de teste,
sempre representando o container "web-server" running. O mesmo vale para outros IDs do mock.

```rust
// tests/support/fixtures.rs — estrutura proposta
pub const RUNNING_CONTAINER_ID: &str = "aabbccdd1122334455667788"; // web-server, nginx:latest
pub const STOPPED_CONTAINER_ID: &str = "112233445566778899aabbcc"; // db, postgres:15, Exited
pub const STANDALONE_CONTAINER_ID: &str = "223344556677889900aabbcc"; // standalone, redis
pub const UNKNOWN_CONTAINER_ID: &str = "nonexistentid0000000000";
pub const MOCK_CONTAINERS_TOTAL: usize = 3;
pub const MOCK_RUNNING_CONTAINERS: usize = 1;
```

Duplicação eliminada: ~30 literais distribuídos em 7 arquivos → 6 constantes nomeadas.

### Fixture compartilhada — factories de use case

```rust
// tests/support/factories.rs — estrutura proposta
pub fn container_uc() -> ContainerUseCase { ... }
pub fn image_uc() -> ImageUseCase { ... }
pub fn network_uc() -> NetworkUseCase { ... }
pub fn mock_driver() -> Arc<MockContainerDriver> { ... }
```

Duplicação eliminada: 4 declarações de `container_uc`, 3 de `fn driver()`, 2 de `fn use_case()`.

### Test Data Builder — Container

```rust
// tests/support/builders.rs — substitui as 4 implementações de make_container
ContainerBuilder::default()
    .name("nginx-proxy")
    .image("nginx:latest")
    .short_id("aabbccdd1122")
    .compose_project("web-stack")
    .status(ContainerStatus::Running)
    .build()
```

Duplicação eliminada: 4 implementações incompatíveis de `make_container` → 1 builder.

### Macro `assert_error_variant!`

```rust
// tests/support/mod.rs
macro_rules! assert_error_variant {
    ($result:expr, $variant:pat) => {
        assert!(matches!($result, Err($variant)), "expected {}, got {:?}", stringify!($variant), $result)
    }
}
// Uso:
assert_error_variant!(result, ContainerError::NotFound(_));
assert_error_variant!(result, ContainerError::NotRunning(_));
```

Duplicação eliminada: padrão `format!()+contains()` em 4 testes → verificação estrutural.

---

## Checklist de Ações Corretivas

> Marque cada item com `[x]` após aplicar a correção. Execute `make test` ao final.

### Ações Imediatas (Crítico / Alto)

- [ ] Substituir asserção tautológica em `test_exec_empty_command_handled_gracefully` por `assert!(s.is_empty(), ...)`
- [ ] Substituir 4 dead assignments em `test_prune_system_returns_report` por `assert_eq!` sobre campos específicos
- [ ] Adicionar verificação de existência de recurso em `remove_volume`, `remove_network`, `remove_image` no mock
- [ ] Adicionar verificação de estado em `restart_container`, `pause_container`, `unpause_container` no mock
- [ ] Remover os 2 testes de `status_badge.rs` (duplicatas de `container.rs`) — violação de camada
- [ ] Adicionar testes de transição inválida: `pause(Stopped)`, `start(Running)`, `unpause(not_paused)`, `restart(nonexistent)`
- [ ] Corrigir comparações inúteis `net_rx_bytes >= 0` e `net_tx_bytes >= 0` em `stats_for_running_container_returns_values`

### Ações de Manutenibilidade (Médio)

- [ ] Criar `tests/support/fixtures.rs` com `RUNNING_CONTAINER_ID`, `STOPPED_CONTAINER_ID`, `UNKNOWN_CONTAINER_ID` e constantes numéricas do mock
- [ ] Criar `tests/support/factories.rs` com `container_uc()`, `image_uc()`, `network_uc()`, `mock_driver()` compartilhados
- [ ] Eliminar declarações locais duplicadas de `fn container_uc()` em `container_stats_test.rs`, `inspect_test.rs`, `create_container_test.rs`
- [ ] Eliminar declarações locais duplicadas de `fn driver()` em `pull_image_streaming_test.rs`, `container_logs_test.rs`, `terminal_test.rs`
- [ ] Eliminar declarações locais de `fn use_case()` em `system_events_test.rs` e `compose_lifecycle_test.rs`
- [ ] Consolidar `make_container` de `search_filter_test.rs` e `compose_grouping_test.rs` em `ContainerBuilder` compartilhado
- [ ] Substituir padrão `format!()+contains()` por `matches!()` em 4 testes de erro (`container_stats_test`, `inspect_test`, `create_container_test` × 2)
- [ ] Fragmentar `start_container_makes_it_running` em `stop_running_container_removes_from_running_list` e `start_stopped_container_adds_to_running_list`
- [ ] Remover 7 testes inline duplicados (listados na tabela de redundância) dos use cases
- [ ] Corrigir `system_df()` no mock para refletir o estado real (3 containers, 3 images)
- [ ] Adicionar `// tests/support/mod.rs` com macro `assert_error_variant!`

### Melhorias de Nomenclatura (Baixo)

- [ ] Remover prefixo `test_` dos 18 testes em `terminal_test.rs`, `pull_image_streaming_test.rs`, `dashboard_test.rs`, `runtime_switcher_test.rs`, `container_logs_test.rs`
- [ ] Renomear `events_returns_list` → `events_with_no_filter_returns_all_events`
- [ ] Renomear `layers_have_id_cmd_and_size` → `layers_for_known_image_have_populated_fields`
- [ ] Parametrizar 7 funções de `is_secret_env_key` em `env_masking_test.rs` em tabela única
- [ ] Parametrizar 5 funções duplicadas de `is_secret_env_key` em `container.rs` (após remover as duplicatas da tabela AM-03)
- [ ] Parametrizar 4 funções de `ContainerStatus::from_state` em `container.rs` em tabela única
- [ ] Adicionar comentário `//!` em `compose_lifecycle_test.rs` e `system_events_test.rs`

---

## Resultado de `make test`

```
cargo test
warning: comparison is useless due to type limits
  --> tests/container_stats_test.rs:23:13
   |
23 |     assert!(stats.net_rx_bytes >= 0);
   = note: `#[warn(unused_comparisons)]` on by default

warning: comparison is useless due to type limits
  --> tests/container_stats_test.rs:24:13
24 |     assert!(stats.net_tx_bytes >= 0);

warning: `gtk-cross-platform` (test "container_stats_test") generated 2 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
     Running unittests src/lib.rs

running 68 tests
test result: ok. 68 passed; 0 failed; 0 ignored; 0 measured

[... 19 integration test binaries ...]

     Running tests/widget_test.rs
running 5 tests
test adw_action_row_title_and_subtitle ... ignored
test adw_status_page_title_property ... ignored
test gtk_list_box_accepts_action_rows ... ignored
test status_badge_css_class_applied ... ignored
test status_badge_stopped_css_class ... ignored
test result: ok. 0 passed; 0 failed; 5 ignored; 0 measured

   Doc-tests gtk_cross_platform
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured
```

**Status final:** PASSOU (com 2 warnings de comparação inútil em `container_stats_test.rs`)
