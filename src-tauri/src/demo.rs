use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::path::Path;
use std::vec;

use crate::huffman_node::Node;
use crate::q3_util;

const MAX_MESSAGE_LENGTH: i32 = 16384;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Demo {
	pub file_name: String,
	pub path: String,
	pub protocol: u8,
	pub gamename: String,
	pub mapname: String,
	pub g_gametype: String,
	pub player_pov: DemoPlayer,
	pub snapshots: usize,
	pub sv_hostname: String,
	pub sv_hostname_color: String,
	pub players: Option<Vec<DemoPlayer>>,
	pub server_info: HashMap<String, String>,
	pub system_info: HashMap<String, String>,
	pub server_commands: HashMap<i32, String>,
	pub duration: usize,
	pub df_time: f32,
	pub issue: Option<String>,
	pub version: String,
}

impl PartialEq for Demo {
    fn eq(&self, other: &Self) -> bool {
        self.file_name == other.file_name && self.path == other.path
    }
}

impl Eq for Demo {} // Marker trait

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct DemoPlayer {
	name: String,
	namecolored: String,
}

impl DemoPlayer {
	fn new() -> Self {
		Self {
			name: String::from(""),
			namecolored: String::from(""),
		}
	}
}

impl Demo {
	pub fn new(file_name: String, path: String, protocol: u8) -> Self {
		Self {
			file_name,
			path,
			protocol,
			gamename: String::from(""),
			mapname: String::from(""),
			g_gametype: String::from(""),
			player_pov: DemoPlayer::new(),
			snapshots: 0,
			sv_hostname: String::from(""),
			sv_hostname_color: String::from(""),
			players: None,
			server_info: HashMap::new(),
			system_info: HashMap::new(),
			server_commands: HashMap::new(),
			duration: 0,
			df_time: 0.0,
			issue: None,
			version: String::from(""),
		}
	}

	pub async fn get_q3_demos(dir: &Path, cache: &HashMap<String, String>) -> Result<Vec<Demo>, std::io::Error> {
		let mut demos: Vec<Demo> = vec![];

		for entry in std::fs::read_dir(dir)? {
			let entry = entry?;
			let path = entry.path();

			if path.is_dir() {
				demos.append(&mut Box::pin(Self::get_q3_demos(&path, cache)).await?);
			}

            let demo_p = path.to_str().unwrap();

            if cache.contains_key(demo_p) {
                continue;
            }

			if let Some(ext) = path.extension() {
                let ext_s = ext.to_string_lossy().to_string();
                let mut protocol: u8 = 0;

                if !ext_s.starts_with("dm_") && ext_s != "urtdemo" {
                    continue;
                }

                if ext_s.starts_with("dm_") {
                    let p = ext_s.as_str()[3..].parse::<u8>();
                    if p.is_err() {
                        continue;
                    }
                    protocol = p.unwrap();
                }

                let demo = Demo::new(
                    path.file_stem().unwrap().to_str().unwrap().to_string(),
                    demo_p.to_string(),
                    protocol,
                );

                demos.push(demo);
			}
		}
		Ok(demos)
	}

