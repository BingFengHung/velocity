use crate::cli::IconStyle;
use crate::fs::{open_in_editor, read_directory, FileEntry};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;

pub struct App {
    pub current_dir: PathBuf,
    pub all_items: Vec<FileEntry>,
    pub filtered_items: Vec<FileEntry>,
    pub selected: usize,
    pub scroll: usize,
    pub filter: String,
    pub is_searching: bool,
    pub icon_style: IconStyle,
    pub show_hidden: bool,
    pub should_quit: bool,
}

impl App {
    pub fn new(initial_path: PathBuf, icon_style: IconStyle, show_hidden: bool) -> Self {
        let abs_path = std::fs::canonicalize(&initial_path)
            .unwrap_or_else(|_| initial_path);

        let mut app = Self {
            current_dir: abs_path,
            all_items: Vec::new(),
            filtered_items: Vec::new(),
            selected: 0,
            scroll: 0,
            filter: String::new(),
            is_searching: false,
            icon_style,
            show_hidden,
            should_quit: false,
        };

        app.reload_directory();
        app
    }

    pub fn reload_directory(&mut self) {
        if let Ok(entries) = read_directory(&self.current_dir, self.show_hidden) {
            self.all_items = entries;
        } else {
            self.all_items = Vec::new();
        }
        self.apply_filter();
    }

    pub fn apply_filter(&mut self) {
        if self.filter.is_empty() {
            self.filtered_items = self.all_items.clone();
        } else {
            let lower_filter = self.filter.to_lowercase();
            self.filtered_items = self
                .all_items
                .iter()
                .filter(|item| item.name.to_lowercase().contains(&lower_filter))
                .cloned()
                .collect();
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
        if let Some(entry) = self.filtered_items.get(self.selected) {
            if entry.is_dir {
                self.current_dir = entry.path.clone();
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

            // Locate previously exited directory to keep cursor selection natural
            if let Some(prev_name) = previous_folder_name {
                if let Some(idx) = self.filtered_items.iter().position(|e| e.name == prev_name) {
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

    pub fn open_selected_in_editor(&self) {
        if let Some(entry) = self.filtered_items.get(self.selected) {
            if !entry.is_dir {
                let _ = open_in_editor(&entry.path);
            }
        }
    }

    pub fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
        self.reload_directory();
    }

    pub fn handle_key(&mut self, key: KeyEvent, list_height: usize) {
        if self.is_searching {
            match key.code {
                KeyCode::Enter => {
                    self.is_searching = false;
                }
                KeyCode::Esc => {
                    self.is_searching = false;
                    self.filter.clear();
                    self.apply_filter();
                }
                KeyCode::Backspace => {
                    self.filter.pop();
                    self.apply_filter();
                }
                KeyCode::Char(c) => {
                    if !key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::ALT) {
                        self.filter.push(c);
                        self.apply_filter();
                    }
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.filtered_items.is_empty() && self.selected < self.filtered_items.len() - 1 {
                    self.selected += 1;
                }
            }
            KeyCode::PageUp => {
                self.selected = self.selected.saturating_sub(list_height.max(1));
            }
            KeyCode::PageDown => {
                if !self.filtered_items.is_empty() {
                    self.selected = (self.selected + list_height.max(1)).min(self.filtered_items.len() - 1);
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
                self.is_searching = true;
                self.filter.clear();
                self.apply_filter();
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                self.open_selected_in_editor();
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.reload_directory();
            }
            KeyCode::Char('.') => {
                self.toggle_hidden();
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
