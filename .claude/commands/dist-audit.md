# /project:dist-audit

Audite os artefatos finais de distribuição deste projeto GTK4/Rust e verifique se o que o
usuário recebe está correto e completo. Este comando é auto-contido — execute-o sem contexto
de conversa anterior.

**Repositório:** `gtk-cross-platform`
**Stack:** Rust · GTK4 0.9 · libadwaita 0.7 · Flatpak (GNOME Platform 48)
**Artefatos esperados:** Flatpak x86_64, Flatpak aarch64, macOS DMG (arm64), Windows ZIP (x86_64)

> **Escopo distinto de `/project:release-audit`:** este comando audita o *conteúdo* dos
> artefatos distribuídos (o que o usuário instala e executa), não o pipeline de CI que os produz.
> Não duplique verificações de workflow YAML, cache Cargo, ou runners de CI.

---

## O que ler antes de auditar

Leia os arquivos abaixo na íntegra antes de emitir qualquer diagnóstico:

- `Cargo.toml` — versão do pacote, App ID (`APP_ID` via `build.rs`)
- `build.rs` — variáveis injetadas em tempo de compilação
- `com.example.GtkCrossPlatform.json` — manifesto Flatpak (permissões, módulos, env vars)
- `data/com.example.GtkCrossPlatform.metainfo.xml` — AppStream metainfo
- `data/com.example.GtkCrossPlatform.desktop` — desktop entry
- `data/com.example.GtkCrossPlatform.gschema.xml` — GSettings schema
- `data/resources/resources.gresource.xml` — manifesto GResource
- `data/resources/window.ui` — template principal da janela
- `data/icons/hicolor/` — ícones em múltiplas resoluções
- `CHANGELOG.md` — versão mais recente documentada
- `.github/workflows/ci.yml` e `.github/workflows/release.yml` — nomes dos artefatos publicados
- `Makefile` — targets de `dist-*`
- `CLAUDE.md` — regras de arquitetura e identidade do projeto

---

## Dimensão 1 — Identidade e Consistência de Versão

Verifique se a identidade do aplicativo é coerente em todos os pontos de distribuição:

1. **App ID**
    - `Cargo.toml` → campo `APP_ID` em `[package.metadata]` ou `build.rs`
    - `com.example.GtkCrossPlatform.json` → campo `app-id`
    - `data/com.example.GtkCrossPlatform.desktop` → campo `StartupWMClass` e nome do arquivo
    - `data/com.example.GtkCrossPlatform.metainfo.xml` → campo `<id>`
    - `data/com.example.GtkCrossPlatform.gschema.xml` → atributo `id` do schema
    - Todos devem ser idênticos (`com.example.GtkCrossPlatform`). Reporte qualquer divergência.

2. **Versão**
    - `Cargo.toml [package].version` — fonte de verdade
    - `data/com.example.GtkCrossPlatform.metainfo.xml` → `<release version="...">` mais recente
    - `CHANGELOG.md` → título da release mais recente (`## [x.y.z]`)
    - Nome dos artefatos no `release.yml` — devem conter a versão como `${{ github.ref_name }}`
    - Execute: `grep -E '^version\s*=' Cargo.toml` e compare com
      `grep '<release' data/com.example.GtkCrossPlatform.metainfo.xml`
    - Reporte qualquer mismatch. A fonte de verdade é `Cargo.toml`.

3. **Display name**
    - `data/com.example.GtkCrossPlatform.desktop` → `Name=`
    - `data/com.example.GtkCrossPlatform.metainfo.xml` → `<name>`
    - Devem ser idênticos e usar o mesmo capitalização.

4. **App ID de placeholder**
    - Se `app-id` contiver `com.example`, marque como `[AVISO]` — deve ser substituído por um
      ID real (reverse-domain) antes de publicar no Flathub.

---

## Dimensão 2 — Completude do Artefato Flatpak

Para o bundle `.flatpak`, verifique se o manifesto garante todos os arquivos necessários em
runtime. Analise `com.example.GtkCrossPlatform.json`:

1. **GResource compilado**
    - O passo de build deve executar `glib-compile-resources` ou usar `glib-build-tools` (via
      `build.rs`) para gerar `compiled.gresource`
    - O arquivo `.gresource` deve ser instalado em `$PKGDATADIR` (ex: `share/gtk-cross-platform/`)
    - Verifique que `PKGDATADIR` em `build-options.env` corresponde ao caminho de instalação no
      manifesto

