# /project:release-audit

Audite o pipeline de distribuição multiplataforma deste projeto GTK4/Rust. Este comando é
auto-contido — execute-o sem contexto de conversa anterior.

**Repositório:** `gtk-cross-platform`
**Stack:** Rust · GTK4 0.9 · libadwaita 0.7 · Flatpak (GNOME Platform 48)
**Plataformas-alvo:** Linux x86_64 (Flatpak), Linux aarch64 (Flatpak), macOS arm64 (DMG), Windows x86_64 (ZIP)

---

## Execução imediata

Execute este bloco AGORA, antes de qualquer análise. Cada comando produz evidência usada no
relatório. Não pule etapas; registre a saída literal de cada um.

```bash
# 1. Presença dos arquivos críticos
for f in \
  .github/workflows/ci.yml \
  .github/workflows/release.yml \
  com.example.GtkCrossPlatform.json \
  Cargo.toml \
  data/com.example.GtkCrossPlatform.gschema.xml \
  data/com.example.GtkCrossPlatform.metainfo.xml \
  data/com.example.GtkCrossPlatform.desktop \
  build.rs \
  Makefile; do
  [ -f "$f" ] && echo "FOUND  $f" || echo "MISSING $f"
done

# 2. Versão declarada em Cargo.toml
grep '^version' Cargo.toml | head -1

# 3. Versão declarada em metainfo.xml
grep '<release ' data/com.example.GtkCrossPlatform.metainfo.xml | head -3

# 4. Triggers do ci.yml (deve incluir push: branches: [main])
grep -A5 '^on:' .github/workflows/ci.yml

# 5. Triggers do release.yml (deve ser apenas push: tags:)
grep -A5 '^on:' .github/workflows/release.yml

# 6. Permissões declaradas em cada job do release.yml
grep -n 'permissions' .github/workflows/release.yml

# 7. Versões de actions em release.yml
grep -E 'uses: (actions|flatpak|msys2|dtolnay)' .github/workflows/release.yml

# 8. Versões de actions em ci.yml
grep -E 'uses: (actions|taiki-e|EmbarkStudios|crate-ci)' .github/workflows/ci.yml

# 9. GSettings schema no manifesto Flatpak (deve conter glib-compile-schemas)
grep -n 'glib-compile-schemas\|gschema' com.example.GtkCrossPlatform.json

# 10. flatpak-cargo-generator.py: URL e pin (master = UNPINNED)
grep -n 'flatpak-cargo-generator' .github/workflows/release.yml

# 11. finish-args do Flatpak
grep -A10 'finish-args' com.example.GtkCrossPlatform.json

# 12. runtime-version do manifesto
grep 'runtime-version' com.example.GtkCrossPlatform.json

# 13. build-options.env no manifesto (APP_ID, PROFILE, PKGDATADIR, LOCALEDIR)
grep -A15 'build-options' com.example.GtkCrossPlatform.json

# 14. Nightly Flatpak no ci.yml (deve ter job de build Flatpak)
grep -n 'flatpak\|nightly' .github/workflows/ci.yml

# 15. Job publish no release.yml: needs todos os artefatos
grep -A5 'needs:' .github/workflows/release.yml

# 16. actionlint (se disponível)
command -v actionlint >/dev/null 2>&1 \
  && actionlint .github/workflows/ci.yml .github/workflows/release.yml \
  || echo "[aviso] actionlint não encontrado — instale com: brew install actionlint"

# 17. Runner macOS (deve ser macos-14, não macos-latest)
grep 'runs-on' .github/workflows/release.yml

# 18. Homebrew deps no macOS job
grep -A5 'brew install' .github/workflows/release.yml

# 19. DLL filter no Windows job (mingw64 sem barra inicial)
grep -n 'grep.*mingw64\|ldd' .github/workflows/release.yml

# 20. Windows runtime data obrigatório
grep -n 'gschemas.compiled\|gdk-pixbuf\|index.theme' .github/workflows/release.yml

# 21. gh release create: artefatos referenciados
grep -A15 'gh release create' .github/workflows/release.yml

# 22. Cache Cargo: presença de hashFiles(Cargo.lock) em todos os jobs Rust
grep -n 'hashFiles' .github/workflows/release.yml

# 23. Paralelismo: flatpak-x86_64, flatpak-aarch64, macos, windows sem needs entre si
grep -B2 'needs:' .github/workflows/release.yml
```

---

## Dimensão 0 — Consistência de versão e identidade

Verifique usando os outputs do bloco de execução imediata:

