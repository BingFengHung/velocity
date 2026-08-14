use chrono::{DateTime, Local};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub extension: String,
    pub is_hidden: bool,
}

#[derive(Debug)]
pub enum PreviewContent {
    Directory {
        total_items: usize,
        items: Vec<String>,
    },
    Text {
        lines: Vec<String>,
        total_lines: usize,
        truncated: bool,
    },
    Binary {
        size: u64,
    },
    TooLarge {
        size: u64,
    },
    Empty,
    Error(String),
}

const TEXT_EXTENSIONS: &[&str] = &[
    "txt", "md", "markdown", "ps1", "psm1", "psd1", "json", "xml", "yml", "yaml", "csv",
    "log", "ini", "cfg", "conf", "py", "js", "ts", "tsx", "jsx", "html", "htm", "css", "scss",
    "sass", "less", "cs", "java", "go", "rs", "rb", "php", "c", "h", "cpp", "hpp", "cc", "cxx",
    "sql", "sh", "bash", "zsh", "bat", "cmd", "toml", "env", "gitignore", "dockerfile", "tf",
    "tsv", "properties", "vue", "svelte", "swift", "kt", "kts", "dart", "lua", "r", "scala",
    "zig", "v", "nim", "odin", "graphql", "proto", "rst", "tex", "bib", "diff", "patch",
];

pub fn read_directory(path: &Path, show_hidden: bool) -> io::Result<Vec<FileEntry>> {
    let mut entries = Vec::new();
    let read_dir = fs::read_dir(path)?;

    for item in read_dir {
        let entry = match item {
            Ok(e) => e,
            Err(_) => continue,
        };

        let file_name = entry.file_name().to_string_lossy().to_string();
        let is_hidden = file_name.starts_with('.');

        if is_hidden && !show_hidden {
            continue;
        }

        let file_path = entry.path();
        let metadata = entry.metadata().ok();
        let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        let modified = metadata.and_then(|m| m.modified().ok());

        let extension = if is_dir {
            String::new()
        } else {
            file_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase()
        };

        entries.push(FileEntry {
            name: file_name,
            path: file_path,
            is_dir,
            size,
            modified,
            extension,
            is_hidden,
        });
    }

    // Sort: directories first, then alphabetical (case-insensitive)
    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(entries)
}

pub fn is_text_file(path: &Path, ext: &str, size: u64) -> bool {
    if size == 0 {
        return true;
    }

    if TEXT_EXTENSIONS.contains(&ext) {
        return true;
    }

    // Inspect first 4096 bytes for null bytes or invalid non-text binary indicators
    if let Ok(mut file) = File::open(path) {
        let mut buffer = [0u8; 4096];
        if let Ok(n) = file.read(&mut buffer) {
            if n == 0 {
                return true;
            }
            let slice = &buffer[..n];
            // If contains null byte, treat as binary
            if slice.contains(&0) {
                return false;
            }
            // Check UTF-8 validity or high ASCII text ratio
            if std::str::from_utf8(slice).is_ok() {
                return true;
            }
            // If invalid UTF-8, check if mostly printable ASCII/whitespace
            let printable = slice.iter().filter(|&&b| b == b'\t' || b == b'\n' || b == b'\r' || (32..=126).contains(&b)).count();
            return (printable as f64 / n as f64) > 0.85;
        }
    }

    false
}

pub fn read_preview(entry: &FileEntry, max_lines: usize, max_bytes: u64) -> PreviewContent {
    if entry.is_dir {
        match read_directory(&entry.path, true) {
            Ok(sub_items) => {
                let total_items = sub_items.len();
                let preview_items = sub_items
                    .into_iter()
                    .take(max_lines)
                    .map(|item| {
                        let prefix = if item.is_dir { "[DIR]" } else { "     " };
                        format!("{} {}", prefix, item.name)
                    })
                    .collect();
                PreviewContent::Directory {
                    total_items,
                    items: preview_items,
                }
            }
            Err(e) => PreviewContent::Error(format!("無法讀取目錄內容: {}", e)),
        }
    } else {
        if entry.size == 0 {
            return PreviewContent::Empty;
        }

        if entry.size > max_bytes {
            return PreviewContent::TooLarge { size: entry.size };
        }

        if !is_text_file(&entry.path, &entry.extension, entry.size) {
            return PreviewContent::Binary { size: entry.size };
        }

        match File::open(&entry.path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let mut lines = Vec::new();
                let mut total_lines = 0;
                let mut truncated = false;

                for line in reader.lines() {
                    match line {
                        Ok(l) => {
                            total_lines += 1;
                            if lines.len() < max_lines {
                                lines.push(l);
                            } else {
                                truncated = true;
                            }
                        }
                        Err(_) => {
                            // If encoding error occurs during line reading
                            if lines.is_empty() {
                                return PreviewContent::Binary { size: entry.size };
                            }
                            break;
                        }
                    }
                }

                if lines.is_empty() {
                    PreviewContent::Empty
                } else {
                    PreviewContent::Text {
                        lines,
                        total_lines,
                        truncated,
                    }
                }
            }
            Err(e) => PreviewContent::Error(format!("讀取失敗: {}", e)),
        }
    }
}

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes < KB {
        format!("{} B", bytes)
    } else if bytes < MB {
        format!("{:.1}K", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.1}M", bytes as f64 / MB as f64)
    } else {
        format!("{:.1}G", bytes as f64 / GB as f64)
    }
}

pub fn format_time(time: Option<SystemTime>) -> String {
    match time {
        Some(t) => {
            let datetime: DateTime<Local> = t.into();
            datetime.format("%Y-%m-%d %H:%M").to_string()
        }
        None => "----/--/-- --:--".to_string(),
    }
}

pub fn open_in_editor(path: &Path) -> Result<(), String> {
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .ok();

    if let Some(ed) = editor {
        if !ed.trim().is_empty() {
            let parts: Vec<&str> = ed.split_whitespace().collect();
            if let Some((cmd, args)) = parts.split_first() {
                let status = Command::new(cmd)
                    .args(args)
                    .arg(path)
                    .status();
                if status.is_ok() {
                    return Ok(());
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Try VS Code first, then Notepad
        if Command::new("cmd").args(["/C", "code", path.to_str().unwrap_or_default()]).spawn().is_ok() {
            return Ok(());
        }
        if Command::new("notepad.exe").arg(path).spawn().is_ok() {
            return Ok(());
        }
    }

    #[cfg(target_os = "macos")]
    {
        if Command::new("open").arg(path).spawn().is_ok() {
            return Ok(());
        }
    }

    #[cfg(target_os = "linux")]
    {
        if Command::new("xdg-open").arg(path).spawn().is_ok() {
            return Ok(());
        }
        for fallback in &["nano", "vim", "vi"] {
            if Command::new(fallback).arg(path).status().is_ok() {
                return Ok(());
            }
        }
    }

    Err("找不到可用的文字編輯器（請設定 $EDITOR 環境變數）".to_string())
}
