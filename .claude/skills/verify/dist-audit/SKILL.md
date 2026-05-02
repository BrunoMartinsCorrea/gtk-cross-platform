---
name: verify:dist-audit
version: 1.0.0
description: Audit distributed artifacts (Flatpak, macOS DMG, Windows ZIP) for completeness and store compliance
---

# verify:dist-audit

Audit the final distribution artifacts of this GTK4/Rust project and verify that what the
user receives is correct and complete. This command is self-contained — run it fresh without
prior conversation context.

**Repository:** `gtk-cross-platform`
**Stack:** Rust · GTK4 0.9 · libadwaita 0.7 · Flatpak (GNOME Platform 48)
**Expected artifacts:** Flatpak x86_64, Flatpak aarch64, macOS DMG (arm64), Windows ZIP (x86_64)

> **Distinct scope from `/verify:release-audit`:** this command audits the *content* of
> distributed artifacts (what the user installs and runs), not the CI pipeline that produces them.
> Do not duplicate checks for workflow YAML, Cargo cache, or CI runners.

---

## What to read before auditing

Read the files below in full before emitting any diagnosis:

- `Cargo.toml` — package version, App ID (`APP_ID` via `build.rs`)
- `build.rs` — variables injected at compile time
- `com.example.GtkCrossPlatform.json` — Flatpak manifest (permissions, modules, env vars)
- `data/com.example.GtkCrossPlatform.metainfo.xml` — AppStream metainfo
- `data/com.example.GtkCrossPlatform.desktop` — desktop entry
- `data/com.example.GtkCrossPlatform.gschema.xml` — GSettings schema
- `data/resources/resources.gresource.xml` — GResource manifest
- `data/resources/window.ui` — main window template
- `data/icons/hicolor/` — icons at multiple resolutions
- `CHANGELOG.md` — most recently documented version
- `.github/workflows/ci.yml` and `.github/workflows/release.yml` — published artifact names
- `Makefile` — `dist-*` targets
- `CLAUDE.md` — architecture rules and project identity

---

## Dimension 1 — Identity and Version Consistency

Verify that the application identity is coherent across all distribution points:

1. **App ID**
    - `Cargo.toml` → `APP_ID` field in `[package.metadata]` or `build.rs`
    - `com.example.GtkCrossPlatform.json` → `app-id` field
    - `data/com.example.GtkCrossPlatform.desktop` → `StartupWMClass` field and filename
    - `data/com.example.GtkCrossPlatform.metainfo.xml` → `<id>` field
    - `data/com.example.GtkCrossPlatform.gschema.xml` → schema `id` attribute
    - All must be identical (`com.example.GtkCrossPlatform`). Report any divergence.

2. **Version**
    - `Cargo.toml [package].version` — source of truth
    - `data/com.example.GtkCrossPlatform.metainfo.xml` → most recent `<release version="...">`
    - `CHANGELOG.md` → most recent release title (`## [x.y.z]`)
    - Artifact names in `release.yml` — must contain the version as `${{ github.ref_name }}`
    - Run: `grep -E '^version\s*=' Cargo.toml` and compare with
      `grep '<release' data/com.example.GtkCrossPlatform.metainfo.xml`
    - Report any mismatch. The source of truth is `Cargo.toml`.

3. **Display name**
    - `data/com.example.GtkCrossPlatform.desktop` → `Name=`
    - `data/com.example.GtkCrossPlatform.metainfo.xml` → `<name>`
    - Must be identical and use the same capitalisation.

4. **Placeholder App ID**
    - If `app-id` contains `com.example`, mark as `[WARNING]` — must be replaced with a real
      reverse-domain ID before publishing to Flathub.

---

## Dimension 2 — Flatpak Artifact Completeness

For the `.flatpak` bundle, verify that the manifest guarantees all required files at
runtime. Analyse `com.example.GtkCrossPlatform.json`:

1. **Compiled GResource**
    - The build step must run `glib-compile-resources` or use `glib-build-tools` (via
      `build.rs`) to generate `compiled.gresource`
    - The `.gresource` file must be installed in `$PKGDATADIR` (e.g. `share/gtk-cross-platform/`)
    - Verify that `PKGDATADIR` in `build-options.env` matches the installation path in the manifest

