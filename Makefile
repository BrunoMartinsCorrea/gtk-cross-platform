.DEFAULT_GOAL := run

APP_ID           := com.example.GtkCrossPlatform
BINARY           := gtk-cross-platform
MANIFEST         := $(APP_ID).json
FLATPAK_BUILD_DIR := .flatpak-builder/build

.PHONY: setup build run test clean lint fmt fmt-fix \
        run-mobile \
        flatpak-build flatpak-run flatpak-install flatpak-build-arm \
        setup-macos setup-windows

# ── Local build ───────────────────────────────────────────────────────────────

setup:
	cargo fetch

build:
	cargo build

run:
	cargo run

test:
	cargo test

clean:
	cargo clean
	rm -rf $(FLATPAK_BUILD_DIR) .flatpak-builder repo

lint:
	cargo clippy -- -D warnings

fmt:
	cargo fmt --check

fmt-fix:
	cargo fmt

# ── Mobile emulation (Linux only) ────────────────────────────────────────────
# Simulates GNOME Mobile constraints (360 sp wide, touch-first)

run-mobile:
	GTK_DEBUG=interactive cargo run

# ── Flatpak (x86_64 / native arch) ───────────────────────────────────────────

flatpak-build:
	flatpak-builder --force-clean --user --install-deps-from=flathub \
		$(FLATPAK_BUILD_DIR) $(MANIFEST)

flatpak-run: flatpak-build
	flatpak-builder --run $(FLATPAK_BUILD_DIR) $(MANIFEST) $(BINARY)

flatpak-install: flatpak-build
	flatpak-builder --user --install --force-clean \
		$(FLATPAK_BUILD_DIR) $(MANIFEST)

# ── Flatpak (aarch64 — GNOME Mobile / PinePhone) ─────────────────────────────
# Requires: flatpak-builder + qemu-user-static (for cross-compilation)

flatpak-build-arm:
	flatpak-builder --force-clean --user --install-deps-from=flathub \
		--arch=aarch64 \
		$(FLATPAK_BUILD_DIR)-arm $(MANIFEST)

# ── macOS setup (requires Homebrew) ──────────────────────────────────────────
# NOTE: Adwaita theme applies; native macOS look is not supported — by design.

setup-macos:
	@echo "Installing GTK4 stack via Homebrew..."
	brew install gtk4 libadwaita rust
	@echo "Run 'make build' to compile."

# ── Windows setup (requires MSYS2 + MINGW64) ─────────────────────────────────
# NOTE: Adwaita theme applies; native Windows look is not supported — by design.

setup-windows:
	@echo "Open MSYS2 MINGW64 shell and run:"
	@echo "  pacman -S mingw-w64-x86_64-gtk4 mingw-w64-x86_64-libadwaita \\"
	@echo "            mingw-w64-x86_64-rust"
	@echo "Then: make build"
