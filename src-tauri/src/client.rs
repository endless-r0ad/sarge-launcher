use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Q3Executable {
	pub name: String,
	pub exe_path: String,
    pub parent_path: String
}

