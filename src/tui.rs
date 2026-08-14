use crate::config::{PREVIEW_MAX_BYTES, PREVIEW_MAX_LINES, THEME};
use crate::fs::{format_size, format_time, read_preview, FileEntry, PreviewContent};
use crate::icons::get_icon;
use crate::theme::get_file_color;
use crate::cli::IconStyle;
use crossterm::cursor::MoveTo;
use crossterm::style::{Attribute, Color, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor};
use crossterm::QueueableCommand;
use std::io::{self, Write};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Fit a string to exact display width (in terminal columns), truncating or padding with spaces.
pub fn fit_width(text: &str, target_width: usize) -> String {
    let mut current_width = 0;
    let mut result = String::with_capacity(target_width + 4);

    for ch in text.chars() {
        let ch_width = ch.width().unwrap_or(0);
        if ch == '\t' {
            let space_needed = 4.min(target_width.saturating_sub(current_width));
            for _ in 0..space_needed {
                result.push(' ');
            }
            current_width += space_needed;
        } else if ch.is_control() {
            // Replace control characters with a space
            if current_width + 1 <= target_width {
                result.push(' ');
                current_width += 1;
            } else {
                break;
            }
        } else if current_width + ch_width <= target_width {
            result.push(ch);
            current_width += ch_width;
        } else {
            break;
        }

        if current_width >= target_width {
            break;
        }
    }

    // Pad remaining spaces if needed
    if current_width < target_width {
        let pad = target_width - current_width;
        for _ in 0..pad {
            result.push(' ');
        }
    }

    result
}

pub struct TerminalLayout {
    pub width: u16,
    pub height: u16,
    pub left_w: u16,
    pub right_x: u16,
    pub right_w: u16,
    pub top: u16,
    pub bottom: u16,
    pub inner_h: u16,
}

impl TerminalLayout {
    pub fn calculate(width: u16, height: u16) -> Self {
        let left_w = (width as f32 * 0.40).round() as u16;
        let left_w = left_w.clamp(26, width.saturating_sub(15));
        let right_x = left_w;
        let right_w = width.saturating_sub(left_w);
        let top = 1;
        let bottom = height.saturating_sub(2);
        let inner_h = bottom.saturating_sub(top).saturating_sub(1);

        Self {
            width,
            height,
            left_w,
            right_x,
            right_w,
            top,
            bottom,
            inner_h,
        }
    }
}

