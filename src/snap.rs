use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffLine {
    Unchanged(String),
    Added(String),
    Removed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub content: String,
}

pub struct GhostSnapManager {
    pub snapshots: HashMap<String, Snapshot>,
}

impl GhostSnapManager {
    pub fn new() -> Self {
        let mut manager = Self {
            snapshots: HashMap::new(),
        };
        let _ = manager.load_from_disk();
        manager
    }

    pub fn create_snapshot(&mut self, path: &str, content: &str) {
        self.snapshots.insert(
            path.to_string(),
            Snapshot {
                content: content.to_string(),
            },
        );
    }

    pub fn rollback(&self, path: &str) -> Option<String> {
        self.snapshots.get(path).map(|s| s.content.clone())
    }

    pub fn get_diff(&self, path: &str, current_content: &str) -> Option<Vec<DiffLine>> {
        let snap = self.snapshots.get(path)?;
        let old_lines: Vec<&str> = snap.content.lines().collect();
        let new_lines: Vec<&str> = current_content.lines().collect();

        let mut diff = Vec::new();
        let max_len = old_lines.len().max(new_lines.len());
        for i in 0..max_len {
            match (old_lines.get(i), new_lines.get(i)) {
                (Some(o), Some(n)) if o == n => diff.push(DiffLine::Unchanged(o.to_string())),
                (Some(o), Some(n)) => {
                    diff.push(DiffLine::Removed(o.to_string()));
                    diff.push(DiffLine::Added(n.to_string()));
                }
                (Some(o), None) => diff.push(DiffLine::Removed(o.to_string())),
                (None, Some(n)) => diff.push(DiffLine::Added(n.to_string())),
                (None, None) => {}
            }
        }
        Some(diff)
    }

    pub fn save_all_to_disk(&self) {
        let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("evcode");
        path.push("snaps");
        path.push("ghost_snaps.json");
        if let Ok(json) = serde_json::to_string_pretty(&self.snapshots) {
            let _ = fs::create_dir_all(path.parent().unwrap());
            let _ = fs::write(path, json);
        }
    }

    fn load_from_disk(&mut self) -> anyhow::Result<()> {
        let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("evcode");
        path.push("snaps");
        path.push("ghost_snaps.json");
        if path.exists() {
            let json = fs::read_to_string(path)?;
            if let Ok(snaps) = serde_json::from_str(&json) {
                self.snapshots = snaps;
            }
        }
        Ok(())
    }
}