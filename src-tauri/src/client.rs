use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Q3Executable {
	pub name: String,
	pub exe_path: String,
    pub parent_path: String,
    pub gamename: String
}


impl Q3Executable {
    pub fn set_gamename(&mut self) {
        match self.name.to_lowercase().as_str() {
            "quake3e" | "quake3e-vulkan" | "quake3" | "ioquake3" => self.gamename = String::from("baseq3"),
            "quake3-urt" => self.gamename = String::from("q3ut4"),
            "odfe" | "odfe.vk" | "idfe" => self.gamename = String::from("defrag"),
            "cnq3-x64" | "cnq3-x86" => self.gamename = String::from("cpma"),
            "openarena" | "omega-x64" => self.gamename = String::from("baseoa"),
            "bloodrun" => self.gamename = String::from("base"),
            _ => self.gamename = String::from("baseq3")
        }
    }
}