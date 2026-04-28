---
name: ops:artifact-packaging
description: Packages the binary for each target platform — Flatpak (x86_64/aarch64), macOS .app/.dmg, Windows ZIP — and validates the manifest and AppStream metainfo before packaging.
---

# ops:artifact-packaging

Invoke with `/ops:artifact-packaging` when preparing a release build for distribution.

## When to use

- Before a new version release
- When verifying the packaging pipeline is healthy after infrastructure changes
- After updating the Flatpak manifest or macOS bundle configuration

## Pre-packaging validation

Run these before packaging to catch issues early:

```sh
make ci                    # quality gate — must pass
make validate-metainfo     # AppStream metainfo must be valid
make validate-desktop      # .desktop file must be valid
make check-version         # Cargo.toml version must match metainfo.xml version
```

## Flatpak (Linux — x86_64 and aarch64)

```sh
# x86_64 (primary target)
make dist-flatpak          # builds to $(FLATPAK_BUILD_DIR)/com.example.GtkCrossPlatform.flatpak

# aarch64 (GNOME Mobile / PinePhone)
make dist-flatpak-arm      # builds to $(FLATPAK_BUILD_DIR)-arm/com.example.GtkCrossPlatform.flatpak

# Verify in sandbox (optional but recommended)
make dist-flatpak-run
```

**Manifest:** `com.example.GtkCrossPlatform.json`
**Runtime:** `org.gnome.Platform` — version must match `GNOME Platform 48` (or current)
**SDK:** `org.gnome.Sdk` — same version as Platform

Verify manifest is current:
```sh
flatpak remote-ls flathub | grep org.gnome.Platform  # check available versions
```

## macOS (.app + .dmg)

```sh
make dist-macos            # builds .app bundle + .dmg via dylibbundler + create-dmg
```

**Prerequisites:** `dylibbundler`, `create-dmg` (installed via `make setup-macos`)

**Bundle structure:**
```
GtkCrossPlatform.app/
  Contents/
    MacOS/gtk-cross-platform
    lib/           ← bundled dylibs (dylibbundler)
    Resources/
```

Verify all GTK dylibs are bundled (no homebrew path references):
```sh
otool -L dist/GtkCrossPlatform.app/Contents/MacOS/gtk-cross-platform | grep "/opt/homebrew"
# Must return empty (all deps bundled inside .app)
```

## Windows (ZIP)

Built via CI on Windows runner (GitHub Actions `release.yml`). For local testing, use the CI pipeline.

**Artifact:** `GtkCrossPlatform-vX.Y.Z-windows-x86_64.zip`

## Artifact checklist

- [ ] Version in `Cargo.toml` matches `metainfo.xml` (`make check-version` passes)
- [ ] AppStream metainfo is valid (`make validate-metainfo` passes)
- [ ] `.desktop` file is valid (`make validate-desktop` passes)
- [ ] Flatpak x86_64 artifact exists and is ≥ expected size
- [ ] Flatpak aarch64 artifact exists
- [ ] macOS DMG mounts cleanly and app launches
- [ ] All 4 artifacts ready for GitHub Release upload

## Output

Packaging status per platform: ✅ success / ❌ failure with error detail.
Path of each artifact produced.
