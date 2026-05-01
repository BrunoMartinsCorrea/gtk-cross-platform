---
name: verify:release-audit
version: 1.0.0
description: Audit the cross-platform release pipeline (workflows, bundling, artifacts, publishing)
---

# verify:release-audit

Audit the cross-platform distribution pipeline of this GTK4/Rust project. This command is
self-contained — run it fresh without prior conversation context.

**Repository:** `gtk-cross-platform`
**Stack:** Rust · GTK4 0.9 · libadwaita 0.7 · Flatpak (GNOME Platform 48)
**Target platforms:** Linux x86_64 (Flatpak), Linux aarch64 (Flatpak), macOS arm64 (DMG), Windows x86_64 (ZIP)

---

## Immediate execution

Run this block NOW, before any analysis. Each command produces evidence used in the
report. Do not skip steps; record the literal output of each one.

```bash
# 1. Presence of critical files
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

# 2. Version declared in Cargo.toml
grep '^version' Cargo.toml | head -1

# 3. Version declared in metainfo.xml
grep '<release ' data/com.example.GtkCrossPlatform.metainfo.xml | head -3

# 4. Triggers of ci.yml (must include push: branches: [main])
grep -A5 '^on:' .github/workflows/ci.yml

# 5. Triggers of release.yml (must be only push: tags:)
grep -A5 '^on:' .github/workflows/release.yml

# 6. Permissions declared in each job of release.yml
grep -n 'permissions' .github/workflows/release.yml

# 7. Action versions in release.yml
grep -E 'uses: (actions|flatpak|msys2|dtolnay)' .github/workflows/release.yml

# 8. Action versions in ci.yml
grep -E 'uses: (actions|taiki-e|EmbarkStudios|crate-ci)' .github/workflows/ci.yml

# 9. GSettings schema in Flatpak manifest (must contain glib-compile-schemas)
grep -n 'glib-compile-schemas\|gschema' com.example.GtkCrossPlatform.json

# 10. flatpak-cargo-generator.py: URL and pin (master = UNPINNED)
grep -n 'flatpak-cargo-generator' .github/workflows/release.yml

# 11. Flatpak finish-args
grep -A10 'finish-args' com.example.GtkCrossPlatform.json

# 12. runtime-version in the manifest
grep 'runtime-version' com.example.GtkCrossPlatform.json

# 13. build-options.env in the manifest (APP_ID, PROFILE, PKGDATADIR, LOCALEDIR)
grep -A15 'build-options' com.example.GtkCrossPlatform.json

# 14. Nightly Flatpak in ci.yml (must have a Flatpak build job)
grep -n 'flatpak\|nightly' .github/workflows/ci.yml

# 15. publish job in release.yml: needs all artifacts
grep -A5 'needs:' .github/workflows/release.yml

# 16. actionlint (if available)
command -v actionlint >/dev/null 2>&1 \
  && actionlint .github/workflows/ci.yml .github/workflows/release.yml \
  || echo "[warning] actionlint not found — install with: brew install actionlint"

# 17. macOS runner (must be macos-14, not macos-latest)
grep 'runs-on' .github/workflows/release.yml

# 18. Homebrew deps in the macOS job
grep -A5 'brew install' .github/workflows/release.yml

# 19. DLL filter in the Windows job (mingw64 without leading slash)
grep -n 'grep.*mingw64\|ldd' .github/workflows/release.yml

# 20. Mandatory Windows runtime data
grep -n 'gschemas.compiled\|gdk-pixbuf\|index.theme' .github/workflows/release.yml

# 21. gh release create: referenced artifacts
grep -A15 'gh release create' .github/workflows/release.yml

# 22. Cargo cache: presence of hashFiles(Cargo.lock) in all Rust jobs
grep -n 'hashFiles' .github/workflows/release.yml

# 23. Parallelism: flatpak-x86_64, flatpak-aarch64, macos, windows without needs between them
grep -B2 'needs:' .github/workflows/release.yml
```

---

## Dimension 0 — Version and identity consistency

Verify using the outputs from the immediate execution block:

1. **Cargo.toml == metainfo.xml**
    - `version` in `Cargo.toml` must be identical to the `version` attribute in the latest `<release>`
      in `metainfo.xml`
    - Divergence = `[BLOCKING]`: the release tag `v<X>` does not match the compiled binary