2. **GSettings schema**
    - `data/com.example.GtkCrossPlatform.gschema.xml` deve ser instalado em
      `share/glib-2.0/schemas/`
    - Após `glib-compile-schemas`, deve existir `share/glib-2.0/schemas/gschemas.compiled`
    - Sem este arquivo, qualquer `gio::Settings::new(...)` lança panic em runtime

3. **Ícones**
    - `data/icons/hicolor/` deve conter ícones nas resoluções: 16, 32, 48, 128, 256, 512 px
    - Execute: `find data/icons/hicolor -name '*.png' | sort`
    - Deve existir ao menos um `.svg` em `scalable/apps/` para renderização vetorial
    - Todos os ícones devem ter o nome exato do App ID: `com.example.GtkCrossPlatform.png`

4. **Desktop entry e metainfo**
    - `data/com.example.GtkCrossPlatform.desktop` deve ser instalado em
      `share/applications/`
    - `data/com.example.GtkCrossPlatform.metainfo.xml` deve ser instalado em
      `share/metainfo/` (caminho moderno) — não `share/appdata/` (legado)
    - Execute: `grep -r 'metainfo\|appdata' com.example.GtkCrossPlatform.json`

5. **Localização (i18n)**
    - Arquivos `.mo` compilados devem ser instalados em `$LOCALEDIR/<locale>/LC_MESSAGES/`
    - Verifique que `LOCALEDIR` em `build-options.env` é consistente com o caminho real de
      instalação no manifesto Flatpak
    - Sem os `.mo`, todas as strings aparecem em inglês independentemente do locale do sistema

6. **Permissões de sandbox**
    - `finish-args` obrigatórios: `--socket=wayland`, `--socket=fallback-x11`, `--share=ipc`
    - Permissões que devem estar **ausentes** (princípio do menor privilégio):
        - `--filesystem=home` — nunca necessário para um gerenciador de containers
        - `--share=network` — conexão ao daemon Docker/Podman é via socket Unix, não rede
        - `--device=all` — não requer acesso a hardware de entrada/saída
    - `--socket=session-bus` só deve aparecer se o app usa D-Bus de sessão

---

## Dimensão 3 — Completude do Bundle macOS

Analise o Makefile (`dist-macos` / `dmg`) e o workflow `release.yml` (job `macos`):

1. **Estrutura do `.app`**
   Verifique que o script de bundle cria a estrutura correta:
   ```
   GtkCrossPlatform.app/
   ├── Contents/
   │   ├── Info.plist
   │   ├── MacOS/
   │   │   └── gtk-cross-platform           # binário principal
   │   ├── Frameworks/                      # dylibs re-linkadas por dylibbundler
   │   └── Resources/
   │       ├── share/
   │       │   ├── gtk-cross-platform/      # GResource compilado ($PKGDATADIR)
   │       │   │   └── compiled.gresource
   │       │   ├── glib-2.0/schemas/
   │       │   │   └── gschemas.compiled    # OBRIGATÓRIO
   │       │   ├── icons/hicolor/           # ícones do app
   │       │   └── locale/                  # arquivos .mo ($LOCALEDIR)
   │       └── lib/
   │           └── gdk-pixbuf-2.0/         # pixel buffer loaders
   ```

2. **`Info.plist`**
   Deve conter todos os campos obrigatórios:
    - `CFBundleIdentifier` — deve ser igual ao App ID
    - `CFBundleExecutable` — deve ser `gtk-cross-platform`
    - `CFBundleName` — nome de exibição
    - `CFBundleVersion` e `CFBundleShortVersionString` — versão do app
    - `NSHighResolutionCapable: true` — sem isso, UI fica borrada em Retina
    - `LSMinimumSystemVersion` — deve declarar versão mínima do macOS
    - `NSHumanReadableCopyright` — linha de copyright
      Execute: `grep -A1 'CFBundle\|NSHighResolution\|LSMinimum' <Info.plist>` se o arquivo existir

3. **Dependências runtime (dylibs)**
    - Após `dylibbundler`, nenhum path em `otool -L` deve apontar para `/opt/homebrew/` ou
      `/usr/local/` — todos devem ser `@executable_path/../Frameworks/` ou paths do sistema
      (`/usr/lib`, `/System/`)
    - Contagem mínima de dylibs em `Contents/Frameworks/`: **≥ 20** para GTK4+Adwaita
    - Se o Makefile não executar `dylibbundler`, o bundle não é redistributável

