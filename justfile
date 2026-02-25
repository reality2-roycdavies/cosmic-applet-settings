name := 'cosmic-applet-settings'
appid := 'io.github.reality2_roycdavies.cosmic-applet-settings'

# Default recipe: build release
default: build-release

# Build in debug mode
build-debug:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Run in debug mode
run:
    cargo run

# Run in release mode
run-release:
    cargo run --release

# Check code with clippy
check:
    cargo clippy --all-features

# Format code
fmt:
    cargo fmt

# Clean build artifacts
clean:
    cargo clean

# Install to local user
install-local:
    #!/bin/bash
    set -e

    echo "Stopping any running instances..."
    pkill -x "cosmic-applet-settings" 2>/dev/null || true
    sleep 1

    # Install binary
    mkdir -p ~/.local/bin
    rm -f ~/.local/bin/{{name}}
    cp target/release/{{name}} ~/.local/bin/

    # Install desktop entries (main + per-page)
    mkdir -p ~/.local/share/applications
    cp resources/{{appid}}.desktop ~/.local/share/applications/
    cp resources/{{appid}}.Tailscale.desktop ~/.local/share/applications/
    cp resources/{{appid}}.RunKat.desktop ~/.local/share/applications/
    cp resources/{{appid}}.BingWallpaper.desktop ~/.local/share/applications/
    cp resources/{{appid}}.PieMenu.desktop ~/.local/share/applications/
    cp resources/{{appid}}.Hotspot.desktop ~/.local/share/applications/

    # Install icon
    mkdir -p ~/.local/share/icons/hicolor/scalable/apps
    cp resources/{{appid}}.svg ~/.local/share/icons/hicolor/scalable/apps/

    echo "Installation complete!"

# Uninstall from local user
uninstall-local:
    rm -f ~/.local/bin/{{name}}
    rm -f ~/.local/share/applications/{{appid}}.desktop
    rm -f ~/.local/share/applications/{{appid}}.Tailscale.desktop
    rm -f ~/.local/share/applications/{{appid}}.RunKat.desktop
    rm -f ~/.local/share/applications/{{appid}}.BingWallpaper.desktop
    rm -f ~/.local/share/applications/{{appid}}.PieMenu.desktop
    rm -f ~/.local/share/applications/{{appid}}.Hotspot.desktop
    rm -f ~/.local/share/icons/hicolor/scalable/apps/{{appid}}.svg

# Build and run
br: build-debug run