2. **App ID consistent across all files**
    - `com.example.GtkCrossPlatform` must appear identically in:
        - `com.example.GtkCrossPlatform.json` → `app-id` field
        - `Cargo.toml` → build env `APP_ID`
        - `release.yml` → artifact names and `gh release create`
        - `ci.yml` → upload-artifact names (if any Flatpak nightly)
    - Any variant (case, separator, extra suffix) = `[BLOCKING]`

3. **Release trigger only on semver tags**
    - `release.yml` must fire **only** on `push: tags: ['v[0-9]*.[0-9]*.[0-9]*']`
    - Absence of the exact pattern = risk of accidental release on a wrongly formatted tag

4. **ci.yml must fire on `push: branches: [main]`**
    - Without this trigger, direct pushes to `main` (e.g. squash merges) do not run CI
    - Output of command 4 must show `push:` with `branches: [main]` in addition to `pull_request:`
    - Absence = `[BLOCKING]`: regressions reach `main` unchecked

---

## Dimension 1 — Workflow syntax and validity

Based on the outputs of commands 6–8 and 16:

1. **actionlint** — report each error line from command 16. If not installed, indicate
   `[warning] actionlint not found` and continue with static analysis.

2. **Minimum permissions per job** (command 6)
    - `flatpak-x86_64`, `flatpak-aarch64`: `permissions: contents: read` ✅
    - `macos`, `windows`: must have `permissions:` declared explicitly; implicit inheritance
      from the repository level is a security risk = `[WARNING]`
    - `publish`: `permissions: contents: write` mandatory = `[BLOCKING]` if absent

3. **Pinned action versions** (commands 7–8)
    - `actions/checkout`, `upload-artifact`, `download-artifact`, `cache` → `@v4` or higher
    - `flatpak/flatpak-github-actions/flatpak-builder` → `@v6` or higher
    - `msys2/setup-msys2` → `@v2` or higher
    - `dtolnay/rust-toolchain` → `@stable` is accepted (implicit semver)
    - Any `@v1`, `@v2`, `@v3` on actions that have `@v4` available = `[WARNING]`
    - Any action without a version pin (`@main`, no @) = `[BLOCKING]`

4. **Correct parallelism** (command 23)
    - `flatpak-x86_64`, `flatpak-aarch64`, `macos`, `windows` must NOT have `needs:` between them
    - Only `publish` should have `needs: [flatpak-x86_64, flatpak-aarch64, macos, windows]`
    - Any extra `needs:` between build jobs = `[IMPROVEMENT]` (unnecessarily serialises)

---

## Dimension 2 — Flatpak build

Based on the outputs of commands 9–13:

1. **Container image**
    - Must use `ghcr.io/flathub-infra/flatpak-github-actions:gnome-48`
    - `runtime-version` in the manifest (command 12) must be `"48"` — inconsistency = `[BLOCKING]`

2. **flatpak-cargo-generator.py — version pin** (command 10)
    - URL must point to a fixed commit SHA, **not** `master`
    - Downloading from `master` = silent breakage when the script changes its API
    - If `master` appears in the output: `[WARNING]`
    - Fix: replace `master` with a verified recent SHA:
      ```
      https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/<SHA>/cargo/flatpak-cargo-generator.py
      ```

3. **GSettings schema in the manifest** (command 9)
    - The manifest MUST have `glib-compile-schemas` in `build-commands` if
      `data/com.example.GtkCrossPlatform.gschema.xml` exists in the repository
    - Absence of `glib-compile-schemas` = `[BLOCKING]`: GSettings does not work in the sandbox
    - Fix to add in `build-commands` after the binary install:
      ```
      "install -Dm644 data/com.example.GtkCrossPlatform.gschema.xml /app/share/glib-2.0/schemas/com.example.GtkCrossPlatform.gschema.xml",
      "glib-compile-schemas /app/share/glib-2.0/schemas/"
      ```

4. **Flatpak manifest — finish-args** (command 11)
    - Mandatory: `--socket=wayland`, `--socket=fallback-x11`, `--share=ipc`
    - `--device=dri` only if GPU is required — report presence as `[WARNING]`
    - Absence of `--socket=wayland` = `[BLOCKING]` (app does not open on GNOME)

