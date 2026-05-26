use super::*;
use crate::session::{Session, SessionData};
use anyhow::Context;
use serde_json::json;

impl App {
    pub fn toggle_file_tree(&mut self) {
        self.show_file_tree = !self.show_file_tree;
        self.show_teleport = false;
        if self.show_file_tree {
            self.file_tree.refresh(".", self.show_hidden);
            self.focus = Focus::FileTree;
        } else {
            self.focus = Focus::Editor;
        }
    }

    pub fn toggle_teleport(&mut self) {
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

    pub fn rollback_current_file(&mut self) {
        if let Some(file) = self.editor.current_file_mut() {
            if let Some(path) = &file.path {
                if let Some(old_content) = self.snap.rollback(path) {
                    file.buffer = crate::editor::EditorBuffer::from_str(&old_content);
                    file.modified = false;
                    let title = file.title();
                    self.set_status(format!("↺ Rollback completed: {title}"));
                    self.notify_current_lsp_did_change();
                }
            }
        }
    }

    pub async fn run_current_file(&mut self) {
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

    pub fn start_naming(&mut self, ntype: NamingType) {
        if let Some(path) = self.file_tree.selected_path() {
            self.naming_target = path.clone();
            self.naming_type = ntype;
            self.naming_input.clear();
            self.focus = Focus::Naming;
        }
    }

    pub fn save_session(&self) {
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

    pub fn restore_session(&mut self, session: SessionData) {
        let mut count = 0usize;
        for path in &session.open_files {
            if self.editor.open_file(path).is_ok() {
                self.notify_lsp_did_open(path);
                count += 1;
            }
        }
        if let Some(idx) = session.current_file_index {
            self.editor.current_file =
                idx.min(self.editor.files.len().saturating_sub(1));
        }
        if count > 0 {
            self.set_status(format!("Session restored: {count} files"));
        }
    }

    pub fn clipboard_set(&mut self, text: String) {
        if text.is_empty() {
            return;
        }
        let char_count = text.chars().count();
        match self.get_clipboard() {
            Ok(cb) => match cb.set_text(text) {
                Ok(_) => {
                    self.set_status(format!(
                        "✓ Copied ({char_count} chars) to system clipboard"
                    ));
                }
                Err(e) => self.set_status(format!("✗ Clipboard set: {e}")),
            },
            Err(e) => self.set_status(format!("✗ Clipboard: {e}")),
        }
    }

    pub fn clipboard_get(&mut self) -> Option<String> {
        match self.get_clipboard() {
            Ok(cb) => match cb.get_text() {
                Ok(text) if !text.is_empty() => Some(text),
                Ok(_) => {
                    self.set_status("Clipboard is empty");
                    None
                }
                Err(e) => {
                    self.set_status(format!("✗ Clipboard get: {e}"));
                    None
                }
            },
            Err(e) => {
                self.set_status(format!("✗ Clipboard: {e}"));
                None
            }
        }
    }

    pub fn get_clipboard(&mut self) -> anyhow::Result<&mut arboard::Clipboard> {
        if self.clipboard.is_none() {
            let cb = arboard::Clipboard::new()
                .context("Failed to initialize system clipboard")?;
            self.clipboard = Some(cb);
        }
        Ok(self.clipboard.as_mut().unwrap())
    }

    pub fn send_lsp(&mut self, msg: String) {
        if let Some(tx) = &self.lsp_tx {
            let _ = tx.try_send(msg);
        }
    }

    pub fn notify_lsp_did_open(&mut self, path: &str) {
        let Some(file) = self
            .editor
            .files
            .iter()
            .find(|file| file.path.as_deref() == Some(path))
        else {
            return;
        };
        let Some(uri) = super::lsp_helpers::file_uri(path) else {
            return;
        };
        let msg = crate::lsp::json_rpc_notification(
            "textDocument/didOpen",
            json!({
                "textDocument": {
                    "uri": uri,
                    "languageId": super::lsp_helpers::language_id(path),
                    "version": file.lsp_version,
                    "text": file.buffer.text()
                }
            }),
        );
        self.send_lsp(msg);
    }

    pub fn notify_current_lsp_did_change(&mut self) {
        let Some(file) = self.editor.current_file_mut() else {
            return;
        };
        let Some(path) = file.path.clone() else {
            return;
        };
        let Some(uri) = super::lsp_helpers::file_uri(&path) else {
            return;
        };
        file.lsp_version += 1;
        let version = file.lsp_version;
        let text = file.buffer.text();

        let msg = crate::lsp::json_rpc_notification(
            "textDocument/didChange",
            json!({
                "textDocument": {
                    "uri": uri,
                    "version": version
                },
                "contentChanges": [
                    {
                        "text": text
                    }
                ]
            }),
        );
        self.send_lsp(msg);
    }

    pub fn get_selected_text(&self) -> Option<String> {
        let file = self.editor.current_file_ref()?;
        file.buffer.selected_text()
    }
}