---
name: release:store-submission
version: 1.0.0
description: Publishes release artifacts to distribution channels — Flathub submission, GitHub Release upload, and AppStream metainfo publishing.
---

# release:store-submission

> **Release pipeline — Step 3 of 3:** `release:artifact-packaging` → `release:artifact-signing` →
`release:store-submission`

Invoke with `/release:store-submission` after all artifacts are packaged and signed.

## When to use

- After `/release:artifact-signing` confirms all artifacts are signed and verified
- When submitting a new version to Flathub
- When creating a GitHub Release with all platform artifacts

## GitHub Release

**Create release with all artifacts:**

```sh
# Tag must exist before creating the release
make release-tag           # creates and pushes git tag v$(VERSION)

# Create GitHub Release (--generate-notes uses git log since last tag)
gh release create "v$(VERSION)" \
  --generate-notes \
  --title "v$(VERSION)" \
  com.example.GtkCrossPlatform-x86_64.flatpak \
  com.example.GtkCrossPlatform-aarch64.flatpak \
  GtkCrossPlatform-v$(VERSION).dmg \
  GtkCrossPlatform-v$(VERSION)-windows-x86_64.zip \
  SHA256SUMS.txt
```

Or use the Makefile target (requires all artifacts to be built):

```sh
make release-github
```

**Verify the release:**

```sh
gh release view "v$(VERSION)"
gh release download "v$(VERSION)" --dir /tmp/verify/
# Verify checksums match
sha256sum -c /tmp/verify/SHA256SUMS.txt
```

## Flathub Submission

Flathub uses a separate GitHub repository for each app's manifest.

**First submission:**

1. Fork `https://github.com/flathub/flathub` and submit a new-app PR
2. Submit `com.example.GtkCrossPlatform.json` (the Flatpak manifest)
3. Address review feedback from Flathub maintainers
4. Once merged, Flathub CI builds and publishes automatically

**Version update:**

1. Open a PR against the app's Flathub manifest repository
2. Update the `tag` or `commit` in the manifest to the new version
3. Update the AppStream metainfo `<release>` entry with the new version and date
4. Wait for Flathub CI to build and review

**AppStream metainfo requirements for Flathub:**

- `<releases>` section must have an entry for the new version
- Release date must be in `YYYY-MM-DD` format
- At least one screenshot in `<screenshots>`
- `<content_rating>` must be present

Validate before submitting:

```sh
make validate-metainfo     # must pass with --pedantic
```

## CHANGELOG update

Before submission, ensure `CHANGELOG.md` is updated:

```markdown
## [0.x.y] - YYYY-MM-DD

### Added
- ...

### Fixed
- ...
```

The GitHub Release notes can be auto-generated from git log (`--generate-notes`), but the
`CHANGELOG.md` entry is the authoritative human-readable record.

## Submission checklist

- [ ] `make check-version` passes (Cargo.toml and metainfo.xml versions match)
- [ ] `make validate-metainfo` passes with `--pedantic`
- [ ] `<releases>` section in metainfo.xml has the new version entry
- [ ] All 4 artifacts uploaded to GitHub Release
- [ ] `SHA256SUMS.txt` uploaded to GitHub Release
- [ ] Flathub PR submitted (if applicable)
- [ ] `CHANGELOG.md` updated with release notes

## Output

Submission status: GitHub Release URL, Flathub PR URL (if submitted), and SHA256 checksums of uploaded artifacts.
