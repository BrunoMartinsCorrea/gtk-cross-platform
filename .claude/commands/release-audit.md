# /project:release-audit

Audite o pipeline de distribuição multiplataforma deste projeto GTK4/Rust. Este comando é
auto-contido — execute-o sem contexto de conversa anterior.

**Repositório:** `gtk-cross-platform`
**Stack:** Rust · GTK4 0.9 · libadwaita 0.7 · Flatpak (GNOME Platform 48)
**Plataformas-alvo:** Linux x86_64 (Flatpak), Linux aarch64 (Flatpak), macOS arm64 (DMG), Windows x86_64 (ZIP)

---

## O que ler antes de auditar

Leia os arquivos abaixo na íntegra antes de emitir qualquer diagnóstico:

- `.github/workflows/ci.yml` — quality gates + nightly Flatpak
- `.github/workflows/release.yml` — pipeline de release por tag `v*.*.*`
- `com.example.GtkCrossPlatform.json` — manifesto Flatpak
- `Cargo.toml` — versões de dependências e features habilitadas
- `build.rs` — variáveis de build injetadas (`APP_ID`, `PROFILE`, `PKGDATADIR`, `LOCALEDIR`)
- `Makefile` — targets locais de build e Flatpak
- `CLAUDE.md` — regras de arquitetura e convenções do projeto

---

## Dimensão 1 — Sintaxe e validade dos workflows

Verifique cada item abaixo lendo o YAML dos workflows:

1. **actionlint** — execute `actionlint .github/workflows/ci.yml .github/workflows/release.yml`
   se a ferramenta estiver disponível. Reporte cada linha de erro. Se não estiver instalada,
   registre como `[aviso] actionlint não encontrado — instale com brew install actionlint`.

2. **Triggers corretos**
   - `ci.yml` deve disparar em `pull_request` e `push: branches: [main]`
   - `release.yml` deve disparar **somente** em `push: tags:` com padrão `v[0-9]*.[0-9]*.[0-9]*`
   - Nenhum workflow de release deve disparar em PRs ou push direto de branch

3. **Permissões mínimas**
   - Jobs que não publicam artefatos devem declarar `permissions: contents: read` ou omitir
     (herda o mínimo)
   - Jobs que criam GitHub Releases devem declarar `permissions: contents: write`
   - Nenhum job deve ter permissões mais amplas que o necessário

4. **Versões de actions pinadas**
   - `actions/checkout`, `actions/upload-artifact`, `actions/download-artifact`,
     `actions/cache` devem usar `@v4` ou superior
   - `flatpak/flatpak-github-actions/flatpak-builder` deve usar `@v6` ou superior
   - `msys2/setup-msys2` deve usar `@v2` ou superior
   - Reportar qualquer action em versão desatualizada ou sem pin de versão

5. **Job `publish` depende de todos os jobs de artefato**
   - `needs: [flatpak-x86_64, flatpak-aarch64, macos, windows]` deve estar presente
   - Se qualquer job de artefato falhar, o release não deve ser publicado parcialmente

---

## Dimensão 2 — Build Flatpak

Para cada job Flatpak (`flatpak-x86_64`, `flatpak-aarch64`):

1. **Imagem do container**
   - Deve usar `ghcr.io/flathub-infra/flatpak-github-actions:gnome-48`
   - Versão do SDK no manifesto JSON (`runtime-version`) deve ser `"48"` — verifique coerência

2. **Geração de `cargo-sources.json`**
   - O passo de geração deve usar `flatpak-cargo-generator.py` com `Cargo.lock` como entrada
   - O arquivo `cargo-sources.json` deve ser gerado **antes** do step `flatpak-builder`
   - Verifique se o `curl` usa `-sSfL` (falha em erros HTTP) e não silencia falhas

3. **Arquitetura aarch64**
   - O job `flatpak-aarch64` deve passar `arch: aarch64` para `flatpak-github-actions/flatpak-builder`
   - O bundle deve ter sufixo `-aarch64.flatpak` para diferenciar do x86_64
   - Builds aarch64 usam QEMU — esperado ser mais lento; não é um bug

