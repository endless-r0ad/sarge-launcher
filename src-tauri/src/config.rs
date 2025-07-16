use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use std::collections::HashSet;
use std::process::Child;

use crate::client::Q3Executable;
use crate::master::{self, MasterServer};

pub struct Q3Browser {
	pub client: Mutex<Option<Child>>,
	pub config: Mutex<Option<Config>>,
	pub app_data: Mutex<Option<AppData>>,
}

impl Default for Q3Browser {
	fn default() -> Self {
		Q3Browser {
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
	play_gif: bool,
	server_browser_threads: usize,
	server_timeout: u16,
	show_unreachable: bool,
	manage_q3_instance: bool,
	show_trashed_servers: bool,
	demo_path: Option<String>,
	fs_homepath: Option<String>,
	q3_clients: Vec<Q3Executable>
}

impl Config {
	pub fn new(path: String) -> Self {
		Self {
			path: path,
			welcome_message: true,
			play_gif: true,
			server_browser_threads: 50,
			server_timeout: 400,
			show_unreachable: false,
			manage_q3_instance: true,
			show_trashed_servers: true,
			demo_path: None,
			fs_homepath: None,
			q3_clients: Vec::<Q3Executable>::new(),
		}
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
}
