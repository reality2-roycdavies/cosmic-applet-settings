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

    # Install desktop entry
    mkdir -p ~/.local/share/applications
    cp resources/{{appid}}.desktop ~/.local/share/applications/

    # Install icon
    mkdir -p ~/.local/share/icons/hicolor/scalable/apps
    cp resources/{{appid}}.svg ~/.local/share/icons/hicolor/scalable/apps/

    # Create registry directory for applet descriptors
    mkdir -p ~/.local/share/cosmic-applet-settings/applets

    echo "Installation complete!"

# Uninstall from local user
uninstall-local:
    rm -f ~/.local/bin/{{name}}
    rm -f ~/.local/share/applications/{{appid}}.desktop
    rm -f ~/.local/share/icons/hicolor/scalable/apps/{{appid}}.svg

# Build and run
br: build-debug run