1. **Cargo.toml == metainfo.xml**
   - `version` em `Cargo.toml` deve ser idêntico ao atributo `version` na última `<release>`
     em `metainfo.xml`
   - Divergência = `[BLOQUEANTE]`: o release tag `v<X>` não corresponde ao binário compilado

2. **App ID consistente em todos os arquivos**
   - `com.example.GtkCrossPlatform` deve aparecer idêntico em:
     - `com.example.GtkCrossPlatform.json` → campo `app-id`
     - `Cargo.toml` → build env `APP_ID`
     - `release.yml` → nomes de artefatos e `gh release create`
     - `ci.yml` → upload-artifact names (se houver Flatpak nightly)
   - Qualquer variante (case, separador, sufixo extra) = `[BLOQUEANTE]`

3. **Trigger de release só em tags semver**
   - `release.yml` deve disparar **somente** em `push: tags: ['v[0-9]*.[0-9]*.[0-9]*']`
   - Ausência do padrão exato = risco de release acidental em tags de formato errado

4. **ci.yml deve disparar em `push: branches: [main]`**
   - Sem este trigger, pushes diretos em `main` (e.g., merge squash) não executam CI
   - Output do comando 4 deve mostrar `push:` com `branches: [main]` além de `pull_request:`
   - Ausência = `[BLOQUEANTE]`: regressões chegam a `main` sem verificação

---

## Dimensão 1 — Sintaxe e validade dos workflows

Com base nos outputs dos comandos 6–8 e 16:

1. **actionlint** — reporte cada linha de erro do comando 16. Se não instalado, indique
   `[aviso] actionlint não encontrado` e prossiga com análise estática.

2. **Permissões mínimas por job** (comando 6)
   - `flatpak-x86_64`, `flatpak-aarch64`: `permissions: contents: read` ✅
   - `macos`, `windows`: devem ter `permissions:` declarado explicitamente; herança implícita
     do nível de repositório é um risco de segurança = `[AVISO]`
   - `publish`: `permissions: contents: write` obrigatório = `[BLOQUEANTE]` se ausente

3. **Versões de actions pinadas** (comandos 7–8)
   - `actions/checkout`, `upload-artifact`, `download-artifact`, `cache` → `@v4` ou superior
   - `flatpak/flatpak-github-actions/flatpak-builder` → `@v6` ou superior
   - `msys2/setup-msys2` → `@v2` ou superior
   - `dtolnay/rust-toolchain` → `@stable` é aceito (semver implícito)
   - Qualquer `@v1`, `@v2`, `@v3` em actions que têm `@v4` disponível = `[AVISO]`
   - Qualquer action sem pin de versão (`@main`, sem @) = `[BLOQUEANTE]`

4. **Paralelismo correto** (comando 23)
   - `flatpak-x86_64`, `flatpak-aarch64`, `macos`, `windows` NÃO devem ter `needs:` entre si
   - Apenas `publish` deve ter `needs: [flatpak-x86_64, flatpak-aarch64, macos, windows]`
   - Qualquer `needs:` extra entre jobs de build = `[MELHORIA]` (serializa desnecessariamente)

---

## Dimensão 2 — Build Flatpak

Com base nos outputs dos comandos 9–13:

1. **Imagem do container**
   - Deve usar `ghcr.io/flathub-infra/flatpak-github-actions:gnome-48`
   - `runtime-version` no manifesto (comando 12) deve ser `"48"` — inconsistência = `[BLOQUEANTE]`

2. **flatpak-cargo-generator.py — pin de versão** (comando 10)
   - URL deve apontar para um commit SHA fixo, **não** `master`
   - Download de `master` = quebra silenciosa quando o script muda API
   - Se `master` aparecer na saída: `[AVISO]`
   - Correção: substituir `master` por um SHA recente verificado:
     ```
     https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/<SHA>/cargo/flatpak-cargo-generator.py
     ```

3. **GSettings schema no manifesto** (comando 9)
   - O manifesto DEVE ter `glib-compile-schemas` em `build-commands` se
     `data/com.example.GtkCrossPlatform.gschema.xml` existe no repositório
   - Ausência de `glib-compile-schemas` = `[BLOQUEANTE]`: GSettings não funciona no sandbox
   - Correção a adicionar em `build-commands` após install do binário:
     ```
     "install -Dm644 data/com.example.GtkCrossPlatform.gschema.xml /app/share/glib-2.0/schemas/com.example.GtkCrossPlatform.gschema.xml",
     "glib-compile-schemas /app/share/glib-2.0/schemas/"
     ```

4. **Manifesto Flatpak — finish-args** (comando 11)
   - Obrigatórios: `--socket=wayland`, `--socket=fallback-x11`, `--share=ipc`
   - `--device=dri` só se GPU for necessária — reportar presença como `[AVISO]`
   - Ausência de `--socket=wayland` = `[BLOQUEANTE]` (app não abre no GNOME)

