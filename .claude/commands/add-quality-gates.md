# /project:add-quality-gates

Implemente todos os quality gates ausentes neste projeto GTK4/Rust/Flatpak. Este comando é
auto-contido — execute-o sem contexto de conversa anterior.

**Repositório:** `gtk-cross-platform`
**Stack:** Rust 2024 edition · GTK4 0.9 · libadwaita 0.7 · glib/gio 0.20 · Flatpak/GNOME Platform 48

---

## O que ler antes de implementar

Leia os seguintes arquivos na íntegra antes de fazer qualquer mudança:

- `CLAUDE.md` — regras de arquitetura e convenções do projeto
- `.github/workflows/ci.yml` — pipeline atual (base para as modificações)
- `Makefile` — targets locais existentes (adicionar equivalentes locais dos novos gates)
- `Cargo.toml` — versão atual do pacote e lista de dependências
- `data/com.example.GtkCrossPlatform.metainfo.xml` — versão declarada no AppStream
- `data/com.example.GtkCrossPlatform.desktop` — desktop entry
- `po/POTFILES` — lista atual de arquivos registrados para i18n
- `.config/nextest.toml` — perfis nextest já configurados (ci e default)
- `.editorconfig` — escopo atual do checker

Não modifique `CLAUDE.md`, `README.md`, arquivos de domínio (`src/core/`), nem testes existentes.

---

## Gate 1 — Validação do AppStream metainfo

**Arquivo:** `.github/workflows/ci.yml`
**Job:** `lint` (adicionar passo após `i18n lint`)

Adicione um passo que instala `appstream` e valida o metainfo com flag `--pedantic`:

```yaml
-   name: Validate AppStream metadata
    run: |
        sudo apt-get install -y appstream
        appstreamcli validate --pedantic data/com.example.GtkCrossPlatform.metainfo.xml
```

**Makefile:** adicione o target `validate-metainfo`:

```makefile
validate-metainfo:
	appstreamcli validate --pedantic data/com.example.GtkCrossPlatform.metainfo.xml
```

---

## Gate 2 — Validação do .desktop file

**Arquivo:** `.github/workflows/ci.yml`
**Job:** `lint` (adicionar passo após Gate 1)

```yaml
-   name: Validate .desktop file
    run: |
        sudo apt-get install -y desktop-file-utils
        desktop-file-validate data/com.example.GtkCrossPlatform.desktop
```

**Makefile:** adicione o target `validate-desktop`:

```makefile
validate-desktop:
	desktop-file-validate data/com.example.GtkCrossPlatform.desktop
```

**Observação:** os dois `apt-get install` dos Gates 1 e 2 podem ser combinados em um único
passo de instalação no início do job para economizar tempo:

```yaml
-   name: Install validation tools
    run: sudo apt-get install -y appstream desktop-file-utils gettext
```

Se o job já instala `gettext` separadamente, remova o passo duplicado e consolide em um só.

---

## Gate 3 — Consistência de versão Cargo.toml ↔ metainfo.xml

**Arquivo:** `.github/workflows/ci.yml`
**Job:** `lint` (adicionar passo após Gate 2)

```yaml
-   name: Check version consistency (Cargo.toml vs metainfo.xml)
    run: |
        CARGO_VER=$(grep '^version' Cargo.toml | head -1 | grep -oP '[\d.]+')
        META_VER=$(grep -oP '(?<=version=")[^"]+' data/com.example.GtkCrossPlatform.metainfo.xml | head -1)
        echo "Cargo version: $CARGO_VER"
        echo "Metainfo version: $META_VER"
        [ "$CARGO_VER" = "$META_VER" ] || \
          { echo "ERROR: Version mismatch — Cargo.toml=$CARGO_VER metainfo.xml=$META_VER"; exit 1; }
```

**Makefile:** adicione o target `check-version`:

```makefile
check-version:
	@CARGO_VER=$$(grep '^version' Cargo.toml | head -1 | grep -oP '[\d.]+'); \
	 META_VER=$$(grep -oP '(?<=version=")[^"]+' data/com.example.GtkCrossPlatform.metainfo.xml | head -1); \
	 echo "Cargo: $$CARGO_VER  Metainfo: $$META_VER"; \
	 [ "$$CARGO_VER" = "$$META_VER" ] || { echo "ERROR: version mismatch"; exit 1; }
```

---

## Gate 4 — POTFILES completeness

**Arquivo:** `.github/workflows/ci.yml`
**Job:** `lint` (adicionar passo junto ao bloco de i18n)

