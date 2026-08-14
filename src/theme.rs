use crate::config::THEME;
use crossterm::style::Color;

pub fn get_file_color(ext: &str) -> Color {
    match ext {
        "ps1" | "psm1" | "psd1" => Color::Rgb {
            r: 120,
            g: 200,
            b: 255,
        },
        "py" => Color::Rgb {
            r: 255,
            g: 212,
            b: 90,
        },
        "js" | "jsx" | "ts" | "tsx" => Color::Rgb {
            r: 240,
            g: 220,
            b: 120,
        },
        "json" | "yaml" | "yml" | "toml" | "ini" | "cfg" | "conf" | "env" | "properties" => {
            Color::Rgb {
                r: 200,
                g: 170,
                b: 255,
            }
        }
        "md" | "txt" | "log" | "csv" | "tsv" => Color::Rgb {
            r: 200,
            g: 200,
            b: 210,
        },
        "html" | "htm" | "css" | "scss" => Color::Rgb {
            r: 255,
            g: 160,
            b: 120,
        },
        "c" | "h" | "cpp" | "hpp" | "cc" | "cxx" | "cs" | "java" | "go" | "rs" | "rb" | "php" => {
            Color::Rgb {
                r: 150,
                g: 230,
                b: 180,
            }
        }
        "sh" | "bash" | "zsh" | "bat" | "cmd" => Color::Rgb {
            r: 160,
            g: 220,
            b: 140,
        },
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "ico" | "webp" => Color::Rgb {
            r: 230,
            g: 140,
            b: 200,
        },
        "zip" | "7z" | "rar" | "gz" | "tar" | "xz" => Color::Rgb {
            r: 200,
            g: 140,
            b: 110,
        },
        "exe" | "dll" | "msi" | "bin" | "so" | "dylib" => Color::Rgb {
            r: 180,
            g: 120,
            b: 120,
        },
        _ => THEME.file,
    }
}
