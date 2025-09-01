use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::time::Instant;

use tauri::{AppHandle, Manager};

use crate::config::SargeLauncher;
use crate::server::Quake3Server;

const GETSTATUS: &[u8] = b"\xff\xff\xff\xffgetstatus\x00";

#[tauri::command(async)]
pub async fn refresh_all_servers(
	app: AppHandle,
	mut all_servers: Vec<Quake3Server>,
	num_threads: usize,
	timeout: u64,
) -> Result<Vec<Quake3Server>, String> {

    if all_servers.len() == 0 {
        return Err(String::from("Zero servers to refresh, check network connection or master server status"))
    }

    get_saved_servers(&app, &mut all_servers);
    let serv_chunks = all_servers.chunks(all_servers.len() / num_threads);
	let refreshed_servers_arc: Arc<Mutex<Vec<Quake3Server>>> = Arc::new(Mutex::new(vec![]));

    thread::scope(|s| {
        let mut handles = vec![];

        for chunk in serv_chunks {

            let refreshed = Arc::clone(&refreshed_servers_arc);
            let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
            socket.set_read_timeout(Some(Duration::from_millis(timeout))).unwrap();

            handles.push(s.spawn(move || {
                
                for i in 0..chunk.len() {
                    let mut queried_server: Quake3Server = chunk[i].to_owned();

                    let ping_start = Instant::now();

                    let x = socket.send_to(GETSTATUS, queried_server.address.to_owned()).unwrap_or_else(|err| {
                        queried_server.set_error(err);
                        return 0;
                    });

                    if x == 0 {
                        refreshed.lock().unwrap().push(queried_server);
                        continue;
                    }

                    let mut response_buf: [u8; 2400] = [0; 2400];

                    let response = socket.recv_from(&mut response_buf);

                    match response {
                        Ok((_bytes, _src)) => {
                            queried_server.ping = ping_start.elapsed().as_millis() as u16;
                            queried_server
                                .parse_status_response(&response_buf)
                                .unwrap_or_else(|e| queried_server.errormessage = e.to_string());
                            refreshed.lock().unwrap().push(queried_server);
                        }
                        Err(err) => {
                            queried_server.set_error(err);
                            refreshed.lock().unwrap().push(queried_server);
                        }
                    }
                }
            }));
        }
        for handle in handles {
            handle.join().unwrap();
        }
    });

    let servers = Mutex::into_inner(Arc::try_unwrap(refreshed_servers_arc).unwrap()).map_err(|e| e.to_string())?;

	Ok(servers)
}

#[tauri::command(async)]
pub async fn refresh_single_server(mut refresh_server: Quake3Server, timeout: u64) -> Result<Quake3Server, String> {
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
        return Ok(refresh_server)
	}

	let mut response_buf: [u8; 2400] = [0; 2400];

	let response = socket.recv_from(&mut response_buf);

	match response {
		Ok((_bytes, _src)) => {
			refresh_server.ping = ping_start.elapsed().as_millis() as u16;

			refresh_server
				.parse_status_response(&response_buf)
				.unwrap_or_else(|e| refresh_server.errormessage = e.to_string());
		}
		Err(err) => {
			refresh_server.set_error(err);
		}
	}

	Ok(refresh_server)
}

fn get_saved_servers(app: &AppHandle, servers: &mut Vec<Quake3Server>) -> () {
	let all_servers_addresses: Vec<String> = servers.iter().map(|x| x.address.clone()).collect();

    let state = app.state::<Mutex<SargeLauncher>>();
	let state = state.lock().unwrap();

	if let Some(app_data) = &mut *state.app_data.lock().unwrap() {
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
	};
}
