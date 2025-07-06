use serde::{Deserialize, Serialize};
use std::str;

use crate::server::Quake3Server;

#[derive(Debug, Default, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Clone)]
pub struct MasterServer {
	pub name: String,
	pub address: String,
	pub game: String,
	pub active: bool,
	pub unreachable: bool,
}

impl MasterServer {
	pub fn parse_master_response(
		response: &[u8],
		master: &MasterServer,
		servers_so_far: &Vec<Quake3Server>,
		protocol: &u8,
	) -> Vec<Quake3Server> {
		let mut parsed_master_servers: Vec<Quake3Server> = vec![];

		let mut index: usize = 0;

		if &response[index..22] == b"\xff\xff\xff\xffgetserversResponse" {
			index += 23;
		} else {
			return parsed_master_servers;
		}

		while (index + 5) < response.len() {
			let ip = format!(
				"{0}.{1}.{2}.{3}",
				&response[index],
				&response[index + 1],
				&response[index + 2],
				&response[index + 3]
			);
			let port = u16::from_be_bytes([response[index + 4], response[index + 5]]).to_string();

			if port == "0" {
				break;
			}

			let parsed = Quake3Server::new(ip, port, Some(master.to_owned()), Some(protocol.to_owned()));

			if !servers_so_far.iter().any(|serv| serv.address == parsed.address) {
				parsed_master_servers.push(parsed);
			}

			index += 7;
		}

		parsed_master_servers
	}

	pub fn initial_masters() -> Vec<MasterServer> {
		let initial: Vec<MasterServer> = vec![
			MasterServer {
				name: String::from("master.quake3arena.com"),
				address: String::from("master.quake3arena.com:27950"),
				game: String::from("Quake 3"),
				active: true,
				unreachable: false,
			},
			MasterServer {
				name: String::from("master.ioquake3.org"),
				address: String::from("master.ioquake3.org:27950"),
				game: String::from("Quake 3"),
				active: true,
				unreachable: false,
			},
			MasterServer {
				name: String::from("dpmaster.deathmask.net"),
				address: String::from("dpmaster.deathmask.net:27950"),
				game: String::from("Quake 3"),
				active: true,
				unreachable: false,
			},
			MasterServer {
				name: String::from("master.urbanterror.info"),
				address: String::from("master.urbanterror.info:27900"),
				game: String::from("Urban Terror"),
				active: true,
				unreachable: false,
			},
			MasterServer {
				name: String::from("master.ioquake3.org"),
				address: String::from("master.ioquake3.org:27950"),
				game: String::from("OpenArena"),
				active: true,
				unreachable: false,
			},
			MasterServer {
				name: String::from("dpmaster.deathmask.net"),
				address: String::from("dpmaster.deathmask.net:27950"),
				game: String::from("OpenArena"),
				active: true,
				unreachable: false,
			},
			MasterServer {
				name: String::from("bloodrun master"), // "getservers Quake3Champions 108 full empty"
				address: String::from("164.90.203.227:27950"),
				game: String::from("Bloodrun"),
				active: true,
				unreachable: false,
			},
		];

		initial
	}
}