4. **`gschemas.compiled` no bundle**
    - O arquivo deve ser copiado de `$(brew --prefix)/share/glib-2.0/schemas/gschemas.compiled`
      (prefix global do Homebrew, não o de um formula específico)
    - Verifique no Makefile se o step de bundle copia este arquivo
    - Sem ele, GTK4/Adwaita lança `g_settings_new: assertion schema not found` em runtime

5. **Tamanho esperado do bundle**
    - `.app`: 25–50 MB (GTK4+Adwaita+deps)
    - `.dmg`: 8–15 MB (comprimido)
    - Valores abaixo indicam que dylibs ou dados runtime não foram incluídos

6. **DMG**
    - Deve conter `--app-drop-link` (atalho para `/Applications`) para instalação por drag-and-drop
    - Nome do arquivo deve incluir versão e arquitetura: `GtkCrossPlatform-<version>-macos-arm64.dmg`

---

## Dimensão 4 — Completude do Bundle Windows

Analise o workflow `release.yml` (job `windows`) e o Makefile:

1. **Estrutura do ZIP**
   Verifique que o script de bundle cria:
   ```
   GtkCrossPlatform-<version>-windows-x86_64/
   ├── gtk-cross-platform.exe
   ├── *.dll                               # DLLs MINGW64 copiadas por ldd
   ├── share/
   │   ├── gtk-cross-platform/
   │   │   └── compiled.gresource          # GResource compilado
   │   ├── glib-2.0/schemas/
   │   │   └── gschemas.compiled           # OBRIGATÓRIO
   │   ├── icons/hicolor/
   │   │   ├── index.theme                 # fallback de ícones
   │   │   └── ...
   │   └── locale/                         # arquivos .mo
   └── lib/
       └── gdk-pixbuf-2.0/               # pixel buffer loaders (PNG/SVG)
   ```

2. **DLL bundling**
    - `ldd gtk-cross-platform.exe | grep -i 'mingw64' | awk '{print $3}'` deve produzir lista
      não vazia de DLLs
    - Filtro deve usar `mingw64` sem `/` inicial (algumas versões de `ldd` omitem a barra)
    - DLLs críticas que devem aparecer: `libgtk-4-1.dll`, `libadwaita-1-0.dll`, `libglib-2.0-0.dll`,
      `libgobject-2.0-0.dll`, `libgio-2.0-0.dll`, `libpango-1.0-0.dll`, `libcairo-2.dll`

3. **Runtime data obrigatório**
    - `share/glib-2.0/schemas/gschemas.compiled` — sem este, GTK4 não abre
    - `share/icons/hicolor/index.theme` — sem este, ícones ficam ausentes
    - `lib/gdk-pixbuf-2.0/` com loaders — sem este, imagens PNG não renderizam
    - Fonte: `/mingw64/share/glib-2.0/schemas/`, `/mingw64/share/icons/hicolor/`,
      `/mingw64/lib/gdk-pixbuf-2.0/`

4. **Sem installer nativo**
    - O ZIP é o único formato de distribuição Windows atualmente — reporte como `[MELHORIA]`
      a ausência de um installer NSIS/WiX para integração com "Add/Remove Programs" do Windows

---

## Dimensão 5 — Conformidade com Lojas e Repositórios

1. **Flathub**
    - `app-id` não deve ser `com.example.*` — ID placeholder bloqueia submissão
    - `metainfo.xml` deve passar `appstreamcli validate --pedantic`; execute se disponível:
      ```bash
      appstreamcli validate data/com.example.GtkCrossPlatform.metainfo.xml
      ```
    - `<release>` deve ter `date=` no formato ISO 8601 (`YYYY-MM-DD`)
    - `<url type="homepage">`, `<url type="bugtracker">`, `<url type="vcs-browser">` devem apontar
      para URLs reais (não `https://example.com`)
    - `<screenshots>` — pelo menos uma screenshot válida (URL ou arquivo local) é obrigatório para
      Flathub
    - Licença em `<metadata_license>` deve ser `CC0-1.0`; `<project_license>` deve ser a licença
      real do código (ex: `GPL-3.0-or-later`)

2. **GNOME Software / KDE Discover**
    - `desktop` file deve ter `Categories=` com pelo menos uma categoria válida do FreeDesktop
    - `desktop` file deve ter `Keywords=` para pesquisa no GNOME Software
    - Execute se disponível:
      ```bash
      desktop-file-validate data/com.example.GtkCrossPlatform.desktop
      ```

