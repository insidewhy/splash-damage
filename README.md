# splash-damage

A keyboard remapper daemon for Wayland. Intercepts keyboard input at the evdev level and remaps key combinations based on a TOML config, with per-application rules.

Built to bring macOS-style `Cmd+C`/`Cmd+V` shortcuts to Linux by remapping `Super+key` to `Ctrl+key`, with the ability to exclude specific apps like terminal emulators.

## How it works

```
Physical Keyboard (/dev/input/eventX)
        │  ← exclusive grab (EVIOCGRAB)
        ▼
   splash-damage daemon
        │
        ├─ TOML config (remap rules + per-app exclusions)
        ├─ KWin script + D-Bus (active window detection)
        │
        ▼
   Virtual Keyboard (/dev/uinput)
        │
        ▼
   KDE Plasma / Applications
```

The daemon grabs your physical keyboard exclusively, processes each key event through remapping rules, and emits the result through a virtual keyboard device. Active window detection is handled via a KWin script that reports focus changes over D-Bus.

## Installation

```bash
# Build release binary
make build

# Build and install to ~/.local/bin
make install
```

## Configuration

Default config location: `~/.config/splash-damage/config.toml`

```toml
[[remap]]
from = "super+c"
to = "ctrl+c"
exclude = ["kitty"]

[[remap]]
from = "super+v"
to = "ctrl+v"
exclude = ["kitty"]
```

Each `[[remap]]` entry defines:
- `from` - the key combination to intercept
- `to` - the key combination to emit instead
- `exclude` - list of window classes where the remap should not apply (matched against the active window's `resourceClass`)

### Supported keys

**Modifiers:** `ctrl`, `shift`, `alt`, `super` (also `meta`, `cmd`, `control`)

**Keys:** `a`-`z`, `0`-`9`, `space`, `enter`, `tab`, `escape`, `backspace`, `delete`, `up`, `down`, `left`, `right`, `home`, `end`, `pageup`, `pagedown`, `f1`-`f12`

### Finding window class names

Run `splash-damage` and switch between windows - the log output shows the `resource_class` for each focused window:

```
INFO splash_damage::window: active window changed resource_class="chromium" caption="..."
INFO splash_damage::window: active window changed resource_class="kitty" caption="..."
```

## Usage

The binary uses Linux capabilities (`CAP_DAC_OVERRIDE`) to access input devices without root. `make install` sets this up automatically via `setcap`.

```bash
# With explicit config path
splash-damage /path/to/config.toml

# Uses ~/.config/splash-damage/config.toml by default
splash-damage
```

Stop with `Ctrl+C` - the daemon will clean up the virtual keyboard and KWin script.

## Autostart with KDE

To have splash-damage start automatically when you log in:

```bash
make enable
```

This will:
- Build and install the binary to `~/.local/bin` (with `setcap`)
- Install a systemd user service
- Enable and start the service

To rebuild and restart after making changes:

```bash
make update
```

To check the service status:

```bash
systemctl --user status splash-damage
```

To stop and disable:

```bash
make disable
```

## Requirements

- Linux with evdev and uinput support
- Rust toolchain for building
- Per-app window detection currently only supports **KDE Plasma 6** (Wayland) via KWin scripting + D-Bus. The core remapping works on any Wayland compositor, but exclude lists require window detection.

## Contributing

PRs are welcome - especially for adding active window detection support for other compositors (Hyprland, Sway, GNOME, etc.).
