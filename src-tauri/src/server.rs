use serde::{Deserialize, Serialize};

use crate::master::MasterServer;
use crate::q3_util;
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ServerPlayer {
	name: String,
	namecolored: String,
	frags: i32,
	ping: i32,
}

impl ServerPlayer {
	fn new(name: String, frags: String, ping: String) -> Self {
		let parsed_name = q3_util::parse_colorstring(name);
		Self {
			name: parsed_name.0,
			namecolored: parsed_name.1,
			frags: frags.parse::<i32>().unwrap(),
			ping: ping.parse::<i32>().unwrap(),
		}
	}
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Quake3Server {
	pub master: Option<MasterServer>,
	pub ip: String,
	pub port: String,
	pub address: String,
	pub protocol: Option<u8>,
	pub ping: u16,
	pub errormessage: String,
	pub host: String,
	pub hostcolored: String,
	pub game: String,
	pub playersconnected: u8,
	pub maxclients: String,
	pub bots: u8,
	pub map: String,
	pub othersettings: HashMap<String, String>,
	pub players: Option<Vec<ServerPlayer>>,
	pub list: String,
	pub custom: bool,
	pub version: String,
}

impl Quake3Server {
	pub fn new(ip: String, port: String, master: Option<MasterServer>, protocol: Option<u8>) -> Quake3Server {
		Quake3Server {
			master,
			ip: ip.clone(),
			port: port.clone(),
			address: format!("{}:{}", ip, port),
			protocol: protocol,
			ping: 0,
			errormessage: String::from(""),
			host: String::new(),
			hostcolored: String::new(),
			game: String::new(),
			playersconnected: 0,
			maxclients: String::from("0"),
			bots: 0,
			map: String::new(),
			othersettings: HashMap::new(),
			players: None,
			list: String::from("main"),
			custom: false,
			version: String::from(""),
		}
	}

	pub fn parse_status_response(&mut self, &response_buf: &[u8; 2400]) -> Result<(), tauri::Error> {
		let mut index = 0;

		if &response_buf[index..(index + 19)] != b"\xff\xff\xff\xffstatusResponse\n" {
            return Err(tauri::Error::FailedToReceiveMessage)
		}

		index += 20;

		let mut reverse_response = response_buf;
		reverse_response.reverse();

        let resp_end = reverse_response.iter().position(|&r| r == 0x0a);

        if resp_end.is_none() {
            return Err(tauri::Error::AssetNotFound(String::from("status response end")))
        }

		let end_index: usize = response_buf.len() - resp_end.unwrap();

        let last_gamestate_end = reverse_response.iter().position(|&r| r == 0x5c);

        if last_gamestate_end.is_none() {
            return Err(tauri::Error::AssetNotFound(String::from("last gamestate delimiter")))
        }

		let last_gamestate_delimiter = response_buf.len() - last_gamestate_end.unwrap();

		let last_gamestate_len= response_buf[last_gamestate_delimiter..].iter().position(|&r| r == 0x0a);

        if last_gamestate_len.is_none() {
            return Err(tauri::Error::AssetNotFound(String::from("last gamestate length")))
        }

		let gamestate = String::from_utf8_lossy(&response_buf[index..last_gamestate_delimiter + last_gamestate_len.unwrap()]);
		let gamestate_list: Vec<&str> = gamestate.split("\\").collect();

		for i in (0..gamestate_list.len()).step_by(2) {
			match gamestate_list[i] {
				"sv_hostname" => {
					let parsed_host = q3_util::parse_colorstring(gamestate_list[i + 1].to_owned());
					self.host = parsed_host.0;
					self.hostcolored = parsed_host.1;
				}
				"version" => {
					self.version = gamestate_list[i + 1].to_owned();
				}
				"gamename" => {
					self.game = if self.protocol == Some(114) { String::from("base") } else { gamestate_list[i + 1].to_owned() } 
				}
				"sv_maxclients" => {
					self.maxclients = gamestate_list[i + 1].to_owned();
				}
				"mapname" => {
					self.map = gamestate_list[i + 1].to_owned();
				}
				_ => {
					self.othersettings.entry(gamestate_list[i].to_owned()).or_insert(gamestate_list[i + 1].to_owned());
				}
			}
		}

		index = last_gamestate_delimiter + last_gamestate_len.unwrap();

		if index == end_index - 1 {
			return Ok(());
		}

		let players = String::from_utf8_lossy(&response_buf[index + 1..end_index - 1]);
		let player_list: Vec<&str> = players.split("\n").collect();

		for player in player_list {
			let p: Vec<&str> = player.split(" ").collect();
			let name: String;

			if p.len() > 3 {
				name = p[2..].join(" ").to_string();
			} else {
				name = p[2].to_string();
			}

			let new_player = ServerPlayer::new(name[1..name.len() - 1].to_owned(), p[0].to_owned(), p[1].to_owned());

			if new_player.ping == 0 {
				self.bots += 1;
			} else {
				self.playersconnected += 1;
			}

			if let Some(player_vec) = self.players.as_mut() {
				player_vec.push(new_player);
			} else {
				let _ = self.players.insert(vec![new_player]);
			}
		}

		Ok(())
	}

	pub fn set_error(&mut self, err: std::io::Error) -> () {
		self.ping = 999;
		self.errormessage = err.to_string();
		self.host = err.to_string();
		self.hostcolored = err.to_string();
		self.game = String::from("unreachable");
		self.map = String::from("unreachable");
	}
}
