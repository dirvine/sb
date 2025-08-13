use super::git::{FileStatus, GitRepository};
use anyhow::{Context, Result};
use ratatui::prelude::*;
use std::io;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use std::{
    io::Read,
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};
use tui_textarea::TextArea;
use tui_tree_widget::{TreeItem, TreeState};

// Vim mode removed — keep simple preview editing

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Left,
    Editor,
    Preview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpMode {
    None,
    Copy,
    Move,
    Mkdir,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct FileNode {
    pub path: PathBuf,
    pub label: String,
    pub children: Vec<FileNode>,
    pub is_dir: bool,
}

#[derive(Debug)]
pub struct App {
    pub root: PathBuf,
    pub focus: Focus,
    // Dual panes like MC
    pub left_dir: PathBuf,
    pub right_dir: PathBuf,
    pub left_tree: Vec<TreeItem<'static, String>>,
    pub right_tree: Vec<TreeItem<'static, String>>,
    pub left_state: TreeState<String>,
    pub editor: TextArea<'static>,
    pub opened: Option<PathBuf>,
    pub last_saved_text: Option<String>,
    pub status: String,
    pub show_help: bool,
    pub show_left_pane: bool,
    pub creating_file: bool,
    pub filename_input: TextArea<'static>,
    pub confirming_delete: bool,
    pub delete_target: Option<PathBuf>,
    // File picker overlay
    pub picking_file: bool,
    pub picker_dir: PathBuf,
    pub picker_items: Vec<PathBuf>,
    pub picker_index: usize,
    pub op_mode: OpMode,
    pub op_input: TextArea<'static>,
    pub op_source: Option<PathBuf>,
    // Inline preview editing
    pub preview_cursor: usize,
    pub editing_line: bool,
    pub line_input: TextArea<'static>,
    // Full raw edit mode in the preview pane
    pub show_raw_editor: bool,
    // Remember user preference for raw editor when switching focus
    pub prefer_raw_editor: bool,
    // Editor command mode (minimal)
    pub editor_cmd_mode: bool,
    pub editor_cmd_input: TextArea<'static>,
    // Simple preview editing state
    pub preview_col: usize,
    pub preview_scroll: usize,
    pub preview_viewport: usize,
    #[allow(dead_code)]
    pub undo_stack: Vec<Vec<String>>,
    #[allow(dead_code)]
    pub redo_stack: Vec<Vec<String>>,
    pub autoplay_video: bool,
    // Video playback
    pub video_player: Option<VideoPlayer>,
    pub video_path: Option<PathBuf>,
    // Git integration
    pub git_repo: Option<GitRepository>,
    pub git_status: HashMap<PathBuf, FileStatus>,
    // Move destination picker
    pub showing_move_dest: bool,
    pub move_dest_dir: PathBuf,
    pub move_dest_items: Vec<PathBuf>,
    pub move_dest_index: usize,
    pub move_source: Option<PathBuf>,
    // Git status display
    pub showing_git_status: bool,
    pub git_status_text: String,
}

impl App {
    pub fn new(root: PathBuf) -> Result<Self> {
        let left_tree = build_tree(&root)?;
        let right_tree = build_tree(&root)?;
        let mut left_state = TreeState::<String>::default();
        let mut right_state = TreeState::<String>::default();
        left_state.select(vec![root.display().to_string()]);
        right_state.select(vec![root.display().to_string()]);
        let mut editor = TextArea::default();
        editor.set_placeholder_text("Select a file in the tree (Enter) …");
        let mut filename_input = TextArea::default();
        filename_input.set_placeholder_text("new-note.md");
        let git_repo = GitRepository::open(&root).ok();
        let git_status: HashMap<PathBuf, FileStatus> = if let Some(ref repo) = git_repo {
            repo.status().unwrap_or_default()
        } else {
            HashMap::new()
        };

        Ok(Self {
            root: root.clone(),
            focus: Focus::Left,
            left_dir: root.clone(),
            right_dir: root.clone(),
            left_tree,
            right_tree,
            left_state,
            editor,
            opened: None,
            last_saved_text: None,
            status: "Ready".into(),
            show_help: false,
            show_left_pane: true,
            creating_file: false,
            filename_input,
            confirming_delete: false,
            delete_target: None,
            picking_file: false,
            picker_dir: root.clone(),
            picker_items: vec![],
            picker_index: 0,
            op_mode: OpMode::None,
            op_input: TextArea::default(),
            op_source: None,
            preview_cursor: 0,
            editing_line: false,
            line_input: TextArea::default(),
            show_raw_editor: false,
            prefer_raw_editor: false,
            editor_cmd_mode: false,
            editor_cmd_input: TextArea::default(),
            preview_col: 0,
            preview_scroll: 0,
            preview_viewport: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            autoplay_video: false,
            video_player: None,
            video_path: None,
            git_repo,
            git_status,
            showing_move_dest: false,
            move_dest_dir: root.clone(),
            move_dest_items: vec![],
            move_dest_index: 0,
            move_source: None,
            showing_git_status: false,
            git_status_text: String::new(),
        })
    }

    pub fn open_selected(&mut self) -> Result<()> {
        if let Some(path) = self.current_selection_path() {
            if path.is_dir() {
                return Ok(());
            }
            let text =
                fs::read_to_string(&path).with_context(|| format!("Reading {}", path.display()))?;
            self.editor = TextArea::from(text.lines().map(|s| s.to_string()).collect::<Vec<_>>());
            self.opened = Some(path);
            self.last_saved_text = Some(text);
            self.status = "File opened".into();
            self.focus = Focus::Preview;
        }
        Ok(())
    }

    pub fn activate_on_tree(&mut self) -> Result<()> {
        // If dir: toggle open; if file: open
        let current_path = self.left_state.selected().last().cloned();
        if let Some(id) = current_path {
            let p = PathBuf::from(&id);
            if p.is_dir() {
                let _ = self.left_state.toggle(self.left_state.selected().to_vec());
                return Ok(());
            }
        }
        self.open_selected()
    }

    pub fn save(&mut self) -> Result<()> {
        if let Some(path) = &self.opened {
            let text = self.editor.lines().join("\n");
            fs::write(path, text).with_context(|| format!("Saving {}", path.display()))?;
            self.last_saved_text = Some(self.editor.lines().join("\n"));
            self.status = "Saved".into();
        }
        Ok(())
    }

    pub fn open_externally(&mut self) -> Result<()> {
        if let Some(path) = &self.opened {
            opener::open(path)?;
        }
        Ok(())
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn toggle_left_pane(&mut self) {
        self.show_left_pane = !self.show_left_pane;
        if !self.show_left_pane && matches!(self.focus, Focus::Left) {
            self.focus = Focus::Preview;
        }
    }

    // --- Editor command mode ----------------------------------------------
    pub fn begin_editor_cmd(&mut self) {
        self.editor_cmd_mode = true;
        self.editor_cmd_input = TextArea::default();
        self.editor_cmd_input.insert_str(":");
    }

    pub fn cancel_editor_cmd(&mut self) {
        self.editor_cmd_mode = false;
    }

    pub fn confirm_editor_cmd(&mut self) -> Result<()> {
        let cmd = self.editor_cmd_input.lines().join("");
        let cmd = cmd.trim_start_matches(':').trim();
        match cmd {
            "w" => {
                self.save()?;
                self.status = "Saved".into();
            }
            "q" => {
                /* Optional: set a flag the main loop reads to quit */
                self.status = "Use F10/Q to quit".into();
            }
            "wq" => {
                self.save()?;
                self.status = "Saved (use F10/Q to quit)".into();
            }
            _ => {
                self.status = format!("Unknown :{cmd}");
            }
        }
        self.editor_cmd_mode = false;
        Ok(())
    }

    #[allow(dead_code)]
    fn push_undo(&mut self, lines: &[String]) {
        self.undo_stack.push(lines.to_vec());
        self.redo_stack.clear();
    }

    #[allow(dead_code)]
    fn save_lines(&mut self, lines: Vec<String>) {
        self.editor = TextArea::from(lines.clone());
        if let Some(path) = &self.opened {
            let _ = std::fs::write(path, lines.join("\n"));
        }
    }

    #[allow(dead_code)]
    pub fn insert_char_preview(&mut self, ch: char) {
        if self.preview_cursor >= self.editor.lines().len() {
            return;
        }
        let mut lines = self.editor.lines().to_vec();
        self.push_undo(&lines);
        let line = &mut lines[self.preview_cursor];
        let mut s = String::with_capacity(line.len() + 1);
        let mut idx = 0usize;
        for (count, c) in line.chars().enumerate() {
            if count == self.preview_col {
                break;
            }
            s.push(c);
            idx += c.len_utf8();
        }
        s.push(ch);
        s.push_str(&line[idx..]);
        *line = s;
        self.preview_col += 1;
        self.save_lines(lines);
    }

    #[allow(dead_code)]
    pub fn backspace_preview(&mut self) {
        if self.preview_cursor >= self.editor.lines().len() {
            return;
        }
        let mut lines = self.editor.lines().to_vec();
        self.push_undo(&lines);
        let line = &mut lines[self.preview_cursor];
        if self.preview_col == 0 {
            return;
        }
        let mut out = String::with_capacity(line.len());
        let mut idx = 0usize;
        for (count, c) in line.chars().enumerate() {
            if count + 1 == self.preview_col {
                break;
            }
            out.push(c);
            idx += c.len_utf8();
        }
        let mut skip_idx = idx;
        if let Some(c) = line[idx..].chars().next() {
            skip_idx += c.len_utf8();
        }
        out.push_str(&line[skip_idx..]);
        *line = out;
        self.preview_col -= 1;
        self.save_lines(lines);
    }

    #[allow(dead_code)]
    pub fn insert_newline_preview(&mut self) {
        let mut lines = self.editor.lines().to_vec();
        self.push_undo(&lines);
        let line = &mut lines[self.preview_cursor];
        let mut split_idx = 0usize;
        for (count, c) in line.chars().enumerate() {
            if count == self.preview_col {
                break;
            }
            split_idx += c.len_utf8();
        }
        let right = line[split_idx..].to_string();
        line.truncate(split_idx);
        let idx = self.preview_cursor + 1;
        lines.insert(idx, right);
        self.preview_cursor = idx;
        self.preview_col = 0;
        self.save_lines(lines);
    }

    #[allow(dead_code)]
    pub fn insert_newline_above_preview(&mut self) {
        let mut lines = self.editor.lines().to_vec();
        self.push_undo(&lines);
        let idx = self.preview_cursor;
        lines.insert(idx, String::new());
        self.save_lines(lines);
        self.preview_col = 0;
    }

    pub fn refresh_tree(&mut self) -> Result<()> {
        self.left_tree = build_tree(&self.left_dir)?;
        self.right_tree = build_tree(&self.right_dir)?;
        Ok(())
    }

    pub fn begin_create_file(&mut self) {
        self.creating_file = true;
        self.filename_input = TextArea::default();
        self.filename_input.set_placeholder_text("new-note.md");
    }

    pub fn cancel_create_file(&mut self) {
        self.creating_file = false;
        self.status = "Create canceled".into();
    }

    pub fn confirm_create_file(&mut self) -> Result<()> {
        let name = self
            .filename_input
            .lines()
            .first()
            .cloned()
            .unwrap_or_default();
        let name = name.trim();
        if name.is_empty() {
            self.status = "Name required".into();
            return Ok(());
        }
        let filename = if name.ends_with(".md") {
            name.to_string()
        } else {
            format!("{name}.md")
        };
        // Determine target directory from selection
        let sel_dir = self.current_dir_for_new_file();
        std::fs::create_dir_all(&sel_dir)?;
        let new_path = sel_dir.join(&filename);
        if new_path.exists() {
            self.status = "File exists".into();
            return Ok(());
        }
        let title = std::path::Path::new(&filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .replace(['-', '_'], " ");
        let initial = format!(
            "# {}\n\n",
            if title.is_empty() { "New Note" } else { &title }
        );
        fs::write(&new_path, &initial)?;
        self.opened = Some(new_path.clone());
        self.editor = TextArea::from(initial.lines().map(|s| s.to_string()).collect::<Vec<_>>());
        self.last_saved_text = Some(initial);
        self.creating_file = false;
        self.refresh_tree()?;
        // Select the new file in the left tree
        let _ = self.left_state.select(vec![new_path.display().to_string()]);
        self.status = "File created".into();
        self.focus = Focus::Editor;
        Ok(())
    }

    fn current_dir_for_new_file(&self) -> PathBuf {
        if let Some(p) = self.current_selection_path() {
            if p.is_dir() {
                p
            } else {
                p.parent().unwrap_or(Path::new(&self.root)).to_path_buf()
            }
        } else {
            self.root.clone()
        }
    }

    pub fn begin_delete(&mut self) {
        // Determine target from selection
        if let Some(id) = self.left_state.selected().last() {
            self.delete_target = Some(PathBuf::from(id));
            self.confirming_delete = true;
        }
    }

    pub fn cancel_delete(&mut self) {
        self.confirming_delete = false;
        self.delete_target = None;
    }

    #[allow(dead_code)]
    pub fn confirm_delete(&mut self) -> Result<()> {
        if let Some(path) = self.delete_target.clone() {
            if path.is_dir() {
                std::fs::remove_dir_all(&path)?;
            } else if path.exists() {
                std::fs::remove_file(&path)?;
            }
            if self.opened.as_ref().map(|p| p == &path).unwrap_or(false) {
                self.opened = None;
                self.editor = TextArea::default();
            }
            self.refresh_tree()?;
            self.status = format!("Deleted {}", path.display());
        }
        self.confirming_delete = false;
        self.delete_target = None;
        Ok(())
    }

    // --- File picker -------------------------------------------------------
    pub fn begin_file_picker(&mut self) -> Result<()> {
        self.status = "Opening file picker...".to_string();
        self.picking_file = true; // Set this BEFORE loading dir so it stays true even if load fails

        let start = self
            .opened
            .as_ref()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| self.root.clone());

        match self.load_picker_dir(start) {
            Ok(_) => {
                self.status = format!("File picker opened with {} items", self.picker_items.len());
            }
            Err(e) => {
                self.status = format!("Failed to load directory: {}", e);
                // Keep picking_file = true so user can still see the picker and navigate
            }
        }
        Ok(())
    }

    pub fn picker_up(&mut self) {
        if self.picker_index > 0 {
            self.picker_index -= 1;
        }
    }
    pub fn picker_down(&mut self) {
        if self.picker_index + 1 < self.picker_items.len() {
            self.picker_index += 1;
        }
    }
    pub fn picker_cancel(&mut self) {
        self.picking_file = false;
    }

    pub fn picker_activate(&mut self) -> Result<()> {
        if let Some(path) = self.picker_items.get(self.picker_index).cloned() {
            if path.is_dir() {
                self.load_picker_dir(path)?;
            } else {
                self.insert_link_to(&path)?;
                self.picking_file = false;
            }
        }
        Ok(())
    }

    fn load_picker_dir(&mut self, dir: PathBuf) -> Result<()> {
        let mut items: Vec<PathBuf> = std::fs::read_dir(&dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                !p.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .starts_with('.')
            })
            .collect();
        items.sort_by_key(|p| {
            (
                !p.is_dir(),
                p.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_lowercase(),
            )
        });
        // Add parent entry if not at root
        if let Some(parent) = dir.parent() {
            if parent != dir {
                items.insert(0, parent.to_path_buf());
            }
        }
        self.picker_dir = dir;
        self.picker_items = items;
        self.picker_index = 0;

        // Refresh Git status when loading new directory
        self.refresh_git_status();

        Ok(())
    }

    fn insert_link_to(&mut self, target: &Path) -> Result<()> {
        // Determine relative path against opened file dir (or root)
        let base_dir = self
            .opened
            .as_ref()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| self.root.clone());
        let rel = pathdiff::diff_paths(target, &base_dir).unwrap_or_else(|| target.to_path_buf());
        let rel_str = rel.to_string_lossy();
        let name = target.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let ext = target
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let is_image = matches!(
            ext.as_str(),
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp"
        );
        let is_video = matches!(ext.as_str(), "mp4" | "mov" | "mkv" | "webm" | "avi" | "m4v");
        let link = if is_image {
            format!("![{name}]({rel_str})")
        } else if is_video {
            format!("[video]({rel_str})")
        } else {
            format!("[{name}]({rel_str})")
        };
        // Insert at cursor
        self.editor.insert_str(&link);
        self.status = format!("Inserted link to {}", rel.display());
        Ok(())
    }

    // --- Git-aware file picker enhancements -----------------------------------

    /// Navigate to parent directory in file picker (P command)
    pub fn picker_parent_dir(&mut self) -> Result<()> {
        if let Some(parent) = self.picker_dir.parent() {
            self.load_picker_dir(parent.to_path_buf())?;
        }
        Ok(())
    }

    /// Start move operation (M command)
    pub fn picker_start_move(&mut self) -> Result<()> {
        // Don't switch overlays while in picker - just show status
        if let Some(path) = self.picker_items.get(self.picker_index).cloned() {
            if path.is_file() {
                // For now, just show a message that move is not supported in picker
                self.status = "Move not supported in picker. Use F6 in main view".to_string();
            } else {
                self.status = "Can only move files, not directories".to_string();
            }
        } else {
            self.status = "No item selected".to_string();
        }
        Ok(())
    }

    /// Show Git status (S command)
    pub fn picker_show_git_status(&mut self) {
        // Don't switch overlays while in picker - just show status in the status bar
        if let Some(ref repo) = self.git_repo {
            match repo.status_summary() {
                Ok(summary) => {
                    self.git_status_text = summary.clone();
                    // Don't set showing_git_status = true to avoid overlay
                    self.status = format!("Git: {}", summary);
                }
                Err(e) => {
                    self.status = format!("Failed to get Git status: {}", e);
                }
            }
        } else {
            self.status = "Not a Git repository".to_string();
        }
    }

    /// Delete with Git awareness (D command)
    pub fn picker_delete_with_git_check(&mut self) -> Result<()> {
        if let Some(path) = self.picker_items.get(self.picker_index).cloned() {
            if path.is_file() {
                if self.is_in_git_repo(&path) {
                    self.status =
                        "Delete will use 'git rm'. Press 'd' again to confirm.".to_string();
                    self.delete_target = Some(path);
                    self.confirming_delete = true;
                } else {
                    self.delete_target = Some(path);
                    self.confirming_delete = true;
                    self.status = "Press 'd' again to confirm deletion".to_string();
                }
            }
        }
        Ok(())
    }

    /// Confirm deletion with Git support
    pub fn confirm_delete_with_git(&mut self) -> Result<()> {
        if let Some(path) = self.delete_target.take() {
            if self.is_in_git_repo(&path) {
                if let Some(ref repo) = self.git_repo {
                    match repo.remove_file(&path) {
                        Ok(()) => {
                            self.status = format!(
                                "Git removed: {}",
                                path.file_name().unwrap_or_default().to_string_lossy()
                            );
                        }
                        Err(_) => {
                            // Fallback to regular file deletion
                            if path.exists() {
                                std::fs::remove_file(&path)?;
                                self.status = format!(
                                    "Deleted: {}",
                                    path.file_name().unwrap_or_default().to_string_lossy()
                                );
                            }
                        }
                    }
                }
            } else if path.exists() {
                std::fs::remove_file(&path)?;
                self.status = format!(
                    "Deleted: {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                );
            }

            // Refresh the UI - picker if in picker mode, otherwise main tree
            if self.picking_file {
                self.load_picker_dir(self.picker_dir.clone())?;
            } else {
                self.refresh_tree()?;
            }
            self.refresh_git_status();
        }
        self.confirming_delete = false;
        Ok(())
    }

    /// Check if a path is within the Git repository
    fn is_in_git_repo(&self, path: &Path) -> bool {
        if let Some(ref repo) = self.git_repo {
            path.starts_with(repo.root())
        } else {
            false
        }
    }

    /// Load directory for move destination picker
    fn load_move_dest_dir(&mut self, dir: PathBuf) -> Result<()> {
        let mut items: Vec<PathBuf> = std::fs::read_dir(&dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir()) // Only show directories for move destination
            .filter(|p| {
                !p.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .starts_with('.')
            })
            .collect();

        items.sort_by_key(|p| {
            p.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase()
        });

        // Add parent entry if not at root
        if let Some(parent) = dir.parent() {
            if parent != dir {
                items.insert(0, parent.to_path_buf());
            }
        }

        self.move_dest_dir = dir;
        self.move_dest_items = items;
        self.move_dest_index = 0;
        Ok(())
    }

    /// Navigate in move destination picker
    pub fn move_dest_up(&mut self) {
        if self.move_dest_index > 0 {
            self.move_dest_index -= 1;
        }
    }

    pub fn move_dest_down(&mut self) {
        if self.move_dest_index + 1 < self.move_dest_items.len() {
            self.move_dest_index += 1;
        }
    }

    /// Navigate to directory in move destination picker
    pub fn move_dest_enter(&mut self) -> Result<()> {
        if let Some(path) = self.move_dest_items.get(self.move_dest_index).cloned() {
            self.load_move_dest_dir(path)?;
        }
        Ok(())
    }

    /// Confirm move operation
    pub fn confirm_move(&mut self) -> Result<()> {
        if let Some(source) = self.move_source.take() {
            let dest_dir = &self.move_dest_dir;
            let filename = source.file_name().unwrap_or_default();
            let dest = dest_dir.join(filename);

            if self.is_in_git_repo(&source) && self.is_in_git_repo(&dest) {
                // Both source and dest are in the Git repo, use git mv
                if let Some(ref repo) = self.git_repo {
                    match repo.move_file(&source, &dest) {
                        Ok(()) => {
                            self.status = format!(
                                "Git moved: {} -> {}",
                                source.file_name().unwrap_or_default().to_string_lossy(),
                                dest.display()
                            );
                        }
                        Err(_) => {
                            // Fallback to regular move
                            std::fs::rename(&source, &dest)?;
                            self.status = format!(
                                "Moved: {} -> {}",
                                source.file_name().unwrap_or_default().to_string_lossy(),
                                dest.display()
                            );
                        }
                    }
                }
            } else {
                // Regular move
                std::fs::rename(&source, &dest)?;
                self.status = format!(
                    "Moved: {} -> {}",
                    source.file_name().unwrap_or_default().to_string_lossy(),
                    dest.display()
                );
            }

            // Refresh views
            self.load_picker_dir(self.picker_dir.clone())?;
            self.refresh_git_status();
        }

        self.showing_move_dest = false;
        Ok(())
    }

    /// Cancel move operation
    pub fn cancel_move(&mut self) {
        self.showing_move_dest = false;
        self.move_source = None;
        self.status = "Move cancelled".to_string();
    }

    /// Close Git status display
    pub fn close_git_status(&mut self) {
        self.showing_git_status = false;
        self.status = "Ready".to_string();
    }

    /// Refresh Git status cache
    pub fn refresh_git_status(&mut self) {
        if let Some(ref repo) = self.git_repo {
            if let Ok(status) = repo.status() {
                self.git_status = status;
            }
        }
    }

    /// Get the Git status of a file
    pub fn get_file_git_status(&self, path: &Path) -> Option<FileStatus> {
        // Try both absolute and canonical paths
        if let Some(status) = self.git_status.get(path).copied() {
            return Some(status);
        }

        // Try canonical path if available
        if let Ok(canonical_path) = path.canonicalize() {
            if let Some(status) = self.git_status.get(&canonical_path).copied() {
                return Some(status);
            }
        }

        None
    }

    /// Check if file preview should show diff
    pub fn should_show_diff(&self, path: &Path) -> bool {
        // Never auto-diff Markdown; keep normal rendered preview for .md files
        if path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("md") || e.eq_ignore_ascii_case("markdown"))
            .unwrap_or(false)
        {
            return false;
        }
        if let Some(status) = self.get_file_git_status(path) {
            matches!(status, FileStatus::Modified | FileStatus::Added)
        } else {
            false
        }
    }

    /// Get diff content for a file
    pub fn get_file_diff(&self, path: &Path) -> Option<String> {
        if let Some(ref repo) = self.git_repo {
            repo.file_diff(path).ok()
        } else {
            None
        }
    }

    pub fn current_selection_path(&self) -> Option<PathBuf> {
        let id = self.left_state.selected().last()?.clone();
        Some(PathBuf::from(id))
    }

    // --- MC style operations ----------------------------------------------
    pub fn begin_copy(&mut self) {
        self.op_mode = OpMode::Copy;
        self.op_input = TextArea::default();
        if let Some(src) = self.current_selection_path() {
            let name = src
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            self.op_input.insert_str(&name);
            self.op_source = Some(src);
        }
    }

    pub fn begin_move(&mut self) {
        self.op_mode = OpMode::Move;
        self.op_input = TextArea::default();
        if let Some(src) = self.current_selection_path() {
            let name = src
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            self.op_input.insert_str(&name);
            self.op_source = Some(src);
        }
    }

    pub fn begin_mkdir(&mut self) {
        self.op_mode = OpMode::Mkdir;
        self.op_input = TextArea::default();
    }

    pub fn cancel_op(&mut self) {
        self.op_mode = OpMode::None;
        self.op_source = None;
    }

    pub fn confirm_op(&mut self) -> Result<()> {
        match self.op_mode {
            OpMode::None => {}
            OpMode::Copy => {
                if let Some(src) = self.op_source.clone() {
                    let name = self.op_input.lines().first().cloned().unwrap_or_default();
                    let target_dir = if matches!(self.focus, Focus::Left) {
                        &self.right_dir
                    } else {
                        &self.left_dir
                    };
                    let dest = if name.is_empty() {
                        PathBuf::from(target_dir).join(src.file_name().unwrap_or_default())
                    } else {
                        PathBuf::from(target_dir).join(name)
                    };
                    if src.is_dir() {
                        copy_dir_all(&src, &dest)?;
                    } else {
                        std::fs::create_dir_all(dest.parent().unwrap_or(Path::new(".")))?;
                        std::fs::copy(&src, &dest)?;
                    }
                    self.status = format!("Copied → {}", dest.display());
                }
            }
            OpMode::Move => {
                if let Some(src) = self.op_source.clone() {
                    let name = self.op_input.lines().first().cloned().unwrap_or_default();
                    let target_dir = if matches!(self.focus, Focus::Left) {
                        &self.right_dir
                    } else {
                        &self.left_dir
                    };
                    let dest = if name.is_empty() {
                        PathBuf::from(target_dir).join(src.file_name().unwrap_or_default())
                    } else {
                        PathBuf::from(target_dir).join(name)
                    };
                    std::fs::create_dir_all(dest.parent().unwrap_or(Path::new(".")))?;
                    std::fs::rename(&src, &dest)?;
                    self.status = format!("Moved → {}", dest.display());
                }
            }
            OpMode::Mkdir => {
                let name = self.op_input.lines().first().cloned().unwrap_or_default();
                if name.trim().is_empty() {
                    self.status = "Name required".into();
                    self.op_mode = OpMode::None;
                    return Ok(());
                }
                let base = if matches!(self.focus, Focus::Left) {
                    &self.left_dir
                } else {
                    &self.right_dir
                };
                let dir = PathBuf::from(base).join(name);
                std::fs::create_dir_all(&dir)?;
                self.status = format!("Created dir {}", dir.display());
            }
        }
        self.refresh_tree()?;
        self.op_mode = OpMode::None;
        self.op_source = None;
        Ok(())
    }

    // --- Inline editing in Preview ----------------------------------------
    #[allow(dead_code)]
    pub fn begin_line_edit(&mut self) {
        if self.preview_cursor >= self.editor.lines().len() {
            return;
        }
        let current = self
            .editor
            .lines()
            .get(self.preview_cursor)
            .cloned()
            .unwrap_or_default();
        self.line_input = TextArea::default();
        self.line_input.insert_str(&current);
        self.preview_col = current.chars().count();
        self.editing_line = true; // activate bottom overlay editor
    }

    pub fn cancel_line_edit(&mut self) {
        self.editing_line = false;
    }

    pub fn confirm_line_edit(&mut self) {
        if self.preview_cursor < self.editor.lines().len() {
            let new_line = self.line_input.lines().join("");
            // Replace the specific line in the editor buffer
            let mut lines = self.editor.lines().to_vec();
            lines[self.preview_cursor] = new_line.clone();
            self.editor = TextArea::from(lines.clone());
            // Immediate save if file open
            if let Some(path) = &self.opened {
                let text = lines.join("\n");
                let _ = std::fs::write(path, text);
            }
            self.status = format!("Updated line {}", self.preview_cursor + 1);
        }
        self.editing_line = false;
    }

    // --- Vim helpers -------------------------------------------------------
    pub fn move_cursor_up(&mut self) {
        if self.preview_cursor > 0 {
            self.preview_cursor -= 1;
        }
        let vp = self.preview_viewport.max(1);
        if self.preview_cursor < self.preview_scroll {
            self.preview_scroll = self.preview_cursor;
        }
        if self.preview_cursor >= self.preview_scroll.saturating_add(vp) {
            self.preview_scroll = self.preview_cursor.saturating_sub(vp.saturating_sub(1));
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.preview_cursor + 1 < self.editor.lines().len() {
            self.preview_cursor += 1;
        }
        let vp = self.preview_viewport.max(1);
        if self.preview_cursor < self.preview_scroll {
            self.preview_scroll = self.preview_cursor;
        }
        if self.preview_cursor >= self.preview_scroll.saturating_add(vp) {
            self.preview_scroll = self.preview_cursor.saturating_sub(vp.saturating_sub(1));
        }
    }

    // Line operations (simple)
    #[allow(dead_code)]
    pub fn delete_current_line(&mut self) {
        let mut lines = self.editor.lines().to_vec();
        if self.preview_cursor < lines.len() {
            lines.remove(self.preview_cursor);
            if self.preview_cursor >= lines.len() && self.preview_cursor > 0 {
                self.preview_cursor -= 1;
            }
            self.save_lines(lines);
            self.status = "Deleted line".into();
        }
    }

    // Column and word motions
    #[allow(dead_code)]
    pub fn move_col_to_start(&mut self) {
        self.preview_col = 0;
    }
    #[allow(dead_code)]
    pub fn move_col_to_end(&mut self) {
        if let Some(line) = self.editor.lines().get(self.preview_cursor) {
            self.preview_col = line.chars().count();
        }
    }
    #[allow(dead_code)]
    pub fn move_word_forward(&mut self) {
        if let Some(line) = self.editor.lines().get(self.preview_cursor) {
            let cs: Vec<char> = line.chars().collect();
            let len = cs.len();
            let mut i = self.preview_col.min(len);
            let is_word = |c: char| c.is_alphanumeric();
            // skip non-word chars
            while i < len && !is_word(cs[i]) {
                i += 1;
            }
            // advance within word with camelCase boundary awareness
            while i + 1 < len && is_word(cs[i]) {
                if cs[i].is_lowercase() && cs[i + 1].is_uppercase() {
                    break;
                }
                i += 1;
            }
            // move to next position (end-of-word)
            if i < len {
                i += 1;
            }
            self.preview_col = i.min(len);
        }
    }
    #[allow(dead_code)]
    pub fn move_word_back(&mut self) {
        if let Some(line) = self.editor.lines().get(self.preview_cursor) {
            let cs: Vec<char> = line.chars().collect();
            let len = cs.len();
            if self.preview_col == 0 {
                return;
            }
            let mut i = self.preview_col.min(len);
            // step back one to inspect boundary
            i = i.saturating_sub(1);
            let is_word = |c: char| c.is_alphanumeric();
            // skip non-word chars backwards
            while i > 0 && !is_word(cs[i]) {
                i = i.saturating_sub(1);
            }
            // move to start of word with camelCase awareness
            while i > 0 && is_word(cs[i - 1]) {
                if cs[i - 1].is_lowercase() && cs[i].is_uppercase() {
                    break;
                }
                i -= 1;
            }
            self.preview_col = i;
        }
    }

    pub fn move_col_right(&mut self) {
        if let Some(line) = self.editor.lines().get(self.preview_cursor) {
            let len = line.chars().count();
            if self.preview_col < len {
                self.preview_col += 1;
            }
        }
    }
    #[allow(dead_code)]
    pub fn delete_char_under(&mut self) {
        if self.preview_cursor >= self.editor.lines().len() {
            return;
        }
        let mut lines = self.editor.lines().to_vec();
        let line_len = lines[self.preview_cursor].chars().count();
        if self.preview_col >= line_len {
            return;
        }
        self.push_undo(&lines);
        let line = &mut lines[self.preview_cursor];
        let mut out = String::with_capacity(line.len());
        let mut idx = 0usize;
        for (count, c) in line.chars().enumerate() {
            if count == self.preview_col {
                break;
            }
            out.push(c);
            idx += c.len_utf8();
        }
        let mut skip_idx = idx;
        if let Some(c) = line[idx..].chars().next() {
            skip_idx += c.len_utf8();
        }
        out.push_str(&line[skip_idx..]);
        *line = out;
        self.save_lines(lines);
    }
    #[allow(dead_code)]
    pub fn delete_char_before(&mut self) {
        self.backspace_preview();
    }
    #[allow(dead_code)]
    pub fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            let current = self.editor.lines().to_vec();
            self.redo_stack.push(current);
            self.save_lines(prev);
        }
    }
    #[allow(dead_code)]
    pub fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            let current = self.editor.lines().to_vec();
            self.undo_stack.push(current);
            self.save_lines(next);
        }
    }

    // --- Video controls -----------------------------------------------------
    pub fn start_video(&mut self, path: PathBuf) {
        self.stop_video();
        match VideoPlayer::spawn(path.clone()) {
            Ok(vp) => {
                self.video_player = Some(vp);
                self.video_path = Some(path);
                self.status = "Playing video".into();
            }
            Err(e) => {
                self.status = format!("Video error: {e}");
                self.video_player = None;
                self.video_path = None;
            }
        }
    }

    pub fn stop_video(&mut self) {
        if let Some(mut vp) = self.video_player.take() {
            vp.stop();
        }
        self.video_path = None;
    }

    pub fn toggle_pause_video(&mut self) {
        if let Some(vp) = &self.video_player {
            vp.toggle_pause();
        }
    }
}

