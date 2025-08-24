# Spotify Controls

Your launcher now includes comprehensive Spotify and media player controls using `playerctl`.

## How to Use

1. **Type `spotify`** in the launcher to see all available controls
2. **Type `spotify [command]`** to filter specific commands (e.g., `spotify vol` for volume controls)

## Available Commands

### 🎵 Basic Playback Controls
- **play** - ▶️ Start playback
- **pause** - ⏸️ Pause playback  
- **play-pause** - ⏯️ Toggle play/pause
- **stop** - ⏹️ Stop playback
- **next** - ⏭️ Next track
- **previous** - ⏮️ Previous track

### 🔊 Volume Controls
- **vol-50** - 🔉 Set volume to 50%
- **vol-80** - 🔊 Set volume to 80%
- **vol-up** - 🔊 Increase volume by 10%
- **vol-down** - 🔉 Decrease volume by 10%
- **vol-max** - 🔊 Set volume to 100%
- **vol-mute** - 🔇 Mute volume

### 🎛️ Advanced Controls
- **shuffle** - 🔀 Toggle shuffle mode
- **repeat** - 🔁 Toggle repeat mode
- **status** - ℹ️ Show current player status
- **metadata** - 📋 Show current track info

## Examples

1. **Quick play/pause**: Type `spotify play-pause`
2. **Volume control**: Type `spotify vol` then select from volume options
3. **Basic controls**: Type `spotify` and browse all available commands

## Requirements

- `playerctl` must be installed (already installed on your system ✅)
- Works with Spotify, VLC, and other MPRIS-compatible media players

## Technical Details

The commands use `playerctl` under the hood:
- `playerctl play` - for playback
- `playerctl volume 0.5` - for 50% volume
- `playerctl next` - for next track
- etc.

This means it works with any media player that supports the MPRIS D-Bus interface, not just Spotify!