5. **build-options.env** (command 13)
    - `APP_ID`, `PROFILE`, `PKGDATADIR`, `LOCALEDIR` must be present
    - `PKGDATADIR=/app/share/gtk-cross-platform` (Flatpak path)
    - `LOCALEDIR=/app/share/locale`
    - Absence of any variable = `[BLOCKING]`: binary compiled with wrong paths

6. **Flatpak artifact names**
    - `release.yml`: `com.example.GtkCrossPlatform-x86_64.flatpak` and
      `com.example.GtkCrossPlatform-aarch64.flatpak`
    - Name in `bundle:`, `upload-artifact path:`, and path in `publish` must be identical
    - Any divergence = `[BLOCKING]`: `download-artifact` cannot find the file

7. **aarch64 architecture**
    - Job `flatpak-aarch64` must pass `arch: aarch64` to the action
    - Absence = x86_64 build sent with `aarch64` name = `[BLOCKING]`

---

## Dimension 3 — macOS build

Based on the outputs of commands 17–18 and local validation below:

1. **Runner** (command 17)
    - Must be `macos-14` — not `macos-latest` (changes between runs), not `macos-13` (Intel)
    - `macos-latest` = `[WARNING]`: artifact may be x86_64 instead of arm64

2. **Homebrew dependencies** (command 18)
    - Mandatory: `gtk4`, `libadwaita`, `dylibbundler`, `create-dmg`
    - Absence of any one = `[BLOCKING]`

3. **Build variables**
    - `APP_ID=com.example.GtkCrossPlatform`
    - `PROFILE=default`
    - `PKGDATADIR=../Resources/share/gtk-cross-platform`
    - `LOCALEDIR=../Resources/share/locale`
    - Absence of `env:` in the build step = `[BLOCKING]`: `config.rs` compiles with wrong paths

4. **Info.plist — mandatory fields**
    - `CFBundleIdentifier`: must be `com.example.GtkCrossPlatform`
    - `CFBundleExecutable`: must be `gtk-cross-platform`
    - `NSHighResolutionCapable`: `true`
    - `LSMinimumSystemVersion`: must exist (e.g. `12.0`)
    - Absence of any field = `[BLOCKING]` (Gatekeeper rejects the bundle)

5. **`gschemas.compiled` path**
    - Must use `$(brew --prefix)/share/glib-2.0/schemas/gschemas.compiled`
    - NOT `$(brew --prefix glib)/share/...` (wrong path — GTK4/Adwaita does not open)
    - This is a silent bug; CI only detects it at runtime = `[BLOCKING]`

6. **`dylibbundler` — mandatory flags**
    - Must have `-od -b -x <binary> -d <Frameworks/> -p @executable_path/../Frameworks/`
    - Absence of `-p @executable_path/...` = absolute Homebrew links in the binary = `[BLOCKING]`

7. **Homebrew cache**
    - Absence of `actions/cache` for Homebrew = `[IMPROVEMENT]`: +5–10 min per build
    - Minimum acceptable mitigation: `HOMEBREW_NO_AUTO_UPDATE=1 HOMEBREW_NO_INSTALL_CLEANUP=1` as `env:`

---

## Dimension 4 — Windows build

Based on the outputs of commands 19–20:

1. **Runner and shell**
    - Must use `windows-latest` with `defaults: run: shell: msys2 {0}`
    - Steps with `Compress-Archive` must declare `shell: pwsh` explicitly

2. **MSYS2 MINGW64 packages**
    - Mandatory: `gtk4`, `libadwaita`, `rust`, `pkg-config`, `gettext`, `gettext-tools`,
      `adwaita-icon-theme`
    - Without `adwaita-icon-theme` = missing icons at runtime = `[WARNING]`

3. **DLL bundling — correct filter** (command 19)
    - Filter must be `grep -i 'mingw64'` (without leading `/`)
    - `grep -i '/mingw64/'` may omit DLLs in some `ldd` versions = `[WARNING]`
    - Verify the `awk` column: `$3` is the full path when `ldd` uses the format
      `name => /path (0x...)` — confirm from the observed output

