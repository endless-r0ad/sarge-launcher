use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Q3Executable {
	pub name: String,
	pub exe_path: String,
	pub parent_path: String,
	pub gamename: String,
    pub extra_launch_args: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Q3Config {
	pub name: String,
	pub path: String
}

impl Q3Executable {
    pub fn game_uses_baseq3_paths(&self) -> bool {
        match self.gamename.to_lowercase().as_str() {
            "baseq3" => return true,
            "cpma" => return true, 
            "defrag" => return true, 
            "excessiveplus" => return true, 
            "osp" => return true,
            _ => return false
        }
    }
    pub fn game_uses_baseoa_paths(&self) -> bool {
        match self.gamename.to_lowercase().as_str() {
            "baseoa" => return true,
            "rat" => return true, 
            "missionpack" => return true, 
            _ => return false
        }
    }
}