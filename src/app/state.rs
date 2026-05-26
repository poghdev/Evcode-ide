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