# Autocomplete Feature

The Q7 launcher now includes an autocomplete feature that allows you to quickly access and copy frequently used words to your clipboard.

## How to Use

1. **Toggle Autocomplete Mode**: Press the `Tab` key to switch between normal mode and autocomplete mode
2. **In Autocomplete Mode**: Type letters to see word suggestions that start with your input
3. **Select and Copy**: Use arrow keys to navigate suggestions and press `Enter` to copy the selected word to clipboard

## Configuration

The autocomplete feature is configured in your `config.json` file:

```json
{
  "autocomplete_words_file": "/path/to/your/words.txt"
}
```

## Words File Format

Create a text file with comma-separated words:

```
rust, python, javascript, programming, development, algorithm, data, structure
```

## Default Words File

A sample words file is provided at `assets/autocomplete_words.txt` with programming-related terms.

## Visual Indicators

- **Mode Indicator**: Shows "🔤 Autocomplete Mode" when active
- **Hint Text**: Search bar shows different hints based on current mode
- **Results**: In autocomplete mode, shows "Copy to clipboard" as the action

## Keyboard Shortcuts

- `Tab`: Toggle between normal and autocomplete mode
- `↑` / `↓`: Navigate through suggestions
- `Enter`: Copy selected word to clipboard (closes launcher)
- `Escape`: Close launcher without action

## Requirements

- **Linux**: `xclip` or `xsel` must be installed for clipboard functionality
- **Windows**: Uses built-in `clip` command

## Adding Your Own Words

You can create your own words file and point to it in the config:

1. Create a text file with comma-separated words
2. Update `autocomplete_words_file` in your config to point to your file
3. Restart the launcher to load the new words

Example custom words file:
```
project1, project2, email@example.com, frequently, used, phrases, commands
```
