use crate::huffman_node::Node;
use rayon::prelude::*;
use std::path::Path;
use std::fs::{File, remove_file};
use std::io::prelude::*;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

use crate::demo::Demo;
use crate::client::Q3Executable;

const SARGE_CFG: &str = "sarge-launcher-demo.cfg";

#[tauri::command(async)]
pub async fn get_demos(search_paths: Vec<String>, cache: HashMap<String, String>, all_data: bool) -> Result<Vec<Demo>, tauri::Error> {
	let mut demos: Vec<Demo> = vec![];
	const Q3_HUFFMAN_TREE: [Node; 514] = Node::create_tree();

    for p in search_paths {
        let path = Path::new(&p).join("demos");
        let demo_path = path.as_path();
        if demo_path.is_dir() {
            demos.append(&mut Demo::get_q3_demos(demo_path, &cache).await?);
        }
    }

	demos.par_iter_mut().for_each(|re| {
		re.parse_demo(Q3_HUFFMAN_TREE, all_data).unwrap_or_else(|error| re.issue = Some(error.to_string()));
	});

	Ok(demos)
}

#[tauri::command(async)]
pub async fn create_demo_script(app: AppHandle, active_client: Q3Executable, fs_game: String, demo_path: String, close: bool, loop_d: bool) -> Result<(), String> {
    let client_path = Path::new(&active_client.parent_path);

    let mut cfg_path = client_path.join(&fs_game);

    if !cfg_path.is_dir() {
        let home_dir = app.path().home_dir().unwrap();
        cfg_path = home_dir.join(&fs_game);
    }

    if !cfg_path.is_dir() {
        return Err(String::from("Cannot find baseq3 path for active client, does it exist?"));
    }

    let new_file = cfg_path.join(SARGE_CFG);

	let mut file = File::create(&new_file).unwrap();

    file.write(b"set cpma_menu togglemenu\r\n").unwrap();
    file.write(b"set close quit\r\n").unwrap();
    file.write(b"set demo_load \"demo ").unwrap();
    file.write(demo_path.as_bytes()).unwrap();
    file.write(b";").unwrap();

    if close {
        file.write(b" set nextdemo vstr close").unwrap();
    }
    if loop_d {
        file.write(b" set nextdemo vstr demo_load").unwrap();
    }   
    file.write(b"\"\r\nvstr demo_load").unwrap();

	Ok(())
}

#[tauri::command(async)]
pub async fn delete_temp_script(file: String) -> Result<(), tauri::Error> {
    let file_path = Path::new(&file);

    if !file_path.exists() {
        return Ok(())
    }

    remove_file(file_path)?;

	Ok(())
}