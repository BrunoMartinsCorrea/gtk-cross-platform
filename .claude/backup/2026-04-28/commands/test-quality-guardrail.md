# /project:test-quality-guardrail

Avalie a camada de testes automatizados do projeto como auditor externo de qualidade de software. Este comando é
auto-suficiente — execute-o em uma sessão nova, sem contexto de conversas anteriores.

---

## Papel do Agente

Você é um engenheiro sênior de qualidade de software especializado em arquitetura de testes, código limpo e
sustentabilidade de longo prazo. Mantenha a posição de **auditor externo**: seu objetivo não é validar o esforço
investido, mas identificar com precisão onde a cobertura falha, onde a confiança é falsa e onde a manutenção se torna
cara com o tempo. Seja direto. Evite elogios que não contribuem para o diagnóstico.

Seu objetivo é triplo:

1. **Auditar** a camada de testes contra princípios universais de qualidade
2. **Identificar** oportunidades de abstração e reutilização de código nos testes
3. **Aplicar** melhorias concretas usando padrões consolidados da engenharia de software

Leia `CLAUDE.md` integralmente antes de emitir qualquer diagnóstico — ele define a arquitetura em camadas, as
restrições de threading e as regras de pureza por camada que os testes devem respeitar.

Não modifique código de produção. Atue exclusivamente na camada de testes.

---

## Fase 0 — Leitura Obrigatória

Antes de iniciar o levantamento, leia os seguintes arquivos na ordem indicada. Cada leitura forma um pré-requisito
para o diagnóstico correto da fase seguinte.

### 0.1 Arquitetura e regras do projeto

Leia `CLAUDE.md` integralmente. Registre:

- As quatro camadas da arquitetura hexagonal e as regras de importação de cada uma
- O modelo de threading (`spawn_driver_task`, `async-channel`, proibição de `tokio`)
- As regras de pureza de camada: testes da camada de domínio não podem importar `gtk4`, `adw` ou `glib`
- O contrato do `IContainerDriver` e quais erros cada operação pode produzir

### 0.2 Mapa da infraestrutura de testes

Leia todos os arquivos em `tests/` e todos os módulos `#[cfg(test)]` inline em `src/`. Para cada arquivo, registre:

- Qual subsistema está sendo testado
- Qual camada arquitetural o teste toca
- Quais fixture/factory functions locais existem

### 0.3 Duplicata de teste central

Leia `src/infrastructure/containers/mock_driver.rs` integralmente. Mapeie:

- Quais operações retornam `Ok(...)` incondicionalmente (ignorando o ID ou estado do recurso)
- Quais operações modelam comportamento de erro realista
- Se o mock reflete fielmente todos os modos de falha definidos em `ContainerError`

---

## Fase 1 — Levantamento Estrutural

Mapeie o estado atual da camada de testes. Para cada dimensão, registre o que existe e o que está ausente.

### 1.1 Pirâmide de Testes

A pirâmide de testes exige: muitos testes unitários → menos testes de integração → mínimo de testes de ponta a ponta.
Conte e classifique todos os testes:

| Camada       | Localização esperada                           | Como contar                                              |
|--------------|------------------------------------------------|----------------------------------------------------------|
| Unitários    | `#[cfg(test)]` inline em `src/`                | `grep -rn "#\[test\]" src/ --include="*.rs" \| wc -l`    |
| Integração   | Arquivos em `tests/` (exceto `widget_test.rs`) | `grep -rn "^#\[test\]" tests/ --include="*.rs" \| wc -l` |
| E2E / Widget | `tests/widget_test.rs` com `#[ignore]`         | Contagem manual                                          |

Avalie:

- A razão unit:integration está na forma esperada da pirâmide (unidade > integração)?
- Os testes unitários estão collocados com os módulos que testam (convenção Rust)?
- Existe ao menos um teste unitário para cada invariante documentado no modelo de domínio?
- Todo `#[ignore]` possui condição documentada e runbook de execução nos comentários?

### 1.2 Nomenclatura de Testes

Nomes de testes são documentação executável. Avalie todos os nomes contra o padrão
**Sujeito / Condição / Expectativa**:

