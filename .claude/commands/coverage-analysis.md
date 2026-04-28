# /project:coverage-analysis — Rust Testing Coverage Audit & Execution Plan

## Papel do Agente

Você é um engenheiro de software sênior especialista em qualidade de software e ecossistema Rust. Sua função nesta
tarefa é conduzir um **audit completo da infraestrutura de testes** do projeto gtk-cross-platform, cruzar os achados
com as melhores práticas da indústria, e entregar um plano de execução priorizado e acionável — sem depender de
follow-up.

---

## Contexto do Projeto

O projeto é uma **aplicação GTK4 + Adwaita** escrita em Rust, targeting Linux, macOS, Windows e GNOME Mobile. Segue
**Arquitetura Hexagonal** (Ports & Adapters) com quatro camadas:

| Camada   | Caminho               | Regra de pureza                                  |
|----------|-----------------------|--------------------------------------------------|
| Domain   | `src/core/`           | Sem GTK/Adw/GLib; lógica pura de negócio         |
| Ports    | `src/ports/`          | Traits Rust consumidas pelo core e pela UI       |
| Adapters | `src/infrastructure/` | Implementam ports; podem usar GLib/IO, nunca GTK |
| UI       | `src/window/`         | Widgets GTK/Adw; depende apenas de ports         |

**Tipos críticos para testes:**

- `IContainerDriver` (`src/ports/i_container_driver.rs`) — porta central; implementada por Docker, Podman, containerd e
  Mock.
- `ContainerDriverFactory` (`src/infrastructure/containers/factory.rs`) — auto-detecta runtime disponível.
- `spawn_driver_task` (`src/infrastructure/containers/background.rs`) — bridge bloqueante ↔ GLib main loop via
  `async-channel`; `tokio` é proibido no projeto.
- `MockContainerDriver` (`src/infrastructure/containers/mock_driver.rs`) — fake in-memory usado em testes de integração.
- `ContainerStatus` (`src/core/domain/container.rs`) — enum com variante `Exited(i32)` e lógica `from_state`.

**Restrições de ambiente obrigatórias:**

- Testes de widget em `tests/widget_test.rs` exigem display (GTK init). Em Linux: `xvfb-run cargo test --test
  widget_test -- --test-threads=1 --ignored`. Em macOS: idem sem xvfb. Nunca rodar esses testes em CI headless sem
  Xvfb.
- `tokio` é explicitamente proibido — conflita com o GLib event loop. Usar `async-channel` e `glib::spawn_local`.
- Testes na camada `src/core/` não devem importar `gtk4` ou `adw`.

**Estado atual do tooling (conhecido antes do levantamento):**

- Runner: `cargo test` padrão. Nenhum `cargo-nextest`, nenhum `.config/nextest.toml`.
- Coverage: nenhuma ferramenta configurada (`cargo-llvm-cov`, `cargo-tarpaulin` ausentes).
- Dev-dependencies: **nenhuma** em `Cargo.toml` (`proptest`, `insta`, `mockall` ausentes).
- CI: `.github/workflows/ci.yml` executa apenas `cargo test --lib` — **os testes em `tests/` nunca rodam em CI**.

As melhores práticas de referência para este audit são:

**Organização de testes**

- Unit tests com `#[cfg(test)]` dentro do próprio módulo para acessar funções privadas
- Integration tests em `tests/` consumindo apenas a API pública
- Doc tests em `///` para manter exemplos de documentação executáveis

**Ferramental de execução**

- `cargo-nextest` como runner principal (paralelismo, isolamento por processo, JUnit XML nativo)
- `cargo-llvm-cov` para cobertura via LLVM (suporta LCOV, Cobertura XML, HTML)
- `insta` para snapshot testing de outputs complexos
- `proptest` ou `quickcheck` para property-based testing em domínios críticos

**Métricas e thresholds**

- Cobertura mínima por camada: domain ≥ 90%, infrastructure ≥ 60%, UI (testável sem display) ≥ 40%
- Métrica preferida: **regions** (captura branches dentro de linha, mais preciso que lines)
- `cargo-mutants` para validar eficácia dos testes existentes

**Formatos de output desacoplados de vendor**

- Cobertura: LCOV (`lcov.info`) ou Cobertura XML
- Resultados de teste: JUnit XML via nextest
- Sumário no terminal: `cargo llvm-cov --summary-only`
- GitHub Actions: `$GITHUB_STEP_SUMMARY` para reporting inline no PR

**CI/CD**

- `cargo test --no-run --locked` como step de compilação de testes antes da execução
- `RUSTFLAGS="-D warnings"` para tratar warnings como erros
- Branch protection com status checks obrigatórios no GitHub
- Threshold enforcement via `--fail-under-lines`, `--fail-under-functions`, `--fail-under-regions`

---

## Tarefa

Execute as três fases a seguir em sequência. Não interrompa para pedir confirmação entre elas.

### Fase 1 — Levantamento da Estrutura Atual

Inspecione o projeto e mapeie o estado atual. Para cada item abaixo, registre o que existe e o que está ausente.
Considere o estado conhecido descrito no contexto acima como ponto de partida, mas verifique e complemente lendo os
arquivos relevantes:

