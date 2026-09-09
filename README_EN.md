<p align="center">
  <img src="public/ptools-icon.png" width="112" alt="PTools Logo" />
</p>

<h1 align="center">PTools</h1>

<p align="center">A local-first cross-platform clipboard history app</p>

<p align="center">
  <a href="README.md">简体中文</a> | <a href="README_EN.md">English</a>
</p>

<p align="center">
  <a href="#screenshots">Screenshots</a> · <a href="#download-and-install">Download</a> · <a href="#highlights">Features</a> · <a href="#formatting-tools">Formatting Tools</a> · <a href="#todo">TODO</a> · <a href="#how-to-use">How to Use</a> · <a href="#privacy-and-permissions">Privacy</a> · <a href="#license">License</a>
</p>

PTools is a utility built around copy and paste. It keeps copied text, images, and files on your Mac, so your clipboard history remains local and private.

## Screenshots

### Clipboard Panel

![PTools clipboard panel](images/Panel.png)

### Settings

![PTools settings](images/Setting.png)

## Download and Install

1. Download the matching installer from [Releases](https://github.com/white-super/PTools/releases):
   - `aarch64` for Apple Silicon Macs.
   - `x86_64` for Intel Macs.
   - Windows users should download the `x64-setup.exe` installer.
2. On macOS, open the `.dmg` and drag **PTools** into Applications. On Windows, run the `.exe` installer and follow the prompts.
3. Press `Ctrl + V` on macOS or `Alt + V` on Windows to open the clipboard panel.

> If macOS blocks the first launch, allow the app in **System Settings → Privacy & Security**. Windows may show a SmartScreen prompt for an unsigned installer; choose **More info → Run anyway** if you trust the download.

## Highlights

- **Local history** for copied text, images, and files.
- **Global shortcut** that opens without interrupting your current app.
- **Filters and tags** for finding the right item quickly.
- **Restore and paste** with double-click, Enter, or number keys.
- **Flexible retention** controls for content types, duration, and record limit.
- **Software updates** from Settings, with a GitHub download fallback.
- **Text formatting tools** for JSON/JSONC, XML, HTML, URL, Base64, and date content, with a dedicated formatter or conversion workspace for each type.

## Formatting Tools

1. Select a text entry in the clipboard panel.
2. Press `F` to open an independent formatting workspace. Escaped JSON text is also recognized.
3. Right-click a text card and open “Use Tool” to view and select all text tools.
4. JSON supports formatting, folding, comment removal, minified or escaped copies, and XML/TypeScript conversion. URL, Base64, and date entries expose their corresponding conversion actions.
5. All formatting tools support shortcuts: use `Ctrl + R` for regular formatting; JSON comment removal, minifying, escaping, and conversions use `Ctrl + Q/W/A/S/T`; URL, Base64, and date conversions use `Ctrl + E/D`.
6. The toolbar includes an auto-wrap toggle enabled by default. When text is selected, formatting and encoding/decoding apply only to the selection; otherwise they apply to the full content.
7. Press `Command + D` or `Alt + D` to pin or unpin the window. Pinned windows stay visible when unfocused, and multiple formatter windows can be open at the same time.

## Text Comparison (1.4.0 Development Build)

1. Select a text or text-file card and press `D` to mark the left source. Select another card and press `D` to open a side-by-side comparison. The context menu provides the same actions.
2. A Diff icon appears in the card's fixed-height header without resizing the card. Press `D` on the marked card again or click the icon to clear it. The snapshot survives search, filters, pagination, and panel hiding, and clears after opening a comparison or exiting the app. Multi-file cards prompt you to select one file.
3. Raw comparison is the default. Enable formatted comparison for JSON/JSONC, XML, or HTML, with automatic or manual type selection and support for escaped JSON. XML/HTML retain meaningful whitespace; formatted equality does not imply byte-identical source files.
4. Includes inline highlights, synchronized scrolling, difference navigation, swapping sides, line wrapping, and collapsing unchanged regions. Use `Ctrl + Q/E` for previous/next difference and `Ctrl + R` to recompute.
5. Edit either side directly with live difference highlighting and undo/redo. Typing does not reformat your text; press `Ctrl + R` to reformat when formatted comparison is enabled. Each side supports importing UTF-8 text files (including UTF-8 BOM), restoring the source, and copying displayed content. Images, binary files, and directories are unsupported. Edits affect temporary copies only; files and history entries are never overwritten automatically.
6. Multiple independent windows follow the app theme and remain open when unfocused. Use `Command + D` / `Alt + D` to toggle always-on-top. The global clipboard shortcut remains available. Closing an edited draft requires confirmation.
7. Full formatting and comparison run in a cancellable background worker on your device. Direct edits use local incremental comparison to preserve the caret and undo history. Content is not uploaded.

## TODO

- [ ] **Password manager**: Generate random passwords and store them encrypted in the local database.
- [x] **Extended formatting**: Support XML, HTML, URL encode/decode, date formatting and conversion, and Base64 encoding/decoding.
- [x] **Text comparison**: Select text or file cards with two presses of `D`, with side-by-side differences and formatted JSON/XML/HTML comparison.
- [ ] **Sequential paste**: When enabled, arrange multiple clipboard items in order and paste them one by one with a shortcut.
- [ ] **cURL request tool**: Parse cURL text in a dedicated request workspace and inspect responses.
- [ ] **Image crop tool**: Crop, resize, copy, or save clipboard images.

These features are planned for version `1.4.0`; the exact interactions and shortcuts may change during implementation.

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

On macOS, allow **PTools** in **System Settings → Privacy & Security → Accessibility** to use global shortcuts and auto-paste. Windows does not require macOS Accessibility permission. This permission is only used to handle shortcuts and send paste actions after you explicitly choose an item.

## Feedback and Contributing

Report bugs or share ideas through [Issues](https://github.com/white-super/PTools/issues). See [CONTRIBUTING.md](CONTRIBUTING.md) to contribute code.

## License

PTools is open source under the [Apache License 2.0](LICENSE).
