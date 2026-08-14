use chrono::{DateTime, Local};
use image::{imageops::FilterType, DynamicImage, GenericImageView, ImageReader};
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

#[derive(Clone, Debug)]
pub struct PixelCell {
    pub top: (u8, u8, u8),
    pub bottom: (u8, u8, u8),
}

#[derive(Clone, Debug)]
pub struct ImagePreviewInfo {
    pub orig_width: u32,
    pub orig_height: u32,
    pub format_name: String,
    pub grid: Vec<Vec<PixelCell>>,
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
    Image(ImagePreviewInfo),
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

const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "bmp", "ico",
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

    if IMAGE_EXTENSIONS.contains(&ext) {
        return false;
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
            // Check UTF-8 validity
            if std::str::from_utf8(slice).is_ok() {
                return true;
            }
            // If invalid UTF-8, check if mostly printable ASCII/whitespace
            let printable = slice
                .iter()
                .filter(|&&b| b == b'\t' || b == b'\n' || b == b'\r' || (32..=126).contains(&b))
                .count();
            return (printable as f64 / n as f64) > 0.85;
        }
    }

    false
}

fn blend_color(fg: (u8, u8, u8), alpha: u8, bg: (u8, u8, u8)) -> (u8, u8, u8) {
    if alpha == 255 {
        return fg;
    }
    if alpha == 0 {
        return bg;
    }
    let a = alpha as f32 / 255.0;
    let r = (fg.0 as f32 * a + bg.0 as f32 * (1.0 - a)).round() as u8;
    let g = (fg.1 as f32 * a + bg.1 as f32 * (1.0 - a)).round() as u8;
    let b = (fg.2 as f32 * a + bg.2 as f32 * (1.0 - a)).round() as u8;
    (r, g, b)
}

pub fn load_image_preview(path: &Path, max_w: usize, max_h: usize) -> Option<ImagePreviewInfo> {
    if max_w == 0 || max_h == 0 {
        return None;
    }

    let reader = ImageReader::open(path).ok()?.with_guessed_format().ok()?;
    let format_str = reader
        .format()
        .map(|f| format!("{:?}", f).to_uppercase())
        .unwrap_or_else(|| "IMAGE".to_string());

    let dynamic_img: DynamicImage = reader.decode().ok()?;
    let (orig_w, orig_h) = dynamic_img.dimensions();

    if orig_w == 0 || orig_h == 0 {
        return None;
    }

    // Terminal characters have ~2:1 height/width ratio.
    // Since each character cell displays 2 vertical pixels (via \u{2580}),
    // the target pixel canvas is max_w (columns) by max_h * 2 (pixel rows).
    let target_pixel_w = max_w as u32;
    let target_pixel_h = (max_h * 2) as u32;

    // Calculate aspect ratio preserving thumbnail
    let thumb = dynamic_img.resize(target_pixel_w, target_pixel_h, FilterType::Triangle);
    let rgba_img = thumb.to_rgba8();
    let (thumb_w, thumb_h) = rgba_img.dimensions();

    let mut grid = Vec::new();
    let panel_bg = (30, 30, 38); // Panel background color for alpha blending

    let row_count = (thumb_h as usize + 1) / 2;
    for row in 0..row_count {
        let mut row_cells = Vec::with_capacity(thumb_w as usize);
        let top_y = (row * 2) as u32;
        let bottom_y = (row * 2 + 1) as u32;

        for x in 0..thumb_w {
            let top_pixel = rgba_img.get_pixel(x, top_y);
            let top_rgb = blend_color(
                (top_pixel[0], top_pixel[1], top_pixel[2]),
                top_pixel[3],
                panel_bg,
            );

            let bottom_rgb = if bottom_y < thumb_h {
                let bottom_pixel = rgba_img.get_pixel(x, bottom_y);
                blend_color(
                    (bottom_pixel[0], bottom_pixel[1], bottom_pixel[2]),
                    bottom_pixel[3],
                    panel_bg,
                )
            } else {
                panel_bg
            };

            row_cells.push(PixelCell {
                top: top_rgb,
                bottom: bottom_rgb,
            });
        }
        grid.push(row_cells);
    }

    Some(ImagePreviewInfo {
        orig_width: orig_w,
        orig_height: orig_h,
        format_name: format_str,
        grid,
    })
}

pub fn read_preview(
    entry: &FileEntry,
    max_lines: usize,
    max_bytes: u64,
    avail_width: usize,
) -> PreviewContent {
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

        // Check for image preview
        if IMAGE_EXTENSIONS.contains(&entry.extension.as_str()) {
            let img_max_h = max_lines.saturating_sub(2);
            let img_max_w = avail_width.saturating_sub(4);
            if let Some(info) = load_image_preview(&entry.path, img_max_w, img_max_h) {
                return PreviewContent::Image(info);
            }
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
        if Command::new("cmd")
            .args(["/C", "code", path.to_str().unwrap_or_default()])
            .spawn()
            .is_ok()
        {
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
