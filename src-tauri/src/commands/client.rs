use std::fs::File;
use std::io::Read;
use std::thread;
use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager, WebviewWindow};
//use std::sync::Arc;

//use std::path::Path;
use std::process::{Command, Stdio};
use tauri_plugin_dialog::DialogExt;

use crate::client::Q3Executable;
use crate::config::Q3Browser;

#[tauri::command(async)]
pub async fn pick_client(app: AppHandle, window: WebviewWindow) -> Result<(), String> {
    //tauri::async_runtime::spawn( async move {

    //let os_family = tauri_plugin_os::family();
    //println!("os_family is {:?}", os_family);

    let picking = app.dialog().file().set_title("Select a Quake 3 Client");

    picking.pick_file(move |mut file| {
        if let Some(picked_file) = &mut file {
            let file_name = picked_file
                .as_path()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            let picked_client = Q3Executable::new(
                file_name.to_string(),
                picked_file.clone().simplified(),
                false,
            );
            println!("pick_client - CHOSEN FILE IS - {:?}", picked_client);

            let mut client_bytes = std::fs::File::open(&picked_file.as_path().unwrap()).unwrap();
            let mut first_bytes = [0; 24];

            let _ = client_bytes.read_exact(&mut first_bytes);

            println!("first 24 bytes of client is -- ");
            println!("{:?}", first_bytes);

            let _ = window.emit("picked_client", picked_client).unwrap();
        } else {
            println!("pick_client - CHOSEN FILE IS - {:?}", file);
        }
    });
    // });

    //let _ = picking_client.join();

    Ok(())
}

#[tauri::command]
pub async fn pick_client_blocking(app: AppHandle) -> Result<Option<Q3Executable>, String> {
    //let os_family = tauri_plugin_os::family();
    //println!("os_family is {:?}", os_family);
    let chosen_client: Option<Q3Executable>;
    let mut is_valid_client: bool;
    let mut chosen_file_bytes = [0; 12];

    let mut file_path = app
        .dialog()
        .file()
        .set_title("Select a Quake 3 Client")
        //.add_filter("Quake3 Exe", &["exe", "x64", "x86_64"])
        .blocking_pick_file();

    if let Some(picked_file) = &mut file_path {
        let file_name = picked_file
            .as_path()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();

        chosen_client = Some(Q3Executable::new(
            file_name.to_string(),
            picked_file.clone().simplified(),
            false,
        ));

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

    match chosen_client.clone().unwrap().name.to_lowercase() {
        x if x.contains("ded") => is_valid_client = false,
        x if x.contains("serv") => is_valid_client = false,
        _ => (),
    }

    if is_valid_client {
        Ok(chosen_client)
    } else {
        Err(format!("{:?} is not a quake 3 client executable", chosen_client.unwrap().name))
    }
}

#[tauri::command]
pub async fn spawn_quake(app: AppHandle, active_client: Q3Executable, q3_args: Vec<String>, manage_instance: bool) -> Result<u32, String> {
    let state = app.state::<Mutex<Q3Browser>>();
    let state = state.lock().unwrap();
    let mut client = state.client.lock().unwrap();
    let process_id: u32;

    if let Some(child) = &mut *client {
        if child.try_wait().unwrap().is_none() && manage_instance {
            child.kill().unwrap();
        }

        let mut new_child = Command::new(active_client.path.to_string())
            .args(q3_args)
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to open {}: {}", active_client.name, e))?;

        process_id = new_child.id();

        if manage_instance {
            client.replace(new_child);
        } else {
            thread::spawn(move || { new_child.wait().unwrap(); });
        }
    } else {
        //println!("the child in the Mutex was NONE, so spawning new q3 instance");

        let mut new_child = Command::new(active_client.path.to_string())
            .args(q3_args)
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to open {}: {}", active_client.name, e))?;

        //let mut stdin = new_child.stdin.take().unwrap();
        process_id = new_child.id();
        //stdin.write_all(b"\\demo current-xas.dm_68").unwrap();

        if manage_instance {
            client.replace(new_child);
        } else {
            thread::spawn(move || { new_child.wait().unwrap(); });
        }
    }

    Ok(process_id)
}

#[tauri::command]
pub async fn kill_q3_client(app: AppHandle, process_id: u32) -> Result<(), String> { 
    let state = app.state::<Mutex<Q3Browser>>();
    let state = state.lock().unwrap();

    let mut client = state.client.lock().unwrap();

    if let Some(current_client) = &mut *client {
        if current_client.try_wait().unwrap().is_none() {
            if current_client.id() == process_id {
                current_client.kill().unwrap();
            }
        }
    }

    Ok(())
}
