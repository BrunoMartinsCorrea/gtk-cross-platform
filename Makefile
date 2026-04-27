.DEFAULT_GOAL := run

APP_ID           := com.example.GtkCrossPlatform
BINARY           := gtk-cross-platform
MANIFEST         := $(APP_ID).json
FLATPAK_BUILD_DIR := .flatpak-builder/build
VERSION          := $(shell grep '^version' Cargo.toml | head -1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')
SCHEMA_DIR       := $(shell pwd)/data
SVG_ICON         := data/icons/hicolor/scalable/apps/$(APP_ID).svg
ICON_SIZES       := 16 32 48 64 128 256 512
BREW             := /opt/homebrew
BUNDLE           := GtkCrossPlatform.app
DMG_OUT          := GtkCrossPlatform.dmg
OS               := $(shell uname 2>/dev/null || echo Windows)
GIT_TAG          := $(shell git describe --tags --abbrev=0 2>/dev/null || echo "v$(VERSION)")
NEXTEST_PROFILE  ?= default

# ── Meta ──────────────────────────────────────────────────────────────────────

help: ## Show all available targets
	@grep -E '^[a-zA-Z0-9_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
	  awk 'BEGIN {FS = ":.*?## "}; {printf "  %-26s %s\n", $$1, $$2}'

# ── Setup ─────────────────────────────────────────────────────────────────────

setup: setup-rust setup-platform setup-cargo-deps ## Configure environment for the detected platform

setup-rust: ## Install Rust via rustup if cargo is not found
	@which cargo >/dev/null 2>&1 || \
	  (curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
	   . "$$HOME/.cargo/env")

setup-platform: ## Delegate to platform-specific setup target
	@case "$(OS)" in \
	  Darwin)  $(MAKE) setup-macos ;; \
	  Linux)   $(MAKE) setup-linux ;; \
	  *)       $(MAKE) setup-windows ;; \
	esac

setup-macos: ## Install GTK4 stack via Homebrew (idempotent)
	@echo "Installing GTK4 stack via Homebrew (idempotent)..."
	@brew list gtk4           >/dev/null 2>&1 || brew install gtk4
	@brew list libadwaita     >/dev/null 2>&1 || brew install libadwaita
	@brew list adwaita-icon-theme >/dev/null 2>&1 || brew install adwaita-icon-theme
	@brew list librsvg        >/dev/null 2>&1 || brew install librsvg
	@brew list dylibbundler   >/dev/null 2>&1 || brew install dylibbundler
	@brew list create-dmg     >/dev/null 2>&1 || brew install create-dmg
	@echo "macOS setup complete. Run 'make build' to compile."

setup-linux: ## Install GTK4 dev libraries via apt or dnf (idempotent)
	@echo "Installing GTK4 dev libraries..."
	@if command -v apt-get >/dev/null 2>&1; then \
	  dpkg -l libgtk-4-dev >/dev/null 2>&1 || sudo apt-get install -y libgtk-4-dev libadwaita-1-dev; \
	elif command -v dnf >/dev/null 2>&1; then \
	  rpm -q gtk4-devel >/dev/null 2>&1 || sudo dnf install -y gtk4-devel libadwaita-devel; \
	else \
	  echo "ERROR: Unsupported package manager — install libgtk-4-dev and libadwaita-1-dev manually."; exit 1; \
	fi
	@echo "Linux setup complete. Run 'make build' to compile."

setup-windows: ## Print MSYS2/MINGW64 installation instructions
	@echo "ERROR: Automated setup is not supported on Windows."
	@echo ""
	@echo "Open MSYS2 MINGW64 shell and run:"
	@echo "  pacman -S mingw-w64-x86_64-gtk4 mingw-w64-x86_64-libadwaita \\"
	@echo "            mingw-w64-x86_64-glib2 mingw-w64-x86_64-rust \\"
	@echo "            mingw-w64-x86_64-pkg-config mingw-w64-x86_64-gettext"
	@echo ""
	@echo "Then: make build"
	@exit 1

setup-cargo-deps: ## Pre-fetch Cargo dependencies
	cargo fetch

# ── Build ─────────────────────────────────────────────────────────────────────

build: build-debug ## Build the application (debug)

