use crate::editor::EditorState;
use crate::filesystem::FileTree;
use crate::lsp::LspState;
use crate::session::{Session, SessionData};
use crate::snap::{DiffLine, GhostSnapManager};
use crate::terminal::TerminalState;
use crate::ui::draw;
use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{backend::Backend, Terminal};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum Focus {
    Editor,
    FileTree,
    Teleport,
    Terminal,
    Naming,
    ConfirmDelete,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NamingType {
    CreateFile,
    CreateFolder,
    Rename,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Theme {
    Dark,
    Light,
}

pub struct App {
    pub focus: Focus,
    pub theme: Theme,
    pub editor: EditorState,
    pub file_tree: FileTree,
    pub teleport: TeleportState,
    pub term: TerminalState,
    pub lsp: LspState,
    pub snap: GhostSnapManager,
    pub snap_diff: Option<Vec<DiffLine>>,
    pub show_snap_diff: bool,
    pub naming_input: String,
    pub naming_type: NamingType,
    pub naming_target: String,
    pub problems: Vec<String>,
    pub show_file_tree: bool,
    pub show_teleport: bool,
    pub show_terminal: bool,
    pub show_hidden: bool,
    pub ghost_mode: bool,
    pub should_quit: bool,
    pub status_msg: String,
    clipboard: Option<arboard::Clipboard>,
}

pub struct TeleportState {
    pub query: String,
    pub results: Vec<String>,
    pub selected: usize,
    pub all_files: Vec<String>,
}

impl TeleportState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            results: Vec::new(),
            selected: 0,
            all_files: Vec::new(),
        }
    }

    pub fn refresh_files(&mut self, root: &str) {
        self.all_files = crate::filesystem::collect_files(root);
        self.filter();
    }

    pub fn filter(&mut self) {
        let q = self.query.to_lowercase();
        self.results = if q.is_empty() {
            self.all_files.clone()
        } else {
            self.all_files
                .iter()
                .filter(|f| f.to_lowercase().contains(&q))
                .cloned()
                .collect()
        };
        self.selected = 0;
    }

    pub fn selected_path(&self) -> Option<&String> {
        self.results.get(self.selected)
    }
}

impl App {
    pub async fn new() -> Self {
        let mut app = Self {
            focus: Focus::Editor,
            theme: Theme::Dark,
            editor: EditorState::new(),
            file_tree: FileTree::new("."),
            teleport: TeleportState::new(),
            term: TerminalState::new(),
            lsp: LspState::new(),
            snap: GhostSnapManager::new(),
            snap_diff: None,
            show_snap_diff: false,
            naming_input: String::new(),
            naming_type: NamingType::CreateFile,
            naming_target: String::new(),
            problems: Vec::new(),
            show_file_tree: false,
            show_teleport: false,
            show_terminal: false,
            show_hidden: false,
            ghost_mode: false,
            should_quit: false,
            status_msg: String::from(
                "Evcode — Ctrl+C copy | Ctrl+V paste | Ctrl+X cut | Ctrl+A select all | Ctrl+Q quit",
            ),
            clipboard: None,
        };

        app.lsp.start_async().await;

        match Session::load() {
            Ok(session) => app.restore_session(session),
            Err(_) => {}
        }

        app
    }

    fn get_clipboard(&mut self) -> Result<&mut arboard::Clipboard> {
        if self.clipboard.is_none() {
            let cb = arboard::Clipboard::new()
                .context("Failed to initialize system clipboard")?;
            self.clipboard = Some(cb);
        }
        Ok(self.clipboard.as_mut().unwrap())
    }

    fn clipboard_set(&mut self, text: String) {
        if text.is_empty() {
            return;
        }
        let char_count = text.chars().count();
        match self.get_clipboard() {
            Ok(cb) => match cb.set_text(text) {
                Ok(_) => {
                    self.status_msg = format!("✓ Copied ({char_count} chars) to system clipboard");
                }
                Err(e) => self.status_msg = format!("✗ Clipboard set: {e}"),
            },
            Err(e) => self.status_msg = format!("✗ Clipboard: {e}"),
        }
    }