```yaml
-   name: i18n POTFILES completeness
    run: |
        grep -rl 'gettext!(' src/ | sort > /tmp/has_gettext.txt
        sort po/POTFILES | grep '\.rs$' > /tmp/potfiles_rs.txt
        MISSING=$(comm -23 /tmp/has_gettext.txt /tmp/potfiles_rs.txt)
        if [ -n "$MISSING" ]; then
          echo "ERROR: Files with gettext!() not registered in po/POTFILES:"
          echo "$MISSING"
          exit 1
        fi
        echo "POTFILES completeness OK"
```

**Makefile:** adicione o target `check-potfiles`:

```makefile
check-potfiles:
	@grep -rl 'gettext!(' src/ | sort > /tmp/has_gettext.txt; \
	 sort po/POTFILES | grep '\.rs$$' > /tmp/potfiles_rs.txt; \
	 MISSING=$$(comm -23 /tmp/has_gettext.txt /tmp/potfiles_rs.txt); \
	 if [ -n "$$MISSING" ]; then \
	   echo "Files with gettext!() not in POTFILES:"; echo "$$MISSING"; exit 1; \
	 fi
```

---

## Gate 5 — cargo deny (licenças + dependências banidas)

### 5.1 Criar `deny.toml` na raiz do repositório

```toml
[graph]
targets = []

[advisories]
ignore = []

[licenses]
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "GPL-3.0-or-later",
    "LGPL-2.1-or-later",
    "Unicode-3.0",
]
exceptions = []

[bans]
multiple-versions = "warn"
deny = [
    # tokio conflicts with the GLib event loop (see CLAUDE.md — Threading section)
    { name = "tokio" },
]

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
```

### 5.2 Adicionar job `deny` em `.github/workflows/ci.yml`

Crie um job independente (não depende de `lint`):

```yaml
deny:
    name: cargo deny
    runs-on: ubuntu-latest
    steps:
        -   uses: actions/checkout@v4

        -   uses: EmbarkStudios/cargo-deny-action@v2
            with:
                command: check
                arguments: --all-features
```

### 5.3 Makefile

```makefile
deny:
	cargo deny check
```

---

## Gate 6 — typos (detecção de erros ortográficos)

**Arquivo:** `.github/workflows/ci.yml`
**Job:** adicionar job independente `typos`

```yaml
typos:
    name: Spell check (typos)
    runs-on: ubuntu-latest
    steps:
        -   uses: actions/checkout@v4

        -   uses: crate-ci/typos-action@v1
```

Se houver falsos positivos legítimos (termos técnicos, nomes de API), crie `.typos.toml` na
raiz com as exceções:

```toml
[default.extend-words]
# Adicione aqui exceções reais, por exemplo:
# "gio" = "gio"
```

Só crie `.typos.toml` se o passo falhar por falsos positivos ao rodar localmente com
`typos .` — não crie o arquivo preventivamente.

**Makefile:**

```makefile
spell-check:
	typos .
```

---

## Gate 7 — Coverage threshold

**Arquivo:** `.github/workflows/ci.yml`
**Job:** `coverage` (modificar o passo `Run coverage`)

Adicione `--fail-under-lines 60` ao comando existente:

```yaml
-   name: Run coverage (summary + threshold)
    run: |
        cargo llvm-cov --lib --summary-only --fail-under-lines 60
        cargo llvm-cov --test container_driver_test --summary-only
        cargo llvm-cov --test greet_use_case_test --summary-only
```

O threshold de 60% aplica-se apenas ao `--lib` (domínio + infraestrutura). Os testes de
integração são verificados separadamente sem threshold por serem de API pública.

**Makefile:** atualize o target `coverage` existente para refletir o threshold:

```makefile
coverage:
	cargo llvm-cov --lib --test container_driver_test --test greet_use_case_test \
	  --summary-only --fail-under-lines 60
```

---

## Gate 8 — Migração para cargo nextest no CI

**Arquivo:** `.github/workflows/ci.yml`
**Job:** `lint`

O `.config/nextest.toml` já define o perfil `ci` com `fail-fast = true` e
`status-level = "all"`. O CI ainda usa `cargo test`. Substitua todos os passos de `cargo test`
no job `lint` por `cargo nextest run --profile ci`:

| Comando atual                             | Substituir por                                                |
|-------------------------------------------|---------------------------------------------------------------|
| `cargo test --lib`                        | `cargo nextest run --profile ci --lib`                        |
| `cargo test --test container_driver_test` | `cargo nextest run --profile ci --test container_driver_test` |
| `cargo test --test greet_use_case_test`   | `cargo nextest run --profile ci --test greet_use_case_test`   |
| `cargo test --test i18n_test`             | `cargo nextest run --profile ci --test i18n_test`             |