4. **Mandatory runtime data in the ZIP** (command 20)
    - `dist/share/glib-2.0/schemas/gschemas.compiled` — absent = GTK4 does not open = `[BLOCKING]`
    - `dist/share/icons/hicolor/index.theme` — absent = icons without fallback = `[WARNING]`
    - `dist/lib/gdk-pixbuf-2.0/` — absent = PNG/SVG do not render = `[WARNING]`

5. **CARGO_HOME and cache on Windows/MSYS2**
    - `actions/cache` runs in PowerShell (not MSYS2); `~` resolves to `C:\Users\runneradmin`
    - If MSYS2 sets `CARGO_HOME` as a Unix path, cache never hits = `[WARNING]`
    - Check: the `key:` uses `hashFiles('**/Cargo.lock')` — confirm presence in the output
      of command 22

6. **ZIP name**
    - Must include the version: `GtkCrossPlatform-${{ github.ref_name }}-windows-x86_64.zip`
    - Name in the compression step, `upload-artifact`, and `publish` must be identical = `[BLOCKING]`

---

## Dimension 5 — Release publishing

Based on the output of command 21:

1. **`download-artifact` without `name:`**
    - Must download all artifacts into separate subdirectories (`path: artifacts`)
    - Specifying `name:` forces download of a single artifact = `[BLOCKING]`

2. **Exact paths in `gh release create`**
    - `artifacts/flatpak-x86_64/com.example.GtkCrossPlatform-x86_64.flatpak`
    - `artifacts/flatpak-aarch64/com.example.GtkCrossPlatform-aarch64.flatpak`
    - `artifacts/macos-dmg/GtkCrossPlatform-<tag>-macos-arm64.dmg`
    - `artifacts/windows-zip/GtkCrossPlatform-<tag>-windows-x86_64.zip`
    - `<tag>` must be `${{ github.ref_name }}` — verify interpolation
    - Wrong path = `[BLOCKING]`: artifact absent from the release

3. **`GH_TOKEN`**
    - Must use `secrets.GITHUB_TOKEN` — not personal PATs
    - Personal PAT = unnecessary attack surface = `[WARNING]`

4. **`--generate-notes`**
    - Must be present in `gh release create` or replaced by `--notes-file CHANGELOG.md`
    - Absence = release without changelog = `[WARNING]`

5. **Nightly vs. versioned**
    - `ci.yml` must have a Flatpak nightly job (`nightly` release, `--prerelease`) on `push: branches: [main]`
    - `release.yml` must create a versioned release without `--prerelease`
    - If `ci.yml` does not have the `push: branches: [main]` trigger nor the Flatpak nightly job:
      report as `[WARNING]`: intermediate builds are not published

---

## Dimension 6 — Cache and CI performance

Based on the outputs of commands 22–23:

1. **Cargo cache in all Rust jobs**
    - All jobs with `cargo build` must cache:
      `~/.cargo/registry/index/`, `~/.cargo/registry/cache/`, `~/.cargo/git/db/`, `target/`
    - Key must include `${{ hashFiles('**/Cargo.lock') }}`
    - `restore-keys` must have a fallback without hash
    - Job without cache = `[WARNING]`: +2–5 min per build

2. **Flatpak cache — unique keys per job**
    - `flatpak-release-x86_64-<tag>` and `flatpak-release-aarch64-<tag>` must not share a prefix
    - Sharing = wrong-architecture artifact restored = `[BLOCKING]`

3. **Parallel jobs**
    - `flatpak-x86_64`, `flatpak-aarch64`, `macos`, `windows` without `needs:` between them
    - Any serialisation = `[IMPROVEMENT]`: increases total release time

---

## Local validation (macOS)

If the current environment is macOS, run this block **immediately**:

