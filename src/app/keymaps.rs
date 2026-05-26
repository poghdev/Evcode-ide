use super::*;
use crate::editor::BufferMove;
use crossterm::event::{KeyCode, KeyModifiers};

impl App {
    pub async fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        use KeyCode::*;

        match (key.modifiers, key.code) {
            (m, Char('q')) if is_primary_modifier(m) => {
                self.should_quit = true;
                return;
            }
            (m, Char('r')) if is_primary_modifier(m) => {
                self.run_current_file().await;
                return;
            }
            (m, Char('b')) if is_primary_modifier(m) => {
                self.toggle_file_tree();
                return;
            }
            (m, Char('p')) if is_primary_modifier(m) => {
                self.toggle_teleport();
                return;
            }
            (m, Char('g')) if is_primary_modifier(m) => {
                self.ghost_mode = !self.ghost_mode;
                self.set_status(if self.ghost_mode {
                    "Ghost Mode ON (Ctrl+Enter apply, Esc cancel)"
                } else {
                    "Ghost Mode OFF"
                });
                return;
            }
            (m, Char('t')) if is_primary_modifier(m) => {
                self.theme = match self.theme {
                    Theme::Dark => Theme::Light,
                    Theme::Light => Theme::Dark,
                };
                return;
            }
            (m, Char('s')) if m.contains(KeyModifiers::ALT) => {
                if let Some(file) = self.editor.current_file_ref() {
                    if let Some(path) = &file.path {
                        self.snap.create_snapshot(path, &file.buffer.text());
                        let title = file.title();
                        self.set_status(format!("✓ Snapshot created: {title}"));
                    }
                }
                return;
            }
            (m, Char('d')) if m.contains(KeyModifiers::ALT) => {
                self.show_snap_diff = !self.show_snap_diff;
                if self.show_snap_diff {
                    if let Some(file) = self.editor.current_file_ref() {
                        if let Some(path) = &file.path {
                            let content = file.buffer.text();
                            self.snap_diff = self.snap.get_diff(path, &content);
                        }
                    }
                }
                return;
            }
            (m, Char('r')) if m.contains(KeyModifiers::ALT) => {
                self.rollback_current_file();
                return;
            }
            _ => {}
        }