Adicione a instalação do nextest antes dos passos de teste:

```yaml
-   name: Install cargo-nextest
    run: cargo install cargo-nextest --locked
```

Ou use a action oficial que é mais rápida:

```yaml
-   uses: taiki-e/install-action@cargo-nextest
```

**Makefile:** o target `test-nextest` já existe — não alterar.

---

## Gate 9 — cargo machete (dependências não usadas)

**Arquivo:** `.github/workflows/ci.yml`
**Job:** adicionar job independente `unused-deps`

```yaml
unused-deps:
    name: Unused dependencies
    runs-on: ubuntu-latest
    steps:
        -   uses: actions/checkout@v4

        -   uses: dtolnay/rust-toolchain@stable

        -   name: Install cargo-machete
            run: cargo install cargo-machete --locked

        -   name: Check for unused dependencies
            run: cargo machete
```

**Makefile:**

```makefile
check-unused-deps:
	cargo machete
```

---

## Gate 10 — Escopo do EditorConfig ampliado

**Arquivo:** `.github/workflows/ci.yml`
**Job:** `editorconfig`

O passo atual valida apenas `src/ data/ po/ tests/`. Amplie para incluir `Cargo.toml`,
`.github/` e `build.rs`:

```yaml
-   name: Run editorconfig-checker
    run: npx editorconfig-checker src/ data/ po/ tests/ Cargo.toml build.rs .github/
```

---

## Makefile — target agregador

Após implementar todos os gates, adicione um target `validate` que agrupa os checks
executáveis localmente sem CI:

```makefile
validate: check-version check-potfiles validate-metainfo validate-desktop lint lint-i18n fmt
	@echo "All local validations passed."
```

Adicione `validate` à linha `.PHONY` existente.

---

## Ordem de implementação

Execute nesta sequência para manter o CI verde a cada passo:

1. **Gates 1 e 2** — instalar ferramentas, validar metainfo e .desktop (apenas leitura; não
   falham se o arquivo já é válido)
2. **Gate 3** — verificar versão (confirmar manualmente que `Cargo.toml` e `metainfo.xml`
   estão em sincronia antes de ativar)
3. **Gate 4** — POTFILES completeness (verificar localmente com `make check-potfiles` antes
   de commitar)
4. **Gate 5** — criar `deny.toml` e job `deny` (rodar `cargo deny check` localmente primeiro;
   ajustar `allow` de licenças se necessário)
5. **Gate 6** — job `typos` (rodar `typos .` localmente; criar `.typos.toml` só se houver
   falsos positivos)
6. **Gate 8** — migrar para nextest (baixo risco; perfil `ci` já existe)
7. **Gate 7** — threshold de coverage (pode falhar se coverage atual < 60%; ajustar threshold
   para o valor real atual e depois elevar gradualmente)
8. **Gate 9** — `cargo machete` (pode identificar dependências não usadas que precisam de
   remoção ou justificativa)
9. **Gate 10** — escopo do EditorConfig (verificar localmente se novos arquivos passam)
10. **Makefile `validate`** — target agregador após todos os outros estarem estáveis

---

## Verificação final

Após implementar todos os gates:

1. Execute `make validate` localmente — deve passar sem erros.
2. Execute `cargo deny check` — deve passar sem erros de licença ou CVE.
3. Execute `typos .` — deve retornar sem erros ou apenas com exceções documentadas em `.typos.toml`.
4. Abra um PR e confirme que todos os novos jobs aparecem no CI e passam.
5. Não mergear se qualquer job novo estiver em estado `skipped` — investigar a condição de ativação.

---

## Restrições

- Não remova gates existentes (`cargo fmt`, `cargo clippy`, `cargo audit`, `editorconfig`).
- Não altere os testes em `tests/` nem o código em `src/`.
- Não adicione dependências novas ao `Cargo.toml` — todos os tools são instalados via CI action
  ou `cargo install`.
- Targets Makefile novos devem seguir o padrão existente: nome em kebab-case, sem comentários
  desnecessários, listados em `.PHONY`.
- Se um gate falhar durante a implementação por estado atual do repositório (ex: coverage < 60%,
  typo em comentário legado), corrija o problema ou ajuste o threshold antes de ativar o gate —
  nunca marque o gate como `continue-on-error: true`.