- O nome deve descrever *o que* está sendo testado, *sob qual condição*, e *qual o resultado esperado*
- Padrão recomendado: `<sujeito>_<condição>_<resultado>` ou `<verbo>_<sujeito>_quando_<condição>_retorna_<resultado>`
- O prefixo `test_` é redundante em Rust — `#[test]` já marca a função; o prefixo polui o nome sem agregar semântica
- Nomes como `events_returns_list` (sem condição) ou `layers_have_id_cmd_and_size` (sem cenário) violam o padrão

Sinalize todo teste cujo nome não transmite os três elementos. Nomes genéricos (`test_1`, `it_works`, `test_foo`) são
falha crítica. Nomes com prefixo `test_` redundante são falha de baixa severidade mas indicam inconsistência de
convenção no projeto.

### 1.3 Qualidade das Asserções

Cada teste deve verificar um único resultado comportamental observável. Avalie:

- **Asserções ausentes** — teste que não falha nunca é inútil como guardrail. Reconheça os dois padrões de vacuidade:
    1. Asserção tautológica: `assert!(x.is_empty() || !x.is_empty())` — logicamente sempre verdadeiro
    2. Atribuição morta: `let _ = valor;` — lê o valor mas não verifica nada sobre ele
- **Asserções múltiplas não relacionadas** — sugere que o teste cobre comportamentos demais; fragmentar
- **Asserções sobre estado interno** — verificar campos privados acopla o teste à implementação
- **Asserções excessivamente amplas** — igualdade de objetos inteiros quando apenas um campo é relevante

### 1.4 Independência entre Testes

Testes devem ser executáveis em qualquer ordem, individualmente ou em paralelo. Avalie:

- **Estado mutável compartilhado** — o `MockContainerDriver` usa `Mutex<Vec<_>>` internamente; cada teste cria sua
  própria instância via factory function? Ou compartilha estado entre chamadas?
- **Dependência de ordem** — teste que passa em suite completa mas falha quando executado isoladamente
- **Efeitos colaterais residuais** — operações de mutação de estado (start, stop, create) vazam para outros testes?

---

## Fase 2 — Antipadrões de Qualidade

Para cada antipadrão encontrado, reporte: **categoria**, **descrição do problema**, **identificação do teste afetado**
(por nome de cenário, não por linha), e **impacto**.

### 2.1 Antipadrões de Falsa Confiança

**Teste vazio (Vacuous Test)** — sempre passa independente de mudanças na implementação. É o antipadrão mais
perigoso: dá confiança falsa e deixa regressões passarem silenciosamente. Dois padrões de ocorrência neste projeto:

*Padrão 1 — Tautologia lógica:*

```rust
// ANTIPADRÃO: always true regardless of what `s` contains
assert!(s.is_empty() || !s.is_empty(), "no panic is the contract")
```

A expressão `x || !x` é a tautologia de De Morgan — verdadeira para qualquer valor de `x`. Qualquer mudança na
produção que altere o valor retornado não será detectada.

*Padrão 2 — Atribuição morta (dead assignment):*

```rust
// ANTIPADRÃO: reads fields but asserts nothing
let _ = report.containers_deleted.len();
let _ = report.images_deleted.len();
let _ = report.space_reclaimed;
```

O `let _ = expr` descarta o valor sem verificar nada. O único contrato exercitado é "a chamada não panicopu", que o
próprio `.expect()` anterior já cobre. Estas linhas são ruído que mascara a ausência de asserções reais.

**Apenas caminho feliz (Happy Path Only)** — arquivo de testes que cobre somente o cenário de sucesso, sem testes para
condições de erro, valores-limite ou entradas inválidas. Toda operação do `IContainerDriver` com mais de um modo de
falha deve ter ao menos um teste por variante de `ContainerError` — caso contrário, o mapeamento de erros nunca é
exercitado.

**Duplicata de teste que não falha (Non-Failing Fake)** — test double que retorna `Ok(...)` incondicionalmente para
operações que deveriam falhar sob certas condições. Se a duplicata não representa fielmente os modos de falha do
contrato, os testes de integração exercitam apenas os caminhos felizes do código de produção.