pub fn render_ui<W: Write>(
    stdout: &mut W,
    current_dir: &str,
    items: &[FileEntry],
    selected: usize,
    scroll: usize,
    filter: &str,
    is_searching: bool,
    icon_style: IconStyle,
    layout: &TerminalLayout,
) -> io::Result<()> {
    // 1. Top Title Bar
    stdout.queue(MoveTo(0, 0))?;
    stdout.queue(SetBackgroundColor(THEME.title_bg))?;
    stdout.queue(SetForegroundColor(THEME.title))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;

    let folder_icon = match icon_style {
        IconStyle::Ascii => "[DIR]",
        IconStyle::Emoji => "📁",
        IconStyle::Nerd => "\u{f07c}",
    };
    let title_text = format!("  {}  {}", folder_icon, current_dir);
    let title_line = fit_width(&title_text, layout.width as usize);
    write!(stdout, "{}", title_line)?;
    stdout.queue(ResetColor)?;
    stdout.queue(SetAttribute(Attribute::Reset))?;

    // 2. Borders
    stdout.queue(SetForegroundColor(THEME.border))?;
    stdout.queue(SetBackgroundColor(THEME.bg))?;

    // Top Border
    let left_bar_len = (layout.left_w.saturating_sub(1)) as usize;
    let right_bar_len = (layout.right_w.saturating_sub(2)) as usize;
    let top_border = format!(
        "┌{}┬{}┐",
        "─".repeat(left_bar_len),
        "─".repeat(right_bar_len)
    );
    stdout.queue(MoveTo(0, layout.top))?;
    write!(stdout, "{}", top_border)?;

    // Bottom Border
    let bottom_border = format!(
        "└{}┴{}┘",
        "─".repeat(left_bar_len),
        "─".repeat(right_bar_len)
    );
    stdout.queue(MoveTo(0, layout.bottom))?;
    write!(stdout, "{}", bottom_border)?;

    // Left Panel Header (Items Count)
    stdout.queue(MoveTo(2, layout.top))?;
    stdout.queue(SetForegroundColor(THEME.accent))?;
    let items_badge = format!(" 檔案 ({}) ", items.len());
    write!(stdout, "{}", items_badge)?;

    let selected_entry = if !items.is_empty() && selected < items.len() {
        Some(&items[selected])
    } else {
        None
    };

    // Right Panel Header (Selected item details)
    let right_title_max_w = layout.right_w.saturating_sub(4) as usize;
    if let Some(entry) = selected_entry {
        stdout.queue(MoveTo(layout.right_x + 2, layout.top))?;
        stdout.queue(SetForegroundColor(THEME.search))?;

        let header_str = if entry.is_dir {
            format!(" {}  {}", folder_icon, entry.name)
        } else {
            format!(
                " {} · {} · {}",
                entry.name,
                format_size(entry.size),
                format_time(entry.modified)
            )
        };
        write!(stdout, "{}", fit_width(&header_str, right_title_max_w))?;
    }

    stdout.queue(ResetColor)?;

    // 3. Render Left List & Right Preview Lines
    let list_height = layout.inner_h as usize;
    let preview_content = selected_entry.map(|e| read_preview(e, PREVIEW_MAX_LINES, PREVIEW_MAX_BYTES));

    for row in 0..list_height {
        let y = layout.top + 1 + (row as u16);

        // Render Vertical Borders
        stdout.queue(MoveTo(0, y))?;
        stdout.queue(SetForegroundColor(THEME.border))?;
        write!(stdout, "│")?;

        stdout.queue(MoveTo(layout.left_w, y))?;
        write!(stdout, "│")?;

        stdout.queue(MoveTo(layout.width.saturating_sub(1), y))?;
        write!(stdout, "│")?;

        // Render Left File List Cell
        let item_idx = scroll + row;
        let left_cell_w = layout.left_w.saturating_sub(2) as usize;

        stdout.queue(MoveTo(1, y))?;
        if item_idx < items.len() {
            let item = &items[item_idx];
            let is_cur = item_idx == selected;

            let icon = get_icon(item.is_dir, &item.extension, icon_style);
            let size_str = if item.is_dir {
                String::new()
            } else {
                format_size(item.size)
            };

            let name_avail_w = left_cell_w.saturating_sub(3).saturating_sub(7);
            let name_str = fit_width(&item.name, name_avail_w);
            let size_pad = format!("{:>6}", size_str);

            let row_line = format!(" {} {} {}", icon, name_str, size_pad);
            let final_left_line = fit_width(&row_line, left_cell_w);

            if is_cur {
                stdout.queue(SetBackgroundColor(THEME.sel_bg))?;
                stdout.queue(SetForegroundColor(THEME.sel_fg))?;
                stdout.queue(SetAttribute(Attribute::Bold))?;
                write!(stdout, "{}", final_left_line)?;
                stdout.queue(SetAttribute(Attribute::Reset))?;
            } else {
                let fg_color = if item.is_dir {
                    THEME.dir
                } else {
                    get_file_color(&item.extension)
                };
                stdout.queue(SetBackgroundColor(THEME.panel))?;
                stdout.queue(SetForegroundColor(fg_color))?;
                write!(stdout, "{}", final_left_line)?;
            }
        } else {
            stdout.queue(SetBackgroundColor(THEME.panel))?;
            write!(stdout, "{}", " ".repeat(left_cell_w))?;
        }

        // Render Right Preview Cell
        let right_cell_w = layout.right_w.saturating_sub(3) as usize;
        stdout.queue(MoveTo(layout.right_x + 1, y))?;
        stdout.queue(SetBackgroundColor(THEME.panel))?;
        stdout.queue(SetForegroundColor(THEME.file))?;

        let line_text = match &preview_content {
            Some(PreviewContent::Directory { items: sub_items, total_items }) => {
                if row == 0 {
                    format!(" 📁 目錄包含 {} 個項目", total_items)
                } else if row == 1 {
                    String::new()
                } else {
                    let sub_idx = row - 2;
                    if sub_idx < sub_items.len() {
                        format!("   {}", sub_items[sub_idx])
                    } else {
                        String::new()
                    }
                }
            }
            Some(PreviewContent::Text { lines, .. }) => {
                if row < lines.len() {
                    format!(" {}", lines[row])
                } else {
                    String::new()
                }
            }
            Some(PreviewContent::Binary { size }) => {
                if row == 1 {
                    format!("   <二進位檔案或不支援預覽 — {}>", format_size(*size))
                } else {
                    String::new()
                }
            }
            Some(PreviewContent::TooLarge { size }) => {
                if row == 1 {
                    format!(
                        "   <檔案過大 ({}) — 按 'e' 使用編輯器開啟>",
                        format_size(*size)
                    )
                } else {
                    String::new()
                }
            }
            Some(PreviewContent::Empty) => {
                if row == 1 {
                    "   <空檔案>".to_string()
                } else {
                    String::new()
                }
            }
            Some(PreviewContent::Error(err)) => {
                if row == 1 {
                    format!("   <{}>", err)
                } else {
                    String::new()
                }
            }
            None => String::new(),
        };

        write!(stdout, "{}", fit_width(&line_text, right_cell_w))?;
    }

    // 4. Bottom Status / Search Bar
    let status_y = layout.height.saturating_sub(1);
    stdout.queue(MoveTo(0, status_y))?;
    stdout.queue(SetBackgroundColor(THEME.title_bg))?;

    if is_searching {
        stdout.queue(SetForegroundColor(THEME.search))?;
        stdout.queue(SetAttribute(Attribute::Bold))?;
        let search_text = format!("  / 搜尋: {}█   (Enter 定案 · Esc 取消)", filter);
        write!(stdout, "{}", fit_width(&search_text, layout.width as usize))?;
    } else {
        stdout.queue(SetForegroundColor(THEME.muted))?;
        let pos_str = if !items.is_empty() {
            format!("{}/{}", selected + 1, items.len())
        } else {
            "0/0".to_string()
        };
        let filter_str = if !filter.is_empty() {
            format!("  過濾: {}", filter)
        } else {
            String::new()
        };
        let shortcuts_str = "↑↓/kj 移動  →/l 進入  ←/h 上層  / 搜尋  e 編輯  r 整理  . 隱藏檔  q 離開";
        let status_text = format!(" {} {}   {}", pos_str, filter_str, shortcuts_str);
        write!(stdout, "{}", fit_width(&status_text, layout.width as usize))?;
    }

    stdout.queue(ResetColor)?;
    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.flush()?;

    Ok(())
}