1. **Estrutura de arquivos**: presença e contagem de testes em `tests/`, módulos `#[cfg(test)]` inline por camada,
   doc tests em uso. Preste atenção especial ao status `#[ignore]` em `tests/widget_test.rs`.
2. **Cargo.toml**: dev-dependencies declaradas (ou ausência total); flags `cfg(test)`; features de teste.
3. **Runner**: `cargo test` padrão ou `cargo-nextest`; existência de `.config/nextest.toml`.
4. **Coverage**: `cargo-llvm-cov`, `cargo-tarpaulin` ou ausência.
5. **CI/CD**: inspecione `.github/workflows/ci.yml` — identifique exatamente quais flags são passadas a `cargo test`
   e se os testes em `tests/` são executados. Avalie também `.github/workflows/flatpak.yml`.
6. **Qualidade dos testes existentes**: por camada — verifique se cobrem apenas happy path, se testam edge cases
   (`ContainerStatus::from_state` com estado desconhecido, `Exited(i32)` com diferentes códigos), se
   `MockContainerDriver::unavailable()` é suficientemente exercitado, e se `spawn_driver_task` tem qualquer cobertura.
7. **Reporting**: algum sumário de cobertura no stdout ou PR? Qual formato de export?

Apresente este levantamento em uma tabela com colunas: `Aspecto | Estado Atual | Observação`.

---

### Fase 2 — Análise de Gap

Com base no levantamento, cruze cada aspecto com as melhores práticas. Priorize os seguintes gaps conhecidos e
identifique gaps adicionais:

- **CI executa apenas `--lib`**: os 11 testes em `tests/container_driver_test.rs` e demais integração nunca rodaram em
  CI.
- **Widget tests excluídos permanentemente**: `tests/widget_test.rs` tem 5 testes todos `#[ignore]` sem caminho de CI
  para executá-los (nem mesmo com Xvfb).
- **Zero dev-dependencies**: sem `proptest` para `ContainerStatus::from_state`, sem `insta` para snapshots de
  `IContainerDriver` outputs.
- **Nenhuma cobertura configurada**: impossível medir maturidade por camada.
- **`spawn_driver_task` sem cobertura**: bridge crítica de threading sem nenhum teste.

Para cada gap, classifique:

- **Impacto**: Alto / Médio / Baixo — quão crítico é o gap para qualidade e manutenibilidade
- **Esforço**: Alto / Médio / Baixo — estimativa de complexidade de implementação
- **Risco de não corrigir**: o que acontece se este gap permanecer

Apresente como lista estruturada, um item por gap.

---

### Fase 3 — Plano de Execução

Produza um plano de execução priorizado, agrupado em fases de entrega incremental. Cada item deve conter:

- **O que fazer**: descrição objetiva
- **Como fazer**: comandos, configurações ou código mínimo — específico o suficiente para executar sem pesquisar.
  Referencie arquivos concretos do projeto (ex: `Cargo.toml`, `.github/workflows/ci.yml`,
  `.config/nextest.toml`).
- **Critério de conclusão**: como saber que está feito
- **Dependências**: quais itens precisam estar concluídos antes

Organize por matriz Impacto × Esforço:

1. **Quick wins** (Alto impacto, Baixo esforço) — inclui obrigatoriamente a correção do CI para rodar `tests/`
2. **Grandes apostas** (Alto impacto, Alto esforço) — inclui cobertura + thresholds por camada
3. **Fill-ins** (Baixo impacto, Baixo esforço)
4. **Reconsiderar** (Baixo impacto, Alto esforço) — documente o motivo de postergar

---

## Restrições

- Não proponha mudanças na lógica de negócio — escopo exclusivo: infraestrutura de testes e qualidade.
- Não use ferramentas proprietárias ou pagas como solução primária; prefira o ecossistema Cargo e formatos abertos.
- Não remova testes existentes mesmo que fracos — o plano deve evoluir sobre o que existe.
- Respeite as restrições do projeto: nenhum `tokio`, widget tests exigem display, camada `core/` sem imports GTK.
- Thresholds iniciais devem ser conservadores (atingíveis no estado atual + margem de crescimento), não ideais.
- O projeto **não é um workspace** — é um único crate com `[lib]` + `[[bin]]`.

---

## Formato de Entrega

Entregue um único documento Markdown estruturado:

```
# Rust Testing Audit — gtk-cross-platform

## Sumário Executivo
(3-5 linhas: estado atual em uma frase, os 3 gaps mais críticos, horizonte estimado para atingir maturidade)

## Fase 1: Estrutura Atual
(tabela de levantamento)

## Fase 2: Análise de Gap
(lista estruturada com Impacto, Esforço e Risco)

## Fase 3: Plano de Execução
(agrupado por prioridade, com o que fazer / como fazer / critério de conclusão / dependências)

## Referências de Configuração
(snippets prontos para uso: ci.yml corrigido, nextest.toml, llvm-cov step, Cargo.toml additions)
```

O documento deve ser auto-suficiente: um engenheiro sem contexto prévio deve conseguir executar o plano do início ao
fim lendo apenas este arquivo.