        match self.focus {
            Focus::Editor => self.handle_editor_key(key),
            Focus::FileTree => self.handle_file_tree_key(key),
            Focus::Teleport => self.handle_teleport_key(key),
            Focus::Terminal => self.handle_terminal_key(key).await,
            Focus::Naming => self.handle_naming_key(key),
            Focus::ConfirmDelete => self.handle_confirm_delete_key(key),
        }
    }

    pub fn handle_editor_key(&mut self, key: crossterm::event::KeyEvent) {
        use KeyCode::*;

        match (key.modifiers, key.code) {
            (m, Char('s')) if is_primary_modifier(m) => {
                match self.editor.save_current() {
                    Ok(_) => {
                        if let Some(path) =
                            self.editor.current_file_ref().and_then(|f| f.path.as_deref())
                        {
                            let prefix = format!(
                                "file://{}",
                                std::fs::canonicalize(path)
                                    .unwrap_or_default()
                                    .display()
                            );
                            self.problems.retain(|p| !p.starts_with(&prefix));
                            self.lsp.diagnostics.retain(|p| !p.starts_with(&prefix));
                        }
                        self.set_status("✓ Saved");
                    }
                    Err(e) => self.set_status(format!("✗ Error: {e}")),
                }
            }

            (m, Char('z')) if is_primary_modifier(m) => {
                self.editor.undo();
                self.notify_current_lsp_did_change();
            }

            (m, Char('y')) if is_primary_modifier(m) => {
                self.editor.redo();
                self.notify_current_lsp_did_change();
            }

            (m, Char('a')) if is_primary_modifier(m) => {
                if let Some(file) = self.editor.current_file_mut() {
                    file.buffer.select_all();
                    self.set_status("All text selected (Ctrl+C copy, Ctrl+X cut)");
                }
            }

            (m, Char('c')) if is_primary_modifier(m) => {
                if let Some(text) = self.get_selected_text() {
                    self.clipboard_set(text);
                } else {
                    self.set_status("No selection — use Shift+Arrows or Ctrl+A");
                }
            }

            (m, Char('x')) if is_primary_modifier(m) => {
                if let Some(text) = self.get_selected_text() {
                    self.clipboard_set(text);
                    if let Some(file) = self.editor.current_file_mut() {
                        file.buffer.delete_char_forward();
                        file.modified = true;
                    }
                    self.notify_current_lsp_did_change();
                } else {
                    if let Some(file) = self.editor.current_file_mut() {
                        let (row, _) = file.buffer.cursor();
                        let line_text = file
                            .buffer
                            .line_text(row)
                            .map(|l| format!("{l}\n"))
                            .unwrap_or_default();
                        self.clipboard_set(line_text);
                        if let Some(file) = self.editor.current_file_mut() {
                            file.buffer.delete_current_line();
                            file.modified = true;
                        }
                        self.notify_current_lsp_did_change();
                    }
                }
            }

            (m, Char('v')) if is_primary_modifier(m) => {
                if let Some(text) = self.clipboard_get() {
                    let char_count = text.chars().count();
                    if let Some(file) = self.editor.current_file_mut() {
                        file.buffer.insert_str(&text);
                        file.modified = true;
                        self.set_status(format!("✓ Pasted ({char_count} chars)"));
                    }
                    self.notify_current_lsp_did_change();
                }
            }

            (m, Char('e')) if is_primary_modifier(m) => {
                self.editor.next_file();
                if let Some(f) = self.editor.current_file_ref() {
                    self.set_status(format!(
                        "Tab [{}/{}]: {}",
                        self.editor.current_file + 1,
                        self.editor.file_count(),
                        f.title()
                    ));
                }
            }

            (m, Char('A')) if is_primary_modifier(m) => {
                self.editor.prev_file();
                if let Some(f) = self.editor.current_file_ref() {
                    self.set_status(format!(
                        "Tab [{}/{}]: {}",
                        self.editor.current_file + 1,
                        self.editor.file_count(),
                        f.title()
                    ));
                }
            }

            (m, Char('w')) if is_primary_modifier(m) => {
                let title = self
                    .editor
                    .current_file_ref()
                    .map(|f| f.title())
                    .unwrap_or_default();
                self.editor.close_current();
                self.set_status(format!("Closed: {title}"));
            }

            (m, Char('`')) if is_primary_modifier(m) => {
                self.show_terminal = !self.show_terminal;
                self.focus = if self.show_terminal {
                    Focus::Terminal
                } else {
                    Focus::Editor
                };
            }

            (m, Enter) if self.ghost_mode && is_primary_modifier(m) => {
                self.ghost_mode = false;
                self.set_status("Ghost: changes applied");
            }
            (m, Esc) if self.ghost_mode || is_primary_modifier(m) => {
                self.ghost_mode = false;
                self.rollback_current_file();
                self.set_status("Ghost: changes cancelled");
            }

            (m, Left | Right | Up | Down | Home | End)
                if m.contains(KeyModifiers::SHIFT)
                    || m.contains(KeyModifiers::CONTROL)
                    || m.contains(KeyModifiers::ALT)
                    || m.contains(KeyModifiers::SUPER) =>
            {
                if let Some(file) = self.editor.current_file_mut() {
                    if let Some(movement) = key_to_buffer_move(key.code, m) {
                        file.buffer
                            .move_cursor(movement, m.contains(KeyModifiers::SHIFT));
                    }
                }
            }

            (m, Esc)
                if !m.contains(KeyModifiers::CONTROL) && !m.contains(KeyModifiers::ALT) =>
            {
                if let Some(file) = self.editor.current_file_mut() {
                    file.buffer.cancel_selection();
                }
            }

            (m, Enter)
                if !m.contains(KeyModifiers::CONTROL) && !m.contains(KeyModifiers::ALT) =>
            {
                if let Some(file) = self.editor.current_file_mut() {
                    let (row, col) = file.buffer.cursor();
                    let current_line = file.buffer.line_text(row).unwrap_or_default();
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
                    file.buffer.insert_newline();
                    file.buffer.insert_str(&indent);
                    if extra {
                        file.buffer.insert_str("    ");
                    }
                    file.modified = true;
                }
                self.notify_current_lsp_did_change();
            }

            _ => {
                let mut changed = false;
                if let Some(file) = self.editor.current_file_mut() {
                    if apply_editor_key(&mut file.buffer, key) {
                        file.modified = true;
                        changed = true;
                    }
                }
                if changed {
                    self.notify_current_lsp_did_change();
                }
            }
        }
    }

    pub fn handle_file_tree_key(&mut self, key: crossterm::event::KeyEvent) {
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
                self.set_status(if self.show_hidden {
                    "Hidden files shown"
                } else {
                    "Hidden files hidden"
                });
            }
            Enter => {
                if let Some(path) = self.file_tree.selected_path() {
                    let path = path.clone();
                    match self.editor.open_file(&path) {
                        Ok(_) => {
                            self.notify_lsp_did_open(&path);
                            self.set_status(format!("Opened: {path}"));
                        }
                        Err(e) => self.set_status(format!("Error: {e}")),
                    }
                    self.show_file_tree = false;
                    self.focus = Focus::Editor;
                }
            }
            _ => {}
        }
    }

    pub fn handle_teleport_key(&mut self, key: crossterm::event::KeyEvent) {
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
                        Ok(_) => {
                            self.notify_lsp_did_open(&path);
                            self.set_status(format!("Opened: {path}"));
                        }
                        Err(e) => self.set_status(format!("Error: {e}")),
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

    pub async fn handle_terminal_key(&mut self, key: crossterm::event::KeyEvent) {
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
                        self.problems.insert(output);
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

    pub fn handle_naming_key(&mut self, key: crossterm::event::KeyEvent) {
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
                            let full_path = std::path::PathBuf::from(&base)
                                .join(&name)
                                .to_string_lossy()
                                .into_owned();
                            let r = crate::filesystem::create_file(&full_path);
                            if r.is_ok() {
                                created_file_path = Some(full_path);
                            }
                            r
                        }
                        NamingType::CreateFolder => {
                            let full_path = std::path::PathBuf::from(&base)
                                .join(&name)
                                .to_string_lossy()
                                .into_owned();
                            crate::filesystem::create_dir(&full_path)
                        }
                        NamingType::Rename => {
                            let parent = std::path::Path::new(&self.naming_target)
                                .parent()
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_else(|| ".".into());
                            let new_path = std::path::PathBuf::from(&parent)
                                .join(&name)
                                .to_string_lossy()
                                .into_owned();
                            crate::filesystem::rename_item(&self.naming_target, &new_path)
                        }
                    };

                    match res {
                        Ok(_) => {
                            self.set_status(format!("Done: {name}"));
                            if let Some(path) = created_file_path {
                                if self.editor.open_file(&path).is_ok() {
                                    self.notify_lsp_did_open(&path);
                                }
                            }
                        }
                        Err(e) => self.set_status(format!("Error: {e}")),
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

    pub fn handle_confirm_delete_key(&mut self, key: crossterm::event::KeyEvent) {
        use KeyCode::*;
        match key.code {
            Char('y') | Char('Y') | Enter => {
                let path = self.naming_target.clone();
                match crate::filesystem::delete_item(&path) {
                    Ok(_) => {
                        self.editor.close_path(&path);
                        self.file_tree.refresh(".", self.show_hidden);
                        self.set_status(format!("Deleted: {path}"));
                    }
                    Err(e) => self.set_status(format!("Delete error: {e}")),
                }
                self.focus = Focus::FileTree;
            }
            Char('n') | Char('N') | Esc => self.focus = Focus::FileTree,
            _ => {}
        }
    }
}

pub fn key_to_buffer_move(code: KeyCode, modifiers: KeyModifiers) -> Option<BufferMove> {
    match code {
        #[cfg(target_os = "macos")]
        KeyCode::Left if modifiers.contains(KeyModifiers::SUPER) => Some(BufferMove::Home),
        #[cfg(target_os = "macos")]
        KeyCode::Right if modifiers.contains(KeyModifiers::SUPER) => Some(BufferMove::End),
        KeyCode::Left
            if modifiers.contains(KeyModifiers::CONTROL)
                || modifiers.contains(KeyModifiers::ALT) =>
        {
            Some(BufferMove::PreviousWord)
        }
        KeyCode::Right
            if modifiers.contains(KeyModifiers::CONTROL)
                || modifiers.contains(KeyModifiers::ALT) =>
        {
            Some(BufferMove::NextWord)
        }
        KeyCode::Left => Some(BufferMove::Left),
        KeyCode::Right => Some(BufferMove::Right),
        KeyCode::Up => Some(BufferMove::Up),
        KeyCode::Down => Some(BufferMove::Down),
        KeyCode::Home => Some(BufferMove::Home),
        KeyCode::End => Some(BufferMove::End),
        _ => None,
    }
}

pub fn is_primary_modifier(modifiers: KeyModifiers) -> bool {
    modifiers.contains(KeyModifiers::CONTROL) || modifiers.contains(KeyModifiers::SUPER)
}

pub fn apply_editor_key(
    buffer: &mut crate::editor::EditorBuffer,
    key: crossterm::event::KeyEvent,
) -> bool {
    use KeyCode::*;

    match key.code {
        Left | Right | Up | Down | Home | End
            if !key.modifiers.contains(KeyModifiers::SHIFT) =>
        {
            if let Some(movement) = key_to_buffer_move(key.code, key.modifiers) {
                buffer.move_cursor(movement, false);
            }
            false
        }
        Backspace if !key.modifiers.contains(KeyModifiers::ALT) => {
            buffer.delete_char_backward();
            true
        }
        Delete if !key.modifiers.contains(KeyModifiers::ALT) => {
            buffer.delete_char_forward();
            true
        }
        Char(c)
            if !key.modifiers.contains(KeyModifiers::CONTROL)
                && !key.modifiers.contains(KeyModifiers::ALT)
                && !key.modifiers.contains(KeyModifiers::SUPER) =>
        {
            let pair = match c {
                '(' => Some("()"),
                '[' => Some("[]"),
                '{' => Some("{}"),
                '"' => Some("\"\""),
                '\'' => Some("''"),
                _ => None,
            };

            if let Some(pair) = pair {
                buffer.insert_str(pair);
                buffer.move_cursor(BufferMove::Left, false);
            } else {
                buffer.insert_char(c);
            }
            true
        }
        _ => false,
    }
}