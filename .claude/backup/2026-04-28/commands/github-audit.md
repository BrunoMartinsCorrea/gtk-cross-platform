# /project:github-audit

Audite este repositório GTK4/Rust/Flatpak nas seis dimensões abaixo. Este comando é
auto-contido — execute-o sem contexto de conversa anterior.

**Repositório:** `gtk-cross-platform`
**Stack:** Rust 2024 edition · GTK4 0.9 · libadwaita 0.7 · glib/gio 0.20 · gettext-rs 0.7
**Tipo de projeto:** Aplicativo desktop GNOME (Flatpak), arquitetura hexagonal, alvo
multiplataforma (Linux, macOS, Windows, GNOME Mobile)

---

## O que ler antes de auditar

Leia os seguintes arquivos na íntegra antes de emitir qualquer diagnóstico:

- `CLAUDE.md` — regras de arquitetura, threading, HIG, i18n, breakpoints
- `Cargo.toml` e `Cargo.lock` — versões reais de dependências
- `com.example.GtkCrossPlatform.json` — manifesto Flatpak (SDK, permissões de sandbox)
- `.github/workflows/ci.yml` — pipeline de lint e testes unitários
- `.github/workflows/flatpak.yml` — build Flatpak + publicação nightly
- `.github/workflows/editorconfig.yml` — validação de `.editorconfig`
- `Makefile` — targets locais de build/test/lint/flatpak
- `meson.build` e `meson_options.txt` — sistema de build alternativo (legado)
- `README.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CHANGELOG.md`
- `data/resources/window.ui` — breakpoints e templates de widgets
- `data/resources/style.css` — touch targets e classes CSS
- `tests/` — `container_driver_test.rs`, `greet_use_case_test.rs`, `widget_test.rs`, `unit/`
- `src/` completo — arquitetura real vs. documentada

---

## Dimensão 1 — Manutenibilidade

**Analise:**

- Estrutura de `src/` — coerência com a arquitetura hexagonal documentada em `CLAUDE.md`:
  `core/` (domínio puro), `ports/` (traits), `infrastructure/` (adaptadores), `window/` (UI)
- Arquivos mortos, código comentado, `TODO` / `FIXME` não rastreados em nenhuma issue
- `README.md` — seções presentes: Overview, Quickstart, Development, Architecture, Contributing,
  Versioning, License; badges de CI; comandos `make` documentados
- `CONTRIBUTING.md` — guia de contribuição atualizado com fluxo de PR, estilo de commit,
  requisitos de build local
- `CODE_OF_CONDUCT.md`, `CHANGELOG.md`, `AUTHORS`, `GOVERNANCE.md` — presença e qualidade
- Rustdoc (`///`) em tipos públicos de `src/ports/` e `src/core/domain/`; ausência de blocos
  longos desnecessários em código interno
- Testes — localização em `tests/` (`container_driver_test.rs`, `greet_use_case_test.rs`,
  `widget_test.rs`) e módulos `#[cfg(test)]` inline em `src/core/`; nomenclatura descritiva;
  padrão Arrange-Act-Assert; cobertura de todos os métodos de `IContainerDriver` via
  `MockContainerDriver`

**Melhores práticas esperadas:**

- `README.md` com seção de arquitetura que menciona `AdwNavigationSplitView`, `AdwBreakpoint`,
  `GestureLongPress`, `spawn_driver_task`, e os quatro adaptadores de runtime