O `MockContainerDriver` neste projeto contém operações que ignoram completamente seus parâmetros de entrada:

```rust
// ANTIPADRÃO no mock — operações que nunca falham, independente do estado do recurso
fn remove_volume(&self, _name: &str, _force: bool) -> Result<(), ContainerError> { Ok(()) }
fn remove_network(&self, _id: &str) -> Result<(), ContainerError> { Ok(()) }
fn remove_image(&self, _id: &str, _force: bool) -> Result<(), ContainerError> { Ok(()) }
fn restart_container(&self, _id: &str, _timeout_secs: Option<u32>) -> Result<(), ContainerError> { Ok(()) }
fn pause_container(&self, _id: &str) -> Result<(), ContainerError> { Ok(()) }
fn unpause_container(&self, _id: &str) -> Result<(), ContainerError> { Ok(()) }
```

Nenhuma dessas operações verifica se o ID existe, se o recurso está em estado correto ou se a operação é válida naquele
contexto. Qualquer teste que chame `remove_image("id-inexistente")` esperando `NotFound` passará silenciosamente — e
mais importante, a ausência desses testes significa que os paths de erro do código de produção nunca são exercitados.

### 2.2 Antipadrões de Manutenibilidade

**Duplicação de fixture (Fixture Duplication)** — código de setup idêntico repetido no início de múltiplos testes.

No projeto, `fn container_uc()` é copy-paste em 4 arquivos distintos com corpo idêntico:

```rust
// Declarada identicamente em:
// tests/container_driver_test.rs
// tests/container_stats_test.rs
// tests/inspect_test.rs
// tests/create_container_test.rs
fn container_uc() -> ContainerUseCase {
    ContainerUseCase::new(Arc::new(MockContainerDriver::new()))
}
```

Da mesma forma, `fn driver() -> Arc<MockContainerDriver>` é copy-paste em:

- `tests/pull_image_streaming_test.rs`
- `tests/container_logs_test.rs`
- `tests/terminal_test.rs`

E `fn make_container(...)` tem duas implementações com assinaturas incompatíveis:

- `tests/search_filter_test.rs`: 4 parâmetros `(name, image, short_id, compose)`
- `tests/compose_grouping_test.rs`: 2 parâmetros `(name, compose)`

A cópia produz drift: uma atualização na assinatura de `ContainerUseCase::new` deve ser propagada manualmente para
cada arquivo.

**Valores mágicos (Magic Values)** — literais sem nome em asserções que exigem leitura do código de produção para
serem compreendidos.

No projeto, IDs de container são repetidos como literais em 7+ arquivos sem constantes nomeadas:

```rust
// "aabbccdd1122334455667788" aparece em container_stats_test, inspect_test,
// container_logs_test, terminal_test, container_driver_test — sempre representando
// o mesmo container "web-server" running
uc.stats("aabbccdd1122334455667788").expect("stats");

// O leitor não sabe o que este ID representa sem consultar mock_driver.rs
```

Valores numéricos derivados do mock também aparecem hardcoded:

```rust
assert_eq!(usage.containers_total, 2);  // por que 2? qual é o contrato?
assert_eq!(report.containers_deleted.len(), 1);  // depende do estado interno do mock
assert!((stats.memory_usage_mb() - 50.0).abs() < 1.0);  // 50 MiB hardcoded no mock
```

**Teste deus (God Test)** — único teste que cobre múltiplas operações encadeadas. Quando falha, é impossível
identificar qual passo falhou sem depurar todo o caminho.

```rust
// ANTIPADRÃO: cobre stop + verify + start + verify em um único teste
fn start_container_makes_it_running() {
    let uc = container_uc();
    uc.stop("aabbccdd1122", None).expect("stop");    // operação 1
    let before = uc.list(false).expect("list").len(); // asserção intermediária
    assert_eq!(before, 0);
    uc.start("aabbccdd1122").expect("start");         // operação 2
    let after = uc.list(false).expect("list").len();  // asserção final
    assert_eq!(after, 1);
}
```

Deve ser fragmentado em dois testes focados: `stop_running_container_removes_from_running_list` e
`start_stopped_container_adds_to_running_list`.

