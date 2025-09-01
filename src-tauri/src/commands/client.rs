use std::sync::Mutex;
use std::path::Path;
use std::process::Stdio;

use std::process::Command;
use is_executable::IsExecutable;

use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

use crate::client::Q3Executable;
use crate::config::SargeLauncher;

#[tauri::command(async)]
pub async fn pick_client(app: AppHandle) -> Result<Option<Q3Executable>, String> {
	let mut q3_exe: Q3Executable;
    let exe_path: &Path;

	let mut file_path = app.dialog().file().set_title("Select a Quake 3 Client").blocking_pick_file();
    
	if let Some(picked_file) = &mut file_path {
		let file_name = picked_file.as_path().unwrap().file_stem().unwrap().to_string_lossy().into_owned();
        exe_path = picked_file.as_path().unwrap();
        let path = picked_file.clone().simplified();
        let parent_path = path.as_path().unwrap().parent().unwrap().to_str().unwrap().to_string();

        q3_exe = Q3Executable{name: file_name.to_string(), exe_path: path.to_string(), parent_path: parent_path, gamename: String::from("")};
        q3_exe.set_gamename();

	} else {
		return Ok(None);
	}

	if exe_path.is_executable() {
		Ok(Some(q3_exe))
	} else {
		Err(format!("{:?} is not an executable file, please choose a quake 3 client executable", q3_exe.name))
	}
}

#[tauri::command(async)]
pub async fn spawn_client(app: AppHandle, active_client: Q3Executable, q3_args: Vec<String>) -> Result<u32, tauri::Error> {
	let state = app.state::<Mutex<SargeLauncher>>();
	let process_id: u32;
    let mut new_command = Command::new(active_client.exe_path);

    new_command.args(q3_args)
                .stdin(Stdio::inherit())
			.stderr(Stdio::inherit())
			.stdout(Stdio::inherit());

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
    let oa_path = home.join(".openarena").join(&client.gamename);
    let home_baseq3 = home.join(".q3a").join("baseq3");
    let exe_baseq3 = Path::new(&client.parent_path).join("baseq3");

    if fs_homepath.is_dir() {
        search_paths.push(fs_homepath.into_os_string().into_string().unwrap());
    }
    
    if exe_path.is_dir() {
        search_paths.push(exe_path.into_os_string().into_string().unwrap());
    }

    if oa_path.is_dir() && client.gamename == "baseoa" {
        search_paths.push(oa_path.into_os_string().into_string().unwrap());
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
