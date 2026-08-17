use crate::cli::IconStyle;
use std::sync::OnceLock;

static NERD_FONT_SUPPORTED: OnceLock<bool> = OnceLock::new();

/// Detects whether the current environment and terminal likely support Nerd Fonts.
pub fn detect_nerd_font_support() -> bool {
    *NERD_FONT_SUPPORTED.get_or_init(|| {
        // 1. Check explicit environment variables
        if let Ok(val) = std::env::var("NERD_FONT") {
            let lower = val.to_lowercase();
            if lower == "1" || lower == "true" || lower == "yes" {
                return true;
            }
            if lower == "0" || lower == "false" || lower == "no" {
                return false;
            }
        }
        if let Ok(val) = std::env::var("VELOCITY_ICONS") {
            let lower = val.to_lowercase();
            if lower == "nerd" {
                return true;
            }
            if lower == "emoji" || lower == "ascii" {
                return false;
            }
        }

        // 2. Platform-specific terminal & font detection
        #[cfg(windows)]
        {
            // On Windows, if running in legacy conhost / cmd.exe without Windows Terminal (WT_SESSION),
            // VSCode, WezTerm, Alacritty or Ghostty, the default console font is almost never a Nerd Font.
            let in_modern_term = std::env::var("WT_SESSION").is_ok()
                || std::env::var("TERM_PROGRAM")
                    .map(|v| v == "vscode" || v == "WezTerm" || v == "ghostty")
                    .unwrap_or(false)
                || std::env::var("ALACRITTY_LOG").is_ok()
                || std::env::var("WEZTERM_EXECUTABLE").is_ok();

            if !in_modern_term {
                return false;
            }

            has_nerd_font_installed_windows()
        }

        #[cfg(not(windows))]
        {
            has_nerd_font_installed_unix()
        }
    })
}

#[cfg(windows)]
fn has_nerd_font_installed_windows() -> bool {
    let mut dirs = Vec::new();
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        dirs.push(
            std::path::PathBuf::from(local_app_data)
                .join("Microsoft")
                .join("Windows")
                .join("Fonts"),
        );
    }
    if let Ok(windir) = std::env::var("WINDIR") {
        dirs.push(std::path::PathBuf::from(windir).join("Fonts"));
    } else {
        dirs.push(std::path::PathBuf::from(r"C:\Windows\Fonts"));
    }

    for dir in dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains("nerd")
                    || name.contains("nf.")
                    || name.contains("nfm.")
                    || name.contains("nf-")
                    || name.contains("nerdfont")
                {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(not(windows))]
fn has_nerd_font_installed_unix() -> bool {
    let mut dirs = Vec::new();
    if let Ok(home) = std::env::var("HOME") {
        let home_p = std::path::PathBuf::from(home);
        dirs.push(home_p.join(".local/share/fonts"));
        dirs.push(home_p.join(".fonts"));
        dirs.push(home_p.join("Library/Fonts"));
    }
    dirs.push(std::path::PathBuf::from("/usr/share/fonts"));
    dirs.push(std::path::PathBuf::from("/usr/local/share/fonts"));
    dirs.push(std::path::PathBuf::from("/Library/Fonts"));
    dirs.push(std::path::PathBuf::from("/System/Library/Fonts"));

    for dir in dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains("nerd") || name.contains("nf") {
                    return true;
                }
            }
        }
    }
    false
}

pub fn resolve_icon_style(style: IconStyle) -> IconStyle {
    match style {
        IconStyle::Auto => {
            if detect_nerd_font_support() {
                IconStyle::Nerd
            } else {
                IconStyle::Emoji
            }
        }
        other => other,
    }
}

pub fn get_folder_icon(style: IconStyle) -> &'static str {
    match resolve_icon_style(style) {
        IconStyle::Ascii => "[DIR]",
        IconStyle::Emoji => "📁",
        IconStyle::Nerd => "\u{f07c}",
        IconStyle::Auto => unreachable!(),
    }
}

pub fn get_git_branch_icon(style: IconStyle) -> &'static str {
    match resolve_icon_style(style) {
        IconStyle::Ascii => "git:",
        IconStyle::Emoji => "🌿",
        IconStyle::Nerd => "\u{e0a0}",
        IconStyle::Auto => unreachable!(),
    }
}

pub fn get_icon(is_dir: bool, ext: &str, style: IconStyle) -> &'static str {
    match resolve_icon_style(style) {
        IconStyle::Ascii => {
            if is_dir {
                "[+]"
            } else {
                "   "
            }
        }
        IconStyle::Emoji => {
            if is_dir {
                "📂"
            } else {
                match ext {
                    "rs" => "🦀",
                    "py" => "🐍",
                    "js" | "jsx" | "ts" | "tsx" => "📜",
                    "json" | "yaml" | "yml" | "toml" | "xml" | "ini" | "env" => "⚙️",
                    "md" | "txt" | "log" | "doc" | "docx" => "📝",
                    "html" | "htm" | "css" | "scss" => "🌐",
                    "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "ico" | "webp" => "🖼️",
                    "zip" | "7z" | "rar" | "gz" | "tar" | "xz" => "📦",
                    "exe" | "dll" | "msi" | "bin" | "so" | "dylib" => "⚡",
                    "mp3" | "wav" | "flac" | "ogg" => "🎵",
                    "mp4" | "mkv" | "avi" | "mov" => "🎬",
                    "pdf" => "📕",
                    "sh" | "bash" | "zsh" | "bat" | "cmd" | "ps1" => "💻",
                    _ => "📄",
                }
            }
        }
        IconStyle::Nerd => {
            if is_dir {
                "\u{f07c}" // nf-fa-folder_open
            } else {
                match ext {
                    "ps1" | "psm1" | "psd1" => "\u{f0a0a}",
                    "py" => "\u{e73c}",
                    "js" | "jsx" => "\u{e74e}",
                    "ts" | "tsx" => "\u{e628}",
                    "json" => "\u{e60b}",
                    "yaml" | "yml" | "toml" | "ini" | "cfg" | "conf" | "env" | "properties" => {
                        "\u{f0219}"
                    }
                    "md" => "\u{f48a}",
                    "html" | "htm" => "\u{f13b}",
                    "css" | "scss" => "\u{f13c}",
                    "cs" => "\u{f031b}",
                    "java" => "\u{e738}",
                    "go" => "\u{e627}",
                    "rs" => "\u{e7a8}",
                    "c" | "h" => "\u{e61e}",
                    "cpp" | "hpp" | "cc" | "cxx" => "\u{e61d}",
                    "sh" | "bash" | "zsh" | "bat" | "cmd" => "\u{f489}",
                    "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "ico" | "webp" => "\u{f03e}",
                    "zip" | "7z" | "rar" | "gz" | "tar" | "xz" => "\u{f410}",
                    "exe" | "dll" | "msi" | "bin" => "\u{f17a}",
                    "sql" => "\u{f1c0}",
                    "pdf" => "\u{f1c1}",
                    "dockerfile" => "\u{f308}",
                    _ => "\u{f15b}", // generic file
                }
            }
        }
        IconStyle::Auto => unreachable!(),
    }
}