2. **GSettings schema**
    - `data/com.example.GtkCrossPlatform.gschema.xml` must be installed in
      `share/glib-2.0/schemas/`
    - After `glib-compile-schemas`, `share/glib-2.0/schemas/gschemas.compiled` must exist
    - Without this file, any `gio::Settings::new(...)` panics at runtime

3. **Icons**
    - `data/icons/hicolor/` must contain icons at resolutions: 16, 32, 48, 128, 256, 512 px
    - Run: `find data/icons/hicolor -name '*.png' | sort`
    - At least one `.svg` must exist in `scalable/apps/` for vector rendering
    - All icons must be named exactly after the App ID: `com.example.GtkCrossPlatform.png`

4. **Desktop entry and metainfo**
    - `data/com.example.GtkCrossPlatform.desktop` must be installed in
      `share/applications/`
    - `data/com.example.GtkCrossPlatform.metainfo.xml` must be installed in
      `share/metainfo/` (modern path) — not `share/appdata/` (legacy)
    - Run: `grep -r 'metainfo\|appdata' com.example.GtkCrossPlatform.json`

5. **Localisation (i18n)**
    - Compiled `.mo` files must be installed in `$LOCALEDIR/<locale>/LC_MESSAGES/`
    - Verify that `LOCALEDIR` in `build-options.env` is consistent with the actual installation
      path in the Flatpak manifest
    - Without the `.mo` files, all strings appear in English regardless of the system locale

6. **Sandbox permissions**
    - Mandatory `finish-args`: `--socket=wayland`, `--socket=fallback-x11`, `--share=ipc`
    - Permissions that must be **absent** (principle of least privilege):
        - `--filesystem=home` — never required for a container manager
        - `--share=network` — connection to the Docker/Podman daemon is via Unix socket, not network
        - `--device=all` — does not require access to I/O hardware
    - `--socket=session-bus` should only appear if the app uses the session D-Bus

---

## Dimension 3 — macOS Bundle Completeness

Analyse the Makefile (`dist-macos` / `dmg`) and the `release.yml` workflow (job `macos`):

1. **`.app` structure**
   Verify that the bundle script creates the correct structure:
   ```
   GtkCrossPlatform.app/
   ├── Contents/
   │   ├── Info.plist
   │   ├── MacOS/
   │   │   └── gtk-cross-platform           # main binary
   │   ├── Frameworks/                      # dylibs re-linked by dylibbundler
   │   └── Resources/
   │       ├── share/
   │       │   ├── gtk-cross-platform/      # compiled GResource ($PKGDATADIR)
   │       │   │   └── compiled.gresource
   │       │   ├── glib-2.0/schemas/
   │       │   │   └── gschemas.compiled    # MANDATORY
   │       │   ├── icons/hicolor/           # app icons
   │       │   └── locale/                  # .mo files ($LOCALEDIR)
   │       └── lib/
   │           └── gdk-pixbuf-2.0/         # pixel buffer loaders
   ```

2. **`Info.plist`**
   Must contain all mandatory fields:
    - `CFBundleIdentifier` — must equal the App ID
    - `CFBundleExecutable` — must be `gtk-cross-platform`
    - `CFBundleName` — display name
    - `CFBundleVersion` and `CFBundleShortVersionString` — app version
    - `NSHighResolutionCapable: true` — without this, the UI is blurry on Retina
    - `LSMinimumSystemVersion` — must declare the minimum macOS version
    - `NSHumanReadableCopyright` — copyright line
      Run: `grep -A1 'CFBundle\|NSHighResolution\|LSMinimum' <Info.plist>` if the file exists

3. **Runtime dependencies (dylibs)**
    - After `dylibbundler`, no path in `otool -L` should point to `/opt/homebrew/` or
      `/usr/local/` — all must be `@executable_path/../Frameworks/` or system paths
      (`/usr/lib`, `/System/`)
    - Minimum dylib count in `Contents/Frameworks/`: **≥ 20** for GTK4+Adwaita
    - If the Makefile does not run `dylibbundler`, the bundle is not redistributable