```bash
# Release build with production env vars
APP_ID=com.example.GtkCrossPlatform \
PROFILE=default \
PKGDATADIR=../Resources/share/gtk-cross-platform \
LOCALEDIR=../Resources/share/locale \
cargo build --release 2>&1 | tail -5

# Verify correct path for gschemas.compiled
SCHEMA_PATH="$(brew --prefix)/share/glib-2.0/schemas/gschemas.compiled"
[ -f "$SCHEMA_PATH" ] && echo "PASS gschemas: $SCHEMA_PATH" || echo "FAIL gschemas: not found"

# Test bundle
APP="AuditTest.app"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Frameworks"
cp target/release/gtk-cross-platform "$APP/Contents/MacOS/"

# dylibbundler — "replacing existing signature" is expected (ad-hoc codesign)
dylibbundler -od -b \
  -x "$APP/Contents/MacOS/gtk-cross-platform" \
  -d "$APP/Contents/Frameworks/" \
  -p "@executable_path/../Frameworks/" 2>&1 | grep -v 'replacing existing signature'

# FAIL if there are residual absolute Homebrew paths
ABSOLUTE=$(otool -L "$APP/Contents/MacOS/gtk-cross-platform" \
  | grep -v "@executable_path\|/usr/lib\|/System\|gtk-cross-platform:" \
  | wc -l | tr -d ' ')
[ "$ABSOLUTE" -eq 0 ] \
  && echo "PASS dylibs: zero absolute Homebrew paths" \
  || echo "FAIL dylibs: $ABSOLUTE residual absolute paths"

# Minimum bundled dylib count (≥ 20 for GTK4+Adwaita)
COUNT=$(ls "$APP/Contents/Frameworks/" | wc -l | tr -d ' ')
[ "$COUNT" -ge 20 ] \
  && echo "PASS bundle: $COUNT dylibs" \
  || echo "WARN bundle: $COUNT dylibs — fewer than expected (≥ 20)"

# Bundle size (expected: 25–50 MB)
du -sh "$APP/" | awk '{print "Bundle size: "$1" (expected: 25–50 MB)"}'

rm -rf "$APP"
```

Expected warnings that must **not** be reported as failures:

- `replacing existing signature` — ad-hoc codesign; normal
- `hdiutil does not support internet-enable` — removed in macOS 10.15; ignore

---

## Report format

```markdown
# Release Pipeline Audit — gtk-cross-platform

## Scorecard

| Dimension                       | Status   | Blocking | Warnings |
|---------------------------------|----------|----------|----------|
| 0. Version consistency          | ✅/⚠️/❌ | n        | n        |
| 1. Workflow syntax              | ✅/⚠️/❌ | n        | n        |
| 2. Flatpak build                | ✅/⚠️/❌ | n        | n        |
| 3. macOS build                  | ✅/⚠️/❌ | n        | n        |
| 4. Windows build                | ✅/⚠️/❌ | n        | n        |
| 5. Release publishing           | ✅/⚠️/❌ | n        | n        |
| 6. Cache and performance        | ✅/⚠️/❌ | n        | n        |

✅ = no issues · ⚠️ = non-blocking warnings · ❌ = blocking CI failure

---

## Issues found

For each issue:

**[SEVERITY] Dimension N → Item → file:line**
> Evidence: literal output of the command that detected the issue.
> Impact: what breaks in CI or on the user's device.
> Exact fix: diff or complete command.

Severities:
- `[BLOCKING]` — pipeline fails or artifact does not run on the user's device
- `[WARNING]` — silent degradation (cache miss, missing icon, i18n in English, security risk)
- `[IMPROVEMENT]` — does not break, but reduces quality or increases CI time

---

## Local validations (if run)

- `cargo build --release`: PASS / FAIL (last 5 lines)
- `gschemas.compiled`: PASS path / FAIL
- `dylibbundler` + absolute paths: PASS / FAIL (N residual paths)
- dylib count: N (PASS ≥ 20 / WARN < 20)
- Bundle size: N MB (PASS 25–50 MB / WARN outside range)

---

## Fix plan (BLOCKING and WARNINGS only)

Order by impact on the end user: artifact that does not run > incomplete artifact > slow CI.

| # | File | Line | Fix summary | Effort |
|---|------|------|-------------|--------|
| 1 | …    | …    | …           | 5 min  |
```

---

## Constraints

- Base all diagnoses on the literal output of the commands run; never assume unobserved behaviour
- When an item cannot be verified locally (Windows, aarch64), indicate
  `[static analysis only]` and detail the reasoning
- Do not repeat gaps covered by `/verify:compliance-audit` (i18n, A11Y, hexagonal architecture)
- For each `[BLOCKING]`, provide the exact diff or fix command — not just a description