3. **macOS Gatekeeper**
    - Sem assinatura com Apple Developer certificate, o app é bloqueado no primeiro launch
    - Codesign ad-hoc (`codesign --sign -`) feito pelo `dylibbundler` não satisfaz Gatekeeper
    - Reporte como `[MELHORIA]` a ausência de notarização — não é bloqueante para distribuição
      via GitHub Releases, mas bloqueia a Mac App Store

4. **Windows SmartScreen**
    - Sem Authenticode signing, Windows exibe aviso de "Unknown publisher" no primeiro launch
    - Reporte como `[MELHORIA]` — não bloqueia execução, mas reduz confiança do usuário

---

## Dimensão 6 — Experiência de Primeira Instalação

Simule mentalmente o fluxo de um usuário novo em cada plataforma:

1. **Flatpak (Linux)**
    - O usuário executa `flatpak install com.example.GtkCrossPlatform.flatpak`
    - Dependência de runtime: `org.gnome.Platform//48` deve estar no Flathub; se não estiver,
      a instalação falha — verifique que o manifesto declara `runtime-version: "48"`
    - O usuário executa o app: não deve haver mensagem de erro sobre schema ausente, ícone ausente,
      ou locale não encontrado

2. **macOS (DMG)**
    - O usuário monta o DMG e arrasta o `.app` para `/Applications`
    - O usuário abre o app — Gatekeeper solicita confirmação (esperado sem cert)
    - O app não deve falhar com `dyld: Library not loaded` (indica que dylibbundler não rodou)
    - O app não deve falhar com `g_settings_new: Failed to get schema` (indica falta de
      `gschemas.compiled`)

3. **Windows (ZIP)**
    - O usuário extrai o ZIP e executa `gtk-cross-platform.exe`
    - O app não deve falhar com `The code execution cannot proceed because libgtk-4-1.dll was
     not found` (indica DLL ausente no ZIP)
    - O app não deve falhar com schema ou GResource ausente

4. **Verificação de locale**
    - Se o sistema do usuário estiver em PT-BR, o app deve exibir strings em português
    - Valide que os arquivos `.mo` estão no path correto para cada plataforma

---

## Dimensão 7 — Integridade dos Ícones

1. **Resoluções presentes**
   Execute: `find data/icons -name '*.png' -o -name '*.svg' | sort`
   Resoluções mínimas esperadas: 16, 32, 48, 128, 256, 512 px + SVG scalable
   Reporte resoluções ausentes.

2. **Nomenclatura correta**
    - Todos os ícones devem se chamar `com.example.GtkCrossPlatform.<ext>`
    - Ícones com nome incorreto não são encontrados pelo GNOME Shell / Launcher

3. **SVG válido**
    - O SVG em `scalable/apps/` deve ter `viewBox` quadrado (ex: `0 0 256 256`)
    - viewBox não-quadrado causa distorção na renderização pelo GNOME Shell
    - Execute: `grep viewBox data/icons/hicolor/scalable/apps/com.example.GtkCrossPlatform.svg`

4. **Ícone no bundle macOS**
    - O bundle macOS deve conter um `.icns` ou copiar os PNGs do hicolor para
      `Contents/Resources/` com o nome correto
    - Sem ícone no bundle, o macOS exibe o ícone genérico de aplicação

---

## Como executar esta auditoria

### Passo 1 — Análise estática (sempre)

Leia todos os arquivos listados em "O que ler antes de auditar" e execute as verificações de
todas as dimensões com base no conteúdo observado.

### Passo 2 — Verificações locais (se no macOS)

