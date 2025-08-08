# q7 Launcher

A lightweight launcher for Linux written in Rust using egui/eframe.

Features
- Launch GUI apps by parsing .desktop files
- Fuzzy search over app names
- File search via `fd` (prefix: `f `)
- Command runner (fallback when no app/file chosen)
- Web search (prefix `?query` or `g query` → opens in browser)
- i3-friendly: undecorated, on-top, and self-centers on open

Current UI
- Centered, frameless input with larger text
- Icon + two-line result rows with a subtle full-width highlight for the selected row
- Enter or click executes and closes the launcher

Requirements
- Rust toolchain (https://rustup.rs)
- `fd` (fd-find): used for fast file search
- Optional: `rsvg-convert` for crisp SVG icons

Build
```sh
cargo build --release
```

Run
```sh
./target/release/q7-launcher
```

i3 integration
- Bind to a hotkey:
```
bindsym $mod+space exec --no-startup-id /path/to/q7-launcher/target/release/q7-launcher
```

- Float + center (robust rule):
```
for_window [title="^q7 launcher$"] \
	floating enable, \
	focus, \
	move position center, \
	sticky enable, \
	border pixel 0
```

- If it occasionally spawns off-center, use a post-launch nudge on the keybind:
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
