use crate::cli::IconStyle;

pub fn get_icon(is_dir: bool, ext: &str, style: IconStyle) -> &'static str {
    match style {
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
    }
}
