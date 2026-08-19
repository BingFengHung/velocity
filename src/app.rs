use crate::cli::IconStyle;
use crate::config::SortMode;
use crate::fs::{create_file_or_dir, delete_entry, read_directory, rename_entry, FileEntry};
use crate::fuzzy::{fuzzy_match, FuzzyMatch};
use crate::git::{get_git_status, GitFileStatus, GitRepoInfo};
use crate::graphics::GraphicsProtocol;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Searching,
    Creating,
    Renaming,
    ConfirmDelete,
}

#[derive(Clone, Debug)]
pub struct FilteredEntry {
    pub entry: FileEntry,
    pub fuzzy: Option<FuzzyMatch>,
    pub git_status: Option<GitFileStatus>,
}

pub struct App {
    pub current_dir: PathBuf,
    pub all_items: Vec<FileEntry>,
    pub filtered_items: Vec<FilteredEntry>,
    pub selected: usize,
    pub scroll: usize,
    pub filter: String,
    pub input_buffer: String,
    pub input_mode: InputMode,
    pub sort_mode: SortMode,
    pub icon_style: IconStyle,
    pub image_protocol: GraphicsProtocol,
    pub show_hidden: bool,
    pub should_quit: bool,
    pub git_info: Option<GitRepoInfo>,
    pub status_message: Option<(String, Instant)>,
    /// When set, the main loop will suspend the TUI, open this path in editor, then resume.
    pub pending_open_path: Option<PathBuf>,
}

impl App {
    pub fn new(
        initial_path: PathBuf,
        icon_style: IconStyle,
        image_protocol: GraphicsProtocol,
        show_hidden: bool,
    ) -> Self {
        let abs_path = std::fs::canonicalize(&initial_path).unwrap_or(initial_path);
        let abs_path = if let Ok(stripped) = abs_path.strip_prefix(r"\\?\") {
            stripped.to_path_buf()
        } else {
            abs_path
        };

        let mut app = Self {
            current_dir: abs_path,
            all_items: Vec::new(),
            filtered_items: Vec::new(),
            selected: 0,
            scroll: 0,
            filter: String::new(),
            input_buffer: String::new(),
            input_mode: InputMode::Normal,
            sort_mode: SortMode::Name,
            icon_style,
            image_protocol,
            show_hidden,
            should_quit: false,
            git_info: None,
            status_message: None,
            pending_open_path: None,
        };

        app.reload_directory();
        app
    }

    pub fn set_status(&mut self, msg: String) {
        self.status_message = Some((msg, Instant::now()));
    }

    pub fn reload_directory(&mut self) {
        self.git_info = get_git_status(&self.current_dir);

        if let Ok(entries) = read_directory(&self.current_dir, self.show_hidden, self.sort_mode) {
            self.all_items = entries;
        } else {
            self.all_items = Vec::new();
        }
        self.apply_filter();
    }

    pub fn apply_filter(&mut self) {
        if self.filter.is_empty() {
            self.filtered_items = self
                .all_items
                .iter()
                .map(|item| {
                    let git_stat = self
                        .git_info
                        .as_ref()
                        .and_then(|g| g.file_statuses.get(&item.path).copied());
                    FilteredEntry {
                        entry: item.clone(),
                        fuzzy: None,
                        git_status: git_stat,
                    }
                })
                .collect();
        } else {
            let mut matches = Vec::new();
            for item in &self.all_items {
                if let Some(f_match) = fuzzy_match(&self.filter, &item.name) {
                    let git_stat = self
                        .git_info
                        .as_ref()
                        .and_then(|g| g.file_statuses.get(&item.path).copied());
                    matches.push(FilteredEntry {
                        entry: item.clone(),
                        fuzzy: Some(f_match),
                        git_status: git_stat,
                    });
                }
            }

            // Sort fuzzy matches by score (highest first, with directories prioritized)
            matches.sort_by(|a, b| {
                match (a.entry.is_dir, b.entry.is_dir) {
                    (true, false) => return std::cmp::Ordering::Less,
                    (false, true) => return std::cmp::Ordering::Greater,
                    _ => {}
                }
                let score_a = a.fuzzy.as_ref().map(|f| f.score).unwrap_or(0);
                let score_b = b.fuzzy.as_ref().map(|f| f.score).unwrap_or(0);
                score_b.cmp(&score_a)
            });

            self.filtered_items = matches;
        }

        if self.filtered_items.is_empty() {
            self.selected = 0;
            self.scroll = 0;
        } else if self.selected >= self.filtered_items.len() {
            self.selected = self.filtered_items.len().saturating_sub(1);
        }
    }

