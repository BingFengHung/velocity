# Velocity (`vl` / `velocity`)

[English](README.md) | [繁體中文](README.zh-TW.md)

**Velocity** is a blazingly fast, lightweight, and modern terminal file browser written in Rust. Features a two-pane layout, true-color theme, instant search filter, file-type syntax coloring, 24-bit image previews, and Nerd Font icons.

Designed as a high-velocity, zero-overhead file navigation tool for terminal power users, developers, and AI-assisted terminal workflows.

---

## ✨ Features

- ⚡ **High Velocity & Zero Dependencies**: Written in pure Rust with native performance and instant sub-millisecond startup.
- 🪟 **Two-Pane Layout**: Left pane for interactive directory tree navigation; right pane for instant real-time file, folder, and image previews.
- 🖼️ **24-bit True Color Image Previews**: Built-in instant pixel-block thumbnail rendering for PNG, JPEG, GIF, WebP, BMP, and ICO without external dependencies.
- 🎨 **True Color & Syntax Categorization**: 24-bit True Color theme with automatic syntax categorization for source code, configuration files, media, archives, and executables.
- 🔎 **Instant Search & Filter**: Press `/` to start typing and filter files in real-time with instant matching.
- 🔤 **Multiple Icon Themes**: Built-in support for **Nerd Font** icons, **Emoji** icons, and **ASCII** fallback mode.
- 🌐 **Accurate CJK & Unicode Width Handling**: Fully prevents terminal box border misalignment caused by East Asian wide characters or emojis.
- 📝 **Seamless Editor Integration**: Press `e` to open the highlighted file directly in your `$EDITOR`, VS Code, or system default text editor.
- 🌍 **Cross-Platform**: First-class support for Windows, Linux, and macOS.

---

## 📦 Installation

### Pre-built Binaries

Download pre-compiled release binaries for your operating system from GitHub Releases.

### Building via GitHub Actions

This repository adheres to the **Zero Local Heavy Compilation** rule. All release binaries across multiple targets (Windows, Linux, macOS) are automatically compiled and published through GitHub Actions CI/CD workflows upon pushing release tags (`v*.*.*`).

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
| `/` | Instant search / filter mode (`Enter` to confirm, `Esc` to cancel) |
| `e` | Open selected file in `$EDITOR` / VS Code / default editor |
| `r` | Refresh current directory |
| `.` | Toggle show/hide hidden files |
| `q` / `Esc` | Quit Velocity |

---

## 🛠️ CLI Options

```
Usage: velocity.exe [OPTIONS] [PATH]

Arguments:
  [PATH]  Starting directory path [default: current working directory]

Options:
  -i, --icons <ICONS>  Icon display style [default: nerd] [possible values: nerd, emoji, ascii]
  -a, --all            Show hidden files and directories (names starting with '.')
  -h, --help           Print help
  -V, --version        Print version
```

---

## 📄 License

Licensed under either of [MIT License](LICENSE-MIT) or [Apache License 2.0](LICENSE-APACHE) at your option.
