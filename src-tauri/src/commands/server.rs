use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::io::{Error, ErrorKind};

use tauri::{AppHandle, Manager};

use crate::config::SargeLauncher;
use crate::server::{Quake3Server, GETSTATUS};

const TIMEOUT_MS: u64 = 1500;
const QUERY_ATTEMPTS: usize = 2;

#[tauri::command(async)]
pub async fn refresh_all_servers(app: AppHandle, mut all_servers: Vec<Quake3Server>) -> Result<Vec<Quake3Server>, String> {
	if all_servers.len() == 0 {
		return Err(String::from("Zero servers to refresh, check network connection or master server status"))
	}

    tauri::async_runtime::spawn_blocking(move || {
        for s in &mut all_servers {
            s.reset_data();
        }

        get_saved_servers(&app, &mut all_servers);

        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        socket.set_read_timeout(Some(Duration::from_millis(TIMEOUT_MS))).unwrap();

        query_servers_batch(&socket, &mut all_servers, QUERY_ATTEMPTS, TIMEOUT_MS);

        Ok(all_servers)

    }).await.map_err(|e| e.to_string())?
	
}

#[tauri::command(async)]
pub async fn refresh_single_server(mut refresh_server: Quake3Server) -> Result<Quake3Server, String> {

    let server_list = refresh_server.list.clone();
    let is_custom = refresh_server.custom.clone();

	refresh_server.reset_data();
    refresh_server.list = server_list;
    refresh_server.custom = is_custom;

	let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
	let _ = socket.set_read_timeout(Some(Duration::from_millis(TIMEOUT_MS))).unwrap();
    
	refresh_server.query_server(&socket, 0);

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

pub fn query_servers_batch(socket: &UdpSocket, chunk: &mut [Quake3Server], max_attempts: usize, timeout_ms: u64) {
    let mut pending: HashMap<SocketAddr, usize> = HashMap::new();

    for (i, serv) in chunk.iter_mut().enumerate() {
        if serv.list == "trash" {
            serv.set_trash();
            continue;
        }

        match serv.address.to_socket_addrs().ok().and_then(|mut it| it.next()) {
            Some(addr) => {
                pending.insert(addr, i);
            }
            None => {
                serv.set_error(Error::new(ErrorKind::InvalidInput, "Could not resolve server address"));
            }
        }
    }

    if pending.is_empty() {
        return;
    }

    let ping_start = Instant::now();
    let mut attempt = 0;

    while attempt < max_attempts && !pending.is_empty() {
        for &addr in pending.keys() {
            if let Err(e) = socket.send_to(GETSTATUS, addr) {
                log::error!("send_to {} failed: {}", addr, e);
            }
        }

        let deadline = Instant::now() + Duration::from_millis(timeout_ms);

        while !pending.is_empty() {

            let remaining = deadline.saturating_duration_since(Instant::now());

            if remaining.is_zero() {
                break;
            }

            let mut response_buf: [u8; 2400] = [0; 2400];
            let _ = socket.set_read_timeout(Some(remaining));

            match socket.recv_from(&mut response_buf) {
                Ok((_bytes, src)) => {
                    if let Some(i) = pending.remove(&src) {
                        let serv = &mut chunk[i];
                        serv.ping = ping_start.elapsed().as_millis().try_into().unwrap_or(u16::MAX);
                        serv.parse_getstatus(&response_buf).unwrap_or_else(|e| serv.errormessage = e.to_string());
                    }
                }
                Err(_) => break
            }
        }

        attempt += 1;
    }

    for (_addr, i) in pending {
        chunk[i].set_error(Error::new(ErrorKind::TimedOut, "No response after max retries"));
    }

}