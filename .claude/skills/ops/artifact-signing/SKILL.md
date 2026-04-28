---
name: ops:artifact-signing
description: Signs and notarizes distribution artifacts — macOS notarization via Apple, Flatpak GPG signing, and SHA256 checksums for all platforms.
---

# ops:artifact-signing

Invoke with `/ops:artifact-signing` after packaging is complete and before store submission.

## When to use

- After `/ops:artifact-packaging` produces all platform artifacts
- Before uploading artifacts to GitHub Release or Flathub
- When verifying existing artifact integrity

## macOS Code Signing + Notarization

**Prerequisites:**
- Apple Developer ID certificate installed in Keychain
- `xcrun notarytool` configured with `--keychain-profile`

**Sign the .app bundle:**

```sh
# Sign with Developer ID Application certificate
codesign --sign "Developer ID Application: <Your Name> (<TeamID>)" \
  --deep --force --options runtime \
  "dist/GtkCrossPlatform.app"

# Verify signature
codesign --verify --verbose=4 "dist/GtkCrossPlatform.app"
spctl --assess --verbose "dist/GtkCrossPlatform.app"
```

**Create and sign the DMG:**

```sh
# DMG must be created AFTER signing the .app
create-dmg "GtkCrossPlatform.dmg" "dist/GtkCrossPlatform.app"

# Sign the DMG itself
codesign --sign "Developer ID Application: <Your Name> (<TeamID>)" "GtkCrossPlatform.dmg"
```

**Notarize:**

```sh
xcrun notarytool submit "GtkCrossPlatform.dmg" \
  --keychain-profile "<profile-name>" \
  --wait

# Staple the notarization ticket
xcrun stapler staple "GtkCrossPlatform.dmg"

# Verify notarization
spctl --assess --type open --context context:primary-signature --verbose "GtkCrossPlatform.dmg"
```

## Flatpak GPG Signing

Flatpak artifacts submitted to Flathub are signed by Flathub's build infrastructure automatically.
For self-hosted distribution:

```sh
# Generate GPG key if not exists
gpg --gen-key

# Sign the Flatpak bundle
flatpak build-sign <build-dir> --gpg-sign=<KEY-ID>

# Export the public key for users
gpg --export <KEY-ID> > com.example.GtkCrossPlatform.gpg
```

## SHA256 Checksums (all platforms)

Generate checksums for all release artifacts:

```sh
sha256sum \
  com.example.GtkCrossPlatform-x86_64.flatpak \
  com.example.GtkCrossPlatform-aarch64.flatpak \
  GtkCrossPlatform-vX.Y.Z.dmg \
  GtkCrossPlatform-vX.Y.Z-windows-x86_64.zip \
  > SHA256SUMS.txt

# Verify a specific artifact
sha256sum -c SHA256SUMS.txt
```

Include `SHA256SUMS.txt` in the GitHub Release assets.

## Signing checklist

- [ ] macOS .app signed with Developer ID Application certificate
- [ ] macOS .app notarization ticket stapled
- [ ] DMG signed and notarized
- [ ] `spctl --assess` passes for both .app and .dmg
- [ ] Flatpak artifacts do not need manual GPG signing (Flathub handles it)
- [ ] `SHA256SUMS.txt` generated for all artifacts
- [ ] All checksums verified locally before upload

## Output

Signing status per artifact: ✅ signed and verified / ❌ failure with error detail.
`SHA256SUMS.txt` path and contents.
