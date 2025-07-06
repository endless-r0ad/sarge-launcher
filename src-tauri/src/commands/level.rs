use std::collections::HashMap;
use std::path::Path;
use tauri::{AppHandle, Manager};

use crate::level::Level;
use tauri_plugin_dialog::{DialogExt, FilePath};

#[tauri::command(async)]
pub async fn pick_fs_homepath(app: AppHandle) -> Result<Option<FilePath>, String> {
	let file_dialog = app.dialog().file().set_title("Select your fs_homepath folder");

	let file = file_dialog.blocking_pick_folder();

	Ok(file)
}

#[tauri::command(async)]
pub async fn get_cached_levelshots(app: AppHandle) -> Result<Option<HashMap<String, String>>, tauri::Error> {
	let mut levelshots: HashMap<String, String> = HashMap::new();
	let mut cache_dir = app.path().app_cache_dir().unwrap();
	cache_dir.push("levelshots");

	for entry in std::fs::read_dir(cache_dir)? {
		let entry = entry?;
		let path = entry.path();
		let name = path.file_stem().unwrap().to_str().unwrap().to_string();
		levelshots.entry(name.to_lowercase()).or_insert(path.to_str().unwrap().to_string());
	}

	if levelshots.len() == 0 {
		return Ok(None);
	}

	Ok(Some(levelshots))
}

#[tauri::command(async)]
pub async fn get_levels(fs_homepath: &Path) -> Result<Vec<Level>, String> {
	let levels: Vec<Level>;

	if fs_homepath.is_dir() {
		levels = Level::get_q3_levels(fs_homepath)
			.await
			.map_err(|e| format!("failed to parse levels in {} - {}", fs_homepath.display(), e))?;
	} else {
		return Err(format!("The specified path does not exist, or is not a directory - {}", fs_homepath.display()));
	}

	Ok(levels)
}

#[tauri::command(async)]
pub async fn extract_levelshots_to_cache(app: AppHandle, levels: Vec<Level>) -> Result<(), String> {
	Level::extract_levelshots(&app, levels)
		.await
		.map_err(|e| format!("failed to extract levelshots {}", e))?;

	Ok(())
}
