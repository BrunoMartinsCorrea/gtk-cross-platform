# /project:redesign-workflows

> **Pré-requisito**: execute `/project:redesign-makefile` antes deste comando. Os targets
> do Makefile precisam existir para que o `ci.yml` possa delegar para eles.

Consolida os GitHub Actions workflows em exatamente **2 arquivos** e atualiza o `ci.yml`
para delegar aos targets do `Makefile` em vez de duplicar scripts inline:

1. **`.github/workflows/ci.yml`** — quality gate completo, dispara apenas em pull request
2. **`.github/workflows/release.yml`** — build e publicação de artefatos, dispara apenas em tag de versão

---

## Step 1 — Audit existing workflows

Leia todos os arquivos em `.github/workflows/`. Para cada arquivo, registre:

- O trigger `on:`
- Cada nome de job e o que ele faz
- Condições `if:` que limitam jobs a eventos específicos

Leia também o `Makefile` para confirmar que os targets referenciados no Step 2 existem.
Se `/project:redesign-makefile` ainda não foi executado, os targets `validate-metainfo`,
`check-version`, `check-potfiles`, `test-unit`, `test-integration`, `test-i18n` podem
não existir — nesse caso, execute `/project:redesign-makefile` primeiro.

---

## Step 2 — Rewrite `ci.yml`

Substitua `.github/workflows/ci.yml` pela estrutura abaixo.

**Regras:**

- Trigger **apenas** em `pull_request` com types `[opened, synchronize, reopened]`.
- Sem trigger `push:`, `schedule:`, nem `workflow_dispatch:`.
- O job `lint` **delega para targets do Makefile** onde possível, passando
  `NEXTEST_PROFILE=ci` para ativar fail-fast nos testes.
- Incluir exatamente estes jobs:
    - `lint` — todos os checks de qualidade via Makefile targets (detalhado abaixo)
    - `editorconfig` — `npx editorconfig-checker` nos paths atuais
    - `audit` — cargo-audit (job paralelo; ferramenta instala via `cargo install`)
    - `deny` — cargo-deny action
    - `typos` — typos-action
    - `unused-deps` — cargo-machete (job paralelo; `make check-unused-deps`)
- **Remover** o job `flatpak` se presente (publicação nightly pertence a outra concern).
- **Remover** o job `coverage` se presente (não faz parte do gate de CI).

### Job `lint` — steps via Makefile targets

```yaml
lint:
  name: Lint & test
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4

    - uses: dtolnay/rust-toolchain@stable
      with:
        components: clippy, rustfmt

    - name: Cache cargo
      uses: actions/cache@v4
      with:
        path: |
          ~/.cargo/registry/index/
          ~/.cargo/registry/cache/
          ~/.cargo/git/db/
          target/
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
        restore-keys: ${{ runner.os }}-cargo-

    - uses: taiki-e/install-action@cargo-nextest

    - name: Install validation tools
      run: sudo apt-get install -y appstream desktop-file-utils gettext

    - name: Check formatting
      run: make fmt

    - name: Clippy
      run: make lint

    - name: Unit tests
      run: make test-unit NEXTEST_PROFILE=ci

    - name: Integration tests — container driver + greet use case
      run: make test-integration NEXTEST_PROFILE=ci

    - name: Validate AppStream metadata
      run: make validate-metainfo

    - name: Validate .desktop file
      run: make validate-desktop

    - name: Check version consistency (Cargo.toml vs metainfo.xml)
      run: make check-version

    - name: i18n lint (msgfmt --check)
      run: make lint-i18n

    - name: i18n POTFILES completeness
      run: make check-potfiles

    - name: i18n structural tests
      run: make test-i18n NEXTEST_PROFILE=ci
```

### Jobs paralelos (sem dependência de Makefile)

```yaml
  editorconfig:
    name: EditorConfig compliance
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run editorconfig-checker
        run: npx editorconfig-checker src/ data/ po/ tests/ Cargo.toml build.rs .github/

  audit:
    name: cargo audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
          key: ${{ runner.os }}-cargo-audit-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: ${{ runner.os }}-cargo-audit-
      - name: Install cargo-audit
        run: cargo install cargo-audit --locked
      - name: Run audit
        run: make audit

  deny:
    name: cargo deny
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: EmbarkStudios/cargo-deny-action@v2
        with:
          command: check
          arguments: --all-features

  typos:
    name: Spell check (typos)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: crate-ci/typos-action@v1

  unused-deps:
    name: Unused dependencies
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install cargo-machete
        run: cargo install cargo-machete --locked
      - name: Check for unused dependencies
        run: make check-unused-deps
```

**Estrutura final do arquivo:**

```yaml
name: CI

on:
    pull_request:
        types: [ opened, synchronize, reopened ]

jobs:
    lint:        # via make targets
    editorconfig:
    audit:
    deny:
    typos:
    unused-deps:
```

---

## Step 3 — Leave `release.yml` untouched

`.github/workflows/release.yml` já tem o trigger correto (`push: tags: ['v[0-9]*.[0-9]*.[0-9]*']`)
e os jobs corretos (`flatpak-x86_64`, `flatpak-aarch64`, `macos`, `windows`, `publish`).
**Não modificar.**

O `make release` local cria e publica a tag git, o que dispara este workflow automaticamente.

---

## Step 4 — Delete redundant workflow files

Se existirem, deletar (a lógica já está consolidada em `ci.yml`):

- `.github/workflows/audit.yml`
- `.github/workflows/editorconfig.yml`

---

## Step 5 — Verify

Execute e reporte os resultados:

```sh
# Confirmar exatamente 2 workflow files
ls .github/workflows/

# Confirmar trigger do ci.yml
grep -A8 "^on:" .github/workflows/ci.yml

# Confirmar trigger do release.yml
grep -A8 "^on:" .github/workflows/release.yml

# Validar YAML de ambos os arquivos
python3 -c "import yaml, sys; yaml.safe_load(open('.github/workflows/ci.yml')); print('ci.yml OK')"
python3 -c "import yaml, sys; yaml.safe_load(open('.github/workflows/release.yml')); print('release.yml OK')"

# Confirmar que steps do lint job usam make targets
grep "run: make" .github/workflows/ci.yml
```

Resultado esperado:

- `ls` mostra apenas `ci.yml` e `release.yml`
- `ci.yml` `on:` contém apenas `pull_request:` com `types: [opened, synchronize, reopened]`
- `release.yml` `on:` contém apenas `push: tags: ['v[0-9]*.[0-9]*.[0-9]*']`
- Ambos os arquivos fazem parse como YAML válido
- Steps do job `lint` chamam `make fmt`, `make lint`, `make test-unit`, etc.
