---
description: Reestrutura o Makefile como instrumento de ciclo de vida SOLID — agregadores genéricos (setup, test, validate, dist) delegam para específicos (setup-macos, test-unit, validate-lint), setup é idempotente, `make` sem argumentos configura e roda o projeto automaticamente, e `make ci` espelha exatamente o pipeline de CI.
---

# Redesign Makefile

> Leia `CLAUDE.md` antes de começar — especialmente as seções **Build and Run Commands** e
> **Architecture** para entender a cadeia de ferramentas e as regras de camada.

Reestrutura o `Makefile` seguindo o princípio de separação de responsabilidades:
**agregadores** têm nomes genéricos e apenas listam dependências; **específicos** executam
exatamente uma ferramenta. O resultado é um Makefile que serve como mapa completo do ciclo
de vida de desenvolvimento **e como fonte de verdade para os pipelines de CI/CD**.

---

## Princípio de Design

```
AGREGADORES (nomes genéricos — apenas listam dependências)
    setup      test      validate      dist      clean
       │          │           │           │          │
       ▼          ▼           ▼           ▼          ▼
ESPECÍFICOS (qualificados — executam exatamente um tool/comando)
 setup-rust  test-unit  validate-lint  dist-flatpak  clean-build
 setup-macos test-integration          dist-macos     clean-icons
 setup-linux test-i18n  validate-deps  dist-flatpak-arm
```

**Regra absoluta**: um target agregador não executa nenhum comando diretamente — ele apenas
lista dependências que são targets específicos.

**Regra de pipeline**: o `Makefile` é a fonte de verdade. Os workflows `.github/workflows/ci.yml`
e `.github/workflows/release.yml` devem chamar targets do Makefile onde possível.
`make ci` deve ser a réplica local exata do pipeline `ci.yml`.

---

## Phase 0: Auditoria de Inconsistências

Antes de implementar, leia os seguintes arquivos e registre as diferenças entre o estado
atual e este spec:

```sh
cat Makefile
cat .github/workflows/ci.yml
cat .github/workflows/release.yml
cat .config/nextest.toml
```

Inconsistências conhecidas a corrigir durante a implementação:

1. **Perfil nextest**: CI usa `--profile ci` (fail-fast=true, definido em `.config/nextest.toml`);
   Makefile atual usa `cargo test` ou `--profile default`. O Makefile redesenhado deve aceitar
   `NEXTEST_PROFILE ?= default` para que o mesmo target funcione localmente (default) e em CI
   (`make test-unit NEXTEST_PROFILE=ci`).

2. **`make ci` incorreto**: o spec anterior dizia `ci: validate test coverage` — ERRADO.
   Coverage não faz parte do pipeline de CI (foi removido intencionalmente). O target `ci`
   correto é: `ci: validate test`.

3. **`cargo audit` ausente**: o job `audit` do `ci.yml` roda `cargo audit`, mas o Makefile atual
   e o spec anterior não têm esse target. Adicionar `audit` como target específico e incluí-lo
   em `validate-deps`.

4. **`release-github` incompleto**: o spec anterior listava apenas DMG + Flatpak x86_64.
   O `release.yml` real publica 4 artefatos: flatpak-x86_64, flatpak-aarch64, macos-dmg e
   windows-zip. O target `release-github` deve incluir todos os 4.

5. **`run` não deve depender de `setup`**: `run: setup build schema` causaria `cargo fetch` +
   verificações de pacotes em cada `make run`. Correto: `run: build schema`.

6. **CI com scripts inline**: `ci.yml` duplica lógica de `check-version`, `check-potfiles`,
   `validate-metainfo`, `validate-desktop` em vez de chamar `make`. Após redesenhar o Makefile,
   atualizar `ci.yml` para chamar os targets (veja seção Pipeline Alignment).

---

## Critério de Aceitação Principal: Desenvolvedor Novo

```sh
git clone <repo> && cd gtk-cross-platform && make
```

Resultado esperado: o ambiente é configurado automaticamente para a plataforma detectada,
o binário é compilado, e a aplicação é executada — sem leitura de documentação prévia.

Requisitos:

- `.DEFAULT_GOAL := run`
- `run` depende de `build schema` (não de `setup`)
- `setup` é **idempotente** (verifica presença de cada dependência antes de instalar)
- `make` sozinho executa `run`, que executa `build schema` automaticamente

---

## Variáveis Globais

Manter todas as variáveis atuais. Adicionar ao bloco de variáveis:

```makefile
OS             := $(shell uname 2>/dev/null || echo Windows)
GIT_TAG        := $(shell git describe --tags --abbrev=0 2>/dev/null || echo "v$(VERSION)")
NEXTEST_PROFILE ?= default
```

`NEXTEST_PROFILE` permite que CI passe `make test-unit NEXTEST_PROFILE=ci` para ativar
`fail-fast = true` sem duplicar os comandos.

---

## Seção 0: Meta — `help`

```makefile
.DEFAULT_GOAL := run

help: ## Mostra todos os targets disponíveis
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
	  awk 'BEGIN {FS = ":.*?## "}; {printf "  %-26s %s\n", $$1, $$2}'
```

Cada target deve ter um comentário `## descrição` na mesma linha para aparecer em `make help`.

---

## Seção 1: SETUP

| Target             | Tipo          | Implementação                                                                                                         |
|--------------------|---------------|-----------------------------------------------------------------------------------------------------------------------|
| `setup`            | **agregador** | `setup-rust setup-platform setup-cargo-deps`                                                                          |
| `setup-rust`       | específico    | verifica `which cargo`; se ausente, instala via rustup e sourcia `$HOME/.cargo/env`                                   |
| `setup-platform`   | específico    | `case "$(OS)" in Darwin) … Linux) … *) … esac` → delega para `setup-macos/linux/windows`                              |
| `setup-macos`      | específico    | `brew list gtk4 >/dev/null 2>&1 \|\| brew install gtk4 libadwaita adwaita-icon-theme librsvg dylibbundler create-dmg` |
| `setup-linux`      | específico    | detecta `apt` ou `dnf`; verifica presença do pacote antes de instalar `libgtk-4-dev libadwaita-1-dev`                 |
| `setup-windows`    | específico    | imprime instruções MSYS2/MINGW64 e encerra com `exit 1` e mensagem orientativa clara                                  |
| `setup-cargo-deps` | específico    | `cargo fetch`                                                                                                         |

`setup-macos` e `setup-linux` devem ser idempotentes: verificar se o pacote já está instalado
antes de chamar o gerenciador de pacotes.

---

## Seção 2: BUILD

| Target          | Tipo          | Implementação                                                                                 |
|-----------------|---------------|-----------------------------------------------------------------------------------------------|
| `build`         | **agregador** | `build-debug`                                                                                 |
| `build-debug`   | específico    | `cargo build`                                                                                 |
| `build-release` | específico    | `cargo build --release`                                                                       |
| `schema`        | específico    | `glib-compile-schemas $(SCHEMA_DIR)`                                                          |
| `run`           | **agregador** | dependências `build schema`; body executa `GSETTINGS_SCHEMA_DIR=$(SCHEMA_DIR) cargo run`      |
| `run-mobile`    | **agregador** | dependências `build schema`; body executa com `GTK_DEBUG=interactive`                         |
| `watch`         | específico    | instala `cargo-watch` se ausente (`cargo install cargo-watch`), depois `cargo watch -x 'run'` |

---

## Seção 3: FORMAT & LINT

| Target      | Tipo          | Implementação                                     |
|-------------|---------------|---------------------------------------------------|
| `format`    | **agregador** | `fmt-fix lint lint-i18n`                          |
| `fmt`       | específico    | `cargo fmt --check`                               |
| `fmt-fix`   | específico    | `cargo fmt`                                       |
| `lint`      | específico    | `cargo clippy -- -D warnings`                     |
| `lint-i18n` | específico    | loop `msgfmt --check --check-format` em `po/*.po` |

---

## Seção 4: TEST

Usar `NEXTEST_PROFILE ?= default` (declarado nas Variáveis Globais) em todos os targets
de teste. CI passa `make test-unit NEXTEST_PROFILE=ci` para ativar fail-fast.

