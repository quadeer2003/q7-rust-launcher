# q7 Launcher

A minimal, fast cross‑platform (Linux + Windows) launcher written in Rust using `eframe`/`egui`.

## Features
* App launch:
	* Linux: parses `.desktop` entries (with description + icon resolution)
	* Windows: enumerates Start Menu `.lnk` shortcuts and standalone `.exe` files
* Fuzzy search over app names (Skim matcher)
* File search:
	* Linux: `fd` (prefix: `f <term>`)
	* Windows: PowerShell scan of home + PATH plus `where.exe` fallback
* Command runner (fallback when no app/file / prefix match)
* Web search with configurable prefixes (`?`, `g `, `yt `, `w `, `gh `, etc.)
* Theme switching: type `theme` to list & apply built‑in color schemes (persisted)
* Minimal UI: centered (Linux) or screen‑centered (Windows), borderless, always-on-top
* Icon caching and startup optimizations for snappy feel

Planned / partial:
* Windows icon extraction for `.exe` / `.lnk` (stub in code – can be extended)
* Better DPI-aware multi‑monitor centering via winit APIs

## Current UI
* Single input bar, large font
* Two-line result rows (title + description / exec)
* Highlighted selection follows arrow keys (auto-scroll)
* Enter or click executes then exits

## Requirements
Mandatory:
* Rust toolchain (https://rustup.rs)

Linux extras:
* `fd` (fd-find) for file search (omit if you only use web/app/command)
* `rsvg-convert` (optional) for better SVG icon rasterization

Windows extras:
* PowerShell (built-in) used for broader file search
* Optional future: COM Shell icon extraction (already partially scaffolded)

## Build
```sh
cargo build --release
```

## Run
```sh
./target/release/q7-launcher   # Linux
target\\release\\q7-launcher.exe  # Windows (PowerShell / cmd)
```

## Assigning a Hotkey

### Linux (i3 / sway)
Add to your config:
```
bindsym $mod+space exec --no-startup-id /absolute/path/to/q7-launcher/target/release/q7-launcher
```

Optional centering rule (i3):
```
for_window [title="^q7 launcher$"] floating enable, focus, move position center, sticky enable, border pixel 0
```

### Linux (GNOME) via gsettings
1. Install / build binary somewhere stable (e.g. ~/bin/q7-launcher)
2. Create a custom keybinding slot:
```sh
gsettings set org.gnome.settings-daemon.plugins.media-keys custom-keybindings "['/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/q7launcher/']"
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/q7launcher/ name 'q7 Launcher'
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/q7launcher/ command '/home/you/bin/q7-launcher'
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/q7launcher/ binding '<Super>space'
```

### Linux (KDE Plasma)
System Settings → Shortcuts → Custom Shortcuts → Add Command:
* Trigger: Meta+Space
* Action: /path/to/q7-launcher

### Windows (Auto Hotkey - recommended)
Create a file `q7-hotkey.ahk`:
```
; Launch q7 launcher with Win+Space
#Space::
Run, C:\\Path\\To\\q7-launcher.exe
return
```
Run AutoHotkey (v2 adjust syntax accordingly) and double-click script. To autostart place script in `%AppData%\Microsoft\Windows\Start Menu\Programs\Startup`.

### Windows (Task Scheduler + Shortcut)
* Create a shortcut to the exe.
* Right-click → Properties → set Shortcut key field (e.g., Ctrl+Alt+Space).
Note: Windows reserves Win+Space for layout switching; override via AutoHotkey if desired.

### XFCE
Settings → Keyboard → Application Shortcuts → Add:
```
/path/to/q7-launcher/target/release/q7-launcher
```
Bind to e.g. Super+Space.

### Generic (systemd user service + hotkey daemon)
Use `sxhkd` or `autokey` mapping a key to the binary path.

Legacy i3 centering nudge (only if needed):
```
bindsym $mod+space exec --no-startup-id sh -lc '
	/path/to/q7-launcher/target/release/q7-launcher &
	sleep 0.08
	i3-msg [title="^q7 launcher$"] floating enable, move position center, focus'
```

Troubleshooting
- Confirm the window title if matching fails:
```
xprop | grep -E 'WM_NAME|_NET_WM_NAME'
```
Then update the i3 rule’s title accordingly.

## Custom prefixes and search URLs

You can define your own prefixes that open a URL with the typed text substituted in place of `%s`:

1) Copy the example to your config folder:
	 - System-wide default example: `assets/config.json`
	 - User override: `~/.config/q7-launcher/config.json`

2) Format:

```
{
	"search_engines": [
		{ "name": "DuckDuckGo", "prefix": "?",  "url": "https://duckduckgo.com/?q=%s" },
		{ "name": "Google",     "prefix": "g ", "url": "https://www.google.com/search?q=%s" },
		{ "name": "StackOverflow","prefix": "so ","url": "https://stackoverflow.com/search?q=%s" }
	]
}
```

Notes
- `prefix` is matched at the start of the query, the rest becomes the search term.
- `%s` is replaced with the URL-encoded term.
- First matching prefix wins; keep them distinct (e.g., `g ` vs `gh `).

## Themes
Type `theme` to list built-in themes, then select one. Theme persists via config (`current_theme`).

## Windows Notes
* Start Menu scan happens at startup (recursive). Large environments can add a slight delay; consider pruning paths if needed.
* Icon extraction is not yet implemented – currently shows placeholder (text) until implemented.
* PowerShell search depth is limited for speed; adjust in `search.rs` if you want deeper indexing.

## Installation

### Download Pre-built Binaries

Download the latest release from the [GitHub Releases](https://github.com/quadeer2003/q7-rust-launcher/releases) page:

- **Linux**: Download `q7-launcher-linux-x86_64.tar.gz`
- **Windows**: Download `q7-launcher-windows-x86_64.zip`

Extract and run the executable.

### Building from Source

Clone the repository and build:

```bash
git clone https://github.com/quadeer2003/q7-rust-launcher.git
cd q7-rust-launcher
cargo build --release
```

The binary will be in `target/release/q7-launcher` (Linux) or `target/release/q7-launcher.exe` (Windows).

### Creating a Release

To create a new release with automatic binary builds:

1. Create and push a new tag:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. GitHub Actions will automatically build binaries for Linux and Windows and create a release.

## Performance Tips
* Build with `--release` for significant speed.
* Remove `fd` if you don't use file search (comment out calls and dependency) to shrink binary.
* Consider stripping debug symbols (already enabled in release profile).

## License
MIT (adjust if you add third-party code requiring attribution).