#[derive(Debug)]
pub struct VideoPlayer {
    child: Child,
    last_frame: Arc<Mutex<Option<image::DynamicImage>>>,
    stop_flag: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl VideoPlayer {
    pub fn spawn(path: PathBuf) -> anyhow::Result<Self> {
        let mut child = Command::new("ffmpeg")
            .arg("-hide_banner")
            .arg("-loglevel")
            .arg("error")
            .arg("-i")
            .arg(path)
            .arg("-f")
            .arg("image2pipe")
            .arg("-vcodec")
            .arg("mjpeg")
            .arg("-")
            .stdout(Stdio::piped())
            .spawn()
            .context("spawning ffmpeg")?;
        let mut stdout = child.stdout.take().context("ffmpeg stdout")?;
        let last_frame = Arc::new(Mutex::new(None));
        let stop_flag = Arc::new(AtomicBool::new(false));
        let paused = Arc::new(AtomicBool::new(false));
        let lf = Arc::clone(&last_frame);
        let sf = Arc::clone(&stop_flag);
        let pf = Arc::clone(&paused);
        let handle = thread::spawn(move || {
            let mut buf: Vec<u8> = Vec::with_capacity(1 << 20);
            let mut chunk = [0u8; 8192];
            let mut frame_start = None;
            while !sf.load(Ordering::Relaxed) {
                match stdout.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(n) => {
                        buf.extend_from_slice(&chunk[..n]);
                        // scan for JPEG SOI/EOI markers
                        let mut i = 0;
                        while i + 1 < buf.len() {
                            if frame_start.is_none() && buf[i] == 0xFF && buf[i + 1] == 0xD8 {
                                frame_start = Some(i);
                            }
                            if buf[i] == 0xFF && buf[i + 1] == 0xD9 {
                                if let Some(start) = frame_start {
                                    let end = i + 2;
                                    let frame = &buf[start..end];
                                    if !pf.load(Ordering::Relaxed) {
                                        if let Ok(img) = image::load_from_memory(frame) {
                                            if let Ok(mut guard) = lf.lock() {
                                                *guard = Some(img);
                                            }
                                        }
                                    }
                                    // drain consumed bytes
                                    buf.drain(..end);
                                    frame_start = None;
                                    i = 0;
                                    continue;
                                }
                            }
                            i += 1;
                        }
                    }
                    Err(_) => {
                        thread::sleep(Duration::from_millis(10));
                    }
                }
            }
        });
        Ok(Self {
            child,
            last_frame,
            stop_flag,
            paused,
            handle: Some(handle),
        })
    }

    pub fn last_frame(&self) -> Option<image::DynamicImage> {
        self.last_frame.lock().ok().and_then(|g| g.clone())
    }

    pub fn toggle_pause(&self) {
        self.paused
            .store(!self.paused.load(Ordering::Relaxed), Ordering::Relaxed);
    }

    pub fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        let _ = self.child.kill();
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

// --- Tree helpers -----------------------------------------------------------

fn build_tree(root: &Path) -> Result<Vec<TreeItem<'static, String>>> {
    fn build_node(dir: &Path) -> TreeItem<'static, String> {
        let mut children: Vec<TreeItem<'static, String>> = std::fs::read_dir(dir)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
            .map(|e| {
                let p = e.path();
                if p.is_dir() {
                    build_node(&p)
                } else {
                    let text = Line::from(e.file_name().to_string_lossy().to_string());
                    TreeItem::new_leaf(p.display().to_string(), text)
                }
            })
            .collect();
        children.sort_by_key(|item| item.identifier().clone());
        let text = Line::from(
            dir.file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| dir.display().to_string()),
        );
        TreeItem::new(dir.display().to_string(), text, children).expect("unique ids")
    }

    let root_item = build_node(root);
    Ok(vec![root_item])
}

fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            if let Some(parent) = to.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}