**Duplicação inline vs. integração (Redundant Test Coverage)** — 7 testes unitários inline duplicam exatamente os
cenários cobertos pelos testes de integração em `tests/`:

| Teste inline em `src/`                             | Duplicado em `tests/`                                       |
|----------------------------------------------------|-------------------------------------------------------------|
| `container_use_case.rs::list_all_returns_all`      | `container_driver_test.rs::list_containers_all_returns_all` |
| `container_use_case.rs::list_running_only`         | `container_driver_test.rs::list_containers_running_only`    |
| `greet_use_case.rs::returns_greeting`              | `greet_use_case_test.rs::returns_greeting`                  |
| `image_use_case.rs::list_returns_images`           | `container_driver_test.rs::list_images_returns_images`      |
| `network_use_case.rs::list_networks_returns_two`   | `container_driver_test.rs::list_networks_returns_two`       |
| `network_use_case.rs::prune_system_returns_report` | `container_driver_test.rs::prune_system_returns_report`     |
| `volume_use_case.rs::list_returns_volumes`         | `container_driver_test.rs::list_volumes_returns_volumes`    |

Testes duplicados dobram o custo de manutenção sem aumentar a cobertura.

### 2.3 Antipadrões de Confiabilidade

**Falta de cobertura das transições de estado** — o domínio modela `ContainerStatus` com 7 variantes e um ciclo de
vida implícito. Os testes cobrem apenas as transições felizes (running → stopped → running). Não existem testes para:

- Tentar iniciar um container já em estado `Running` (idempotência ou erro?)
- Tentar pausar um container em estado `Stopped` (deve retornar `NotRunning`)
- Tentar remover um container em estado `Running` sem `force=true` (deve retornar erro ou forçar?)
- A variante `ContainerStatus::Paused` é testada no modelo de domínio mas nunca exercitada via driver

**Violação de pureza de camada** — `src/window/components/status_badge.rs` reside na camada UI mas seus testes
exercitam exclusivamente lógica do domínio (`ContainerStatus::css_class()`, `.label()`):

```rust
// Em src/window/components/status_badge.rs — camada UI
#[cfg(test)]
mod tests {
    use gtk_cross_platform::core::domain::container::ContainerStatus; // importa domínio ✓
    // Mas testa apenas métodos de ContainerStatus — não testa nada do componente UI
    fn css_class_matches_domain() { ... }
    fn label_is_non_empty_for_all_variants() { ... }
}
```

Esses testes pertencem ao módulo `src/core/domain/container.rs` onde `ContainerStatus` é definido — e de fato a mesma
lógica já está coberta por `status_css_classes` e `status_labels` naquele módulo. É duplicação com violação de
localização.

---

## Fase 3 — Oportunidades de Abstração e Reutilização

### 3.1 Object Mother — Constantes de IDs do Mock

**Problema:** O ID `"aabbccdd1122334455667788"` aparece como literal em pelo menos 7 arquivos de teste, representando
sempre o mesmo container "web-server" do `MockContainerDriver`. O mesmo vale para o container "db" parado e o ID
inexistente usado em testes de `NotFound`.

**Padrão a aplicar:** Módulo `tests/support/fixtures.rs` (ou `mod fixtures` dentro de cada arquivo) com constantes
nomeadas que documentam a semântica de cada ID:

```rust
// Conceito — estrutura esperada
pub mod fixtures {
    // IDs do MockContainerDriver (coincidem com mock_driver.rs)
    pub const RUNNING_CONTAINER_ID: &str = "aabbccdd1122334455667788"; // web-server, nginx:latest
    pub const STOPPED_CONTAINER_ID: &str = "112233445566778899aabbcc"; // db, postgres:15, Exited
    pub const STANDALONE_CONTAINER_ID: &str = "223344556677889900aabbcc"; // standalone, redis
    pub const UNKNOWN_CONTAINER_ID: &str = "nonexistentid0000000000";

    // Valores numéricos derivados do estado fixo do mock
    pub const MOCK_CONTAINERS_TOTAL: usize = 3;
    pub const MOCK_RUNNING_CONTAINERS: usize = 1;
    pub const MOCK_IMAGES_TOTAL: usize = 2;
    pub const MOCK_WEB_SERVER_MEMORY_MIB: f64 = 50.0;
}
```