	pub fn parse_demo(&mut self, huffman_tree: [Node; 514], get_all_data: bool) -> Result<(), std::io::Error> {
		let mut demo_bytes = std::fs::read(Path::new(&self.path))?;
		let demo_length_bits = demo_bytes.len() * 8;

		let mut bit_position: usize = 0;
		let mut byte_pos: usize = 0;
		let mut msg_length: i32;
		let mut msg_start_pos: usize;
		let mut servertime_start: i32 = 0;
		let mut last_snapshot_position: usize = 0;

		'msg: loop {
			let seq_bytes: [u8; 4] = [
				demo_bytes[byte_pos],
				demo_bytes[byte_pos + 1],
				demo_bytes[byte_pos + 2],
				demo_bytes[byte_pos + 3],
			];
			let _sequence = i32::from_le_bytes(seq_bytes);

			let msg_bytes: [u8; 4] = [
				demo_bytes[byte_pos + 4],
				demo_bytes[byte_pos + 5],
				demo_bytes[byte_pos + 6],
				demo_bytes[byte_pos + 7],
			];
			msg_length = i32::from_le_bytes(msg_bytes);

			bit_position += 64;
			msg_start_pos = bit_position;

			if msg_length < 0 {
				break;
			}

			if msg_length > MAX_MESSAGE_LENGTH {
				self.issue = Some(String::from("Error: demoMsglen > MAX_MSGLEN"));
				self.finish_bad_demo();
				break;
			}

			let msg_length_bits = msg_length * 8;

			if bit_position + msg_length_bits as usize + 64 > demo_length_bits {
				self.issue = Some(String::from("Demo file was truncated"));
				self.finish_bad_demo();
				break;
			}

			let _msg_ack = Self::huffman_readbits(32, &mut demo_bytes, huffman_tree, &mut bit_position);

			loop {
				if bit_position > msg_start_pos + msg_length_bits as usize {
					self.issue = Some(String::from("Error: read past end of server message"));
					self.finish_bad_demo();
					break 'msg;
				}

				let cmd = Self::huffman_read(&mut demo_bytes, huffman_tree, &mut bit_position);

				match cmd {
					8 => break,
					2 => {
						let mut gamestate =
							self.load_gamestate(&mut demo_bytes, huffman_tree, &mut bit_position, msg_length, &mut msg_start_pos);
						self.parse_gamestate(&mut gamestate);

                        if !get_all_data {
                            return Ok(());
                        }

						break;
					}
					7 => {
						self.snapshots += 1;
						last_snapshot_position = bit_position;

						if self.snapshots == 1 {
							servertime_start = Self::huffman_readbits(32, &mut demo_bytes, huffman_tree, &mut bit_position);
						}

						Self::skip_rest_of_message(&mut bit_position, msg_length, &mut msg_start_pos);
						break;
					}
					5 => {
						let index = Self::huffman_readbits(32, &mut demo_bytes, huffman_tree, &mut bit_position);
						let mut value = Self::huffman_readstring(&mut demo_bytes, huffman_tree, &mut bit_position);

						match &value {
							x if x.starts_with(&"print \"") || x.starts_with(&"tchat \"") => {
								let parsed = q3_util::parse_colorstring(&x[7..x.len() - 1]);
								value = parsed.1;
							}
							x if x.starts_with(&"chat \"") || x.starts_with(&"cpsm \"") => {
								let parsed = q3_util::parse_colorstring(&x[6..x.len() - 1]);
								value = parsed.1;
							}
							x if x.starts_with(&"cp \"") => {
								let parsed = q3_util::parse_colorstring(&x[4..x.len() - 1]);
								value = parsed.1;
							}
							x if x.starts_with(&"pcp \"") => {
								let parsed = q3_util::parse_colorstring(&x[5..x.len() - 1]);
								value = parsed.1;
							}
							x if x.starts_with(&"cwhisper \"") => {
								let parsed = q3_util::parse_colorstring(&x[10..x.len() - 1]);
								value = parsed.1;
							}
							_ => continue, // ()
						}
						self.server_commands.entry(index).or_insert(value);
					}
					_ => {
						self.issue = Some(String::from("Illegible server message"));
						self.finish_bad_demo();
						break 'msg;
					}
				}
			}

            // realignment (1 in 10k demos needed this?)
			if bit_position != msg_start_pos + msg_length_bits as usize {
				bit_position = msg_start_pos + msg_length_bits as usize;
			}

			byte_pos = bit_position / 8;
		}

		let servertime_end = Self::huffman_readbits(32, &mut demo_bytes, huffman_tree, &mut last_snapshot_position);
		self.duration = (servertime_end - servertime_start) as usize;