    pub fn adjust_scroll(&mut self, list_height: usize) {
        if list_height == 0 {
            self.scroll = 0;
            return;
        }

        if self.selected < self.scroll {
            self.scroll = self.selected;
        }
        if self.selected >= self.scroll + list_height {
            self.scroll = self.selected.saturating_sub(list_height) + 1;
        }
        if self.scroll >= self.filtered_items.len() {
            self.scroll = self.filtered_items.len().saturating_sub(1);
        }
    }

    pub fn enter_directory(&mut self) {
        if let Some(item) = self.filtered_items.get(self.selected) {
            if item.entry.is_dir {
                self.current_dir = item.entry.path.clone();
                self.filter.clear();
                self.selected = 0;
                self.scroll = 0;
                self.reload_directory();
            }
        }
    }

    pub fn go_to_parent(&mut self) {
        if let Some(parent) = self.current_dir.parent().map(|p| p.to_path_buf()) {
            let previous_folder_name = self
                .current_dir
                .file_name()
                .map(|n| n.to_string_lossy().to_string());

            self.current_dir = parent;
            self.filter.clear();
            self.reload_directory();

            if let Some(prev_name) = previous_folder_name {
                if let Some(idx) = self
                    .filtered_items
                    .iter()
                    .position(|e| e.entry.name == prev_name)
                {
                    self.selected = idx;
                } else {
                    self.selected = 0;
                }
            } else {
                self.selected = 0;
            }
            self.scroll = 0;
        }
    }

    pub fn cycle_sort_mode(&mut self) {
        self.sort_mode = self.sort_mode.next();
        self.set_status(format!("已切換排序方式: {}", self.sort_mode.display_name()));
        self.reload_directory();
    }

    pub fn cycle_icon_style(&mut self) {
        self.icon_style = match self.icon_style {
            IconStyle::Auto => IconStyle::Nerd,
            IconStyle::Nerd => IconStyle::Emoji,
            IconStyle::Emoji => IconStyle::Ascii,
            IconStyle::Ascii => IconStyle::Auto,
        };
        let desc = match self.icon_style {
            IconStyle::Auto => {
                let effective = crate::icons::resolve_icon_style(self.icon_style);
                let effective_name = match effective {
                    IconStyle::Nerd => "Nerd Font",
                    IconStyle::Emoji => "Emoji",
                    IconStyle::Ascii => "ASCII",
                    IconStyle::Auto => "Auto",
                };
                format!("自動偵測 (當前生效: {})", effective_name)
            }
            IconStyle::Nerd => "Nerd Font".to_string(),
            IconStyle::Emoji => "Emoji".to_string(),
            IconStyle::Ascii => "ASCII".to_string(),
        };
        self.set_status(format!("已切換圖示樣式: {}", desc));
    }