5. **build-options.env** (comando 13)
   - `APP_ID`, `PROFILE`, `PKGDATADIR`, `LOCALEDIR` devem estar presentes
   - `PKGDATADIR=/app/share/gtk-cross-platform` (path Flatpak)
   - `LOCALEDIR=/app/share/locale`
   - Ausência de qualquer variável = `[BLOQUEANTE]`: binário compilado com paths errados

6. **Nomes de artefatos Flatpak**
   - `release.yml`: `com.example.GtkCrossPlatform-x86_64.flatpak` e
     `com.example.GtkCrossPlatform-aarch64.flatpak`
   - Nome no `bundle:`, `upload-artifact path:` e path no `publish` devem ser idênticos
   - Qualquer divergência = `[BLOQUEANTE]`: `download-artifact` não encontra o arquivo

7. **Arquitetura aarch64**
   - Job `flatpak-aarch64` deve passar `arch: aarch64` para a action
   - Ausência = build x86_64 enviado com nome `aarch64` = `[BLOQUEANTE]`

---

## Dimensão 3 — Build macOS

Com base nos outputs dos comandos 17–18 e validação local abaixo:

1. **Runner** (comando 17)
   - Deve ser `macos-14` — não `macos-latest` (muda entre execuções), não `macos-13` (Intel)
   - `macos-latest` = `[AVISO]`: artefato pode ser x86_64 em vez de arm64

2. **Dependências Homebrew** (comando 18)
   - Obrigatórias: `gtk4`, `libadwaita`, `dylibbundler`, `create-dmg`
   - Ausência de qualquer uma = `[BLOQUEANTE]`

3. **Variáveis de build**
   - `APP_ID=com.example.GtkCrossPlatform`
   - `PROFILE=default`
   - `PKGDATADIR=../Resources/share/gtk-cross-platform`
   - `LOCALEDIR=../Resources/share/locale`
   - Ausência de `env:` no step de build = `[BLOQUEANTE]`: `config.rs` compila com paths errados

4. **Info.plist — campos obrigatórios**
   - `CFBundleIdentifier`: deve ser `com.example.GtkCrossPlatform`
   - `CFBundleExecutable`: deve ser `gtk-cross-platform`
   - `NSHighResolutionCapable`: `true`
   - `LSMinimumSystemVersion`: deve existir (e.g., `12.0`)
   - Ausência de qualquer campo = `[BLOQUEANTE]` (Gatekeeper rejeita o bundle)

5. **Caminho do `gschemas.compiled`**
   - Deve usar `$(brew --prefix)/share/glib-2.0/schemas/gschemas.compiled`
   - NÃO `$(brew --prefix glib)/share/...` (path errado — GTK4/Adwaita não abre)
   - Este é um bug silencioso; CI só detecta em runtime = `[BLOQUEANTE]`

6. **`dylibbundler` — flags obrigatórias**
   - Deve ter `-od -b -x <binary> -d <Frameworks/> -p @executable_path/../Frameworks/`
   - Ausência de `-p @executable_path/...` = links absolutos Homebrew no binário = `[BLOQUEANTE]`

7. **Cache do Homebrew**
   - Ausência de `actions/cache` para Homebrew = `[MELHORIA]`: +5–10 min por build
   - Mitigação mínima aceitável: `HOMEBREW_NO_AUTO_UPDATE=1 HOMEBREW_NO_INSTALL_CLEANUP=1` como `env:`

---

## Dimensão 4 — Build Windows

Com base nos outputs dos comandos 19–20:

1. **Runner e shell**
   - Deve usar `windows-latest` com `defaults: run: shell: msys2 {0}`
   - Steps com `Compress-Archive` devem declarar `shell: pwsh` explicitamente

2. **Pacotes MSYS2 MINGW64**
   - Obrigatórios: `gtk4`, `libadwaita`, `rust`, `pkg-config`, `gettext`, `gettext-tools`,
     `adwaita-icon-theme`
   - Sem `adwaita-icon-theme` = ícones ausentes em runtime = `[AVISO]`

3. **DLL bundling — filtro correto** (comando 19)
   - Filtro deve ser `grep -i 'mingw64'` (sem `/` inicial)
   - `grep -i '/mingw64/'` pode omitir DLLs em algumas versões do `ldd` = `[AVISO]`
   - Verificar coluna do `awk`: `$3` é o path completo quando `ldd` usa formato
     `name => /path (0x...)` — confirmar pela saída observada