**Duplicação eliminada:** ~30 literais de ID distribuídos em 7 arquivos colapsados em 4 constantes nomeadas.

### 3.2 Fixture Compartilhada — Factory Functions de Use Case

**Problema:** `fn container_uc()`, `fn driver()` e variantes são declaradas localmente em cada arquivo de teste com
corpo idêntico. Qualquer mudança na assinatura de `ContainerUseCase::new` ou `MockContainerDriver::new` exige
atualização em 4–5 lugares.

**Padrão a aplicar:** Módulo de suporte compartilhado que cada arquivo de integração importa:

```rust
// Conceito — tests/support/mod.rs
pub fn container_uc() -> ContainerUseCase {
    ContainerUseCase::new(Arc::new(MockContainerDriver::new()))
}

pub fn image_uc() -> ImageUseCase {
    ImageUseCase::new(Arc::new(MockContainerDriver::new()))
}

pub fn mock_driver() -> Arc<MockContainerDriver> {
    Arc::new(MockContainerDriver::new())
}
```

**Duplicação eliminada:** 4 declarações de `container_uc()`, 3 de `fn driver()`, 2 de `fn use_case()` para image.

### 3.3 Test Data Builder — Container Builder

**Problema:** `fn make_container(...)` tem duas implementações incompatíveis em `search_filter_test.rs` (4 parâmetros)
e `compose_grouping_test.rs` (2 parâmetros), construindo estruturalmente o mesmo tipo com assinaturas diferentes. A
divergência crescerá quando o domínio `Container` evoluir.

**Padrão a aplicar:** Builder fluente que torna explícita a intenção de cada campo:

```rust
// Conceito — elimina as duas implementações incompatíveis
ContainerBuilder::default ()
.name("nginx-proxy")
.image("nginx:latest")
.short_id("aabbccdd1122")
.compose_project("web-stack")
.status(ContainerStatus::Running)
.build()
```

**Duplicação eliminada:** 2 implementações incompatíveis de `make_container` colapsadas em 1 builder com semântica
explícita por campo.

### 3.4 Asserção Customizada — Verificação de Variante de Erro

**Problema:** O padrão de verificação de variante de erro é repetido em múltiplos testes com um anti-padrão comum:
usar `format!("{}", err)` e `contains("string")` em vez de verificar a variante diretamente. Isso acopla o teste
à mensagem de texto do erro, não ao tipo do erro.

```rust
// ANTIPADRÃO — frágil contra renomear strings de erro
let msg = format!("{}", result.unwrap_err());
assert!(msg.contains("Not found") || msg.contains("not found"), "...");
```

**Padrão a aplicar:** Macro de asserção customizada que verifica a variante via pattern matching:

```rust
// Conceito — macro assertiva expressiva e robusta
assert_error_variant!(result, ContainerError::NotFound(_));
assert_error_variant!(result, ContainerError::NotRunning(_));
assert_error_variant!(result, ContainerError::RuntimeNotAvailable(_));
```

A macro é declarada uma vez em `tests/support/mod.rs` dentro de `#[cfg(test)]`.

**Duplicação eliminada:** Padrão `format!() + contains()` usado em 8+ testes substituído por verificação estrutural
de tipo.

### 3.5 Teste Parametrizado — `is_secret_env_key`

**Problema:** `env_masking_test.rs` contém 7 funções de teste que diferem apenas nos valores de entrada e na asserção
booleana esperada. Cada função testa uma variante do comportamento de mascaramento.

```rust
// 7 testes quase idênticos — diferem apenas nos valores
fn mask_password_suffix() {
    assert!(is_secret_env_key("POSTGRES_PASSWORD"));
    ...
}
fn mask_password_lowercase() {
    assert!(is_secret_env_key("password"));
    ...
}
fn mask_secret_substring() {
    assert!(is_secret_env_key("API_SECRET"));
    ...
}
fn mask_token_substring() { ... }
fn mask_key_substring() { ... }
fn safe_key_not_masked() { ... }
fn empty_key_not_masked() { ... }
```