		Ok(())
	}

	pub fn bitbuffer_read(msg: &mut Vec<u8>, bit_position: &mut usize) -> i32 {
		let bit_offset = *bit_position & 7;
		let byte_offset = *bit_position / 8;

		let result = (&msg[byte_offset] >> bit_offset) as i32;

		*bit_position += 1;

		result & 1
	}

	pub fn huffman_read(msg: &mut Vec<u8>, tree: [Node; 514], bit_position: &mut usize) -> i32 {
		let mut node = tree[2];

		while node.value == 257 {
			if Self::bitbuffer_read(msg, bit_position) == 0 {
				node = tree[node.left];
			} else {
				node = tree[node.right];
			}
		}

		return node.value as i32;
	}

	pub fn huffman_readbits(bits: i32, msg: &mut Vec<u8>, tree: [Node; 514], bit_position: &mut usize) -> i32 {
		let mut result = 0;

		let bit_count = bits & 7;

		if bit_count > 0 {
			for i in 0..bit_count {
				result |= Self::bitbuffer_read(msg, bit_position) << i
			}
		}

		let byte_count = bits / 8;

		if byte_count > 0 {
			for i in 0..byte_count {
				let bit_offset = (i * 8) + bit_count;
				result |= Self::huffman_read(msg, tree, bit_position) << bit_offset;
			}
		}

		result
	}

	pub fn load_gamestate(
		&mut self,
		msg: &mut Vec<u8>,
		tree: [Node; 514],
		bit_position: &mut usize,
		msg_length: i32,
		msg_start_pos: &mut usize,
	) -> HashMap<i32, String> {
		let mut gamestate: HashMap<i32, String> = Default::default();
		let _sequence = Self::huffman_readbits(32, msg, tree, bit_position);
		let mut cmd = Self::huffman_read(msg, tree, bit_position);

		while cmd != 8 {
			if cmd == 3 {
				let index = Self::huffman_readbits(16, msg, tree, bit_position);
				let config_string = Self::huffman_readstring(msg, tree, bit_position);
				gamestate.entry(index).or_insert(config_string.clone());

				if config_string.starts_with("n\\") {
					let mut new_player = DemoPlayer::new();
					let player_stuff: Vec<&str> = config_string.split("\\").collect();
					let parsed = q3_util::parse_colorstring(player_stuff[1]);
					new_player.namecolored = parsed.1;
					new_player.name = parsed.0;

					if let Some(player_vec) = self.players.as_mut() {
						player_vec.push(new_player);
					} else {
						self.player_pov = new_player.clone();
						let _ = self.players.insert(vec![new_player]);
					}
				}
			}
			if cmd == 4 {
				Self::skip_rest_of_message(bit_position, msg_length, msg_start_pos);
				break;
			}
			cmd = Self::huffman_read(msg, tree, bit_position);
		}

		gamestate
	}

	pub fn huffman_readstring(msg: &mut Vec<u8>, tree: [Node; 514], bit_position: &mut usize) -> String {
		let mut string_bytes = vec![];

		let mut next_byte = Self::huffman_read(msg, tree, bit_position);

		while next_byte != 0 {
			string_bytes.push(next_byte as u8);
			next_byte = Self::huffman_read(msg, tree, bit_position);
		}

		return String::from_utf8(string_bytes).unwrap_or(String::from(""));
	}

	pub fn skip_rest_of_message(bit_position: &mut usize, msg_length: i32, msg_start_pos: &mut usize) -> () {
		let msg_bits_parsed = *bit_position - *msg_start_pos;
		let remaining_message = ((msg_length * 8) as usize) - msg_bits_parsed;
		*bit_position += remaining_message;
	}

	pub fn parse_gamestate(&mut self, gamestate: &mut HashMap<i32, String>) -> () {
		let mut server_stuff: Vec<&str> = gamestate.get(&0).unwrap().split("\\").collect();
		let mut system_stuff: Vec<&str> = gamestate.get(&1).unwrap().split("\\").collect();

		server_stuff.remove(0);
		system_stuff.remove(0);

		if server_stuff[server_stuff.len() - 1] == "" && server_stuff.len() % 2 != 0 {
			server_stuff.remove(server_stuff.len() - 1);
		}

		if system_stuff[system_stuff.len() - 1] == "" && system_stuff.len() % 2 != 0 {
			system_stuff.remove(system_stuff.len() - 1);
		}

		for i in (0..server_stuff.len()).step_by(2) {
			match server_stuff[i] {
				"sv_hostname" => {
					let parsed_host = q3_util::parse_colorstring(server_stuff[i + 1]);
					self.sv_hostname = parsed_host.0;
					self.sv_hostname_color = parsed_host.1;
				}
				"gamename" => {
					self.gamename = server_stuff[i + 1].to_owned();
				}
				"g_gametype" => {
					self.g_gametype = server_stuff[i + 1].to_owned();
				}
				"mapname" => {
					self.mapname = server_stuff[i + 1].to_owned();
				}
				"version" => {
					self.version = server_stuff[i + 1].to_owned();
				}
				_ => {
					self.server_info.entry(server_stuff[i].to_string()).or_insert(server_stuff[i + 1].to_string());
				}
			}
		}

		if self.gamename == "" {
			self.gamename = String::from("unknown");
		}

		for i in (0..system_stuff.len()).step_by(2) {
			self.system_info.entry(system_stuff[i].to_string()).or_insert(system_stuff[i + 1].to_string());
		}
	}

	pub fn finish_bad_demo(&mut self) -> () {
		self.server_info.entry(String::from("gamename")).or_insert(String::from("???"));
		self.server_info.entry(String::from("g_gametype")).or_insert(String::from("???"));
		self.server_info.entry(String::from("mapname")).or_insert(String::from("???"));

		if self.player_pov.name == "" {
			self.player_pov.name = String::from("???");
			self.player_pov.namecolored = String::from("???");
		}
	}
}