build-debug: ## Compile with cargo build (debug profile)
	cargo build

build-release: ## Compile with cargo build --release
	cargo build --release

schema: ## Compile GSettings schemas
	glib-compile-schemas $(SCHEMA_DIR)

run: build schema ## Build and run the application
	GSETTINGS_SCHEMA_DIR=$(SCHEMA_DIR) cargo run

run-mobile: build schema ## Run with GTK_DEBUG=interactive (simulates narrow screen)
	GSETTINGS_SCHEMA_DIR=$(SCHEMA_DIR) GTK_DEBUG=interactive cargo run

watch: ## Watch for changes and rebuild (installs cargo-watch if needed)
	@command -v cargo-watch >/dev/null 2>&1 || cargo install cargo-watch
	cargo watch -x 'run'

# ── Format & Lint ─────────────────────────────────────────────────────────────

format: fmt-fix lint lint-i18n ## Auto-format and lint all code

fmt: ## Check formatting (fails if unformatted)
	cargo fmt --check

fmt-fix: ## Auto-format code with cargo fmt
	cargo fmt

lint: ## Run clippy with -D warnings
	cargo clippy -- -D warnings

lint-i18n: ## Validate all .po files with msgfmt --check
	@echo "Validating .po files with msgfmt..."
	@fail=0; \
	for f in po/*.po; do \
		if ! msgfmt --check --check-format --output-file=/dev/null "$$f" 2>&1; then \
			fail=1; \
		fi; \
	done; \
	exit $$fail

# ── Test ──────────────────────────────────────────────────────────────────────

test: test-unit test-integration test-i18n ## Run all tests (unit + integration + i18n)

test-unit: ## Run unit tests via cargo nextest (NEXTEST_PROFILE=default|ci)
	cargo nextest run --profile $(NEXTEST_PROFILE) --lib

test-integration: ## Run integration tests via cargo nextest
	cargo nextest run --profile $(NEXTEST_PROFILE) \
	  --test container_driver_test --test greet_use_case_test

test-i18n: ## Run i18n structural tests via cargo nextest
	cargo nextest run --profile $(NEXTEST_PROFILE) --test i18n_test

coverage: ## Measure line coverage (manual tool; not part of CI)
	cargo llvm-cov --lib --test container_driver_test --test greet_use_case_test \
	  --summary-only --fail-under-lines 25

# ── Validate (Quality Gates) ──────────────────────────────────────────────────

validate: validate-format validate-lint validate-metadata validate-i18n validate-deps ## Run all local quality gates
	@echo "All local validations passed."

validate-format: fmt ## Gate: formatting check

validate-lint: lint ## Gate: clippy

validate-metadata: validate-metainfo validate-desktop check-version ## Gate: AppStream + desktop + version consistency

validate-i18n: lint-i18n check-potfiles ## Gate: .po files + POTFILES completeness

validate-deps: audit deny spell-check check-unused-deps ## Gate: security advisories + licenses + typos + unused deps

validate-metainfo: ## Validate AppStream metainfo XML
	appstreamcli validate --pedantic data/com.example.GtkCrossPlatform.metainfo.xml

validate-desktop: ## Validate .desktop file
	desktop-file-validate data/com.example.GtkCrossPlatform.desktop

check-version: ## Verify Cargo.toml version == metainfo.xml version
	@CARGO_VER=$$(grep '^version' Cargo.toml | head -1 | grep -oP '[\d.]+'); \
	 META_VER=$$(grep -oP '(?<=<release version=")[^"]+' data/com.example.GtkCrossPlatform.metainfo.xml | head -1); \
	 echo "Cargo: $$CARGO_VER  Metainfo: $$META_VER"; \
	 [ "$$CARGO_VER" = "$$META_VER" ] || { echo "ERROR: version mismatch"; exit 1; }

check-potfiles: ## Verify all files with gettext() are listed in po/POTFILES
	@grep -rl 'gettext(' src/ | grep '\.rs$$' | sort > /tmp/has_gettext.txt; \
	 sort po/POTFILES | grep '\.rs$$' > /tmp/potfiles_rs.txt; \
	 MISSING=$$(comm -23 /tmp/has_gettext.txt /tmp/potfiles_rs.txt); \
	 if [ -n "$$MISSING" ]; then \
	   echo "Files with gettext() not in POTFILES:"; echo "$$MISSING"; exit 1; \
	 fi

audit: ## Run cargo audit (security advisories)
	cargo audit

deny: ## Run cargo deny check (licenses + advisories)
	cargo deny check

spell-check: ## Run typos spell-checker on all tracked files
	typos .

check-unused-deps: ## Detect unused Cargo dependencies with cargo machete
	cargo machete

ci: validate test ## Run the full CI pipeline locally (mirrors ci.yml)

# ── Icons & Assets ────────────────────────────────────────────────────────────

icons: icons-png icons-macos icons-windows ## Generate all icon formats (PNG, ICNS, ICO)

icons-png: ## Rasterize SVG icon to PNG at all standard sizes
	@for sz in $(ICON_SIZES); do \
		dir="data/icons/hicolor/$${sz}x$${sz}/apps"; \
		mkdir -p "$$dir"; \
		rsvg-convert -w $$sz -h $$sz -o "$$dir/$(APP_ID).png" "$(SVG_ICON)"; \
		echo "  PNG $${sz}x$${sz} OK"; \
	done

icons-macos: icons-png ## Generate macOS .icns from PNG icons
	@mkdir -p /tmp/$(APP_ID).iconset
	@cp data/icons/hicolor/16x16/apps/$(APP_ID).png   /tmp/$(APP_ID).iconset/icon_16x16.png
	@cp data/icons/hicolor/32x32/apps/$(APP_ID).png   /tmp/$(APP_ID).iconset/icon_16x16@2x.png
	@cp data/icons/hicolor/32x32/apps/$(APP_ID).png   /tmp/$(APP_ID).iconset/icon_32x32.png
	@cp data/icons/hicolor/64x64/apps/$(APP_ID).png   /tmp/$(APP_ID).iconset/icon_32x32@2x.png
	@cp data/icons/hicolor/128x128/apps/$(APP_ID).png /tmp/$(APP_ID).iconset/icon_128x128.png
	@cp data/icons/hicolor/256x256/apps/$(APP_ID).png /tmp/$(APP_ID).iconset/icon_128x128@2x.png
	@cp data/icons/hicolor/256x256/apps/$(APP_ID).png /tmp/$(APP_ID).iconset/icon_256x256.png
	@cp data/icons/hicolor/512x512/apps/$(APP_ID).png /tmp/$(APP_ID).iconset/icon_256x256@2x.png
	@cp data/icons/hicolor/512x512/apps/$(APP_ID).png /tmp/$(APP_ID).iconset/icon_512x512.png
	@rsvg-convert -w 1024 -h 1024 -o /tmp/$(APP_ID).iconset/icon_512x512@2x.png "$(SVG_ICON)"
	@iconutil -c icns -o data/icons/GtkCrossPlatform.icns /tmp/$(APP_ID).iconset
	@echo "  ICNS OK  data/icons/GtkCrossPlatform.icns"

icons-windows: icons-png ## Generate Windows .ico from PNG icons
	@python3 -c "\
import struct, os; \
sizes=[16,32,48,64,128,256]; \
imgs=[(sz, open(f'data/icons/hicolor/{sz}x{sz}/apps/$(APP_ID).png','rb').read()) for sz in sizes]; \
n=len(imgs); hdr=struct.pack('<HHH',0,1,n); off=6+16*n; entries=b''; data=b''; \
[(entries.__iadd__(struct.pack('<BBBBHHII',sz if sz<256 else 0,sz if sz<256 else 0,0,0,1,32,len(img),off+(sum(len(x) for _,x in imgs[:i])))), data.__iadd__(img)) for i,(sz,img) in enumerate(imgs)]; \
open('data/icons/GtkCrossPlatform.ico','wb').write(hdr+entries+data); \
print('  ICO OK  data/icons/GtkCrossPlatform.ico')"

install-icons: icons-png ## Install PNG + SVG icons to system hicolor theme
	@for sz in $(ICON_SIZES); do \
		install -Dm644 "data/icons/hicolor/$${sz}x$${sz}/apps/$(APP_ID).png" \
			"$(DESTDIR)/usr/share/icons/hicolor/$${sz}x$${sz}/apps/$(APP_ID).png"; \
	done
	install -Dm644 "$(SVG_ICON)" \
		"$(DESTDIR)/usr/share/icons/hicolor/scalable/apps/$(APP_ID).svg"
	-gtk-update-icon-cache -f -t "$(DESTDIR)/usr/share/icons/hicolor"

clean-icons: ## Remove generated PNG icons from data/icons/hicolor/
	@for sz in $(ICON_SIZES); do \
		rm -f "data/icons/hicolor/$${sz}x$${sz}/apps/$(APP_ID).png"; \
	done
	@rm -f data/icons/GtkCrossPlatform.icns data/icons/GtkCrossPlatform.ico
	@echo "Generated icons removed."

# ── Package (Distribution) ────────────────────────────────────────────────────

dist: dist-flatpak dist-macos ## Build all distribution packages (Flatpak + macOS DMG)

dist-flatpak: ## Build Flatpak bundle (x86_64)
	flatpak-builder --force-clean --user --install-deps-from=flathub \
		$(FLATPAK_BUILD_DIR) $(MANIFEST)

dist-flatpak-arm: ## Build Flatpak bundle (aarch64 — GNOME Mobile / PinePhone)
	flatpak-builder --force-clean --user --install-deps-from=flathub \
		--arch=aarch64 $(FLATPAK_BUILD_DIR)-arm $(MANIFEST)

dist-flatpak-run: dist-flatpak ## Build and run inside Flatpak sandbox
	flatpak-builder --run $(FLATPAK_BUILD_DIR) $(MANIFEST) $(BINARY)

dist-flatpak-install: dist-flatpak ## Build and install Flatpak for current user
	flatpak-builder --user --install --force-clean \
		$(FLATPAK_BUILD_DIR) $(MANIFEST)

dist-macos: icons-macos build-release ## Build macOS .app bundle and .dmg
	@echo "Creating app bundle..."
	rm -rf "$(BUNDLE)"
	mkdir -p "$(BUNDLE)/Contents/MacOS" "$(BUNDLE)/Contents/Resources" "$(BUNDLE)/Contents/Frameworks"
	@# Info.plist (version is patched dynamically from Cargo.toml)
	cp packaging/macos/Info.plist "$(BUNDLE)/Contents/Info.plist"
	/usr/libexec/PlistBuddy -c "Set :CFBundleVersion $(VERSION)" "$(BUNDLE)/Contents/Info.plist"
	/usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString $(VERSION)" "$(BUNDLE)/Contents/Info.plist"
	@# Launcher script (sets GTK env vars relative to bundle)
	cp packaging/macos/gtk-cross-platform-launcher "$(BUNDLE)/Contents/MacOS/gtk-cross-platform-launcher"
	chmod +x "$(BUNDLE)/Contents/MacOS/gtk-cross-platform-launcher"
	@# Binary and icon
	cp target/release/gtk-cross-platform "$(BUNDLE)/Contents/MacOS/gtk-cross-platform"
	cp data/icons/GtkCrossPlatform.icns "$(BUNDLE)/Contents/Resources/GtkCrossPlatform.icns"
	@# GLib schemas (system + project)
	mkdir -p "$(BUNDLE)/Contents/Resources/share/glib-2.0/schemas"
	cp "$(BREW)/share/glib-2.0/schemas/"*.xml "$(BUNDLE)/Contents/Resources/share/glib-2.0/schemas/"
	cp data/com.example.GtkCrossPlatform.gschema.xml \
		"$(BUNDLE)/Contents/Resources/share/glib-2.0/schemas/"
	glib-compile-schemas "$(BUNDLE)/Contents/Resources/share/glib-2.0/schemas/"
	@# gdk-pixbuf loaders
	mkdir -p "$(BUNDLE)/Contents/Resources/lib/gdk-pixbuf-2.0"
	cp "$$(find $(BREW)/lib/gdk-pixbuf-2.0 -name loaders.cache | head -1)" \
		"$(BUNDLE)/Contents/Resources/lib/gdk-pixbuf-2.0/loaders.cache"
	@# Bundle dylibs
	@echo "Bundling dylibs with dylibbundler..."
	dylibbundler --bundle-deps \
		--fix-file "$(BUNDLE)/Contents/MacOS/gtk-cross-platform" \
		--dest-dir "$(BUNDLE)/Contents/Frameworks" \
		--install-path "@executable_path/../Frameworks/" \
		--overwrite-files
	@# Create DMG
	@echo "Creating DMG..."
	rm -f "$(DMG_OUT)"
	create-dmg \
		--volname "GTK Cross Platform" \
		--volicon "data/icons/GtkCrossPlatform.icns" \
		--window-pos 200 120 \
		--window-size 600 400 \
		--icon-size 100 \
		--icon "$(BUNDLE)" 150 185 \
		--hide-extension "$(BUNDLE)" \
		--app-drop-link 450 185 \
		"$(DMG_OUT)" \
		"$(BUNDLE)"
	@echo "OK  $(DMG_OUT)"

# ── Publish ───────────────────────────────────────────────────────────────────

release: ci dist release-tag release-github ## Full release: validate, build, tag, publish

release-tag: ## Create and push a git tag for the current version
	@command -v gh >/dev/null 2>&1 || { echo "ERROR: gh not found — install via https://cli.github.com"; exit 1; }
	git tag -a "v$(VERSION)" -m "Release v$(VERSION)"
	git push origin "v$(VERSION)"

release-github: ## Create GitHub Release with all platform artifacts
	@command -v gh >/dev/null 2>&1 || { echo "ERROR: gh not found — install via https://cli.github.com"; exit 1; }
	gh release create "v$(VERSION)" --generate-notes \
		"$(FLATPAK_BUILD_DIR)/$(APP_ID).flatpak" \
		"$(FLATPAK_BUILD_DIR)-arm/$(APP_ID).flatpak" \
		"$(DMG_OUT)" \
		"GtkCrossPlatform-v$(VERSION)-windows-x86_64.zip"

# ── Clean & Cache ─────────────────────────────────────────────────────────────

clean: clean-build clean-flatpak ## Remove build artifacts (Cargo + Flatpak)

clean-all: clean clean-icons ## Remove all generated files including icons

clean-build: ## Remove Cargo build artifacts
	cargo clean

clean-flatpak: ## Remove Flatpak build dirs and repo
	rm -rf $(FLATPAK_BUILD_DIR) .flatpak-builder repo

cache-info: ## Show sizes of Cargo and Flatpak caches
	du -sh ~/.cargo/registry ~/.cargo/git .flatpak-builder/ 2>/dev/null || true

cache-prune: ## Prune Cargo registry cache (installs cargo-cache if needed)
	@command -v cargo-cache >/dev/null 2>&1 || cargo install cargo-cache
	cargo cache --autoclean

# ── Aliases (backwards compatibility) ────────────────────────────────────────

flatpak-build:     dist-flatpak         ## [alias] use dist-flatpak
flatpak-run:       dist-flatpak-run     ## [alias] use dist-flatpak-run
flatpak-install:   dist-flatpak-install ## [alias] use dist-flatpak-install
flatpak-build-arm: dist-flatpak-arm     ## [alias] use dist-flatpak-arm
dmg:               dist-macos           ## [alias] use dist-macos
test-nextest:      test                 ## [alias] use test

# ── .PHONY ────────────────────────────────────────────────────────────────────

.PHONY: \
  help \
  setup setup-rust setup-platform setup-macos setup-linux setup-windows setup-cargo-deps \
  build build-debug build-release schema run run-mobile watch \
  format fmt fmt-fix lint lint-i18n \
  test test-unit test-integration test-i18n coverage \
  validate validate-format validate-lint validate-metadata validate-i18n validate-deps \
  validate-metainfo validate-desktop check-version check-potfiles \
  audit deny spell-check check-unused-deps \
  ci \
  icons icons-png icons-macos icons-windows install-icons clean-icons \
  dist dist-flatpak dist-flatpak-arm dist-flatpak-run dist-flatpak-install dist-macos \
  release release-tag release-github \
  clean clean-all clean-build clean-flatpak cache-info cache-prune \
  flatpak-build flatpak-run flatpak-install flatpak-build-arm dmg test-nextest
