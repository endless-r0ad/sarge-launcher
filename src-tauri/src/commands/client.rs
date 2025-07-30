use std::fs::File;
use std::io::Read;
use std::sync::Mutex;
use std::path::Path;

use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

use crate::client::Q3Executable;
use crate::config::SargeLauncher;

#[tauri::command(async)]
pub async fn pick_client(app: AppHandle) -> Result<Option<Q3Executable>, String> {
	let mut chosen_client: Q3Executable;
	let mut is_valid_client: bool;
	let mut chosen_file_bytes = [0; 12];

	let mut file_path = app.dialog().file().set_title("Select a Quake 3 Client").blocking_pick_file();

	if let Some(picked_file) = &mut file_path {
		let file_name = picked_file.as_path().unwrap().file_stem().unwrap().to_string_lossy().into_owned();
        let path = picked_file.clone().simplified();
        let parent_path = path.as_path().unwrap().parent().unwrap().to_str().unwrap().to_string();

        chosen_client = Q3Executable{name: file_name.to_string(), exe_path: path.to_string(), parent_path: parent_path, gamename: String::from("")};
        chosen_client.set_gamename();

		let mut client_bytes = File::open(&picked_file.as_path().unwrap()).unwrap();
		let _ = client_bytes.read_exact(&mut chosen_file_bytes);
	} else {
		return Ok(None);
	}

	match &chosen_file_bytes {
		b"\x4D\x5A\x90\x00\x03\x00\x00\x00\x04\x00\x00\x00" => is_valid_client = true, // win32
		b"\x7F\x45\x4C\x46\x02\x01\x01\x00\x00\x00\x00\x00" => is_valid_client = true, // linux
		_ => is_valid_client = false,
	};

	match chosen_client.name.to_lowercase() {
		x if x.contains("ded") => is_valid_client = false,
		x if x.contains("serv") => is_valid_client = false,
		_ => (),
	}

	if is_valid_client {
		Ok(Some(chosen_client))
	} else {
		Err(format!("{:?} is not a quake 3 client executable", chosen_client.name))
	}
}

#[tauri::command(async)]
pub async fn spawn_client(app: AppHandle, active_client: Q3Executable, mut q3_args: Vec<String>) -> Result<u32, tauri::Error> {
	let state = app.state::<Mutex<SargeLauncher>>();
	let process_id: u32;
    let mut new_command = Command::new(active_client.exe_path);

    #[cfg(windows)]
    let mut demoname = String::from("");

    #[cfg(windows)]
    if q3_args.len() > 2 && q3_args[&q3_args.len()-2] == "+demo" {
        demoname = q3_args.pop().unwrap();
    }
  
    new_command.args(q3_args);

    #[cfg(windows)]
    if demoname.len() > 0 {
        new_command.raw_arg(demoname);
    }

    let new = new_command.spawn()?;	
    process_id = new.id();
	state.lock().unwrap().client.lock().unwrap().replace(new);

	Ok(process_id)
}

#[tauri::command(async)]
pub async fn kill_q3_client(app: AppHandle, process_id: Option<u32>) -> Result<(), tauri::Error> {
    if process_id.is_none() {
        return Ok(())
    }
	let state = app.state::<Mutex<SargeLauncher>>();
	let state = state.lock().unwrap();

	let mut client = state.client.lock().unwrap();

	if let Some(current_client) = &mut *client {
		if current_client.try_wait()?.is_none() {
			if current_client.id() == process_id.unwrap() {
				current_client.kill()?;
			}
		}
	}

	Ok(())
}

#[tauri::command(async)]
pub async fn get_client_paths(app: AppHandle, active_client: Option<Q3Executable>) -> Result<Vec<String>, tauri::Error> {
    let mut search_paths: Vec<String> = vec![];
    let home = app.path().home_dir()?;
    let mut mod_uses_baseq3_path: bool = false;

	if active_client.is_none() {
		return Ok(search_paths);
	}

    let client = active_client.unwrap();

    if client.gamename == "cpma" || client.gamename == "defrag" {
        mod_uses_baseq3_path = true;
    }
    
    let fs_homepath = home.join(".q3a").join(&client.gamename);
    let exe_path = Path::new(&client.parent_path).join(&client.gamename);
    let home_baseq3 = home.join(".q3a").join("baseq3");
    let exe_baseq3 = Path::new(&client.parent_path).join("baseq3");

    if fs_homepath.is_dir() {
        search_paths.push(fs_homepath.into_os_string().into_string().unwrap());
    }
    
    if exe_path.is_dir() {
        search_paths.push(exe_path.into_os_string().into_string().unwrap());
    }

    if mod_uses_baseq3_path {
        if home_baseq3.is_dir() {
            search_paths.push(home_baseq3.into_os_string().into_string().unwrap());
        }
        if exe_baseq3.is_dir() {
            search_paths.push(exe_baseq3.into_os_string().into_string().unwrap());
        }     
    }

	Ok(search_paths)
}