    pub fn copy_selected_path(&mut self) {
        if let Some(item) = self.filtered_items.get(self.selected) {
            let path_str = item.entry.path.to_string_lossy().to_string();
            // Copy to clipboard via OS command
            #[cfg(target_os = "windows")]
            {
                use std::process::Command;
                let _ = Command::new("powershell")
                    .args([
                        "-NoProfile",
                        "-Command",
                        &format!("Set-Clipboard -Value '{}'", path_str.replace('\'', "''")),
                    ])
                    .output();
            }
            self.set_status(format!("📋 已複製路徑: {}", path_str));
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, list_height: usize) {
        match self.input_mode {
            InputMode::Searching => {
                match key.code {
                    KeyCode::Enter => {
                        self.input_mode = InputMode::Normal;
                    }
                    KeyCode::Esc => {
                        self.input_mode = InputMode::Normal;
                        self.filter.clear();
                        self.apply_filter();
                    }
                    KeyCode::Backspace => {
                        self.filter.pop();
                        self.apply_filter();
                    }
                    KeyCode::Char(c)
                        if !key.modifiers.contains(KeyModifiers::CONTROL)
                            && !key.modifiers.contains(KeyModifiers::ALT) =>
                    {
                        self.filter.push(c);
                        self.apply_filter();
                    }
                    _ => {}
                }
                return;
            }
            InputMode::Creating => {
                match key.code {
                    KeyCode::Enter => {
                        let name = self.input_buffer.trim().to_string();
                        if !name.is_empty() {
                            if let Err(e) = create_file_or_dir(&self.current_dir, &name) {
                                self.set_status(format!("❌ 建立失敗: {}", e));
                            } else {
                                self.set_status(format!("✨ 成功建立: {}", name));
                                self.reload_directory();
                            }
                        }
                        self.input_mode = InputMode::Normal;
                        self.input_buffer.clear();
                    }
                    KeyCode::Esc => {
                        self.input_mode = InputMode::Normal;
                        self.input_buffer.clear();
                    }
                    KeyCode::Backspace => {
                        self.input_buffer.pop();
                    }
                    KeyCode::Char(c) => {
                        self.input_buffer.push(c);
                    }
                    _ => {}
                }
                return;
            }
            InputMode::Renaming => {
                match key.code {
                    KeyCode::Enter => {
                        let new_name = self.input_buffer.trim().to_string();
                        if let Some(item) = self.filtered_items.get(self.selected) {
                            if !new_name.is_empty() && new_name != item.entry.name {
                                if let Err(e) = rename_entry(&item.entry.path, &new_name) {
                                    self.set_status(format!("❌ 重命名失敗: {}", e));
                                } else {
                                    self.set_status(format!("✏️ 已重命名為: {}", new_name));
                                    self.reload_directory();
                                }
                            }
                        }
                        self.input_mode = InputMode::Normal;
                        self.input_buffer.clear();
                    }
                    KeyCode::Esc => {
                        self.input_mode = InputMode::Normal;
                        self.input_buffer.clear();
                    }
                    KeyCode::Backspace => {
                        self.input_buffer.pop();
                    }
                    KeyCode::Char(c) => {
                        self.input_buffer.push(c);
                    }
                    _ => {}
                }
                return;
            }
            InputMode::ConfirmDelete => {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        if let Some(item) = self.filtered_items.get(self.selected) {
                            let name = item.entry.name.clone();
                            if let Err(e) = delete_entry(&item.entry.path, item.entry.is_dir) {
                                self.set_status(format!("❌ 刪除失敗: {}", e));
                            } else {
                                self.set_status(format!("🗑️ 已刪除: {}", name));
                                self.reload_directory();
                            }
                        }
                        self.input_mode = InputMode::Normal;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        self.input_mode = InputMode::Normal;
                        self.set_status("已取消刪除".to_string());
                    }
                    _ => {}
                }
                return;
            }
            InputMode::Normal => {}
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.filtered_items.is_empty() && self.selected < self.filtered_items.len() - 1
                {
                    self.selected += 1;
                }
            }
            KeyCode::PageUp => {
                self.selected = self.selected.saturating_sub(list_height.max(1));
            }
            KeyCode::PageDown => {
                if !self.filtered_items.is_empty() {
                    self.selected =
                        (self.selected + list_height.max(1)).min(self.filtered_items.len() - 1);
                }
            }
            KeyCode::Home => {
                self.selected = 0;
            }
            KeyCode::End => {
                if !self.filtered_items.is_empty() {
                    self.selected = self.filtered_items.len() - 1;
                }
            }
            KeyCode::Right | KeyCode::Enter | KeyCode::Char('l') => {
                self.enter_directory();
            }
            KeyCode::Left | KeyCode::Backspace | KeyCode::Char('h') => {
                self.go_to_parent();
            }
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Searching;
                self.filter.clear();
                self.apply_filter();
            }
            KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Char('o') => {
                self.cycle_sort_mode();
            }
            KeyCode::Char('i') | KeyCode::Char('I') => {
                self.cycle_icon_style();
            }
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.copy_selected_path();
            }
            KeyCode::Char('a') | KeyCode::Char('n') => {
                self.input_mode = InputMode::Creating;
                self.input_buffer.clear();
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if let Some(item) = self.filtered_items.get(self.selected) {
                    self.input_buffer = item.entry.name.clone();
                    self.input_mode = InputMode::Renaming;
                }
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if !self.filtered_items.is_empty() {
                    self.input_mode = InputMode::ConfirmDelete;
                }
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                if let Some(item) = self.filtered_items.get(self.selected) {
                    if !item.entry.is_dir {
                        // Signal the main loop to suspend TUI, open editor, then resume
                        self.pending_open_path = Some(item.entry.path.clone());
                    }
                }
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.reload_directory();
                self.set_status("已重新整理目錄".to_string());
            }
            KeyCode::Char('.') => {
                self.show_hidden = !self.show_hidden;
                self.set_status(if self.show_hidden {
                    "已顯示隱藏檔案".to_string()
                } else {
                    "已隱藏點開頭檔案".to_string()
                });
                self.reload_directory();
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_quit = true;
            }
            KeyCode::Esc => {
                if !self.filter.is_empty() {
                    self.filter.clear();
                    self.apply_filter();
                } else {
                    self.should_quit = true;
                }
            }
            _ => {}
        }
    }
}