    fn clipboard_get(&mut self) -> Option<String> {
        match self.get_clipboard() {
            Ok(cb) => match cb.get_text() {
                Ok(text) if !text.is_empty() => Some(text),
                Ok(_) => {
                    self.status_msg = "Clipboard is empty".into();
                    None
                }
                Err(e) => {
                    self.status_msg = format!("✗ Clipboard get: {e}");
                    None
                }
            },
            Err(e) => {
                self.status_msg = format!("✗ Clipboard: {e}");
                None
            }
        }
    }

    fn get_selected_text(&self) -> Option<String> {
        let file = self.editor.current_file_ref()?;
        let (start, end) = file.textarea.selection_range()?;
        let (s, e) = if start <= end { (start, end) } else { (end, start) };

        let lines = file.textarea.lines();
        let mut text = String::new();

        for r in s.0..=e.0 {
            if let Some(line) = lines.get(r) {
                let start_col = if r == s.0 { s.1 } else { 0 };
                let end_col = if r == e.0 { e.1 } else { line.chars().count() };
                let part: String = line
                    .chars()
                    .skip(start_col)
                    .take(end_col.saturating_sub(start_col))
                    .collect();
                text.push_str(&part);
                if r < e.0 {
                    text.push('\n');
                }
            }
        }

        if text.is_empty() { None } else { Some(text) }
    }

