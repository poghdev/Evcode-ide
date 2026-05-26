use walkdir::WalkDir;
use std::fs;

pub struct FileNode {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub depth: usize,
}

pub struct FileTree {
    pub nodes: Vec<FileNode>,
    pub selected: usize,
}

impl FileTree {
    pub fn new(root: &str) -> Self {
        let mut tree = Self {
            nodes: Vec::new(),
            selected: 0,
        };
        tree.refresh(root, false);
        tree
    }

    pub fn refresh(&mut self, root: &str, show_hidden: bool) {
        self.nodes.clear();
        for entry in WalkDir::new(root)
            .max_depth(4)
            .sort_by_file_name()
            .into_iter()
            .filter_entry(|e| {
                if e.depth() == 0 { return true; }
                let name = e.file_name().to_str().unwrap_or("");
                if !show_hidden && name.starts_with('.') {
                    return false;
                }
                name != "target"
            })
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_str().unwrap_or("");
                if !show_hidden && name.starts_with('.') {
                    return false;
                }
                name != "target"
            })
        {
            let depth = entry.depth();
            let is_dir = entry.file_type().is_dir();
            let path = entry.path().display().to_string();
            let name = entry.file_name().to_str().unwrap_or("").to_string();
            self.nodes.push(FileNode {
                path,
                name,
                is_dir,
                depth,
            });
        }
    }

    pub fn selected_path(&self) -> Option<&String> {
        self.nodes.get(self.selected).map(|n| &n.path)
    }

    pub fn select_next(&mut self) {
        if self.selected + 1 < self.nodes.len() {
            self.selected += 1;
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }
}

pub fn collect_files(root: &str) -> Vec<String> {
    WalkDir::new(root)
        .max_depth(6)
        .into_iter()
        .filter_entry(|e| {
            if e.depth() == 0 { return true; }
            let name = e.file_name().to_str().unwrap_or("");
            name != ".git" && name != "target"
        })
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_str().unwrap_or("");
            e.file_type().is_file()
                && !name.starts_with('.')
        })
        .map(|e| e.path().display().to_string())
        .collect()
}

pub fn create_file(path: &str) -> std::io::Result<()> {
    fs::File::create(path)?;
    Ok(())
}

pub fn create_dir(path: &str) -> std::io::Result<()> {
    fs::create_dir_all(path)?;
    Ok(())
}

pub fn delete_item(path: &str) -> std::io::Result<()> {
    let p = std::path::Path::new(path);
    if p.is_dir() { fs::remove_dir_all(p) } else { fs::remove_file(p) }
}

pub fn rename_item(old_path: &str, new_path: &str) -> std::io::Result<()> {
    fs::rename(old_path, new_path)
}