4. **`gschemas.compiled` in the bundle**
    - The file must be copied from `$(brew --prefix)/share/glib-2.0/schemas/gschemas.compiled`
      (Homebrew global prefix, not a formula-specific prefix)
    - Verify in the Makefile that the bundle step copies this file
    - Without it, GTK4/Adwaita throws `g_settings_new: assertion schema not found` at runtime

5. **Expected bundle size**
    - `.app`: 25–50 MB (GTK4+Adwaita+deps)
    - `.dmg`: 8–15 MB (compressed)
    - Values below these ranges indicate that dylibs or runtime data were not included

6. **DMG**
    - Must contain `--app-drop-link` (shortcut to `/Applications`) for drag-and-drop installation
    - Filename must include version and architecture: `GtkCrossPlatform-<version>-macos-arm64.dmg`

---

## Dimension 4 — Windows Bundle Completeness

Analyse the `release.yml` workflow (job `windows`) and the Makefile:

1. **ZIP structure**
   Verify that the bundle script creates:
   ```
   GtkCrossPlatform-<version>-windows-x86_64/
   ├── gtk-cross-platform.exe
   ├── *.dll                               # MINGW64 DLLs copied by ldd
   ├── share/
   │   ├── gtk-cross-platform/
   │   │   └── compiled.gresource          # compiled GResource
   │   ├── glib-2.0/schemas/
   │   │   └── gschemas.compiled           # MANDATORY
   │   ├── icons/hicolor/
   │   │   ├── index.theme                 # icon fallback
   │   │   └── ...
   │   └── locale/                         # .mo files
   └── lib/
       └── gdk-pixbuf-2.0/               # pixel buffer loaders (PNG/SVG)
   ```

2. **DLL bundling**
    - `ldd gtk-cross-platform.exe | grep -i 'mingw64' | awk '{print $3}'` must produce a
      non-empty list of DLLs
    - Filter must use `mingw64` without a leading `/` (some `ldd` versions omit the slash)
    - Critical DLLs that must appear: `libgtk-4-1.dll`, `libadwaita-1-0.dll`, `libglib-2.0-0.dll`,
      `libgobject-2.0-0.dll`, `libgio-2.0-0.dll`, `libpango-1.0-0.dll`, `libcairo-2.dll`

3. **Mandatory runtime data**
    - `share/glib-2.0/schemas/gschemas.compiled` — without this, GTK4 does not open
    - `share/icons/hicolor/index.theme` — without this, icons are absent
    - `lib/gdk-pixbuf-2.0/` with loaders — without this, PNG images do not render
    - Source: `/mingw64/share/glib-2.0/schemas/`, `/mingw64/share/icons/hicolor/`,
      `/mingw64/lib/gdk-pixbuf-2.0/`

4. **No native installer**
    - The ZIP is the only Windows distribution format currently — report as `[IMPROVEMENT]`
      the absence of an NSIS/WiX installer for integration with Windows "Add/Remove Programs"

---

## Dimension 5 — Store and Repository Compliance

1. **Flathub**
    - `app-id` must not be `com.example.*` — a placeholder ID blocks submission
    - `metainfo.xml` must pass `appstreamcli validate --pedantic`; run if available:
      ```bash
      appstreamcli validate data/com.example.GtkCrossPlatform.metainfo.xml
      ```
    - `<release>` must have `date=` in ISO 8601 format (`YYYY-MM-DD`)
    - `<url type="homepage">`, `<url type="bugtracker">`, `<url type="vcs-browser">` must point
      to real URLs (not `https://example.com`)
    - `<screenshots>` — at least one valid screenshot (URL or local file) is mandatory for
      Flathub
    - License in `<metadata_license>` must be `CC0-1.0`; `<project_license>` must be the real
      code license (e.g. `GPL-3.0-or-later`)