4. **Manifesto Flatpak**
   - `finish-args` deve incluir `--socket=wayland` e `--socket=fallback-x11`
   - `--device=dri` só deve estar presente se GPU for necessária
   - O comando `cargo --offline build --release` deve usar `--offline` (fontes pré-geradas)
   - `APP_ID`, `PROFILE`, `PKGDATADIR`, `LOCALEDIR` devem estar em `build-options.env`

5. **Nomes de artefatos**
   - `ci.yml` (nightly): `com.example.GtkCrossPlatform.flatpak` (sem sufixo de arquitetura, para
     compatibilidade com o release `nightly`)
   - `release.yml`: `com.example.GtkCrossPlatform-x86_64.flatpak` e
     `com.example.GtkCrossPlatform-aarch64.flatpak` — verificar que os nomes no
     `upload-artifact` e no step `publish` são consistentes

---

## Dimensão 3 — Build macOS

1. **Runner correto**
   - Deve usar `macos-14` (Apple Silicon, arm64) — não `macos-latest` que pode mudar entre
     execuções, e não `macos-13` (Intel) que não corresponde ao artefato rotulado `arm64`

2. **Dependências Homebrew**
   - Deve instalar: `gtk4`, `libadwaita`, `dylibbundler`, `create-dmg`
   - Nenhuma dessas deve ser assumida como pré-instalada no runner

3. **Variáveis de build (`build.rs`)**
   - `cargo build --release` deve ser executado com:
     ```
     APP_ID=com.example.GtkCrossPlatform
     PROFILE=default
     PKGDATADIR=../Resources/share/gtk-cross-platform
     LOCALEDIR=../Resources/share/locale
     ```
   - Verifique se essas variáveis estão declaradas como `env:` no step de build

4. **Estrutura do `.app` bundle**
   - `Contents/MacOS/gtk-cross-platform` — binário principal
   - `Contents/Frameworks/` — dylibs re-linkadas por `dylibbundler`
   - `Contents/Resources/share/glib-2.0/schemas/gschemas.compiled` — **obrigatório** para
     GTK4/Adwaita funcionar em runtime
   - `Contents/Info.plist` — deve conter `CFBundleIdentifier`, `CFBundleExecutable`,
     `NSHighResolutionCapable: true`, `LSMinimumSystemVersion`

5. **Caminho correto do `gschemas.compiled`**
   - O arquivo NÃO está em `$(brew --prefix glib)/share/glib-2.0/schemas/`
   - Está em `$(brew --prefix)/share/glib-2.0/schemas/gschemas.compiled` (prefix global do
     Homebrew, que agrega schemas de todos os formulários instalados)
   - Este é um bug silencioso: o workflow falharia em CI sem esta distinção

6. **`dylibbundler`**
   - Deve ser invocado com `-od -b -x <binary> -d <Frameworks/> -p @executable_path/../Frameworks/`
   - Após execução, `otool -L` no binário NÃO deve conter nenhum path `/opt/homebrew/` ou
     `/usr/local/`; todos os links devem ser `@executable_path/..` ou `/usr/lib`/`/System`
   - Execute `otool -L Contents/MacOS/gtk-cross-platform | grep -v "@executable_path\|/usr/lib\|/System"`
     e verifique que a saída está vazia
   - **Contagem mínima de dylibs:** `ls Contents/Frameworks/ | wc -l` deve retornar ≥ 20 para
     GTK4+Adwaita; valor abaixo indica que o bundling falhou silenciosamente
   - **Falso positivo esperado:** dylibbundler executa `codesign --sign -` (assinatura ad-hoc)
     e imprime `replacing existing signature` — isso é comportamento correto, não um erro.
     Gatekeeper ainda bloqueará o app no primeiro launch (sem Apple Developer certificate).