    fn restore_session(&mut self, session: SessionData) {
        let mut count = 0usize;
        for path in &session.open_files {
            if self.editor.open_file(path).is_ok() {
                count += 1;
            }
        }
        if let Some(idx) = session.current_file_index {
            self.editor.current_file = idx.min(self.editor.files.len().saturating_sub(1));
        }
        if count > 0 {
            self.status_msg = format!("Session restored: {count} files");
        }
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            self.lsp.poll_messages();
            let new_diags = self.lsp.diagnostic_strings();
            for d in new_diags {
                if !self.problems.contains(&d) {
                    self.problems.push(d);
                }
            }

            terminal.draw(|f| draw(f, self))?;

            if event::poll(Duration::from_millis(16))? {
                match event::read()? {
                    Event::Key(key) => self.handle_key(key),
                    Event::Mouse(mouse) => {
                        self.handle_mouse(mouse);
                    }
                    _ => {}
                }
            }

            if self.should_quit {
                let _ = self.editor.save_all();
                self.snap.save_all_to_disk();
                self.save_session();
                break;
            }
        }
        Ok(())
    }

    fn save_session(&self) {
        let paths: Vec<String> = self
            .editor
            .files
            .iter()
            .filter_map(|f| f.path.clone())
            .collect();
        let session = SessionData {
            open_files: paths,
            current_file_index: Some(self.editor.current_file),
        };
        let _ = Session::save(&session);
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        use KeyCode::*;

        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, Char('q')) => {
                self.should_quit = true;
                return;
            }
            (KeyModifiers::CONTROL, Char('r')) => {
                self.run_current_file();
                return;
            }
            (KeyModifiers::CONTROL, Char('b')) => {
                self.toggle_file_tree();
                return;
            }
            (KeyModifiers::CONTROL, Char('p')) => {
                self.toggle_teleport();
                return;
            }
            (KeyModifiers::CONTROL, Char('g')) => {
                self.ghost_mode = !self.ghost_mode;
                self.status_msg = if self.ghost_mode {
                    "Ghost Mode ON (Ctrl+Enter apply, Esc cancel)".into()
                } else {
                    "Ghost Mode OFF".into()
                };
                return;
            }
            (KeyModifiers::CONTROL, Char('t')) => {
                self.theme = match self.theme {
                    Theme::Dark => Theme::Light,
                    Theme::Light => Theme::Dark,
                };
                return;
            }
            (KeyModifiers::ALT, Char('s')) => {
                if let Some(file) = self.editor.current_file_ref() {
                    if let Some(path) = &file.path {
                        self.snap.create_snapshot(path, &file.textarea.lines().join("\n"));
                        self.status_msg = format!("✓ Snapshot created: {}", file.title());
                    }
                }
                return;
            }
            (KeyModifiers::ALT, Char('d')) => {
                self.show_snap_diff = !self.show_snap_diff;
                if self.show_snap_diff {
                    if let Some(file) = self.editor.current_file_ref() {
                        if let Some(path) = &file.path {
                            let content = file.textarea.lines().join("\n");
                            self.snap_diff = self.snap.get_diff(path, &content);
                        }
                    }
                }
                return;
            }
            (KeyModifiers::ALT, Char('r')) => {
                self.rollback_current_file();
                return;
            }
            _ => {}
        }

        match self.focus {
            Focus::Editor => self.handle_editor_key(key),
            Focus::FileTree => self.handle_file_tree_key(key),
            Focus::Teleport => self.handle_teleport_key(key),
            Focus::Terminal => self.handle_terminal_key(key),
            Focus::Naming => self.handle_naming_key(key),
            Focus::ConfirmDelete => self.handle_confirm_delete_key(key),
        }
    }

    fn handle_mouse(&mut self, event: event::MouseEvent) {
        use event::MouseEventKind::*;

        if let Down(event::MouseButton::Left) = event.kind {
            let x = event.column;
            let y = event.row;

            if y == 0 {
                let mut current_x = 0;
                for (i, file) in self.editor.files.iter().enumerate() {
                    let tab_width = (4 + file.title().len() + 3) as u16;
                    if x >= current_x && x < current_x + tab_width {
                        self.editor.current_file = i;
                        self.focus = Focus::Editor;
                        return;
                    }
                    current_x += tab_width;
                }
            }

            if self.show_file_tree && y >= 1 {
                let tree_width = 30u16;
                if x < tree_width && y >= 2 {
                    let idx = (y - 2) as usize;
                    if idx < self.file_tree.nodes.len() {
                        self.file_tree.selected = idx;
                        if !self.file_tree.nodes[idx].is_dir {
                            let path = self.file_tree.nodes[idx].path.clone();
                            let _ = self.editor.open_file(&path);
                            self.show_file_tree = false;
                            self.focus = Focus::Editor;
                        }
                        return;
                    }
                }
                if x >= tree_width {
                    self.show_file_tree = false;
                    self.focus = Focus::Editor;
                }
            } else if y >= 1 && !self.show_teleport {
                self.focus = Focus::Editor;
            }
        }

        if self.focus == Focus::Editor {
            if let Some(file) = self.editor.current_file_mut() {
                file.textarea.input(event);

                match event.kind {
                    event::MouseEventKind::ScrollUp => {
                        file.scroll_top = file.scroll_top.saturating_sub(3);
                    }
                    event::MouseEventKind::ScrollDown => {
                        file.scroll_top = (file.scroll_top + 3)
                            .min(file.textarea.lines().len().saturating_sub(1));
                    }
                    _ => {}
                }
            }
        }
    }

    fn toggle_file_tree(&mut self) {
        self.show_file_tree = !self.show_file_tree;
        self.show_teleport = false;
        if self.show_file_tree {
            self.file_tree.refresh(".", self.show_hidden);
            self.focus = Focus::FileTree;
        } else {
            self.focus = Focus::Editor;
        }
    }

    fn toggle_teleport(&mut self) {
        self.show_teleport = !self.show_teleport;
        self.show_file_tree = false;
        if self.show_teleport {
            self.teleport.refresh_files(".");
            self.teleport.query.clear();
            self.teleport.filter();
            self.focus = Focus::Teleport;
        } else {
            self.focus = Focus::Editor;
        }
    }

    fn rollback_current_file(&mut self) {
        if let Some(file) = self.editor.current_file_mut() {
            if let Some(path) = &file.path {
                if let Some(old_content) = self.snap.rollback(path) {
                    let lines: Vec<String> = old_content.lines().map(|s| s.to_string()).collect();
                    file.textarea = tui_textarea::TextArea::new(lines);
                    file.modified = false;
                    self.status_msg = format!("↺ Rollback completed: {}", file.title());
                }
            }
        }
    }

    fn run_current_file(&mut self) {
        let _ = self.editor.save_current();
        if let Some(file) = self.editor.current_file_ref() {
            if let Some(path) = &file.path {
                let p = std::path::Path::new(path);
                let abs = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
                let parent = abs.parent().unwrap_or(p).display().to_string();
                let fname = abs
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                let cmd = if path.ends_with(".py") {
                    format!("cd \"{parent}\" && python3 \"{fname}\"")
                } else if path.ends_with(".rs") {
                    format!("cd \"{parent}\" && cargo run")
                } else if path.ends_with(".js") {
                    format!("cd \"{parent}\" && node \"{fname}\"")
                } else {
                    "echo 'No run command for this file type'".into()
                };

                self.show_terminal = true;
                self.focus = Focus::Terminal;
                self.term.execute(&cmd);
            }
        }
    }

    fn handle_editor_key(&mut self, key: crossterm::event::KeyEvent) {
        use KeyCode::*;

        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, Char('s')) => {
                match self.editor.save_current() {
                    Ok(_) => self.status_msg = "✓ Saved".into(),
                    Err(e) => self.status_msg = format!("✗ Error: {e}"),
                }
            }

            (KeyModifiers::CONTROL, Char('z')) => {
                self.editor.undo();
            }

            (KeyModifiers::CONTROL, Char('a')) => {
                if let Some(file) = self.editor.current_file_mut() {
                    file.textarea.move_cursor(tui_textarea::CursorMove::Top);
                    file.textarea.move_cursor(tui_textarea::CursorMove::Head);
                    file.textarea.start_selection();
                    file.textarea.move_cursor(tui_textarea::CursorMove::Bottom);
                    file.textarea.move_cursor(tui_textarea::CursorMove::End);
                    self.status_msg = "All text selected (Ctrl+C copy, Ctrl+X cut)".into();
                }
            }

            (KeyModifiers::CONTROL, Char('c')) => {
                if let Some(text) = self.get_selected_text() {
                    self.clipboard_set(text);
                } else {
                    self.status_msg = "No selection — use Shift+Arrows or Ctrl+A".into();
                }
            }

            (KeyModifiers::CONTROL, Char('x')) => {
                if let Some(text) = self.get_selected_text() {
                    self.clipboard_set(text);
                    if let Some(file) = self.editor.current_file_mut() {
                        file.textarea.cut();
                        file.modified = true;
                    }
                } else {
                    if let Some(file) = self.editor.current_file_mut() {
                        let (row, _) = file.textarea.cursor();
                        let line_text = file
                            .textarea
                            .lines()
                            .get(row)
                            .map(|l| format!("{l}\n"))
                            .unwrap_or_default();

                        self.clipboard_set(line_text);
                        if let Some(file) = self.editor.current_file_mut() {
                            file.textarea.move_cursor(tui_textarea::CursorMove::Head);
                            file.textarea.delete_line_by_end();
                            file.textarea.delete_char();
                            file.modified = true;
                        }
                    }
                }
            }

            (KeyModifiers::CONTROL, Char('v')) => {
                if let Some(text) = self.clipboard_get() {
                    let char_count = text.chars().count();
                    if let Some(file) = self.editor.current_file_mut() {
                        if file.textarea.selection_range().is_some() {
                            file.textarea.cut();
                        }
                        file.textarea.insert_str(&text);
                        file.modified = true;
                        self.status_msg = format!("✓ Pasted ({char_count} chars)");
                    }
                }
            }

            (KeyModifiers::CONTROL, Char('e')) => {
                self.editor.next_file();
                if let Some(f) = self.editor.current_file_ref() {
                    self.status_msg = format!(
                        "Tab [{}/{}]: {}",
                        self.editor.current_file + 1,
                        self.editor.file_count(),
                        f.title()
                    );
                }
            }

            (m, Char('A')) if m.contains(KeyModifiers::CONTROL) => {
                self.editor.prev_file();
                if let Some(f) = self.editor.current_file_ref() {
                    self.status_msg = format!(
                        "Tab [{}/{}]: {}",
                        self.editor.current_file + 1,
                        self.editor.file_count(),
                        f.title()
                    );
                }
            }

            (KeyModifiers::CONTROL, Char('w')) => {
                let title = self
                    .editor
                    .current_file_ref()
                    .map(|f| f.title())
                    .unwrap_or_default();
                self.editor.close_current();
                self.status_msg = format!("Closed: {title}");
            }

            (KeyModifiers::CONTROL, Char('`')) => {
                self.show_terminal = !self.show_terminal;
                self.focus = if self.show_terminal {
                    Focus::Terminal
                } else {
                    Focus::Editor
                };
            }

            (KeyModifiers::CONTROL, Enter) if self.ghost_mode => {
                self.ghost_mode = false;
                self.status_msg = "Ghost: changes applied".into();
            }
            (m, Esc) if self.ghost_mode || m.contains(KeyModifiers::CONTROL) => {
                self.ghost_mode = false;
                self.rollback_current_file();
                self.status_msg = "Ghost: changes cancelled".into();
            }

            (m, Left | Right | Up | Down | Home | End)
                if m.contains(KeyModifiers::SHIFT) =>
            {
                if let Some(file) = self.editor.current_file_mut() {
                    if file.textarea.selection_range().is_none() {
                        file.textarea.start_selection();
                    }
                    file.textarea.input(key);
                }
            }

            (KeyModifiers::NONE, Esc) => {
                if let Some(file) = self.editor.current_file_mut() {
                    file.textarea.cancel_selection();
                }
            }

            (KeyModifiers::NONE, Enter) => {
                if let Some(file) = self.editor.current_file_mut() {
                    let (row, col) = file.textarea.cursor();
                    let lines = file.textarea.lines();
                    let current_line = lines.get(row).map(|s| s.as_str()).unwrap_or("");
                    let indent: String = current_line
                        .chars()
                        .take_while(|c| c.is_whitespace())
                        .collect();
                    let last_non_ws = current_line
                        .chars()
                        .take(col)
                        .filter(|c| !c.is_whitespace())
                        .last();
                    let extra = matches!(last_non_ws, Some('{') | Some(':') | Some('('));
                    file.textarea.insert_newline();
                    file.textarea.insert_str(&indent);
                    if extra {
                        file.textarea.insert_str("    ");
                    }
                    file.modified = true;
                }
            }

            _ => {
                if let Some(file) = self.editor.current_file_mut() {
                    if let KeyCode::Char(c) = key.code {
                        if key.modifiers == KeyModifiers::NONE
                            || key.modifiers == KeyModifiers::SHIFT
                        {
                            let pair = match c {
                                '(' => Some("()"),
                                '[' => Some("[]"),
                                '{' => Some("{}"),
                                '"' => Some("\"\""),
                                '\'' => Some("''"),
                                _ => None,
                            };
                            if let Some(p) = pair {
                                file.textarea.insert_str(p);
                                file.textarea
                                    .move_cursor(tui_textarea::CursorMove::Back);
                                file.modified = true;
                                return;
                            }
                        }
                    }
                    if file.textarea.input(key) {
                        file.modified = true;
                    }
                }
            }
        }
    }

    fn handle_file_tree_key(&mut self, key: crossterm::event::KeyEvent) {
        use KeyCode::*;
        match key.code {
            Esc => {
                self.show_file_tree = false;
                self.focus = Focus::Editor;
            }
            Up => self.file_tree.select_prev(),
            Down => self.file_tree.select_next(),
            Char('n') => self.start_naming(NamingType::CreateFile),
            Char('N') | Char('f') => self.start_naming(NamingType::CreateFolder),
            Char('r') => self.start_naming(NamingType::Rename),
            Char('d') => {
                if let Some(path) = self.file_tree.selected_path() {
                    self.naming_target = path.clone();
                    self.focus = Focus::ConfirmDelete;
                }
            }
            Char('a') => {
                self.show_hidden = !self.show_hidden;
                self.file_tree.refresh(".", self.show_hidden);
                self.status_msg = if self.show_hidden {
                    "Hidden files shown".into()
                } else {
                    "Hidden files hidden".into()
                };
            }
            Enter => {
                if let Some(path) = self.file_tree.selected_path() {
                    let path = path.clone();
                    match self.editor.open_file(&path) {
                        Ok(_) => self.status_msg = format!("Opened: {path}"),
                        Err(e) => self.status_msg = format!("Error: {e}"),
                    }
                    self.show_file_tree = false;
                    self.focus = Focus::Editor;
                }
            }
            _ => {}
        }
    }

    fn handle_teleport_key(&mut self, key: crossterm::event::KeyEvent) {
        use KeyCode::*;
        match key.code {
            Esc => {
                self.show_teleport = false;
                self.focus = Focus::Editor;
            }
            Up => {
                if self.teleport.selected > 0 {
                    self.teleport.selected -= 1;
                }
            }
            Down => {
                if self.teleport.selected + 1 < self.teleport.results.len() {
                    self.teleport.selected += 1;
                }
            }
            Enter => {
                if let Some(path) = self.teleport.selected_path().cloned() {
                    match self.editor.open_file(&path) {
                        Ok(_) => self.status_msg = format!("Opened: {path}"),
                        Err(e) => self.status_msg = format!("Error: {e}"),
                    }
                    self.show_teleport = false;
                    self.focus = Focus::Editor;
                }
            }
            Backspace => {
                self.teleport.query.pop();
                self.teleport.filter();
            }
            Char(c) => {
                self.teleport.query.push(c);
                self.teleport.filter();
            }
            _ => {}
        }
    }

    fn handle_terminal_key(&mut self, key: crossterm::event::KeyEvent) {
        use KeyCode::*;
        match key.code {
            Esc => {
                self.show_terminal = false;
                self.focus = Focus::Editor;
            }
            Enter => {
                let cmd = self.term.input.clone();
                if !cmd.is_empty() {
                    let output = self.term.execute(&cmd);
                    if output.contains("error") || output.contains("Error") {
                        self.problems.push(output);
                    }
                    self.term.input.clear();
                }
            }
            Backspace => {
                self.term.input.pop();
            }
            Char(c) => {
                self.term.input.push(c);
            }
            _ => {}
        }
    }

    fn start_naming(&mut self, ntype: NamingType) {
        if let Some(path) = self.file_tree.selected_path() {
            self.naming_target = path.clone();
            self.naming_type = ntype;
            self.naming_input.clear();
            self.focus = Focus::Naming;
        }
    }

    fn handle_naming_key(&mut self, key: crossterm::event::KeyEvent) {
        use KeyCode::*;
        match key.code {
            Esc => self.focus = Focus::FileTree,
            Enter => {
                let name = self.naming_input.trim().to_string();
                if !name.is_empty() {
                    let base = if std::path::Path::new(&self.naming_target).is_dir() {
                        self.naming_target.clone()
                    } else {
                        std::path::Path::new(&self.naming_target)
                            .parent()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|| ".".into())
                    };

                    let mut created_file_path = None;
                    let res = match self.naming_type {
                        NamingType::CreateFile => {
                            let full_path = format!("{base}/{name}");
                            let r = crate::filesystem::create_file(&full_path);
                            if r.is_ok() {
                                created_file_path = Some(full_path);
                            }
                            r
                        }
                        NamingType::CreateFolder => {
                            crate::filesystem::create_dir(&format!("{base}/{name}"))
                        }
                        NamingType::Rename => {
                            let parent = std::path::Path::new(&self.naming_target)
                                .parent()
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_else(|| ".".into());
                            crate::filesystem::rename_item(
                                &self.naming_target,
                                &format!("{parent}/{name}"),
                            )
                        }
                    };

                    match res {
                        Ok(_) => {
                            self.status_msg = format!("Done: {name}");
                            if let Some(path) = created_file_path {
                                let _ = self.editor.open_file(&path);
                            }
                        }
                        Err(e) => self.status_msg = format!("Error: {e}"),
                    }
                    self.file_tree.refresh(".", self.show_hidden);
                }
                self.focus = Focus::FileTree;
            }
            Char(c) => self.naming_input.push(c),
            Backspace => {
                self.naming_input.pop();
            }
            _ => {}
        }
    }

    fn handle_confirm_delete_key(&mut self, key: crossterm::event::KeyEvent) {
        use KeyCode::*;
        match key.code {
            Char('y') | Char('Y') | Enter => {
                let path = self.naming_target.clone();
                match crate::filesystem::delete_item(&path) {
                    Ok(_) => {
                        self.editor.close_path(&path);
                        self.file_tree.refresh(".", self.show_hidden);
                        self.status_msg = format!("Deleted: {path}");
                    }
                    Err(e) => self.status_msg = format!("Delete error: {e}"),
                }
                self.focus = Focus::FileTree;
            }
            Char('n') | Char('N') | Esc => self.focus = Focus::FileTree,
            _ => {}
        }
    }
}
