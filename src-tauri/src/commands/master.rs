use std::net::UdpSocket;
use std::sync::Mutex;

use std::time::Duration;
use tauri::{AppHandle, Manager};

use crate::config::SargeLauncher;
use crate::master::MasterServer;
use crate::server::Quake3Server;

#[tauri::command(async)]
pub async fn get_q3_server_ips(app: AppHandle, q3_protocol: u8) -> Result<Vec<Quake3Server>, tauri::Error> {
	let mut servers_on_master: Vec<Quake3Server> = vec![];
	let active_masters: Vec<MasterServer> = get_active_masters(&app);

	for mut master in active_masters {
		let socket = UdpSocket::bind("0.0.0.0:0")?;
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

		send_to_master(&socket, &master.game, &q3_protocol)?;
		let protocol_used = get_used_protocol(&master.game, &q3_protocol);

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

	Ok(servers_on_master)
}

fn get_active_masters(app: &AppHandle) -> Vec<MasterServer> {
	let mut active_masters: Vec<MasterServer> = vec![];

	let state = app.state::<Mutex<SargeLauncher>>();
	let state = state.lock().unwrap();

	if let Some(app_data) = &mut *state.app_data.lock().unwrap() {
		for m in &app_data.masters {
			if m.active {
				active_masters.push(m.clone());
			}
		}
	} else {
		active_masters = Vec::from_iter(MasterServer::initial_masters());
	}

	active_masters
}

fn send_to_master(socket: &UdpSocket, game: &String, q3_protocol: &u8) -> Result<(), tauri::Error> {
	match game.as_str() {
		"Quake 3" => match q3_protocol {
			68 => socket.send(b"\xff\xff\xff\xffgetservers 68 full empty")?,
			43 => socket.send(b"\xff\xff\xff\xffgetservers 43 full empty")?,
			_ => socket.send(b"\xff\xff\xff\xffgetservers 68 full empty")?,
		},
		"Urban Terror" => socket.send(b"\xff\xff\xff\xffgetservers 68 full empty")?,
		"OpenArena" => socket.send(b"\xff\xff\xff\xffgetservers 71 full empty")?,
		"Bloodrun" => socket.send(b"\xff\xff\xff\xffgetservers Quake3Champions 114 full empty")?,
		_ => socket.send(b"\xff\xff\xff\xffgetservers 68 full empty")?,
	};

	Ok(())
}

fn get_used_protocol(game: &String, q3_protocol: &u8) -> u8 {
	let used_protocol: u8;

	match game.as_str() {
		"Quake 3" => used_protocol = q3_protocol.to_owned(),
		"Urban Terror" => used_protocol = 68,
		"OpenArena" => used_protocol = 71,
		"Bloodrun" => used_protocol = 114,
		_ => used_protocol = 68,
	};

	used_protocol
}
