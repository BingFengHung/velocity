use crate::app::{App, InputMode};
use crate::config::{PREVIEW_MAX_BYTES, PREVIEW_MAX_LINES, THEME};
use crate::fs::{format_size, format_time, read_preview, PreviewContent};
use crate::git::GitFileStatus;
use crate::icons::{get_folder_icon, get_git_branch_icon, get_icon};
use crate::theme::get_file_color;
use crossterm::cursor::MoveTo;
use crossterm::style::{
    Attribute, Color, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
};
use crossterm::QueueableCommand;
use std::io::{self, Write};
use std::time::Duration;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

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
            if current_width < target_width {
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

pub fn render_ui<W: Write>(stdout: &mut W, app: &App, layout: &TerminalLayout) -> io::Result<()> {
    // 1. Top Title Bar (Path + Git Branch + Sort Mode)
    stdout.queue(MoveTo(0, 0))?;
    stdout.queue(SetBackgroundColor(THEME.title_bg))?;
    stdout.queue(SetForegroundColor(THEME.title))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;

    let folder_icon = get_folder_icon(app.icon_style);

    let git_branch_str = if let Some(ref git) = app.git_info {
        if let Some(ref branch) = git.branch {
            let branch_icon = get_git_branch_icon(app.icon_style);
            format!(" {} {} ", branch_icon, branch)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let sort_badge = format!(" [排序: {}] ", app.sort_mode.display_name());
    let path_str = app.current_dir.to_string_lossy();
    let clean_path = path_str.strip_prefix(r"\\?\").unwrap_or(&path_str);
    let left_title = format!("  {}  {}", folder_icon, clean_path);

    let title_space = (layout.width as usize)
        .saturating_sub(left_title.width())
        .saturating_sub(git_branch_str.width())
        .saturating_sub(sort_badge.width());

    let mut full_top_bar = left_title;
    if title_space > 0 {
        full_top_bar.push_str(&" ".repeat(title_space));
    }
    full_top_bar.push_str(&git_branch_str);
    full_top_bar.push_str(&sort_badge);

    write!(
        stdout,
        "{}",
        fit_width(&full_top_bar, layout.width as usize)
    )?;
    stdout.queue(ResetColor)?;
    stdout.queue(SetAttribute(Attribute::Reset))?;

    // 2. Borders
    stdout.queue(SetBackgroundColor(THEME.bg))?;

    let left_w = layout.left_w as usize;
    let right_w = layout.right_w as usize;

    let selected_entry =
        if !app.filtered_items.is_empty() && app.selected < app.filtered_items.len() {
            Some(&app.filtered_items[app.selected].entry)
        } else {
            None
        };

    // Top Border (Single Pass without overdraw)
    stdout.queue(MoveTo(0, layout.top))?;
    stdout.queue(SetForegroundColor(THEME.border))?;
    write!(stdout, "┌─")?;

    let items_badge = format!(" 檔案 ({}) ", app.filtered_items.len());
    let badge_w = items_badge.width();
    stdout.queue(SetForegroundColor(THEME.accent))?;
    write!(stdout, "{}", items_badge)?;

    stdout.queue(SetForegroundColor(THEME.border))?;
    let left_fill = left_w.saturating_sub(2 + badge_w);
    if left_fill > 0 {
        write!(stdout, "{}", "─".repeat(left_fill))?;
    }

    stdout.queue(MoveTo(layout.left_w, layout.top))?;
    write!(stdout, "┬")?;

    if let Some(entry) = selected_entry {
        let max_title_w = right_w.saturating_sub(4);
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
        let fitted_header = fit_width(&header_str, max_title_w);
        let header_w = fitted_header.width();

        write!(stdout, "─ ")?;
        stdout.queue(SetForegroundColor(THEME.search))?;
        write!(stdout, "{}", fitted_header)?;
        stdout.queue(SetForegroundColor(THEME.border))?;

        let right_fill = right_w.saturating_sub(3 + header_w);
        if right_fill > 0 {
            write!(stdout, "{}", "─".repeat(right_fill))?;
        }
    } else {
        let right_fill = right_w.saturating_sub(2);
        if right_fill > 0 {
            write!(stdout, "{}", "─".repeat(right_fill))?;
        }
    }

    stdout.queue(MoveTo(layout.width.saturating_sub(1), layout.top))?;
    write!(stdout, "┐")?;

    // Bottom Border
    let left_bar_len = (layout.left_w.saturating_sub(1)) as usize;
    let right_bar_len = (layout.right_w.saturating_sub(2)) as usize;
    let bottom_border = format!(
        "└{}┴{}┘",
        "─".repeat(left_bar_len),
        "─".repeat(right_bar_len)
    );
    stdout.queue(MoveTo(0, layout.bottom))?;
    write!(stdout, "{}", bottom_border)?;

    stdout.queue(ResetColor)?;

    // 3. Render Left List & Right Preview Lines
    let list_height = layout.inner_h as usize;
    let right_cell_w = layout.right_w.saturating_sub(3) as usize;
    let preview_content = selected_entry.map(|e| {
        read_preview(
            e,
            PREVIEW_MAX_LINES,
            PREVIEW_MAX_BYTES,
            right_cell_w,
            app.image_protocol,
        )
    });

    for row in 0..list_height {
        let y = layout.top + 1 + (row as u16);

        // Vertical Borders
        stdout.queue(MoveTo(0, y))?;
        stdout.queue(SetForegroundColor(THEME.border))?;
        write!(stdout, "│")?;

        stdout.queue(MoveTo(layout.left_w, y))?;
        write!(stdout, "│")?;

        stdout.queue(MoveTo(layout.width.saturating_sub(1), y))?;
        write!(stdout, "│")?;

        // Left File List Cell
        let item_idx = app.scroll + row;
        let left_cell_w = layout.left_w.saturating_sub(2) as usize;

        stdout.queue(MoveTo(1, y))?;
        if item_idx < app.filtered_items.len() {
            let filtered_item = &app.filtered_items[item_idx];
            let item = &filtered_item.entry;
            let is_cur = item_idx == app.selected;

            let icon = get_icon(item.is_dir, &item.extension, app.icon_style);
            let size_str = if item.is_dir {
                String::new()
            } else {
                format_size(item.size)
            };

            let git_tag = match filtered_item.git_status {
                Some(GitFileStatus::Modified) => " M",
                Some(GitFileStatus::Staged) => " +",
                Some(GitFileStatus::Untracked) => " ?",
                Some(GitFileStatus::Deleted) => " D",
                _ => "  ",
            };

            let name_avail_w = left_cell_w.saturating_sub(5).saturating_sub(7);
            let name_str = fit_width(&item.name, name_avail_w);
            let size_pad = format!("{:>6}", size_str);

            if is_cur {
                stdout.queue(SetBackgroundColor(THEME.sel_bg))?;
                stdout.queue(SetForegroundColor(THEME.sel_fg))?;
                stdout.queue(SetAttribute(Attribute::Bold))?;
                let full_row_str = format!("{} {} {} {}", git_tag, icon, name_str, size_pad);
                write!(stdout, "{}", fit_width(&full_row_str, left_cell_w))?;
                stdout.queue(SetAttribute(Attribute::Reset))?;
            } else {
                stdout.queue(SetBackgroundColor(THEME.panel))?;

                // Git status badge with color
                let git_color = match filtered_item.git_status {
                    Some(GitFileStatus::Modified) => THEME.git_modified,
                    Some(GitFileStatus::Staged) => THEME.git_staged,
                    Some(GitFileStatus::Untracked) => THEME.git_untracked,
                    Some(GitFileStatus::Deleted) => THEME.git_deleted,
                    _ => THEME.muted,
                };
                stdout.queue(SetForegroundColor(git_color))?;
                write!(stdout, "{}", git_tag)?;

                // Icon & Name
                let fg_color = if item.is_dir {
                    THEME.dir
                } else {
                    get_file_color(&item.extension)
                };

                stdout.queue(SetForegroundColor(fg_color))?;
                write!(stdout, " {} ", icon)?;

                // If fuzzy match exists, render matched characters with highlight
                if let Some(ref fuzzy) = filtered_item.fuzzy {
                    let mut rendered_name_w = 0;
                    for (ch_idx, ch) in item.name.chars().enumerate() {
                        let ch_w = ch.width().unwrap_or(0);
                        if rendered_name_w + ch_w > name_avail_w {
                            break;
                        }
                        if fuzzy.matched_indices.contains(&ch_idx) {
                            stdout.queue(SetForegroundColor(THEME.match_highlight))?;
                            stdout.queue(SetAttribute(Attribute::Bold))?;
                            write!(stdout, "{}", ch)?;
                            stdout.queue(SetAttribute(Attribute::Reset))?;
                            stdout.queue(SetBackgroundColor(THEME.panel))?;
                            stdout.queue(SetForegroundColor(fg_color))?;
                        } else {
                            write!(stdout, "{}", ch)?;
                        }
                        rendered_name_w += ch_w;
                    }
                    if rendered_name_w < name_avail_w {
                        write!(stdout, "{}", " ".repeat(name_avail_w - rendered_name_w))?;
                    }
                } else {
                    write!(stdout, "{}", name_str)?;
                }

                // Size
                stdout.queue(SetForegroundColor(THEME.size))?;
                write!(stdout, " {}", size_pad)?;

                let used_w = 2 + 1 + 1 + 1 + name_avail_w + 1 + 6;
                if used_w < left_cell_w {
                    write!(stdout, "{}", " ".repeat(left_cell_w - used_w))?;
                }
            }
        } else {
            stdout.queue(SetBackgroundColor(THEME.panel))?;
            write!(stdout, "{}", " ".repeat(left_cell_w))?;
        }

        // Right Preview Cell
        stdout.queue(MoveTo(layout.right_x + 1, y))?;
        stdout.queue(SetBackgroundColor(THEME.panel))?;
        stdout.queue(SetForegroundColor(THEME.file))?;

        match &preview_content {
            Some(PreviewContent::Image(img_info)) => {
                if row == 0 {
                    let info_line = format!(
                        " 🖼️ 格式: {} · 解析度: {}x{} · 大小: {}",
                        img_info.format_name,
                        img_info.orig_width,
                        img_info.orig_height,
                        selected_entry
                            .map(|e| format_size(e.size))
                            .unwrap_or_default()
                    );
                    stdout.queue(SetForegroundColor(THEME.accent))?;
                    write!(stdout, "{}", fit_width(&info_line, right_cell_w))?;
                } else if row == 1 {
                    write!(stdout, "{}", " ".repeat(right_cell_w))?;
                } else {
                    let img_row = row - 2;
                    if let Some(ref payload) = img_info.protocol_payload {
                        if img_row == 0 {
                            write!(stdout, "{}", payload)?;
                        }
                    } else if img_row < img_info.grid.len() {
                        let cells = &img_info.grid[img_row];
                        let rendered_cols = cells.len().min(right_cell_w);
                        write!(stdout, " ")?;
                        for cell in &cells[..rendered_cols.saturating_sub(1)] {
                            stdout.queue(SetForegroundColor(Color::Rgb {
                                r: cell.top.0,
                                g: cell.top.1,
                                b: cell.top.2,
                            }))?;
                            stdout.queue(SetBackgroundColor(Color::Rgb {
                                r: cell.bottom.0,
                                g: cell.bottom.1,
                                b: cell.bottom.2,
                            }))?;
                            write!(stdout, "▀")?;
                        }
                        stdout.queue(SetBackgroundColor(THEME.panel))?;
                        stdout.queue(ResetColor)?;
                        let remaining_pad = right_cell_w.saturating_sub(rendered_cols);
                        if remaining_pad > 0 {
                            write!(stdout, "{}", " ".repeat(remaining_pad))?;
                        }
                    } else {
                        write!(stdout, "{}", " ".repeat(right_cell_w))?;
                    }
                }
            }
            Some(PreviewContent::Archive(arch_info)) => {
                if row == 0 {
                    let info_line = format!(
                        " 📦 ZIP 壓縮包 · 包含 {} 個項目 · 解壓總計: {}",
                        arch_info.total_files,
                        format_size(arch_info.uncompressed_size)
                    );
                    stdout.queue(SetForegroundColor(THEME.accent))?;
                    write!(stdout, "{}", fit_width(&info_line, right_cell_w))?;
                } else if row == 1 {
                    write!(stdout, "{}", " ".repeat(right_cell_w))?;
                } else {
                    let item_idx = row - 2;
                    let line_text = if item_idx < arch_info.items.len() {
                        format!("   {}", arch_info.items[item_idx])
                    } else {
                        String::new()
                    };
                    write!(stdout, "{}", fit_width(&line_text, right_cell_w))?;
                }
            }
            Some(PreviewContent::Directory {
                items: sub_items,
                total_items,
            }) => {
                let line_text = if row == 0 {
                    let dir_icon = get_folder_icon(app.icon_style);
                    format!(" {} 目錄包含 {} 個項目", dir_icon, total_items)
                } else if row == 1 {
                    String::new()
                } else {
                    let sub_idx = row - 2;
                    if sub_idx < sub_items.len() {
                        format!("   {}", sub_items[sub_idx])
                    } else {
                        String::new()
                    }
                };
                write!(stdout, "{}", fit_width(&line_text, right_cell_w))?;
            }
            Some(PreviewContent::Text { lines, .. }) => {
                if row < lines.len() {
                    let h_line = &lines[row];
                    write!(stdout, " ")?;
                    let mut rendered_w = 1;
                    for span in &h_line.spans {
                        if rendered_w >= right_cell_w {
                            break;
                        }
                        stdout.queue(SetForegroundColor(span.color))?;
                        if span.is_bold {
                            stdout.queue(SetAttribute(Attribute::Bold))?;
                        }
                        let avail = right_cell_w - rendered_w;
                        let text_chunk = fit_width(&span.text, avail.min(span.text.width()));
                        write!(stdout, "{}", text_chunk)?;
                        rendered_w += text_chunk.width();
                        if span.is_bold {
                            stdout.queue(SetAttribute(Attribute::Reset))?;
                            stdout.queue(SetBackgroundColor(THEME.panel))?;
                        }
                    }
                    if rendered_w < right_cell_w {
                        write!(stdout, "{}", " ".repeat(right_cell_w - rendered_w))?;
                    }
                } else {
                    write!(stdout, "{}", " ".repeat(right_cell_w))?;
                }
            }
            Some(PreviewContent::Binary { size }) => {
                let line_text = if row == 1 {
                    format!("   <二進位檔案或不支援預覽 — {}>", format_size(*size))
                } else {
                    String::new()
                };
                write!(stdout, "{}", fit_width(&line_text, right_cell_w))?;
            }
            Some(PreviewContent::TooLarge { size }) => {
                let line_text = if row == 1 {
                    format!(
                        "   <檔案過大 ({}) — 按 'e' 使用編輯器開啟>",
                        format_size(*size)
                    )
                } else {
                    String::new()
                };
                write!(stdout, "{}", fit_width(&line_text, right_cell_w))?;
            }
            Some(PreviewContent::Empty) => {
                let line_text = if row == 1 {
                    "   <空檔案>".to_string()
                } else {
                    String::new()
                };
                write!(stdout, "{}", fit_width(&line_text, right_cell_w))?;
            }
            Some(PreviewContent::Error(err)) => {
                let line_text = if row == 1 {
                    format!("   <{}>", err)
                } else {
                    String::new()
                };
                write!(stdout, "{}", fit_width(&line_text, right_cell_w))?;
            }
            None => {
                write!(stdout, "{}", " ".repeat(right_cell_w))?;
            }
        }
    }

    // 4. Bottom Status / Interactive Bar
    let status_y = layout.height.saturating_sub(1);
    stdout.queue(MoveTo(0, status_y))?;
    stdout.queue(SetBackgroundColor(THEME.title_bg))?;

    match app.input_mode {
        InputMode::Searching => {
            stdout.queue(SetForegroundColor(THEME.search))?;
            stdout.queue(SetAttribute(Attribute::Bold))?;
            let search_text = format!("  / 模糊搜尋: {}█   (Enter 定案 · Esc 取消)", app.filter);
            write!(stdout, "{}", fit_width(&search_text, layout.width as usize))?;
        }
        InputMode::Creating => {
            stdout.queue(SetForegroundColor(THEME.accent))?;
            stdout.queue(SetAttribute(Attribute::Bold))?;
            let create_text = format!(
                "  ✨ 新增檔案或目錄 (以 / 結尾建立目錄): {}█   (Enter 確認 · Esc 取消)",
                app.input_buffer
            );
            write!(stdout, "{}", fit_width(&create_text, layout.width as usize))?;
        }
        InputMode::Renaming => {
            stdout.queue(SetForegroundColor(THEME.search))?;
            stdout.queue(SetAttribute(Attribute::Bold))?;
            let rename_text = format!(
                "  ✏️ 重新命名為: {}█   (Enter 確認 · Esc 取消)",
                app.input_buffer
            );
            write!(stdout, "{}", fit_width(&rename_text, layout.width as usize))?;
        }
        InputMode::ConfirmDelete => {
            stdout.queue(SetForegroundColor(THEME.git_deleted))?;
            stdout.queue(SetAttribute(Attribute::Bold))?;
            let sel_name = selected_entry.map(|e| e.name.as_str()).unwrap_or("");
            let del_text = format!(
                "  ⚠️ 確定要刪除「{}」嗎？  (y: 確認刪除 · n / Esc: 取消)",
                sel_name
            );
            write!(stdout, "{}", fit_width(&del_text, layout.width as usize))?;
        }
        InputMode::Normal => {
            // Check for temporary toast message (lasts 3 seconds)
            if let Some((ref msg, instant)) = app.status_message {
                if instant.elapsed() < Duration::from_secs(3) {
                    stdout.queue(SetForegroundColor(THEME.accent))?;
                    stdout.queue(SetAttribute(Attribute::Bold))?;
                    let toast = format!("  {}", msg);
                    write!(stdout, "{}", fit_width(&toast, layout.width as usize))?;
                    stdout.queue(ResetColor)?;
                    stdout.queue(SetAttribute(Attribute::Reset))?;
                    stdout.flush()?;
                    return Ok(());
                }
            }

            stdout.queue(SetForegroundColor(THEME.muted))?;
            let pos_str = if !app.filtered_items.is_empty() {
                format!("{}/{}", app.selected + 1, app.filtered_items.len())
            } else {
                "0/0".to_string()
            };
            let filter_str = if !app.filter.is_empty() {
                format!("  過濾: {}", app.filter)
            } else {
                String::new()
            };
            let shortcuts_str = "↑↓移動  →進入  ←上層  /搜尋  s排序  i圖示  y複製  a新增  c改名  d刪除  e編輯  .隱藏  q離開";
            let status_text = format!(" {} {}   {}", pos_str, filter_str, shortcuts_str);
            write!(stdout, "{}", fit_width(&status_text, layout.width as usize))?;
        }
    }

    stdout.queue(ResetColor)?;
    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.flush()?;

    Ok(())
}
