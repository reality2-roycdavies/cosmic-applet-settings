# COSMIC Applet Settings

A unified settings application for custom COSMIC desktop applets. Provides a single settings window that dynamically discovers registered applets and renders their configuration inline using a JSON-based CLI protocol.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-2021-orange.svg)
![COSMIC](https://img.shields.io/badge/desktop-COSMIC-purple.svg)

## Features

- **Dynamic Discovery**: Automatically finds registered applets via JSON descriptor files
- **Inline Settings**: Renders applet configuration directly — no separate windows needed
- **Rich Widget Protocol**: Supports toggle, select, slider, text, info, image preview, and scrollable list widgets
- **Per-Item Actions**: List items can have their own action buttons (e.g. Apply, Delete)
- **Confirmation Flow**: Destructive actions show inline Confirm/Cancel before executing
- **Live Refresh**: Applets can specify a refresh interval for automatic UI updates (e.g. timer status)
- **Conditional Visibility**: Settings can be shown/hidden based on the value of other settings
- **Native COSMIC UI**: Built with libcosmic for a native look and feel

## How It Works

### Applet Registration

Each applet registers itself by placing a JSON descriptor in `~/.config/cosmic-applet-settings/registry/`:

```json
{
  "name": "Bing Wallpaper",
  "icon": "io.github.reality2_roycdavies.cosmic-bing-wallpaper-symbolic",
  "applet_id": "io.github.reality2_roycdavies.cosmic-bing-wallpaper",
  "settings_cmd": "cosmic-bing-wallpaper"
}
```

### CLI Settings Protocol

The hub communicates with applets via three CLI commands:

| Command | Purpose |
|---------|---------|
| `<binary> --settings-describe` | Returns a JSON schema describing all settings, sections, and actions |
| `<binary> --settings-set <key> <json_value>` | Updates a setting value |
| `<binary> --settings-action <action_id> [<item_id>]` | Triggers an action (with optional item ID for per-item actions) |

### Schema Format

The `--settings-describe` output defines the full UI:

```json
{
  "title": "My Applet",
  "description": "Optional description text.",
  "refresh_interval": 10,
  "sections": [
    {
      "title": "Section Name",
      "actions": [
        {"id": "fetch", "label": "Fetch Now", "style": "suggested"}
      ],
      "items": [
        {"type": "image", "key": "preview", "label": "", "value": "/path/to/image.jpg", "height": 280.0},
        {"type": "info", "key": "status", "label": "Status", "value": "Active"},
        {"type": "toggle", "key": "enabled", "label": "Enable Feature", "value": true},
        {"type": "select", "key": "mode", "label": "Mode", "value": "auto",
         "options": [{"value": "auto", "label": "Automatic"}, {"value": "manual", "label": "Manual"}]},
        {"type": "slider", "key": "hour", "label": "Hour", "value": 8, "min": 0, "max": 23, "step": 1, "unit": ":00",
         "visible_when": {"key": "enabled", "equals": true}},
        {"type": "text", "key": "name", "label": "Name", "value": "default", "placeholder": "Enter name"},
        {"type": "list", "key": "history", "label": "", "value": null,
         "list_items": [
           {
             "id": "item-1", "image": "/path/to/thumb.jpg",
             "title": "Item Title", "subtitle": "Details",
             "actions": [
               {"id": "apply", "label": "Apply", "style": "suggested"},
               {"id": "delete", "label": "Delete", "style": "destructive", "confirm": "Delete this item?"}
             ]
           }
         ]}
      ]
    }
  ],
  "actions": [
    {"id": "global_action", "label": "Do Something", "style": "standard"}
  ]
}
```

### Widget Types

| Type | Description |
|------|-------------|
| `toggle` | Boolean on/off switch |
| `select` | Dropdown with predefined options |
| `slider` | Numeric range with step and unit |
| `text` | Text input with placeholder and submit-on-enter |
| `info` | Read-only label/value display |
| `image` | Image preview with configurable height, card-styled container |
| `list` | Scrollable list of items with thumbnails and per-item action buttons |

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
# Open settings (discovers all registered applets)
cosmic-applet-settings

# Open directly to a specific applet's page
cosmic-applet-settings io.github.reality2_roycdavies.cosmic-bing-wallpaper
```

## Managed Applets

This settings app provides configuration for the following COSMIC applets:

| Applet | Description |
|--------|-------------|
| **[cosmic-runkat](https://github.com/reality2-roycdavies/cosmic-runkat)** | Animated running cat CPU indicator for the panel |
| **[cosmic-bing-wallpaper](https://github.com/reality2-roycdavies/cosmic-bing-wallpaper)** | Daily Bing wallpaper manager with auto-update |
| **[cosmic-pie-menu](https://github.com/reality2-roycdavies/cosmic-pie-menu)** | Radial/pie menu app launcher with gesture support |
| **[cosmic-tailscale](https://github.com/reality2-roycdavies/cosmic-tailscale)** | Tailscale VPN status and control applet |
| **[cosmic-hotspot](https://github.com/reality2-roycdavies/cosmic-hotspot)** | WiFi hotspot toggle applet |

### Other Related Projects

| Project | Description |
|---------|-------------|
| **[cosmic-konnect](https://github.com/reality2-roycdavies/cosmic-konnect)** | Device connectivity and sync between Linux and Android |
| **[cosmic-konnect-android](https://github.com/reality2-roycdavies/cosmic-konnect-android)** | Android companion app for Cosmic Konnect |

## Adding Protocol Support to Your Applet

To make your applet configurable via this hub:

1. Implement `--settings-describe` to output a JSON schema
2. Implement `--settings-set <key> <json_value>` to apply changes (respond with `{"ok": true, "message": "..."}`)
3. Implement `--settings-action <action_id> [<item_id>]` for buttons (same response format)
4. Place a registry descriptor JSON in `~/.config/cosmic-applet-settings/registry/`

## License

MIT License - See [LICENSE](LICENSE) for details.

## Acknowledgments

- [System76](https://system76.com/) for the COSMIC desktop environment