| Target             | Tipo          | Implementação                                                                                                       |
|--------------------|---------------|---------------------------------------------------------------------------------------------------------------------|
| `test`             | **agregador** | `test-unit test-integration test-i18n`                                                                              |
| `test-unit`        | específico    | `cargo nextest run --profile $(NEXTEST_PROFILE) --lib`                                                              |
| `test-integration` | específico    | `cargo nextest run --profile $(NEXTEST_PROFILE) --test container_driver_test --test greet_use_case_test`            |
| `test-i18n`        | específico    | `cargo nextest run --profile $(NEXTEST_PROFILE) --test i18n_test`                                                   |
| `coverage`         | específico    | `cargo llvm-cov --lib --test container_driver_test --test greet_use_case_test --summary-only --fail-under-lines 25` |

`coverage` existe como target utilitário para uso manual do desenvolvedor — não faz parte
do target `ci` nem do pipeline de CI.

---

## Seção 5: VALIDATE (Quality Gates)

| Target              | Tipo          | Implementação                                                                     |
|---------------------|---------------|-----------------------------------------------------------------------------------|
| `validate`          | **agregador** | `validate-format validate-lint validate-metadata validate-i18n validate-deps`     |
| `validate-format`   | **agregador** | `fmt`                                                                             |
| `validate-lint`     | **agregador** | `lint`                                                                            |
| `validate-metadata` | **agregador** | `validate-metainfo validate-desktop check-version`                                |
| `validate-i18n`     | **agregador** | `lint-i18n check-potfiles`                                                        |
| `validate-deps`     | **agregador** | `audit deny spell-check check-unused-deps`                                        |
| `validate-metainfo` | específico    | `appstreamcli validate --pedantic data/com.example.GtkCrossPlatform.metainfo.xml` |
| `validate-desktop`  | específico    | `desktop-file-validate data/com.example.GtkCrossPlatform.desktop`                 |
| `check-version`     | específico    | grep Cargo.toml vs metainfo.xml (lógica atual preservada)                         |
| `check-potfiles`    | específico    | comm entre `has_gettext.txt` e `potfiles_rs.txt` (lógica atual preservada)        |
| `audit`             | específico    | `cargo audit`                                                                     |
| `deny`              | específico    | `cargo deny check`                                                                |
| `spell-check`       | específico    | `typos .`                                                                         |
| `check-unused-deps` | específico    | `cargo machete`                                                                   |
| `ci`                | **agregador** | `validate test`                                                                   |

**`ci`** espelha exatamente o pipeline do `ci.yml`:

- `validate` = fmt + clippy + validate-metainfo + validate-desktop + check-version + check-potfiles + lint-i18n +
  audit + deny + spell-check + check-unused-deps
- `test` = test-unit + test-integration + test-i18n

O desenvolvedor roda `make ci` antes de abrir um PR. **Não inclui `coverage`** — cobertura
é uma ferramenta de análise manual, não um gate de CI.

---

## Seção 6: ICONS & ASSETS

| Target          | Tipo          | Implementação                                                                         |
|-----------------|---------------|---------------------------------------------------------------------------------------|
| `icons`         | **agregador** | `icons-png icons-macos icons-windows`                                                 |
| `icons-png`     | específico    | loop `rsvg-convert -w $$sz -h $$sz` para cada tamanho (extraído do `icons` atual)     |
| `icons-macos`   | específico    | `iconutil` + cópias (lógica atual preservada; depende de `icons-png`, não de `icons`) |
| `icons-windows` | específico    | Python one-liner ICO (lógica atual preservada; depende de `icons-png`)                |
| `install-icons` | específico    | `install -Dm644` + `gtk-update-icon-cache` (lógica atual preservada)                  |
| `clean-icons`   | específico    | `rm -f` para PNGs gerados em `data/icons/hicolor/`                                    |

---

## Seção 7: PACKAGE (Distribuição)