- `CHANGELOG.md` seguindo [Keep a Changelog](https://keepachangelog.com/) com entradas por versão
- Todos os tipos públicos de `ports/` documentados com `///`; sem `println!` ou `dbg!` no código
  de produção
- `tests/container_driver_test.rs` cobrindo todos os métodos de `IContainerDriver` via mock

---

## Dimensão 2 — CI/CD

**Analise:**

- `.github/workflows/ci.yml` — etapas presentes: `cargo fmt --check`, `cargo clippy -D warnings`,
  `cargo test --lib`; ausência de `permissions:` explícitas; ausência de cache de dependências
  Cargo (`Swatinem/rust-cache` ou `actions/cache`); testes de integração em `tests/` não rodados
  (requerem display Wayland/X11)
- `.github/workflows/flatpak.yml` — triggers: `push` (main) e `pull_request` (main); build via
  `flatpak/flatpak-github-actions/flatpak-builder@v6`; geração de `cargo-sources.json` com
  `flatpak-cargo-generator.py` para build offline; publicação de release nightly via `gh release`
  apenas em push para main (passo `Publish Nightly`); **sem** cross-compilação para aarch64 no
  workflow (disponível apenas localmente via `make flatpak-build-arm`); sem cache Cargo explícito
  (apenas cache do flatpak-builder via `cache-key`)
- `.github/workflows/editorconfig.yml` — validação de `.editorconfig`
- Ausência de `cargo audit` em qualquer workflow
- Ausência de matriz de versões Rust (`stable`, `beta`) ou plataformas nos workflows
- Ausência de `workflow_dispatch` nos workflows para builds manuais
- Ausência de `.github/dependabot.yml` para atualização automática de crates e Actions

**Melhores práticas esperadas:**

- Pipeline completo: `fmt → clippy → test → flatpak-build → release`
- Cache Cargo em `ci.yml` via `Swatinem/rust-cache` (reduz tempo de lint de ~3 min para ~30 s)
- `permissions: contents: read` como padrão em `ci.yml`; `contents: write` apenas no job de
  release em `flatpak.yml` (já presente)
- `workflow_dispatch` adicionado ao `flatpak.yml` para builds manuais sem push
- Job separado para testes de widget GTK (`widget_test.rs` marcado `#[ignore]`) com display
  virtual Xvfb
- `cargo audit` como passo obrigatório no `ci.yml`, com falha em vulnerabilidades HIGH/CRITICAL
- Dependabot ou Renovate configurado para `cargo` e `github-actions`

---

## Dimensão 3 — Versionamento

**Analise:**

- Convenção de commits no histórico git — Conventional Commits (`feat:`, `fix:`, `chore:`,
  `BREAKING CHANGE:`)?
- Alinhamento de versão: `Cargo.toml [package].version` ↔ tag git ↔ `CHANGELOG.md` ↔
  `com.example.GtkCrossPlatform.json` (`<release>` em metainfo)
- `.gitignore` — adequado para Rust (`target/`), GTK (`_build/`, `build/`), IDEs (.iml, .idea,
  .vscode), macOS (`.DS_Store`), variáveis de ambiente (`.env`)
- `.gitattributes` — presente; normaliza line endings (`* text=auto eol=lf`); marca
  `*.flatpak` e `*.gresource` como binário; marca `Cargo.lock` como `merge=text`; marca
  `po/*.po` para GitHub Linguist; **ausente**: atributo `diff=po` para diffs legíveis de `.po`
- Tags semânticas (`v0.1.0`) associadas a GitHub Releases

**Melhores práticas esperadas:**

- Conventional Commits adotado e documentado em `CONTRIBUTING.md`
- `Cargo.toml` version bumped a cada release; `CHANGELOG.md` atualizado no mesmo commit
- `.gitignore` cobrindo `target/`, `_build/`, `*.gresource` gerado, `.env`, `*.swp`
- `.gitattributes` com `po/*.po diff=po` adicionado
- Nenhum artefato compilado (`.gresource`, binários) no histórico git

---

## Dimensão 4 — Distribuição

**Analise:**

- `com.example.GtkCrossPlatform.json` — manifesto Flatpak: SDK pinado (`org.gnome.Sdk//48`),
  runtime pinado, dependências offline via `cargo-sources.json`, permissões de sandbox (Wayland,
  X11, IPC; sem `--device=dri` desnecessário)
- App ID `com.example.GtkCrossPlatform` — placeholder; deve ser substituído por ID real antes
  de publicar no Flathub
- `data/com.example.GtkCrossPlatform.metainfo.xml` — AppStream metainfo: válido, com
  `<release>` atualizado, screenshots, descrição, categorias GNOME
- `data/com.example.GtkCrossPlatform.desktop` — desktop entry: categorias, `TryExec`,
  `StartupWMClass`
- GitHub Releases — bundle Flatpak publicado automaticamente como release nightly a cada push
  em main; tag `nightly` recriada a cada build (não versionada)
- Cross-compilação para aarch64 (GNOME Mobile / PinePhone) disponível apenas localmente
  via `make flatpak-build-arm` — **não** automatizada no CI
- Makefile targets de distribuição: `flatpak-build`, `flatpak-run`, `flatpak-install`,
  `flatpak-build-arm`
- Sistema de build meson (`meson.build` + `meson_options.txt`) presente mas aparentemente
  legado — o build primário usa Cargo; analisar se está sincronizado ou pode ser removido

**Melhores práticas esperadas:**

- App ID real (reverse-domain) antes de qualquer publicação no Flathub
- `metainfo.xml` validado com `appstream-util validate` no CI
- `desktop` file validado com `desktop-file-validate` no CI
- Bundle Flatpak anexado à GitHub Release com nome versionado (`gtk-cross-platform-v0.1.0.flatpak`)
  em releases semânticas; release nightly mantida separada
- Job de aarch64 no `flatpak.yml` via `flatpak-github-actions` para builds de GNOME Mobile
- Esclarecer papel do `meson.build` ou removê-lo se for dead code

---

## Dimensão 5 — Segurança

**Analise:**

- `cargo audit` — ausente nos workflows
- Dependências com vulnerabilidades conhecidas (inferir por `Cargo.lock` + data atual)
- Permissões de sandbox Flatpak no manifesto — princípio do menor privilégio: sem
  `--filesystem=home`, sem `--device=all`, sem `--share=network` desnecessário
- `SECURITY.md` — política de responsible disclosure presente e com contato
- Secrets hardcoded nos workflows (tokens, credenciais de deploy) — verificar
- `.github/dependabot.yml` — ausente; nenhuma atualização automática de Actions e crates Cargo
- Scanning de secrets no CI (`gitleaks`, `trufflehog`, ou GitHub Secret Scanning)
- Permissões de workflows: `ci.yml` sem bloco `permissions:` explícito; `flatpak.yml` com
  `permissions: contents: write` no nível do job (correto para release)

**Melhores práticas esperadas:**

- `cargo audit` como passo obrigatório no CI, com falha em vulnerabilidades HIGH/CRITICAL
- Flatpak sandbox sem permissões além de Wayland, X11 fallback, IPC
- `SECURITY.md` com prazo de resposta e canal de reporte (e-mail ou GitHub Security Advisory)
- Dependabot configurado para `cargo` e `github-actions` em `.github/dependabot.yml`
- `permissions: contents: read` explícito em `ci.yml` (princípio do menor privilégio)
- Nenhum secret hardcoded; todos os tokens via `secrets.*` nos workflows

---

## Dimensão 6 — Configuração de Ferramentas

**Analise:**

- `.editorconfig` — presente; cobre `indent_style`, `indent_size`, `end_of_line`,
  `insert_final_newline`, `trim_trailing_whitespace` para `.rs`, `.toml`, `.yml`, `.md`, `.ui`
- `.gitattributes` — presente; normaliza EOL, marca binários e `Cargo.lock`; falta `diff=po`
- `rustfmt.toml` — ausente; sem configuração explícita de formatação além do padrão `cargo fmt`
- Clippy lints — verificar se `#![deny(clippy::all)]` ou `[lints]` está em `Cargo.toml`; procurar
  `#[allow(clippy::...)]` silenciosos não justificados
- Pre-commit hooks — `pre-commit` framework ou scripts em `.git/hooks/` para `cargo fmt` e
  `cargo clippy` localmente — verificar presença
- `Makefile` — targets bem definidos (`setup`, `build`, `run`, `test`, `lint`, `fmt`, `fmt-fix`,
  `run-mobile`, `flatpak-build`, `flatpak-run`, `flatpak-install`, `flatpak-build-arm`,
  `setup-macos`, `setup-windows`); ausência de `make help` com descrições de uma linha
- Variáveis de ambiente de build — `APP_ID`, `PROFILE`, `PKGDATADIR`, `LOCALEDIR` definidas
  em `build.rs`; ausência de `.env` com valores reais no repositório
- `meson.build` — presente na raiz mas aparentemente legado (build primário é Cargo); analisar
  sincronização com `Cargo.toml`

**Melhores práticas esperadas:**

- `.editorconfig` validado no CI (workflow `editorconfig.yml` já presente — verificar regras cobertas)
- `rustfmt.toml` com configuração mínima explícita (`edition = "2024"`)
- Lints do Clippy centralizados em `Cargo.toml [lints]` (Rust 1.73+), não espalhados por arquivo
- `make help` imprime todos os targets com descrição de uma linha
- `po/*.po diff=po` adicionado ao `.gitattributes`
- Nenhum `.env` real no repositório; variáveis necessárias documentadas em `CONTRIBUTING.md`

---

## Formato de entrega

```
# Auditoria: gtk-cross-platform

## Sumário Executivo
<parágrafo com diagnóstico geral e prioridade das mudanças>

## Scorecard
| Dimensão         | Nota (0–10) | Status    |
|------------------|-------------|-----------|
| Manutenibilidade | X           | 🔴/🟡/🟢 |
| CI/CD            | X           | 🔴/🟡/🟢 |
| Versionamento    | X           | 🔴/🟡/🟢 |
| Distribuição     | X           | 🔴/🟡/🟢 |
| Segurança        | X           | 🔴/🟡/🟢 |
| Configuração     | X           | 🔴/🟡/🟢 |

## Análise por Dimensão
<para cada dimensão: Estado Atual → Gaps → Recomendações>

## Plano de Reestruturação

### Quick Wins (1–3 dias)
<lista de mudanças de alto impacto e baixo esforço>

### Melhorias de Médio Prazo (1–4 semanas)
<lista priorizada>

### Iniciativas Estruturais (1–3 meses)
<lista com dependências entre itens>

## Arquivos a Criar / Modificar
| Arquivo                              | Ação                       | Motivo                              |
|--------------------------------------|----------------------------|-------------------------------------|
| .github/dependabot.yml               | criar                      | atualizações automáticas de crates  |
| rustfmt.toml                         | criar                      | configuração explícita de formato   |
| ...                                  | ...                        | ...                                 |
```

---

## Restrições

- Baseie todas as recomendações no que for **observável no repositório**; não assuma o que não
  está presente
- Quando não tiver acesso a um arquivo (ex: secrets reais), indique explicitamente que a análise
  é parcial
- Priorize recomendações por: **segurança > CI/CD > distribuição > manutenibilidade**
- Use emojis de status apenas no scorecard; mantenha o restante técnico e direto
- Não repita gaps já cobertos por `/project:compliance-audit` (i18n, A11Y, breakpoints,
  threading, arquitetura hexagonal) — foque nos aspectos de repositório GitHub desta auditoria