4. **Runtime data obrigatório no ZIP** (comando 20)
   - `dist/share/glib-2.0/schemas/gschemas.compiled` — ausente = GTK4 não abre = `[BLOQUEANTE]`
   - `dist/share/icons/hicolor/index.theme` — ausente = ícones sem fallback = `[AVISO]`
   - `dist/lib/gdk-pixbuf-2.0/` — ausente = PNG/SVG não renderizam = `[AVISO]`

5. **CARGO_HOME e cache no Windows/MSYS2**
   - `actions/cache` executa em PowerShell (não MSYS2); `~` resolve para `C:\Users\runneradmin`
   - Se MSYS2 definir `CARGO_HOME` como path Unix, cache nunca bate = `[AVISO]`
   - Verificação: o `key:` usa `hashFiles('**/Cargo.lock')` — confirmar presença no output
     do comando 22

6. **Nome do ZIP**
   - Deve incluir a versão: `GtkCrossPlatform-${{ github.ref_name }}-windows-x86_64.zip`
   - Nome no step de compressão, `upload-artifact` e `publish` devem ser idênticos = `[BLOQUEANTE]`

---

## Dimensão 5 — Publicação de release

Com base no output do comando 21:

1. **`download-artifact` sem `name:`**
   - Deve baixar todos os artefatos em subdiretórios separados (`path: artifacts`)
   - Especificar `name:` força download de um único artefato = `[BLOQUEANTE]`

2. **Paths exatos no `gh release create`**
   - `artifacts/flatpak-x86_64/com.example.GtkCrossPlatform-x86_64.flatpak`
   - `artifacts/flatpak-aarch64/com.example.GtkCrossPlatform-aarch64.flatpak`
   - `artifacts/macos-dmg/GtkCrossPlatform-<tag>-macos-arm64.dmg`
   - `artifacts/windows-zip/GtkCrossPlatform-<tag>-windows-x86_64.zip`
   - `<tag>` deve ser `${{ github.ref_name }}` — verificar interpolação
   - Path errado = `[BLOQUEANTE]`: artefato ausente no release

3. **`GH_TOKEN`**
   - Deve usar `secrets.GITHUB_TOKEN` — não PATs pessoais
   - PAT pessoal = superfície de ataque desnecessária = `[AVISO]`

4. **`--generate-notes`**
   - Deve estar presente no `gh release create` ou substituído por `--notes-file CHANGELOG.md`
   - Ausência = release sem changelog = `[AVISO]`

5. **Nightly vs. versionado**
   - `ci.yml` deve ter job de Flatpak nightly (`nightly` release, `--prerelease`) em `push: branches: [main]`
   - `release.yml` deve criar release versionada sem `--prerelease`
   - Se `ci.yml` não tiver o trigger `push: branches: [main]` nem o job Flatpak nightly:
     reportar como `[AVISO]`: builds intermediários não são publicados

---

## Dimensão 6 — Cache e performance de CI

Com base nos outputs dos comandos 22–23:

1. **Cache Cargo em todos os jobs Rust**
   - Todos os jobs com `cargo build` devem cachear:
     `~/.cargo/registry/index/`, `~/.cargo/registry/cache/`, `~/.cargo/git/db/`, `target/`
   - Chave deve incluir `${{ hashFiles('**/Cargo.lock') }}`
   - `restore-keys` deve ter fallback sem hash
   - Job sem cache = `[AVISO]`: +2–5 min por build

2. **Cache Flatpak — keys únicas por job**
   - `flatpak-release-x86_64-<tag>` e `flatpak-release-aarch64-<tag>` não devem compartilhar prefix
   - Compartilhamento = artefato de arquitetura errada restaurado = `[BLOQUEANTE]`

3. **Jobs paralelos**
   - `flatpak-x86_64`, `flatpak-aarch64`, `macos`, `windows` sem `needs:` entre si
   - Qualquer serialização = `[MELHORIA]`: aumenta tempo total de release

---

## Validação local (macOS)

Se o ambiente atual for macOS, execute este bloco **imediatamente**:

```bash
# Build de release com env vars de produção
APP_ID=com.example.GtkCrossPlatform \
PROFILE=default \
PKGDATADIR=../Resources/share/gtk-cross-platform \
LOCALEDIR=../Resources/share/locale \
cargo build --release 2>&1 | tail -5

# Verificar caminho correto do gschemas.compiled
SCHEMA_PATH="$(brew --prefix)/share/glib-2.0/schemas/gschemas.compiled"
[ -f "$SCHEMA_PATH" ] && echo "PASS gschemas: $SCHEMA_PATH" || echo "FAIL gschemas: não encontrado"

# Bundle de teste
APP="AuditTest.app"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Frameworks"
cp target/release/gtk-cross-platform "$APP/Contents/MacOS/"

# dylibbundler — "replacing existing signature" é esperado (codesign ad-hoc)
dylibbundler -od -b \
  -x "$APP/Contents/MacOS/gtk-cross-platform" \
  -d "$APP/Contents/Frameworks/" \
  -p "@executable_path/../Frameworks/" 2>&1 | grep -v 'replacing existing signature'

# FAIL se houver paths absolutos Homebrew residuais
ABSOLUTE=$(otool -L "$APP/Contents/MacOS/gtk-cross-platform" \
  | grep -v "@executable_path\|/usr/lib\|/System\|gtk-cross-platform:" \
  | wc -l | tr -d ' ')
[ "$ABSOLUTE" -eq 0 ] \
  && echo "PASS dylibs: zero paths absolutos Homebrew" \
  || echo "FAIL dylibs: $ABSOLUTE paths absolutos residuais"

# Contagem mínima de dylibs bundled (≥ 20 para GTK4+Adwaita)
COUNT=$(ls "$APP/Contents/Frameworks/" | wc -l | tr -d ' ')
[ "$COUNT" -ge 20 ] \
  && echo "PASS bundle: $COUNT dylibs" \
  || echo "WARN bundle: $COUNT dylibs — menos que o esperado (≥ 20)"

# Tamanho do bundle (esperado: 25–50 MB)
du -sh "$APP/" | awk '{print "Bundle size: "$1" (esperado: 25–50 MB)"}'

rm -rf "$APP"
```

Warnings esperados que **não** devem ser reportados como falha:
- `replacing existing signature` — codesign ad-hoc; normal
- `hdiutil does not support internet-enable` — removido no macOS 10.15; ignorar

---

## Formato do relatório

```markdown
# Auditoria de Release Pipeline — gtk-cross-platform

## Scorecard

| Dimensão                        | Status   | Bloqueantes | Avisos |
|---------------------------------|----------|-------------|--------|
| 0. Consistência de versão       | ✅/⚠️/❌ | n           | n      |
| 1. Sintaxe de workflows         | ✅/⚠️/❌ | n           | n      |
| 2. Build Flatpak                | ✅/⚠️/❌ | n           | n      |
| 3. Build macOS                  | ✅/⚠️/❌ | n           | n      |
| 4. Build Windows                | ✅/⚠️/❌ | n           | n      |
| 5. Publicação de release        | ✅/⚠️/❌ | n           | n      |
| 6. Cache e performance          | ✅/⚠️/❌ | n           | n      |

✅ = sem problemas · ⚠️ = avisos não-bloqueantes · ❌ = falha bloqueante em CI

---

## Problemas encontrados

Para cada problema:

**[SEVERIDADE] Dimensão N → Item → arquivo:linha**
> Evidência: output literal do comando que detectou o problema.
> Impacto: o que quebra em CI ou no dispositivo do usuário.
> Correção exata: diff ou comando completo.

Severidades:
- `[BLOQUEANTE]` — pipeline falha ou artefato não roda no dispositivo do usuário
- `[AVISO]` — degradação silenciosa (cache miss, ícone ausente, i18n em inglês, risco de segurança)
- `[MELHORIA]` — não quebra, mas reduz qualidade ou aumenta tempo de CI

---

## Validações locais (se executadas)

- `cargo build --release`: PASS / FAIL (últimas 5 linhas)
- `gschemas.compiled`: PASS path / FAIL
- `dylibbundler` + paths absolutos: PASS / FAIL (N paths residuais)
- Contagem de dylibs: N (PASS ≥ 20 / WARN < 20)
- Tamanho do bundle: N MB (PASS 25–50 MB / WARN fora da faixa)

---

## Plano de correção (somente BLOQUEANTES e AVISOS)

Ordene por impacto no usuário final: artefato que não roda > artefato incompleto > CI lento.

| # | Arquivo | Linha | Correção resumida | Esforço |
|---|---------|-------|-------------------|---------|
| 1 | …       | …     | …                 | 5 min   |
```

---

## Restrições

- Baseie todos os diagnósticos no output literal dos comandos executados; nunca assuma
  comportamentos não observados
- Quando um item não puder ser verificado localmente (Windows, aarch64), indique
  `[análise estática apenas]` e detalhe o raciocínio
- Não repita gaps cobertos por `/project:compliance-audit` (i18n, A11Y, arquitetura hexagonal)
- Para cada `[BLOQUEANTE]`, forneça o diff ou comando exato de correção — não apenas descrição
