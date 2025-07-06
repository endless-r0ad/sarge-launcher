use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::time::Instant;

use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

use crate::config::{AppData, Q3Browser};
use crate::server::Quake3Server;

const GETSTATUS: &[u8] = b"\xff\xff\xff\xffgetstatus\x00";

#[tauri::command(async)]
pub async fn refresh_all_servers(
	app: AppHandle,
	mut all_servers: Vec<Quake3Server>,
	num_threads: usize,
	timeout: u64,
) -> Result<Vec<Quake3Server>, String> {
	let state = app.state::<Mutex<Q3Browser>>();
	let state = state.lock().unwrap();

	if let Some(state) = &mut *state.app_data.lock().unwrap() {
		get_user_servers(&mut all_servers, state);
		set_server_list(&mut all_servers, state);
	}

	drop(state);

	let all_servers_arc = Arc::new(all_servers);
	let refreshed_servers_arc: Arc<Mutex<Vec<Quake3Server>>> = Arc::new(Mutex::new(vec![]));

	let mut handles = vec![];

	for t in 0..num_threads {
		let all_servers_arc = Arc::clone(&all_servers_arc);
		let refreshed = Arc::clone(&refreshed_servers_arc);

		let servs_per_thread = (&all_servers_arc.len() / num_threads) + 1;
		let mut remaining: isize = servs_per_thread as isize;

		if ((servs_per_thread * t) + servs_per_thread) > all_servers_arc.len() {
			remaining = (all_servers_arc.len() as isize) - ((servs_per_thread * t) as isize);

			if remaining <= 0 {
				break;
			}
		}

		handles.push(thread::spawn(move || {
			let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
			let servers_slice = &all_servers_arc[(servs_per_thread * t)..((servs_per_thread * t) + remaining as usize)];
			let mut serv_counter = 0;

			while serv_counter < servers_slice.len() {
				let mut queried_server: Quake3Server = servers_slice[serv_counter].to_owned();
				let _ = socket.set_read_timeout(Some(Duration::from_millis(timeout)));

				let ping_start = Instant::now();

				let x = socket.send_to(GETSTATUS, queried_server.address.to_owned()).unwrap_or_else(|err| {
					queried_server.set_error(err);
					return 0;
				});

				if x == 0 {
					refreshed.lock().unwrap().push(queried_server);
					serv_counter += 1;
					continue;
				}

				let mut response_buf: [u8; 2400] = [0; 2400];

				let response = socket.recv_from(&mut response_buf);

				match response {
					Ok((_bytes, _src)) => {
						queried_server.ping = ping_start.elapsed().as_millis() as u16;
						queried_server
							.parse_server_response(&response_buf)
							.unwrap_or_else(|e| queried_server.errormessage = e);
						refreshed.lock().unwrap().push(queried_server);
					}
					Err(err) => {
						queried_server.set_error(err);
						refreshed.lock().unwrap().push(queried_server);
					}
				}
				serv_counter += 1;
			}
		}));
	}

	for handle in handles {
		handle.join().unwrap();
	}

	let q3_servers = refreshed_servers_arc.lock().unwrap().clone();

	Ok(q3_servers)
}

#[tauri::command(async)]
pub async fn refresh_single_server(mut refresh_server: Quake3Server, timeout: u64, window: WebviewWindow) -> Result<(), String> {
	refresh_server.players = None;
	refresh_server.playersconnected = 0;
	refresh_server.bots = 0;
	refresh_server.errormessage = String::from("");

	let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

	let _ = socket.set_read_timeout(Some(Duration::from_millis(timeout)));

	let ping_start = Instant::now();

	let x = socket.send_to(GETSTATUS, refresh_server.address.to_string()).unwrap_or_else(|err| {
		refresh_server.set_error(err);
		return 0;
	});

	if x == 0 {
		let _ = window.emit("q3_single_server", refresh_server).unwrap();
		return Ok(());
	}

	let mut response_buf: [u8; 2400] = [0; 2400];

	let response = socket.recv_from(&mut response_buf);

	match response {
		Ok((_bytes, _src)) => {
			refresh_server.ping = ping_start.elapsed().as_millis() as u16;

			refresh_server
				.parse_server_response(&response_buf)
				.unwrap_or_else(|e| refresh_server.errormessage = e);
			let _ = window.emit("q3_single_server", refresh_server).unwrap();
		}
		Err(err) => {
			refresh_server.set_error(err);
			let _ = window.emit("q3_single_server", refresh_server).unwrap();
		}
	}

	Ok(())
}

fn get_user_servers(servers: &mut Vec<Quake3Server>, app_data: &AppData) -> () {
	let all_servers_addresses: Vec<String> = servers.iter().map(|x| x.address.clone()).collect();

	for serv in &app_data.custom {
		if !all_servers_addresses.contains(serv) {
			let ip_port: Vec<&str> = serv.as_str().split(":").collect();
			let mut custom_server = Quake3Server::new(ip_port[0].to_string(), ip_port[1].to_string(), None, None);
			custom_server.list = String::from("pinned");
			custom_server.custom = true;
			servers.push(custom_server);
		}
	}
	for serv in &app_data.pinned {
		if !all_servers_addresses.contains(serv) {
			let ip_port: Vec<&str> = serv.as_str().split(":").collect();
			let mut pinned_server = Quake3Server::new(ip_port[0].to_string(), ip_port[1].to_string(), None, None);
			pinned_server.list = String::from("pinned");
			servers.push(pinned_server);
		}
	}
}

fn set_server_list(servers: &mut Vec<Quake3Server>, app_data: &AppData) -> () {
	for server in servers {
		if app_data.pinned.contains(server.address.as_str()) {
			server.list = String::from("pinned");
		}
		if app_data.custom.contains(server.address.as_str()) {
			server.list = String::from("pinned");
			server.custom = true;
		}
		if app_data.trash.contains(server.address.as_str()) {
			server.list = String::from("trash");
		}
		if app_data.trash_ip.contains(server.ip.as_str()) {
			server.list = String::from("trash");
		}
	}
}