**Padrão a aplicar:** Tabela de casos (table-driven test) que elimina 7 funções e centraliza os dados:

```rust
// Conceito — (chave_env, esperado_secreto)
let cases = [
("POSTGRES_PASSWORD", true),
("DB_PASSWORD",       true),
("password", true),
("API_SECRET", true),
("GITHUB_TOKEN", true),
("AWS_ACCESS_KEY_ID", true),
("NGINX_HOST",        false),
("TZ", false),
("PORT", false),
("", false),
];
for (key, expected) in cases {
assert_eq!(is_secret_env_key(key), expected, "key={key:?}");
}
```

**Duplicação eliminada:** 7 funções de teste colapsadas em 1, com cobertura igual ou maior e adição trivial de novos
casos.

### 3.6 Teste Parametrizado — Parsing de `ContainerStatus`

**Problema:** Os testes inline em `container.rs` cobrem cada variante de `ContainerStatus::from_state` com uma função
separada (`status_from_running_state`, `status_from_paused_state`, `status_from_exited_with_code`,
`status_from_unknown_state`). São candidatos diretos a tabela.

**Padrão a aplicar:** Tabela `(state_str, exit_code, expected_variant)` que cobre todos os casos em um único loop,
tornando simples adicionar novos estados:

```rust
// Conceito — tabela de (estado_string, exit_code, variante_esperada)
let cases: & [( & str, Option<i32>, ContainerStatus)] = & [
("running", None,    ContainerStatus::Running),
("paused", None,    ContainerStatus::Paused),
("exited", Some(0), ContainerStatus::Exited(0)),
("exited", Some(1), ContainerStatus::Exited(1)),
("restarting", None,    ContainerStatus::Restarting),
("dead", None,    ContainerStatus::Dead),
("fancy-new-state", None,    /* matches Unknown(_) */),
];
```

---

## Fase 4 — Laudo Técnico (Entrega Obrigatória)

Esta fase produz o documento de saída. Execute-a **após** as Fases 1–3. O laudo é a entrega principal do comando —
não é opcional e não pode ser substituído por um resumo verbal.

Gere o laudo no arquivo `docs/test-quality-audit.md`. A estrutura obrigatória é:

```markdown
# Laudo de Qualidade — Camada de Testes

**Data:** YYYY-MM-DD
**Auditor:** Claude Code (externo)
**Escopo:** Todos os módulos `#[cfg(test)]` em `src/` e todos os arquivos em `tests/`
**Versão auditada:** (resultado de `git rev-parse --short HEAD`)

## Sumário Executivo

| Dimensão | Estado | Severidade Máxima |
|----------|--------|-------------------|
| Pirâmide de testes | ... | ... |
| Nomenclatura | ... | ... |
| Qualidade das asserções | ... | ... |
| Fidelidade do mock | ... | ... |
| Duplicação de fixtures | ... | ... |
| Pureza de camada | ... | ... |

## Estatísticas

- Testes unitários inline: N
- Testes de integração: N
- Testes E2E/widget (ignored): N
- Total: N
- Razão unit:integration: N:N

## Achados por Severidade

### CRÍTICO

[ ] AC-01 — <nome do antipadrão> — <arquivo(s) afetado(s)>
Descrição: ...
Impacto: ...
Correção: ...

### ALTO

[ ] AA-01 — ...

### MÉDIO

[ ] AM-01 — ...

### BAIXO

[ ] AB-01 — ...

## Checklist de Ações Corretivas

> Marque cada item com [x] após aplicar a correção. Execute `make test` ao final.

### Ações Imediatas (Crítico / Alto)

- [ ] Substituir asserção tautológica em `test_exec_empty_command_handled_gracefully` por asserção real
- [ ] Substituir dead assignments em `test_prune_system_returns_report` por `assert_eq!` sobre campos específicos
- [ ] Adicionar verificação de existência de recurso em `remove_volume`, `remove_network`, `remove_image` no mock
- [ ] Adicionar verificação de estado em `restart_container`, `pause_container`, `unpause_container` no mock
- [ ] Mover testes de `ContainerStatus` de `status_badge.rs` para `container.rs` (violação de camada)
- [ ] Adicionar testes de transição de estado inválida: pause(Stopped), start(Running), remove(Running sem force)

