use is_executable::IsExecutable;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

use crate::client::{Q3Config, Q3Executable};
use crate::config::SargeLauncher;
use crate::q3_util::{get_defrag_recs, get_q3_configs, read_q3config};

#[tauri::command(async)]
pub async fn pick_client(app: AppHandle) -> Result<Option<Q3Executable>, String> {
	let q3_exe: Q3Executable;
    let exe_path: &Path;

	let mut file_path = app.dialog().file().set_title("Select a Quake 3 Client").blocking_pick_file();

	if let Some(picked_file) = &mut file_path {
		let file_name = picked_file.as_path().unwrap().file_stem().unwrap().to_string_lossy().into_owned();
        exe_path = picked_file.as_path().unwrap();
        let path = picked_file.clone().simplified();
        let parent_path = path.as_path().unwrap().parent().unwrap().to_str().unwrap().to_string();

        q3_exe = Q3Executable{
            name: file_name.to_string(), 
            exe_path: path.to_string(), 
            parent_path: parent_path, 
            gamename: String::from(""),
            extra_launch_args: String::from("")
        };

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

	new_command.args(q3_args);

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

/*
    the resulting vec is ordered by priority
    similar to whats done in FS_Startup in qcommon/files.c
*/
#[tauri::command(async)]
pub async fn get_client_search_paths(app: AppHandle, client: Q3Executable) -> Result<Vec<String>, tauri::Error> {
    let mut search_paths: Vec<String> = vec![];
    let home = app.path().home_dir()?;

    let mut exe_path = PathBuf::from(&client.parent_path).join(&client.gamename);
    let mut fs_homepath: PathBuf;

    #[cfg(target_os = "windows")]
    {
        if client.name.to_lowercase().contains("ioquake3") {
            fs_homepath = app.path().app_data_dir()?;
            fs_homepath.pop();
            fs_homepath.extend(["Quake3", &client.gamename]);
        } else {
            fs_homepath = PathBuf::from(&exe_path);
        }
    }

    #[cfg(target_os = "linux")]
    {
        fs_homepath = home.join(if client.uses_openarena_paths() {".openarena"} else {".q3a"}).join(&client.gamename);
    }
    
    #[cfg(target_os = "macos")]
    {
        fs_homepath = home.join("Library/Application Support/Quake3").join(&client.gamename);
    }

    if fs_homepath.is_dir() {
        search_paths.push(fs_homepath.clone().into_os_string().into_string().unwrap());
    }

    if exe_path.is_dir() && !search_paths.contains(&exe_path.clone().into_os_string().into_string().unwrap()) {
        search_paths.push(exe_path.clone().into_os_string().into_string().unwrap());
    }
    
    fs_homepath.pop();
    fs_homepath.push(if client.uses_openarena_paths() {"baseoa"} else {"baseq3"});

    if fs_homepath.is_dir() && 
      !search_paths.contains(&fs_homepath.clone().into_os_string().into_string().unwrap()) &&
      client.game_uses_basegame_paths() 
    {
        search_paths.push(fs_homepath.clone().into_os_string().into_string().unwrap());
    }

    exe_path.pop();
    exe_path.push(if client.uses_openarena_paths() {"baseoa"} else {"baseq3"});

    if exe_path.is_dir() && 
        !search_paths.contains(&exe_path.clone().into_os_string().into_string().unwrap()) &&
        client.game_uses_basegame_paths()
    {
        search_paths.push(exe_path.into_os_string().into_string().unwrap());
    }

	Ok(search_paths)
}

#[tauri::command(async)]
pub async fn get_defrag_rec_files(search_paths: Vec<String>) -> Result<HashMap<String, Vec<Vec<String>>>, tauri::Error> {
	let mut defrag_recs: HashMap<String, Vec<Vec<String>>> = HashMap::new();

	for p in search_paths {
		let mut path = PathBuf::from(p);
		path.extend(["system", "records"]);

		if path.exists() && path.is_dir() {
			defrag_recs.extend(get_defrag_recs(path.as_path()).await?);
		}
	}

	Ok(defrag_recs)
}

#[tauri::command(async)]
pub async fn get_client_available_configs(search_paths: Vec<String>) -> Result<Vec<Q3Config>, tauri::Error> {
    let mut q3_configs: Vec<Q3Config> = vec![];

    for p in search_paths {
        let path = Path::new(&p);
        if path.is_dir() {
            q3_configs.append(&mut get_q3_configs(path).await?);
        }
    }

	Ok(q3_configs)
}

#[tauri::command(async)]
pub async fn get_client_q3config(search_paths: Vec<String>) -> Result<HashMap<String, HashMap<String, String>>, tauri::Error> {
	let mut q3config: HashMap<String, HashMap<String, String>> = HashMap::new();

    for p in &search_paths {
        let mut q3conf = PathBuf::from(&p);
        if q3conf.is_dir() {
            q3conf.push("q3config.cfg");
            if read_q3config(&mut q3config, &q3conf)? {
                break;
            };
        }
    }

    for p in &search_paths {
        let mut autoexec = PathBuf::from(&p);
        if autoexec.is_dir() {
            autoexec.push("autoexec.cfg");
            if read_q3config(&mut q3config, &autoexec)? {
                break;
            };
        }
    }

	Ok(q3config)
}
