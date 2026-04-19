# GTK Cross-Platform

A cross-platform desktop application built with GTK4 + LibAdwaita (Vala),
targeting Linux, macOS, Windows, and GNOME Mobile (Phosh / postmarketOS).

## Screenshots

> Add screenshots to `docs/screenshots/` and link them here.

## Dependencies

| Dependency  | Version |
|-------------|---------|
| GTK4        | ≥ 4.12  |
| LibAdwaita  | ≥ 1.4   |
| Vala        | ≥ 0.56  |
| Meson       | ≥ 0.62  |
| Ninja       | any     |

## Build and Run

### Linux (native)

```sh
make setup      # first time only
make build
make run
make test       # run unit tests
```

### Mobile emulation (Linux)

```sh
make run-mobile   # GTK_DEBUG=interactive — simulates narrow screen
```

### Flatpak

```sh
make flatpak-build      # build x86_64 Flatpak
make flatpak-run        # run in Flatpak sandbox
make flatpak-install    # install locally
make flatpak-build-arm  # cross-compile for aarch64 (PinePhone / Librem 5)
```

### macOS

```sh
make setup-macos   # install deps via Homebrew (one-time)
make setup && make build
```

> Adwaita theme is used on macOS and Windows — native look is not applied by design.

### Windows (MSYS2 / MINGW64)

```sh
make setup-windows   # prints required pacman commands
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

GPL-3.0-or-later — see [LICENSE](LICENSE).
