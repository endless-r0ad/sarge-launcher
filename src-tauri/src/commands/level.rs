use std::collections::HashMap;
use std::fs::create_dir;
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

	if !cache_dir.exists() {
		create_dir(&cache_dir).unwrap();
	}

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
pub async fn get_levels(fs_homepath: Option<&Path>) -> Result<Option<Vec<Level>>, String> {
	let levels: Option<Vec<Level>>;

	if !fs_homepath.is_some() {
		return Ok(None);
	}

	let homepath = fs_homepath.unwrap();

	if homepath.is_dir() {
		levels = Some(
			Level::get_q3_levels(homepath)
				.await
				.map_err(|e| format!("failed to parse levels in {} - {}", homepath.display(), e))?,
		);
	} else {
		return Err(format!("The specified path does not exist, or is not a directory - {}", homepath.display()));
	}

	Ok(levels)
}

#[tauri::command(async)]
pub async fn extract_levelshots_to_cache(app: AppHandle, levels: Option<Vec<Level>>) -> Result<(), String> {
	if levels.is_some() {
		Level::extract_levelshots(&app, levels.unwrap())
			.await
			.map_err(|e| format!("failed to extract levelshots {}", e))?;
	}

	Ok(())
}
