<p align="center">
  <img src="public/ptools-icon.png" width="112" alt="PTools Logo" />
</p>

<h1 align="center">PTools</h1>

<p align="center">A local-first clipboard history app for macOS</p>

<p align="center">
  <a href="README.md">简体中文</a> | <a href="README_EN.md">English</a>
</p>

<p align="center">
  <a href="#screenshots">Screenshots</a> · <a href="#download-and-install">Download</a> · <a href="#highlights">Features</a> · <a href="#json-formatter">JSON Formatter</a> · <a href="#how-to-use">How to Use</a> · <a href="#privacy-and-permissions">Privacy</a> · <a href="#license">License</a>
</p>

PTools is a utility built around copy and paste. It keeps copied text, images, and files on your Mac, so your clipboard history remains local and private.

## Screenshots

### Clipboard Panel

![PTools clipboard panel](images/Panel.png)

### Settings

![PTools settings](images/Setting.png)

## Download and Install

1. Download the matching `.dmg` from [Releases](https://github.com/white-super/PTools/releases):
   - `aarch64` for Apple Silicon Macs.
   - `x86_64` for Intel Macs.
2. Open the `.dmg` and drag **PTools** into Applications.
3. Press the default shortcut `Ctrl + V` to open the clipboard panel.

> If macOS blocks the first launch, allow the app in **System Settings → Privacy & Security**.

## Highlights

- **Local history** for copied text, images, and files.
- **Global shortcut** that opens without interrupting your current app.
- **Filters and tags** for finding the right item quickly.
- **Restore and paste** with double-click, Enter, or number keys.
- **Flexible retention** controls for content types, duration, and record limit.
- **Software updates** from Settings, with a GitHub download fallback.
- **JSON formatting** for JSON/JSONC clipboard content, including formatting, minifying, comment removal, and conversion to XML or TypeScript.

## JSON Formatter

1. Select a JSON or JSONC text entry in the clipboard panel.
2. Press `F` to open an independent JSON formatting window. Escaped JSON text, such as `{\"name\":\"value\"}`, is also recognized.
3. Use the toolbar to format, fold or unfold, remove comments, copy a minified or escaped version, or copy an XML/TypeScript conversion.
4. Press `Command + D` or `Alt + D` to pin or unpin the window. Pinned windows stay visible when unfocused, and multiple formatter windows can be open at the same time.

## How to Use

1. Copy text, an image, or files; PTools records enabled types locally.
2. Press `Ctrl + V` to open the panel. You can change this shortcut in Settings.
3. Filter entries by format or tag, then select an item.
4. Double-click, press Enter, or press `1`–`5` to restore it.
5. With **Auto paste after double-click** enabled, PTools pastes into your previous app; otherwise it only restores the clipboard.

## Settings

- **Clipboard shortcut**: Set the global shortcut for opening the panel.
- **Retention and record limit**: Control how long local history is kept.
- **Content types**: Enable or disable text, image, and file history separately.
- **Accessibility permission**: View its status and open macOS System Settings when needed.
- **Software updates**: Check for updates or open the GitHub download page.

## Privacy and Permissions

PTools does not upload clipboard history. Clipboard items and settings stay on your device. You can shorten retention, disable a content type, or clear all history at any time. See the [Privacy Policy](PRIVACY.md) for details.

To use global shortcuts and auto-paste, allow **PTools** in **System Settings → Privacy & Security → Accessibility**. This permission is only used to handle shortcuts and send paste actions after you explicitly choose an item.

## Feedback and Contributing

Report bugs or share ideas through [Issues](https://github.com/white-super/PTools/issues). See [CONTRIBUTING.md](CONTRIBUTING.md) to contribute code.

## License

PTools is open source under the [Apache License 2.0](LICENSE).
