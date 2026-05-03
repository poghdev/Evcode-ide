use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
pub struct SessionData {
    pub open_files: Vec<String>,
    pub current_file_index: Option<usize>,
}

pub struct Session;

impl Session {
    fn session_path() -> PathBuf {
        let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("evcode");
        std::fs::create_dir_all(&path).ok();
        path.push("session.json");
        path
    }

    pub fn save(data: &SessionData) -> Result<()> {
        let json = serde_json::to_string_pretty(data)?;
        std::fs::write(Self::session_path(), json)?;
        Ok(())
    }

    pub fn load() -> Result<SessionData> {
        let path = Self::session_path();
        if !path.exists() {
            return Ok(SessionData::default());
        }
        let json = std::fs::read_to_string(path)?;
        let data = serde_json::from_str(&json)?;
        Ok(data)
    }
}
