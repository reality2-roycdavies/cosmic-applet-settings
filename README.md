# COSMIC Applet Settings

A unified settings application for custom COSMIC desktop applets. Provides a single settings window with pages for each applet.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-2021-orange.svg)
![COSMIC](https://img.shields.io/badge/desktop-COSMIC-purple.svg)

## Features

- **Unified Settings**: One app to configure all custom COSMIC applets
- **Per-Applet Pages**: Dedicated settings page for each applet:
  - Tailscale VPN
  - RunKat CPU monitor
  - Bing Wallpaper
  - Pie Menu
  - WiFi Hotspot
- **Direct Launch**: Can be opened to a specific page via command line or desktop entry
- **Native COSMIC UI**: Built with libcosmic for a native look and feel

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) toolchain (1.75+)
- [just](https://github.com/casey/just) command runner
- System libraries:

```bash
# Debian/Ubuntu/Pop!_OS
sudo apt install libwayland-dev libxkbcommon-dev libssl-dev pkg-config just

# Fedora
sudo dnf install wayland-devel libxkbcommon-devel openssl-devel just

# Arch
sudo pacman -S wayland libxkbcommon openssl just
```

### Build and Install

```bash
git clone https://github.com/reality2-roycdavies/cosmic-applet-settings.git
cd cosmic-applet-settings

# Build release binary
just build-release

# Install binary, desktop entries, and icon to ~/.local
just install-local
```

### Other just commands

```bash
just build-debug       # Debug build
just run               # Build debug and run
just run-release       # Build release and run
just check             # Run clippy checks
just fmt               # Format code
just clean             # Clean build artifacts
just uninstall-local   # Remove installed files
```

### Uninstalling

```bash
just uninstall-local
```

## Usage

```bash
# Open settings (shows all pages)
cosmic-applet-settings

# Open directly to a specific page
cosmic-applet-settings tailscale
cosmic-applet-settings runkat
cosmic-applet-settings bing-wallpaper
cosmic-applet-settings pie-menu
cosmic-applet-settings hotspot
```

## License

MIT License - See [LICENSE](LICENSE) for details.

## Acknowledgments

- [System76](https://system76.com/) for the COSMIC desktop environment