2. **GNOME Software / KDE Discover**
    - `desktop` file must have `Categories=` with at least one valid FreeDesktop category
    - `desktop` file must have `Keywords=` for search in GNOME Software
    - Run if available:
      ```bash
      desktop-file-validate data/com.example.GtkCrossPlatform.desktop
      ```

3. **macOS Gatekeeper**
    - Without an Apple Developer certificate signature, the app is blocked on first launch
    - Ad-hoc codesign (`codesign --sign -`) performed by `dylibbundler` does not satisfy Gatekeeper
    - Report as `[IMPROVEMENT]` the absence of notarisation — not blocking for distribution
      via GitHub Releases, but blocks the Mac App Store

4. **Windows SmartScreen**
    - Without Authenticode signing, Windows displays an "Unknown publisher" warning on first launch
    - Report as `[IMPROVEMENT]` — does not block execution, but reduces user trust

---

## Dimension 6 — First-Install Experience

Mentally simulate the flow of a new user on each platform:

1. **Flatpak (Linux)**
    - The user runs `flatpak install com.example.GtkCrossPlatform.flatpak`
    - Runtime dependency: `org.gnome.Platform//48` must be on Flathub; if not,
      installation fails — verify that the manifest declares `runtime-version: "48"`
    - The user runs the app: there must be no error messages about a missing schema, missing icon,
      or locale not found

2. **macOS (DMG)**
    - The user mounts the DMG and drags the `.app` to `/Applications`
    - The user opens the app — Gatekeeper prompts for confirmation (expected without cert)
    - The app must not fail with `dyld: Library not loaded` (indicates dylibbundler did not run)
    - The app must not fail with `g_settings_new: Failed to get schema` (indicates missing
      `gschemas.compiled`)

3. **Windows (ZIP)**
    - The user extracts the ZIP and runs `gtk-cross-platform.exe`
    - The app must not fail with `The code execution cannot proceed because libgtk-4-1.dll was
     not found` (indicates a missing DLL in the ZIP)
    - The app must not fail due to missing schema or GResource

4. **Locale verification**
    - If the user's system is set to PT-BR, the app must display strings in Portuguese
    - Validate that `.mo` files are at the correct path for each platform

---

## Dimension 7 — Icon Integrity

1. **Resolutions present**
   Run: `find data/icons -name '*.png' -o -name '*.svg' | sort`
   Minimum expected resolutions: 16, 32, 48, 128, 256, 512 px + scalable SVG
   Report missing resolutions.

2. **Correct naming**
    - All icons must be named `com.example.GtkCrossPlatform.<ext>`
    - Incorrectly named icons are not found by the GNOME Shell / Launcher

3. **Valid SVG**
    - The SVG in `scalable/apps/` must have a square `viewBox` (e.g. `0 0 256 256`)
    - A non-square viewBox causes distortion when rendered by GNOME Shell
    - Run: `grep viewBox data/icons/hicolor/scalable/apps/com.example.GtkCrossPlatform.svg`

4. **Icon in macOS bundle**
    - The macOS bundle must contain a `.icns` or copy the hicolor PNGs to
      `Contents/Resources/` with the correct name
    - Without an icon in the bundle, macOS displays the generic application icon

---

## How to run this audit

### Step 1 — Static analysis (always)

Read all files listed in "What to read before auditing" and run the checks for
all dimensions based on the observed content.

### Step 2 — Local checks (if on macOS)

