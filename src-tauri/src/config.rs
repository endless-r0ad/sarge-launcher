use serde::{Deserialize, Serialize};
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use std::collections::HashSet;
use std::process::Child;

use crate::client::Q3Executable;
use crate::master::{self, MasterServer};

pub struct SargeLauncher {
	pub client: Mutex<Option<Child>>,
	pub config: Mutex<Option<Config>>,
	pub app_data: Mutex<Option<AppData>>,
}

impl Default for SargeLauncher {
	fn default() -> Self {
		SargeLauncher {
			client: Mutex::new(None),
			config: Mutex::new(None),
			app_data: Mutex::new(None),
		}
	}
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
	path: String,
	welcome_message: bool,
	server_browser_threads: usize,
	server_timeout: u16,
	show_unreachable: bool,
	manage_q3_instance: bool,
    refresh_by_mod: bool,
	show_trashed_servers: bool,
    autoclose_demo: bool,
    loop_demo: bool,
	q3_clients: Vec<Q3Executable>
}

impl Config {
	pub fn new(path: String) -> Self {
		Self {
			path: path,
			welcome_message: true,
			server_browser_threads: 50,
			server_timeout: 300,
			show_unreachable: false,
			manage_q3_instance: true,
            refresh_by_mod: false,
			show_trashed_servers: true,
            autoclose_demo: true,
            loop_demo: false,
			q3_clients: Vec::<Q3Executable>::new(),
		}
	}

    pub fn write_to_file(&self, path: &PathBuf) -> Result<(), tauri::Error> {
        let mut file = File::create(&path)?;
        let updated_app_data_string = serde_json::to_string_pretty(&self)?;
        file.write_all(updated_app_data_string.as_bytes())?;

        Ok(())
    }

    pub fn read_from_file(config_path: &PathBuf) -> Result<Self, tauri::Error> {
        let config_str = read_to_string(config_path)?;

        let config_json = serde_json::from_str(&config_str);

        let config: Config = match config_json {
            Ok(config) => config,
            Err(_e) => {
                std::fs::remove_file(config_path)?;
                let new_config = Config::new(config_path.to_str().unwrap().to_string());
                new_config.write_to_file(&config_path)?;
                return Ok(new_config);
            }
        };
        return Ok(config);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppData {
	pub path: String,
	pub pinned: HashSet<String>,
	pub custom: HashSet<String>,
	pub trash: HashSet<String>,
	pub trash_ip: HashSet<String>,
	pub server_password: String,
	pub masters: Vec<MasterServer>,
}

impl AppData {
	pub fn new(path: String) -> Self {
		Self {
			path: path,
			pinned: HashSet::new(),
			custom: HashSet::new(),
			trash: HashSet::new(),
			trash_ip: HashSet::new(),
			masters: master::MasterServer::initial_masters(),
			server_password: String::from(""),
		}
	}

    pub fn write_to_file(&self, path: &PathBuf) -> Result<(), tauri::Error> {
        let mut file = File::create(&path)?;
        let updated_app_data_string = serde_json::to_string_pretty(&self)?;
        file.write_all(updated_app_data_string.as_bytes())?;

        Ok(())
    }

    pub fn read_from_file(data_dir: &PathBuf) -> Result<Self, tauri::Error> {
        let app_data_str = read_to_string(data_dir)?;

        let app_data_json = serde_json::from_str(&app_data_str);

        let app_data: AppData = match app_data_json {
            Ok(app_data) => app_data,
            Err(_e) => {
                std::fs::remove_file(data_dir)?;
                let new_app_data = AppData::new(data_dir.to_str().unwrap().to_string());
                new_app_data.write_to_file(&data_dir)?;
                return Ok(new_app_data);
            }
        };
        return Ok(app_data);
    }
}