7. **DMG**
   - `create-dmg` deve incluir `--app-drop-link` para o atalho de instalação
   - Nome do arquivo deve incluir a versão: `GtkCrossPlatform-<tag>-macos-arm64.dmg`
   - Verificar que o nome no `upload-artifact` e no step `publish` são idênticos
   - **Falso positivo esperado:** create-dmg imprime `hdiutil does not support internet-enable`
     em macOS ≥ 10.15 — a flag foi removida do sistema e não afeta o DMG gerado. Ignorar.
   - **Tamanho esperado:** `.app` bundle ~25–50 MB (GTK4+Adwaita+dependências), DMG comprimido
     ~8–15 MB. Valores muito abaixo indicam que dylibs não foram coletadas.

8. **Cache do Homebrew**
   - O workflow atual não cacheia pacotes Homebrew — `brew install gtk4 libadwaita dylibbundler
     create-dmg` leva ~5–10 min a frio no runner macOS-14.
   - Gap de performance: considere adicionar `actions/cache` para `$(brew --prefix)/Cellar` ou
     usar `HOMEBREW_NO_AUTO_UPDATE=1` + `HOMEBREW_NO_INSTALL_CLEANUP=1` como `env:` para
     acelerar mesmo sem cache completo.

---

## Dimensão 4 — Build Windows

1. **Runner e shell**
   - Deve usar `windows-latest` com `defaults: run: shell: msys2 {0}`
   - Steps que usam PowerShell (`Compress-Archive`) devem declarar `shell: pwsh` explicitamente
     para sobrescrever o default

2. **Pacotes MSYS2 MINGW64**
   - Mínimo obrigatório: `gtk4`, `libadwaita`, `rust`, `pkg-config`, `gettext`, `gettext-tools`,
     `adwaita-icon-theme`
   - Sem `adwaita-icon-theme`, ícones de UI ficam ausentes em runtime

3. **Variáveis de build (`build.rs`)**
   - `cargo build --release` deve ser executado com:
     ```
     APP_ID=com.example.GtkCrossPlatform
     PROFILE=default
     PKGDATADIR=./share/gtk-cross-platform
     LOCALEDIR=./share/locale
     ```

4. **DLL bundling**
   - `ldd` deve filtrar por `mingw64` (sem `/` inicial) para capturar paths como
     `/mingw64/bin/libgtk-4-1.dll`
   - Filtro `grep -i '/mingw64/'` com `/` no início pode silenciosamente omitir DLLs em algumas
     versões do `ldd` — prefira `grep -i 'mingw64'`
   - Execute mentalmente o pipe:
     `ldd gtk-cross-platform.exe | grep -i 'mingw64' | awk '{print $3}'`
     e verifique que a coluna 3 corresponde ao path completo da DLL

5. **Runtime data obrigatório no ZIP**
   - `dist/share/glib-2.0/schemas/gschemas.compiled` — sem este arquivo GTK4 não abre
   - `dist/share/icons/hicolor/index.theme` — fallback de ícones
   - `dist/lib/gdk-pixbuf-2.0/` — loaders para PNG/SVG
   - Fonte: `/mingw64/share/glib-2.0/schemas/gschemas.compiled`,
     `/mingw64/share/icons/hicolor/index.theme`,
     `/mingw64/lib/gdk-pixbuf-2.0/`

6. **Nome do ZIP**
   - Deve incluir a versão: `GtkCrossPlatform-<tag>-windows-x86_64.zip`
   - Verificar coerência entre o nome gerado no step de compressão, `upload-artifact` e `publish`

7. **CARGO_HOME e cache no Windows/MSYS2**
   - O job usa `defaults: run: shell: msys2 {0}`, mas o step `actions/cache` é um `uses:` (não
     um `run:`) — ele executa no shell padrão do runner, que é **PowerShell**, não MSYS2.
   - O `~` em PowerShell resolve para `C:\Users\runneradmin` (Windows home).
   - O `~` em MSYS2 resolve para o home MSYS2, que pode ser diferente.
   - Se MSYS2's rust definir `CARGO_HOME` como `/home/runneradmin/.cargo` (MSYS2 path) em vez
     de `C:\Users\runneradmin\.cargo` (Windows path), o cache do `actions/cache` nunca vai bater.
   - Verificação: o step de cache deve usar o caminho absoluto do Windows ou validar que
     `$CARGO_HOME` (dentro do shell MSYS2) mapeia para o mesmo local que `~/.cargo` no PowerShell.