| Target                 | Tipo          | Implementação                                                                                                          |
|------------------------|---------------|------------------------------------------------------------------------------------------------------------------------|
| `dist`                 | **agregador** | `dist-flatpak dist-macos`                                                                                              |
| `dist-flatpak`         | específico    | `flatpak-builder --force-clean --user --install-deps-from=flathub $(FLATPAK_BUILD_DIR) $(MANIFEST)`                    |
| `dist-flatpak-arm`     | específico    | `flatpak-builder --force-clean --user --install-deps-from=flathub --arch=aarch64 $(FLATPAK_BUILD_DIR)-arm $(MANIFEST)` |
| `dist-flatpak-run`     | específico    | `flatpak-builder --run $(FLATPAK_BUILD_DIR) $(MANIFEST) $(BINARY)`                                                     |
| `dist-flatpak-install` | específico    | `flatpak-builder --user --install --force-clean $(FLATPAK_BUILD_DIR) $(MANIFEST)`                                      |
| `dist-macos`           | **agregador** | dependências `icons-macos build-release`; body contém a lógica de bundle do `dmg` atual                                |

---

## Seção 8: PUBLISH

| Target           | Tipo          | Implementação                                                                                                   |
|------------------|---------------|-----------------------------------------------------------------------------------------------------------------|
| `release`        | **agregador** | `ci dist release-tag release-github`                                                                            |
| `release-tag`    | específico    | verifica `gh` disponível; `git tag -a "v$(VERSION)" -m "Release v$(VERSION)"` + `git push origin "v$(VERSION)"` |
| `release-github` | específico    | `gh release create "v$(VERSION)" --generate-notes` com os 4 artefatos abaixo:                                   |

`release-github` deve publicar exatamente os mesmos artefatos que `release.yml` produz:

```makefile
release-github: ## Cria GitHub Release com os artefatos de todas as plataformas
	@command -v gh >/dev/null 2>&1 || { echo "ERROR: gh não encontrado — instale via https://cli.github.com"; exit 1; }
	gh release create "v$(VERSION)" --generate-notes \
		"$(FLATPAK_BUILD_DIR)/$(APP_ID).flatpak" \
		"$(FLATPAK_BUILD_DIR)-arm/$(APP_ID).flatpak" \
		"$(DMG_OUT)" \
		"GtkCrossPlatform-v$(VERSION)-windows-x86_64.zip"
```

`release` depende de `ci` — publicação só ocorre após a cadeia de qualidade completa passar.

---

## Seção 9: CLEAN & CACHE

| Target          | Tipo          | Implementação                                                                   |
|-----------------|---------------|---------------------------------------------------------------------------------|
| `clean`         | **agregador** | `clean-build clean-flatpak`                                                     |
| `clean-all`     | **agregador** | `clean clean-icons`                                                             |
| `clean-build`   | específico    | `cargo clean`                                                                   |
| `clean-flatpak` | específico    | `rm -rf $(FLATPAK_BUILD_DIR) .flatpak-builder repo`                             |
| `clean-icons`   | específico    | `rm -f` nos PNGs gerados                                                        |
| `cache-info`    | específico    | `du -sh ~/.cargo/registry ~/.cargo/git .flatpak-builder/ 2>/dev/null \|\| true` |
| `cache-prune`   | específico    | instala `cargo-cache` se ausente; `cargo cache --autoclean`                     |

---

## Aliases de Retrocompatibilidade

Adicionar bloco ao final do arquivo (antes do `.PHONY` final) para não quebrar scripts
existentes:

```makefile
# ── Aliases de retrocompatibilidade ───────────────────────────────────────────
flatpak-build:      dist-flatpak         ## [alias] use dist-flatpak
flatpak-run:        dist-flatpak-run     ## [alias] use dist-flatpak-run
flatpak-install:    dist-flatpak-install ## [alias] use dist-flatpak-install
flatpak-build-arm:  dist-flatpak-arm     ## [alias] use dist-flatpak-arm
dmg:                dist-macos           ## [alias] use dist-macos
test-nextest:       test                 ## [alias] use test
```

---

## `.PHONY` Consolidado

Substituir a declaração `.PHONY` atual por:

```makefile
.PHONY: \
  help \
  setup setup-rust setup-platform setup-macos setup-linux setup-windows setup-cargo-deps \
  build build-debug build-release schema run run-mobile watch \
  format fmt fmt-fix lint lint-i18n \
  test test-unit test-integration test-i18n coverage \
  validate validate-format validate-lint validate-metadata validate-i18n validate-deps \
  validate-metainfo validate-desktop check-version check-potfiles \
  audit deny spell-check check-unused-deps \
  ci \
  icons icons-png icons-macos icons-windows install-icons clean-icons \
  dist dist-flatpak dist-flatpak-arm dist-flatpak-run dist-flatpak-install dist-macos \
  release release-tag release-github \
  clean clean-all clean-build clean-flatpak cache-info cache-prune \
  flatpak-build flatpak-run flatpak-install flatpak-build-arm dmg test-nextest
```

---

## Pipeline Alignment: Atualizar `ci.yml` para chamar targets do Makefile

Após implementar o Makefile, o `ci.yml` deve ser atualizado para delegar aos targets
do Makefile em vez de duplicar scripts inline. Isso garante que `make ci` e o pipeline
do GitHub Actions executem exatamente os mesmos comandos.

No job `lint` de `.github/workflows/ci.yml`, substituir os steps individuais pelos
equivalentes em Makefile:

| Step atual (inline)                                               | Target Makefile                            |
|-------------------------------------------------------------------|--------------------------------------------|
| `cargo fmt --check`                                               | `make fmt`                                 |
| `cargo clippy -- -D warnings`                                     | `make lint`                                |
| `cargo nextest run --profile ci --lib`                            | `make test-unit NEXTEST_PROFILE=ci`        |
| `cargo nextest run --profile ci --test container_driver_test ...` | `make test-integration NEXTEST_PROFILE=ci` |
| `appstreamcli validate --pedantic ...`                            | `make validate-metainfo`                   |
| `desktop-file-validate ...`                                       | `make validate-desktop`                    |
| inline script de check-version                                    | `make check-version`                       |
| inline script de check-potfiles                                   | `make check-potfiles`                      |
| `make lint-i18n` (já correto)                                     | manter                                     |
| `cargo nextest run --profile ci --test i18n_test`                 | `make test-i18n NEXTEST_PROFILE=ci`        |

**Nota**: `audit`, `deny`, `typos` e `unused-deps` ficam como jobs separados no CI
(paralelismo) mas também podem ser chamados via `make audit`, `make deny`, `make spell-check`,
`make check-unused-deps` localmente.

Após atualizar o `ci.yml`, ler o arquivo `.claude/commands/redesign-workflows.md` e verificar
se as instruções ainda estão consistentes com os novos targets do Makefile. Se houver
divergências, atualizar o arquivo.

---

## Atualização do CLAUDE.md

Após implementar o Makefile, atualizar a seção **Build and Run Commands** do `CLAUDE.md`:

1. Substituir os exemplos de comandos pelos nomes novos (`dist-flatpak`, `dist-macos`, `ci`, `clean-all`)
2. Adicionar `make ci` como o comando recomendado pré-PR
3. Adicionar `make release` na seção de distribuição
4. Adicionar nota nos aliases antigos: `make flatpak-build # alias para make dist-flatpak`
5. Adicionar `make cache-info` e `make cache-prune` como comandos de manutenção

Atualizar também a tabela **Slash Commands** do `CLAUDE.md` adicionando a linha:

```
| `/project:redesign-makefile` | Reestrutura o Makefile como instrumento SOLID de ciclo de vida alinhado com CI/CD |
```

---

## Verificação

Após implementar, verificar:

1. `make help` — tabela completa com todos os targets e descrições
2. `make setup` — idempotente; segunda execução pula instalações já presentes
3. `make build` → `make lint` → `make test` → `make validate` — cada um passa individualmente
4. `make ci` — executa a cadeia completa; falha no primeiro gate quebrado
5. `make dist` — constrói pacotes sem executar validações
6. `make clean-all` — remove tudo regenerável; `make build` funciona em seguida
7. `make cache-info` — apenas informa tamanhos
8. `make flatpak-build`, `make dmg`, `make test-nextest` — aliases antigos funcionam
9. Todos os targets em `.PHONY`
10. `CLAUDE.md` atualizado com os novos nomes
11. `make test-unit NEXTEST_PROFILE=ci` — usa perfil ci (fail-fast=true)
12. `.github/workflows/ci.yml` atualizado para chamar targets do Makefile
