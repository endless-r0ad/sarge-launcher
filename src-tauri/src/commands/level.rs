use std::collections::HashMap;
use std::fs::create_dir;
use std::path::Path;
use tauri::{AppHandle, Manager};
use itertools::Itertools;

use crate::level::Level;
use tauri_plugin_dialog::{DialogExt, FilePath};

#[tauri::command(async)]
pub async fn get_cached_levelshots(app: AppHandle) -> Result<Option<HashMap<String, String>>, tauri::Error> {
	let mut levelshots: HashMap<String, String> = HashMap::new();
	let mut cache_dir = app.path().app_cache_dir()?;
	cache_dir.push("levelshots");

	if !cache_dir.exists() {
		create_dir(&cache_dir)?;
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
pub async fn get_levels(search_paths: Vec<String>, get_all_data: bool) -> Result<Vec<Level>, tauri::Error> {
	
    let mut levels: Vec<Level> = vec![];
   
    for p in search_paths {
        let path = Path::new(&p);
        levels.append(&mut Level::get_q3_levels(path, get_all_data).await?);
    }

   let unique_levels = levels.into_iter().unique_by(|l| l.to_owned().level_name).collect::<Vec<_>>();

	Ok(unique_levels)
}

#[tauri::command(async)]
pub async fn extract_levelshots_to_cache(app: AppHandle, levels: Option<Vec<Level>>) -> Result<(), tauri::Error> {
	if levels.is_some() {
		Level::extract_levelshots(&app, levels.unwrap()).await?;
	}

	Ok(())
}