---

## Dimensão 5 — Publicação de release

1. **Job `publish`**
   - Deve usar `actions/download-artifact@v4` sem especificar `name` (baixa todos os artefatos
     em subdiretórios separados)
   - `gh release create` deve referenciar todos os 4 artefatos com caminhos exatos:
     - `artifacts/flatpak-x86_64/com.example.GtkCrossPlatform-x86_64.flatpak`
     - `artifacts/flatpak-aarch64/com.example.GtkCrossPlatform-aarch64.flatpak`
     - `artifacts/macos-dmg/GtkCrossPlatform-<tag>-macos-arm64.dmg`
     - `artifacts/windows-zip/GtkCrossPlatform-<tag>-windows-x86_64.zip`
   - `<tag>` deve ser `${{ github.ref_name }}` — verificar interpolação correta

2. **Release nightly vs. versionada**
   - `ci.yml` publica release `nightly` (prerelease) a cada push em `main` — correto
   - `release.yml` publica release versionada (`v*.*.*`) sem `--prerelease` — correto
   - Os dois workflows não devem criar releases com o mesmo nome/tag

3. **`--generate-notes`**
   - O step `gh release create` deve usar `--generate-notes` para auto-gerar changelog
   - Alternativa aceitável: `--notes-file CHANGELOG.md` — mas deve existir e estar atualizado

4. **`GH_TOKEN`**
   - `secrets.GITHUB_TOKEN` é suficiente para criar releases no próprio repositório
   - Não deve usar PATs pessoais (`secrets.GH_PAT`) — desnecessário e aumenta superfície de ataque

---

## Dimensão 6 — Cache e performance de CI

1. **Cache Cargo**
   - Todos os jobs Rust devem cachear: `~/.cargo/registry/index/`, `~/.cargo/registry/cache/`,
     `~/.cargo/git/db/`, `target/`
   - A chave deve incluir `${{ hashFiles('**/Cargo.lock') }}` para invalidar quando deps mudam
   - `restore-keys` deve ter fallback sem hash para aproveitar caches parciais

2. **Cache Flatpak**
   - `cache-key` deve ser único por job: `flatpak-release-x86_64-<tag>` e
     `flatpak-release-aarch64-<tag>`
   - Não compartilhar cache entre x86_64 e aarch64 — arquiteturas diferentes, artefatos
     incompatíveis

3. **Jobs paralelos**
   - `flatpak-x86_64`, `flatpak-aarch64`, `macos`, `windows` devem rodar em paralelo (sem
     `needs` entre eles)
   - Apenas `publish` deve ter `needs` em todos os quatro

---

## Como executar esta auditoria

### Passo 1 — Leitura estática (sempre)
Leia todos os arquivos listados em "O que ler antes de auditar" e execute as verificações das
Dimensões 1–6 com base no conteúdo observado.

### Passo 2 — Validação local (se em macOS)
Se o ambiente for macOS, execute as seguintes verificações ativas:

