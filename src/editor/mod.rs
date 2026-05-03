pub mod highlighter;

use anyhow::{Context, Result};
use tui_textarea::TextArea;

pub use highlighter::Lang;

pub struct OpenFile {
    pub path: Option<String>,
    pub textarea: TextArea<'static>,
    pub modified: bool,
    pub lang: Lang,
    pub scroll_top: usize,
}

impl OpenFile {
    pub fn new_empty() -> Self {
        let mut ta = TextArea::default();
        ta.set_block(ratatui::widgets::Block::default());
        Self {
            path: None,
            textarea: ta,
            modified: false,
            lang: Lang::Generic,
            scroll_top: 0,
        }
    }

    pub fn from_path(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {path}"))?;
        let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        let lang = Lang::from_path(path);
        let ta = TextArea::new(lines);
        Ok(Self {
            path: Some(path.to_string()),
            textarea: ta,
            modified: false,
            lang,
            scroll_top: 0,
        })
    }

    pub fn save(&mut self) -> Result<()> {
        let path = self
            .path
            .as_ref()
            .context("File has no path. Use Ctrl+Shift+S")?;
        let content = self.textarea.lines().join("\n");
        std::fs::write(path, content)?;
        self.modified = false;
        Ok(())
    }

    pub fn title(&self) -> String {
        match &self.path {
            Some(p) => {
                let name = std::path::Path::new(p)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(p);
                if self.modified {
                    format!("●{name}")
                } else {
                    name.to_string()
                }
            }
            None => "[new]".into(),
        }
    }
}

pub struct EditorState {
    pub files: Vec<OpenFile>,
    pub current_file: usize,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            files: vec![OpenFile::new_empty()],
            current_file: 0,
        }
    }

    pub fn current_file_mut(&mut self) -> Option<&mut OpenFile> {
        self.files.get_mut(self.current_file)
    }

    pub fn current_file_ref(&self) -> Option<&OpenFile> {
        self.files.get(self.current_file)
    }

    pub fn open_file(&mut self, path: &str) -> Result<()> {
        for (i, f) in self.files.iter().enumerate() {
            if f.path.as_deref() == Some(path) {
                self.current_file = i;
                return Ok(());
            }
        }
        let file = OpenFile::from_path(path)?;
        self.files.push(file);
        self.current_file = self.files.len() - 1;
        Ok(())
    }

    pub fn save_current(&mut self) -> Result<()> {
        if let Some(f) = self.files.get_mut(self.current_file) {
            f.save()
        } else {
            Ok(())
        }
    }

    pub fn save_all(&mut self) -> Result<()> {
        for file in &mut self.files {
            if file.modified && file.path.is_some() {
                let _ = file.save();
            }
        }
        Ok(())
    }

    pub fn undo(&mut self) {
        if let Some(f) = self.files.get_mut(self.current_file) {
            f.textarea.undo();
        }
    }

    pub fn next_file(&mut self) {
        if self.files.len() > 1 {
            self.current_file = (self.current_file + 1) % self.files.len();
        }
    }

    pub fn prev_file(&mut self) {
        if self.files.len() > 1 {
            if self.current_file == 0 {
                self.current_file = self.files.len() - 1;
            } else {
                self.current_file -= 1;
            }
        }
    }

    pub fn close_path(&mut self, path: &str) {
        self.files.retain(|f| {
            if let Some(f_path) = &f.path {
                !(f_path == path || f_path.starts_with(&format!("{path}/")))
            } else {
                true
            }
        });

        if self.files.is_empty() {
            self.files.push(OpenFile::new_empty());
        }
        self.current_file = self.current_file.min(self.files.len().saturating_sub(1));
    }

    pub fn close_current(&mut self) {
        if self.files.len() > 1 {
            self.files.remove(self.current_file);
            if self.current_file >= self.files.len() {
                self.current_file = self.files.len() - 1;
            }
        } else {
            self.files[0] = OpenFile::new_empty();
        }
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }
}
