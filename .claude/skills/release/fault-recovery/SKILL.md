---
name: release:fault-recovery
version: 1.0.0
description: Recovery playbook for when a deploy or release fails — Flatpak rollback, macOS DMG revert, hotfix procedure, and emergency patch workflow per platform.
---

# release:fault-recovery

Invoke with `/release:fault-recovery` when a release or distribution artifact has a critical defect.

## When to use

- A published Flatpak has a crash or regression in production
- A macOS DMG was signed with the wrong certificate
- A GitHub Release has a broken artifact that needs urgent replacement
- A hotfix must ship before the next planned release

## Platform recovery playbooks

### Flatpak (Flathub)

**Rollback to previous version:**

```sh
# Users can downgrade via Flatpak CLI
flatpak update --commit=<previous-commit-hash> com.example.GtkCrossPlatform

# Maintainer: contact Flathub admins at https://github.com/flathub/flathub/issues
# to request a version rollback on the store
```

**Emergency patch:**

1. Fix the bug on a hotfix branch from the release tag:
   ```sh
   git checkout -b hotfix/v0.x.y v0.x.0   # branch from the release tag
   ```
2. Apply the minimal fix (one commit, no unrelated changes)
3. Bump patch version in `Cargo.toml` and `metainfo.xml`
4. Run `make ci` — must pass
5. Tag and release: `make release`
6. Submit updated Flatpak manifest to Flathub PR queue

### macOS DMG

**Identify the issue:**

- Wrong signing certificate → rebuild with correct certificate
- Notarization rejected → check Apple Developer Portal for rejection reason
- App crashes on launch → run `Console.app` on macOS to read crash log

**Re-release:**

```sh
make dist-macos           # rebuilds .app + .dmg
# Re-sign with correct identity:
codesign --sign "Developer ID Application: <name>" --deep --force "GtkCrossPlatform.app"
xcrun notarytool submit GtkCrossPlatform.dmg --keychain-profile <profile> --wait
# Re-upload to GitHub Release:
gh release upload v0.x.y GtkCrossPlatform.dmg --clobber
```

### Windows ZIP

**Re-release:**

```sh
# Rebuild via CI or locally:
make dist-windows         # if Windows Makefile target exists
# Re-upload:
gh release upload v0.x.y GtkCrossPlatform-vX.Y.Z-windows-x86_64.zip --clobber
```

### GitHub Release (any platform)

**Replace a single artifact without re-releasing:**

```sh
# Upload replacement (--clobber overwrites existing asset)
gh release upload v0.x.y <artifact-path> --clobber
```

**Yank a broken release:**

```sh
# Mark as pre-release to hide from "latest" while fixing
gh release edit v0.x.y --prerelease
# After fix is ready, re-mark as release
gh release edit v0.x.y --latest
```

## Communication checklist

- [ ] Document the regression in `CHANGELOG.md` under the new patch version
- [ ] Update GitHub Release notes to describe what was fixed
- [ ] If the regression affects users already installed: consider a `cargo_config` update or migration note
- [ ] Post to relevant channels (GitHub Discussions, Flathub issue) informing users of the fix

## Hotfix commit convention

```
fix(platform): one-line description of what was broken

Hotfix for regression introduced in vX.Y.Z.
Refs: #<issue-number>
```

## Output

A recovery action log: what was broken, what platform, what action was taken, and confirmation that the fixed artifact
is live.
