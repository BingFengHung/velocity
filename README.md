# Velocity (`vl` / `velocity`)

[English](README.md) | [繁體中文](README.zh-TW.md)

**Velocity** is a blazingly fast, lightweight, and modern terminal file browser written in Rust. Features a two-pane layout, true-color syntax highlighting, instant fuzzy search, Git status integration, high-resolution graphics protocol image previews (Sixel / Kitty / iTerm2 / Lanczos3), archive previews, one-click self-updating, and Nerd Font icons.

Designed as a high-velocity, zero-overhead file navigation and management tool for terminal power users, developers, and AI-assisted pair-programming workflows.

---

## ✨ Features

- ⚡ **High Velocity & Zero Dependencies**: Written in pure Rust with native performance and sub-millisecond startup time.
- 🖼️ **High-Definition Image Previews**: Built-in support for **Sixel**, **Kitty**, and **iTerm2** native graphics protocols, with fallback to crisp **Lanczos3** resampled & contrast-enhanced TrueColor Half-Blocks (PNG, JPEG, GIF, WebP, BMP, ICO).
- 🔄 **One-Command Self Update**: Run `velocity --update` to automatically detect, download, and install the latest release directly from GitHub.
- 🪟 **Two-Pane Layout**: Left pane for interactive directory navigation; right pane for real-time previews.
- 🌈 **True-Color Syntax Highlighting**: Built-in syntax highlighting engine for Rust, Python, JavaScript/TypeScript, Go, C/C++, JSON, TOML, YAML, Markdown, HTML, CSS, Shell, and SQL.
- 📦 **Instant Archive Inspection**: Peek inside `.zip` files to view directory structures and uncompressed sizes without extraction.
- 🌿 **Git Status & Branch Integration**: Live Git branch display and inline file status badges (`M` modified, `+` staged, `?` untracked, `D` deleted).
- 🔍 **Fzf-like Intelligent Fuzzy Search**: Press `/` for instant fuzzy subsequence matching with character highlights and ranking.
- 🔄 **Multi-Dimensional Sorting**: Press `s` to cycle sorting by Name (A-Z), Modification Time (newest first), File Size (largest first), or Extension.
- 🛠️ **Lightweight File Operations**:
  - `y`: Copy absolute file path to clipboard.
  - `a`: Create new file (or directory ending with `/`).
  - `c`: Rename selected item.
  - `d`: Delete item (with confirmation prompt).
- 🔤 **Multiple Icon Themes**: Built-in support for **Nerd Font** icons, **Emoji** icons, and **ASCII** fallback mode.
- 🌐 **Accurate CJK & Unicode Width**: Prevents terminal box border misalignment caused by East Asian wide characters or emojis.
- 📝 **Editor Integration**: Press `e` to open the highlighted file in `$EDITOR`, VS Code, or system default text editor.
- 🌍 **Cross-Platform**: Windows, Linux, and macOS.

---

## 📦 Installation & Auto Update

### Pre-built Binaries

Download pre-compiled release binaries for your operating system from GitHub Releases.

### Automatic Self-Update

Once installed, simply run:
```bash
# Check and automatically upgrade to the latest release
velocity --update
velocity -u

# Check for new versions without upgrading
velocity --check-update
```

---

## 🚀 Quick Start

```bash
# Launch Velocity in current working directory
velocity

# (Optional) Add alias 'vl' for maximum typing speed:
# PowerShell: Set-Alias vl velocity
# Bash/Zsh:   alias vl="velocity"
vl

# Start browsing from a specific path
velocity D:\projects\my_repo

# Choose specific image preview graphics protocol (auto, kitty, iterm2, sixel, blocks)
velocity --image-protocol auto
velocity --image-protocol sixel
velocity --image-protocol kitty

# Use Emoji icons (if no Nerd Font is installed)
velocity --icons emoji
velocity -i emoji

# Use ASCII fallback mode for maximum terminal compatibility
velocity --icons ascii
velocity -i ascii

# Show hidden files (starting with '.')
velocity --all
velocity -a
```

---

## ⌨️ Keybindings

| Key | Action |
|---|---|
| `↑` `↓` / `k` `j` | Move cursor up / down |
| `PgUp` `PgDn` | Scroll page up / down |
| `Home` `End` | Jump to top / bottom |
| `→` / `l` / `Enter` | Enter highlighted directory |
| `←` / `h` / `Backspace` | Go back to parent directory (cursor remembers previous folder) |
| `/` | Instant fuzzy search / filter mode (`Enter` to confirm, `Esc` to cancel) |
| `s` / `o` | Cycle sorting mode (Name → Time → Size → Extension) |
| `y` | Copy absolute file path to system clipboard |
| `a` / `n` | Create new file or folder (append `/` for folder) |
| `c` | Rename selected file or folder |
| `d` | Delete selected item (with confirmation prompt) |
| `e` | Open selected file in `$EDITOR` / VS Code / default editor |
| `r` | Refresh directory & Git status |
| `.` | Toggle show/hide hidden files |
| `q` / `Esc` | Quit Velocity |

---

## 🛠️ CLI Options

```
Usage: velocity.exe [OPTIONS] [PATH]

Arguments:
  [PATH]  Starting directory path [default: current working directory]

Options:
  -i, --icons <ICONS>                     Icon display style [default: nerd] [possible values: nerd, emoji, ascii]
      --image-protocol <IMAGE_PROTOCOL>  Terminal graphics protocol [default: auto] [possible values: auto, kitty, iterm2, sixel, blocks]
  -a, --all                              Show hidden files and directories (names starting with '.')
  -u, --update                           Check for updates and automatically upgrade to the latest release
      --check-update                     Check if a newer version of Velocity is available without upgrading
  -h, --help                             Print help
  -V, --version                          Print version
```

---

## 📄 License

Licensed under either of [MIT License](LICENSE-MIT) or [Apache License 2.0](LICENSE-APACHE) at your option.
