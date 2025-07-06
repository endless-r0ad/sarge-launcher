use std::net::UdpSocket;
use std::sync::Mutex;

use std::time::Duration;
use tauri::{AppHandle, Manager};

use crate::config::Q3Browser;
use crate::master::MasterServer;
use crate::server::Quake3Server;

#[tauri::command(async)]
pub async fn get_q3_server_ips(app: AppHandle, protocol: u8) -> Result<Vec<Quake3Server>, String> {
	let mut servers_on_master: Vec<Quake3Server> = vec![];

	let state = app.state::<Mutex<Q3Browser>>();
	let state = state.lock().unwrap();
	let mut protocol_used = protocol;

	let mut active_masters: Vec<MasterServer> = vec![];

	if let Some(state) = &mut *state.app_data.lock().unwrap() {
		for m in &state.masters {
			if m.active {
				active_masters.push(m.clone());
			}
		}
	} else {
		active_masters = Vec::from_iter(MasterServer::initial_masters());
	}

	drop(state);

	for mut master in active_masters {
		let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
		let _ = socket.set_read_timeout(Some(Duration::from_millis(300)));

		let mut receive_next = true;
		let mut last_bytes = 0;

		let connection = socket.connect(&master.address);

		match connection {
			Ok(()) => connection.unwrap(),
			Err(_err) => {
				master.unreachable = true;
				continue;
			}
		}

		// 'xxxxgetservers {optional: challenge / game} 68 full empty'
		match master.game.as_str() {
			"Quake 3" => match protocol {
				68 => socket.send(b"\xff\xff\xff\xffgetservers 68 full empty").unwrap(),
				43 => socket.send(b"\xff\xff\xff\xffgetservers 43 full empty").unwrap(),
				_ => socket.send(b"\xff\xff\xff\xffgetservers 68 full empty").unwrap(),
			},
			"Urban Terror" => {
				protocol_used = 68;
				socket.send(b"\xff\xff\xff\xffgetservers 68 full empty").unwrap()
			}
			"OpenArena" => {
				protocol_used = 71;
				socket.send(b"\xff\xff\xff\xffgetservers 71 full empty").unwrap()
			}
			"Bloodrun" => {
				protocol_used = 108;
				socket.send(b"\xff\xff\xff\xffgetservers Quake3Champions 108 full empty").unwrap()
			}
			_ => socket.send(b"\xff\xff\xff\xffgetservers 68 full empty").unwrap(),
		};

		while receive_next {
			let mut buf: [u8; 65507] = [0; 65507];

			match socket.recv_from(&mut buf) {
				Ok((bytes, _src)) => {
					let bytes_slice = &buf[..(bytes)];
					let parsed = &mut MasterServer::parse_master_response(bytes_slice, &master, &servers_on_master, &protocol_used);

					servers_on_master.append(parsed);

					receive_next = bytes >= last_bytes;
					last_bytes = bytes;
				}
				Err(_err) => {
					receive_next = false;
				}
			}
		}
	}

	Ok(servers_on_master.into())
}