```bash
# 1. Sintaxe dos workflows
actionlint .github/workflows/ci.yml .github/workflows/release.yml

# 2. Build de release com env vars de produção
APP_ID=com.example.GtkCrossPlatform \
PROFILE=default \
PKGDATADIR=../Resources/share/gtk-cross-platform \
LOCALEDIR=../Resources/share/locale \
cargo build --release 2>&1 | tail -3

# 3. Verificar caminho do gschemas.compiled
ls "$(brew --prefix)/share/glib-2.0/schemas/gschemas.compiled" && echo "OK" || echo "AUSENTE"

# 4. Criar bundle de teste e validar re-linkagem
APP="AuditTest.app"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Frameworks"
cp target/release/gtk-cross-platform "$APP/Contents/MacOS/"

# dylibbundler imprimirá "replacing existing signature" — isso é esperado (codesign ad-hoc)
dylibbundler -od -b -x "$APP/Contents/MacOS/gtk-cross-platform" \
  -d "$APP/Contents/Frameworks/" -p "@executable_path/../Frameworks/" 2>&1 | tail -3

# Verificar zero paths absolutos Homebrew
otool -L "$APP/Contents/MacOS/gtk-cross-platform" \
  | grep -v "@executable_path\|/usr/lib\|/System" \
  | grep -v "gtk-cross-platform:" \
  | tee /dev/stderr | wc -l | xargs -I{} sh -c \
  '[ {} -eq 0 ] && echo "PASS: zero absolute Homebrew paths" || echo "FAIL: {} paths restantes"'

# Verificar contagem mínima de dylibs (≥ 20 para GTK4+Adwaita)
DYLIB_COUNT=$(ls "$APP/Contents/Frameworks/" | wc -l | tr -d ' ')
[ "$DYLIB_COUNT" -ge 20 ] \
  && echo "PASS: $DYLIB_COUNT dylibs bundled" \
  || echo "WARN: apenas $DYLIB_COUNT dylibs — bundling pode ter falhado silenciosamente"

# Verificar tamanho do bundle (esperado: 25–50 MB)
du -sh "$APP/" | awk '{print "Bundle size: "$1" (esperado: 25–50 MB)"}'

rm -rf "$APP"
```

### Passo 3 — Relatório

---

## Formato do relatório

```markdown
# Auditoria de Release Pipeline — gtk-cross-platform

## Scorecard

| Dimensão                   | Status | Problemas |
|----------------------------|--------|-----------|
| 1. Sintaxe de workflows    | ✅/⚠️/❌ | n         |
| 2. Build Flatpak           | ✅/⚠️/❌ | n         |
| 3. Build macOS             | ✅/⚠️/❌ | n         |
| 4. Build Windows           | ✅/⚠️/❌ | n         |
| 5. Publicação de release   | ✅/⚠️/❌ | n         |
| 6. Cache e performance     | ✅/⚠️/❌ | n         |

✅ = sem problemas · ⚠️ = avisos não-bloqueantes · ❌ = falha bloqueante em CI

---

## Problemas encontrados

Para cada problema, reporte:

**[SEVERIDADE] Dimensão → Item → Arquivo:linha**
> Descrição do problema.
> Impacto: o que quebra em CI/runtime se não for corrigido.
> Correção: diff ou instrução exata.

Severidades:
- `[BLOQUEANTE]` — o pipeline falha ou o artefato não funciona no dispositivo do usuário
- `[AVISO]` — degradação silenciosa (cache miss, ícone ausente, i18n em inglês)
- `[MELHORIA]` — não quebra, mas reduz qualidade ou tempo de CI

---

## Validações locais (se executadas)

Reporte o resultado de cada comando do Passo 2:
- `actionlint`: PASS / FAIL (com output completo em caso de FAIL)
- `cargo build --release`: PASS / FAIL (últimas 3 linhas; tempo de compilação observado)
- `gschemas.compiled`: FOUND / ABSENT (reportar o path completo encontrado)
- `dylibbundler` + `otool`: PASS / FAIL (número de paths absolutos restantes)
- Contagem de dylibs: N dylibs (PASS se ≥ 20, WARN se < 20)
- Tamanho do bundle: N MB (PASS se 25–50 MB, WARN fora da faixa)

Warnings esperados que NÃO devem ser reportados como falha:
- `replacing existing signature` — dylibbundler assina ad-hoc; normal
- `hdiutil does not support internet-enable` — removido no macOS 10.15; ignorar

---

## Plano de correção

Liste apenas os itens BLOQUEANTES e AVISOS, em ordem de prioridade:

| # | Arquivo | Linha | Correção | Esforço |
|---|---------|-------|----------|---------|
| 1 | ...     | ...   | ...      | 5 min   |
```

---

## Restrições

- Baseie todos os diagnósticos no conteúdo observável dos arquivos; não assuma comportamentos
  não documentados
- Quando um item não puder ser verificado localmente (Windows, aarch64), indique explicitamente
  `[análise estática apenas]`
- Não repita gaps cobertos por `/project:compliance-audit` (i18n, A11Y, arquitetura hexagonal)
- Priorize por impacto no usuário final: artefato que não roda > artefato incompleto > CI lento