```bash
# 1. Validar metainfo AppStream
appstreamcli validate data/com.example.GtkCrossPlatform.metainfo.xml 2>&1 || true

# 2. Validar desktop file
desktop-file-validate data/com.example.GtkCrossPlatform.desktop 2>&1 || true

# 3. Consistência de versão
CARGO_VERSION=$(grep -m1 '^version' Cargo.toml | sed 's/.*= "\(.*\)"/\1/')
METAINFO_VERSION=$(grep '<release' data/com.example.GtkCrossPlatform.metainfo.xml | head -1 | sed 's/.*version="\([^"]*\)".*/\1/')
echo "Cargo: $CARGO_VERSION | Metainfo: $METAINFO_VERSION"
[ "$CARGO_VERSION" = "$METAINFO_VERSION" ] && echo "PASS: versões coerentes" || echo "FAIL: mismatch de versão"

# 4. Ícones presentes
find data/icons/hicolor -name '*.png' | wc -l | xargs -I{} sh -c \
  '[ {} -ge 6 ] && echo "PASS: {} ícones PNG" || echo "WARN: apenas {} ícones PNG (esperado ≥ 6)"'

# 5. SVG com viewBox quadrado
SVG_VIEWBOX=$(grep -o 'viewBox="[^"]*"' data/icons/hicolor/scalable/apps/com.example.GtkCrossPlatform.svg 2>/dev/null || echo "NÃO ENCONTRADO")
echo "SVG viewBox: $SVG_VIEWBOX"

# 6. GSettings schema presente
[ -f data/com.example.GtkCrossPlatform.gschema.xml ] \
  && echo "PASS: gschema.xml presente" \
  || echo "FAIL: gschema.xml ausente — GTK4 Settings não funcionarão"

# 7. GResource manifest lista os arquivos corretos
GRESOURCE_FILES=$(grep '<file' data/resources/resources.gresource.xml | wc -l)
echo "GResource: $GRESOURCE_FILES arquivos declarados"

# 8. POTFILES lista somente arquivos que existem
while IFS= read -r path; do
  [ -f "$path" ] || echo "STALE POTFILES: $path não existe"
done < po/POTFILES
```

### Passo 3 — Relatório

---

## Formato do relatório

```markdown
# Auditoria de Distribuição — gtk-cross-platform

## Scorecard

| Dimensão                            | Status   | Problemas |
|-------------------------------------|----------|-----------|
| 1. Identidade e versão              | ✅/⚠️/❌ | n         |
| 2. Artefato Flatpak                 | ✅/⚠️/❌ | n         |
| 3. Bundle macOS                     | ✅/⚠️/❌ | n         |
| 4. Bundle Windows                   | ✅/⚠️/❌ | n         |
| 5. Conformidade com lojas           | ✅/⚠️/❌ | n         |
| 6. Experiência de primeira instalação | ✅/⚠️/❌ | n       |
| 7. Integridade de ícones            | ✅/⚠️/❌ | n         |

✅ = sem problemas · ⚠️ = degradação não-bloqueante · ❌ = impede o usuário de instalar ou executar

---

## Problemas encontrados

Para cada problema:

**[SEVERIDADE] Dimensão → Item**
> Arquivo: `<path>:<linha>`
> Problema: descrição objetiva do que está errado.
> Impacto para o usuário: o que falha no dispositivo final.
> Correção: diff ou instrução exata.

Severidades:

- `[BLOQUEANTE]` — o usuário não consegue instalar ou o app trava ao abrir
- `[AVISO]` — degradação silenciosa (ícone ausente, strings em inglês, aviso de segurança do SO)
- `[MELHORIA]` — não quebra, mas reduz qualidade percebida ou compliance com lojas

---

## Verificações locais executadas

- `appstreamcli validate`: PASS / FAIL / NÃO INSTALADO
- `desktop-file-validate`: PASS / FAIL / NÃO INSTALADO
- Consistência de versão: PASS `x.y.z` / FAIL (`Cargo: x.y.z` vs `Metainfo: a.b.c`)
- Contagem de ícones PNG: N (PASS ≥ 6 / WARN < 6)
- SVG viewBox: `<valor>` (PASS quadrado / WARN não-quadrado)
- GSettings schema: PASS / FAIL
- GResource manifest: N arquivos declarados
- POTFILES stale: N caminhos inválidos

---

## Plano de correção

Apenas itens BLOQUEANTES e AVISOS, em ordem de prioridade:

| # | Arquivo | Problema | Correção | Esforço |
|---|---------|----------|----------|---------|
| 1 | ...     | ...      | ...      | 5 min   |
```

---

## Restrições

- Baseie todos os diagnósticos no conteúdo observável dos arquivos; não assuma comportamentos
  não documentados
- Quando não for possível verificar um artefato binário (o bundle não existe localmente), indique
  `[análise estática apenas]` e verifique o script que o gera
- Não duplique gaps de pipeline já cobertos por `/project:release-audit` (sintaxe de workflows,
  cache Cargo, jobs paralelos, runners de CI)
- Não duplique gaps de conformidade já cobertos por `/project:compliance-audit` (i18n inline,
  A11Y, breakpoints, arquitetura hexagonal)
- Priorize pelo impacto no usuário final: **app que não abre > dados ausentes em runtime >
  compliance com lojas > melhorias de polish**
