use serde::{Deserialize, Serialize};
use tauri_plugin_dialog::FilePath;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Q3Executable {
    pub name: String,
    pub path: FilePath,
    pub active: bool,
}

impl Q3Executable {
    pub fn new(name: String, path: FilePath, active: bool) -> Self {
        Self {
            name: name,
            path: path,
            active: active,
        }
    }
}
