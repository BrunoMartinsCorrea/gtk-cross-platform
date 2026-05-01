---
description: Makefile style standards — variable alignment, section headers, .PHONY completeness, tab indentation, help comments, continuation backslash, target naming, and alias conventions. Auto-loaded when editing Makefile.
globs: [ "Makefile", "**/Makefile" ]
---

# Makefile Style Standards

## Variable declarations

Align all `:=` and `?=` operators in the top-level variable block. The column is set
by the longest variable name in the group.

```makefile
# WRONG
APP_ID := com.example.GtkCrossPlatform
BINARY := gtk-cross-platform
FLATPAK_BUILD_DIR := .flatpak-builder/build

# CORRECT
APP_ID           := com.example.GtkCrossPlatform
BINARY           := gtk-cross-platform
FLATPAK_BUILD_DIR := .flatpak-builder/build
```

## Section headers

Every logical group of targets must be preceded by a section header, one blank line
before and one blank line after:

```makefile
# ── Section Name ──────────────────────────────────────────────────────────────
```

- Total width: 80 characters (including the leading `# `)
- Separator: `─` (U+2500, BOX DRAWINGS LIGHT HORIZONTAL)
- Canonical section order for this project:
    1. Meta
    2. Setup
    3. Build
    4. Format & Lint
    5. Test
    6. Validate (Quality Gates)
    7. Icons & Assets
    8. Package (Distribution)
    9. Publish
    10. Clean & Cache
    11. Aliases (backwards compatibility)
    12. .PHONY

## Help comments

Every public target (not prefixed with `_`) must have a `## Description` inline comment.
The `help` target uses `grep -E '^[a-zA-Z0-9_-]+:.*?## .*$$'` to extract these.

```makefile
# WRONG — no help comment
build:
	cargo build

# CORRECT
build: build-debug ## Build the application (debug)
```

Alias targets use this fixed convention:

```makefile
dmg: dist-macos ## [alias] use dist-macos
```

## Tab indentation

Recipe lines must use a **real tab character** (`\t`), never spaces.

```makefile
# WRONG — spaces
build:
    cargo build

# CORRECT — tab
build:
	cargo build
```

## Silent prefix (@)

Prefix with `@` commands that already print their own status messages to avoid
duplicate output. Do NOT silence build commands where seeing the full invocation
is useful.

```makefile
setup-macos:
	@echo "Installing GTK4 stack via Homebrew (idempotent)..."
	brew install gtk4
```

## Continuation backslash

Align continuation `\` consistently within each recipe. Indent continuation lines
with one tab + two extra spaces relative to the first command.

```makefile
dist-flatpak:
	flatpak-builder --force-clean --user --install-deps-from=flathub \
		$(FLATPAK_BUILD_DIR) $(MANIFEST)
```

## .PHONY block

The `.PHONY` declaration must:

- Live at the **bottom** of the file as the last stanza
- List **every** non-file-producing target
- Be grouped by section with a blank line between groups (matching canonical order)
- Be sorted alphabetically within each group

```makefile
.PHONY: \
  help \
  setup setup-rust setup-platform setup-macos setup-linux setup-windows setup-cargo-deps \
  build build-debug build-release schema run run-mobile watch \
  ...
```

Any target present in the file but absent from `.PHONY` is a violation.

## Target naming

- **kebab-case** only: `dist-flatpak`, not `dist_flatpak` or `distFlatpak`
- **Namespace prefixes** matching the section:
    - `setup-*` — setup targets
    - `build-*` — build variants
    - `test-*` — test variants
    - `validate-*` — quality gate checks
    - `dist-*` — distribution packages
    - `clean-*` — cleanup targets
    - `release-*` — release steps
- Bare (unprefixed) targets allowed only for top-level lifecycle commands:
  `help`, `build`, `run`, `test`, `lint`, `format`, `clean`, `ci`, `release`

## No trailing whitespace

No line may end with a space or tab character.