```bash
# 1. Validate AppStream metainfo
appstreamcli validate data/com.example.GtkCrossPlatform.metainfo.xml 2>&1 || true

# 2. Validate desktop file
desktop-file-validate data/com.example.GtkCrossPlatform.desktop 2>&1 || true

# 3. Version consistency
CARGO_VERSION=$(grep -m1 '^version' Cargo.toml | sed 's/.*= "\(.*\)"/\1/')
METAINFO_VERSION=$(grep '<release' data/com.example.GtkCrossPlatform.metainfo.xml | head -1 | sed 's/.*version="\([^"]*\)".*/\1/')
echo "Cargo: $CARGO_VERSION | Metainfo: $METAINFO_VERSION"
[ "$CARGO_VERSION" = "$METAINFO_VERSION" ] && echo "PASS: versions match" || echo "FAIL: version mismatch"

# 4. Icons present
find data/icons/hicolor -name '*.png' | wc -l | xargs -I{} sh -c \
  '[ {} -ge 6 ] && echo "PASS: {} PNG icons" || echo "WARN: only {} PNG icons (expected ≥ 6)"'

# 5. SVG with square viewBox
SVG_VIEWBOX=$(grep -o 'viewBox="[^"]*"' data/icons/hicolor/scalable/apps/com.example.GtkCrossPlatform.svg 2>/dev/null || echo "NOT FOUND")
echo "SVG viewBox: $SVG_VIEWBOX"

# 6. GSettings schema present
[ -f data/com.example.GtkCrossPlatform.gschema.xml ] \
  && echo "PASS: gschema.xml present" \
  || echo "FAIL: gschema.xml missing — GTK4 Settings will not work"

# 7. GResource manifest lists correct files
GRESOURCE_FILES=$(grep '<file' data/resources/resources.gresource.xml | wc -l)
echo "GResource: $GRESOURCE_FILES files declared"

# 8. POTFILES lists only files that exist
while IFS= read -r path; do
  [ -f "$path" ] || echo "STALE POTFILES: $path does not exist"
done < po/POTFILES
```

### Step 3 — Report

---

## Report format

```markdown
# Distribution Audit — gtk-cross-platform

## Scorecard

| Dimension                              | Status   | Issues |
|----------------------------------------|----------|--------|
| 1. Identity and version                | ✅/⚠️/❌ | n      |
| 2. Flatpak artifact                    | ✅/⚠️/❌ | n      |
| 3. macOS bundle                        | ✅/⚠️/❌ | n      |
| 4. Windows bundle                      | ✅/⚠️/❌ | n      |
| 5. Store compliance                    | ✅/⚠️/❌ | n      |
| 6. First-install experience            | ✅/⚠️/❌ | n      |
| 7. Icon integrity                      | ✅/⚠️/❌ | n      |

✅ = no issues · ⚠️ = non-blocking degradation · ❌ = prevents user from installing or running

---

## Issues found

For each issue:

**[SEVERITY] Dimension → Item**
> File: `<path>:<line>`
> Issue: objective description of what is wrong.
> Impact on user: what fails on the end device.
> Fix: exact diff or instruction.

Severities:

- `[BLOCKING]` — the user cannot install or the app crashes on open
- `[WARNING]` — silent degradation (missing icon, strings in English, OS security warning)
- `[IMPROVEMENT]` — does not break, but reduces perceived quality or store compliance

---

## Local checks run

- `appstreamcli validate`: PASS / FAIL / NOT INSTALLED
- `desktop-file-validate`: PASS / FAIL / NOT INSTALLED
- Version consistency: PASS `x.y.z` / FAIL (`Cargo: x.y.z` vs `Metainfo: a.b.c`)
- PNG icon count: N (PASS ≥ 6 / WARN < 6)
- SVG viewBox: `<value>` (PASS square / WARN non-square)
- GSettings schema: PASS / FAIL
- GResource manifest: N files declared
- POTFILES stale: N invalid paths

---

## Fix plan

Only BLOCKING and WARNING items, in priority order:

| # | File | Issue | Fix | Effort |
|---|------|-------|-----|--------|
| 1 | ...  | ...   | ... | 5 min  |
```

---

## Constraints

- Base all diagnoses on the observable content of the files; do not assume undocumented behaviour
- When a binary artifact cannot be verified (the bundle does not exist locally), indicate
  `[static analysis only]` and verify the script that generates it
- Do not duplicate pipeline gaps already covered by `/verify:release-audit` (workflow syntax,
  Cargo cache, parallel jobs, CI runners)
- Do not duplicate compliance gaps already covered by `/verify:compliance-audit` (inline i18n,
  A11Y, breakpoints, hexagonal architecture)
- Prioritise by impact on the end user: **app that does not open > missing runtime data >
  store compliance > polish improvements**