### Ações de Manutenibilidade (Médio)

- [ ] Criar `tests/support/fixtures.rs` com `RUNNING_CONTAINER_ID`, `STOPPED_CONTAINER_ID`, `UNKNOWN_CONTAINER_ID`
- [ ] Criar `tests/support/factories.rs` com `container_uc()`, `image_uc()`, `mock_driver()` compartilhados
- [ ] Eliminar declarações locais duplicadas de `fn container_uc()` em 4 arquivos de teste
- [ ] Eliminar declarações locais duplicadas de `fn driver()` em 3 arquivos de teste
- [ ] Consolidar `make_container` em único builder ou factory compartilhada
- [ ] Parametrizar 7 testes de `is_secret_env_key` em tabela única
- [ ] Parametrizar testes de `ContainerStatus::from_state` em tabela única
- [ ] Fragmentar `start_container_makes_it_running` em dois testes focados
- [ ] Remover testes inline duplicados (os 7 listados na tabela de Redundant Test Coverage)
- [ ] Substituir `format!() + contains()` por macro `assert_error_variant!` nos 8+ testes afetados

### Melhorias de Nomenclatura (Baixo)

- [ ] Remover prefixo `test_` dos testes em `dashboard_test.rs`
- [ ] Remover prefixo `test_` dos testes em `pull_image_streaming_test.rs`
- [ ] Remover prefixo `test_` dos testes em `runtime_switcher_test.rs`
- [ ] Remover prefixo `test_` dos testes em `container_logs_test.rs`
- [ ] Remover prefixo `test_` dos testes em `terminal_test.rs`
- [ ] Adicionar condição aos nomes: `events_returns_list` → `events_with_no_filter_returns_all_events`
- [ ] Adicionar condição: `layers_have_id_cmd_and_size` → `layers_for_known_image_have_populated_fields`
- [ ] Adicionar doc comment `//!` em `compose_lifecycle_test.rs` e `system_events_test.rs`

## Resultado de `make test`

```

(output completo aqui)

```

**Status final:** PASSOU / FALHOU
```

---

## Formato de Entrega nas Fases Intermediárias

### Fase 1 — Levantamento Estrutural

| Dimensão                   | Estado Atual | Prática Recomendada | Gap |
|----------------------------|--------------|---------------------|-----|
| Forma da pirâmide          | ...          | ...                 | ... |
| Nomenclatura               | ...          | ...                 | ... |
| Qualidade das asserções    | ...          | ...                 | ... |
| Independência entre testes | ...          | ...                 | ... |

### Fase 2 — Antipadrões

Para cada antipadrão encontrado:

```
## [SEVERIDADE] ANTIPADRÃO — <nome>

- **Testes afetados:** <nomes dos cenários, não linhas de código>
- **Problema:** <por que é perigoso ou enganoso>
- **Impacto:** <o que falha ou passa despercebido>
- **Correção recomendada:** <padrão ou técnica a aplicar>
```

Níveis de severidade:

- **CRÍTICO** — o teste nunca detectará a regressão que deveria detectar
- **ALTO** — viola uma regra de arquitetura ou gera falsa confiança sistêmica
- **MÉDIO** — dificulta manutenção ou diagnóstico de falhas
- **BAIXO** — cosmético ou oportunidade de melhoria menor

### Fase 3 — Oportunidades de Abstração

Para cada oportunidade:

```
## PADRÃO — <nome do padrão>

- **Aplica-se a:** <cenários de teste, não caminhos de arquivo>
- **Duplicação eliminada:** <quantas ocorrências colapsadas em uma>
- **Estrutura esperada:** <pseudocódigo mostrando a intenção, não a implementação>
```

---

## Guardrails deste Comando

Este comando nunca deve:

- Modificar qualquer arquivo do projeto (código de produção ou testes)
- Reportar como gap algo que já está corretamente implementado — foque apenas em problemas reais
- Omitir a Fase 4 — o laudo em `docs/test-quality-audit.md` é a entrega obrigatória do comando
