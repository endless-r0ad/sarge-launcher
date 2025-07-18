use std::fs::create_dir;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

use crate::config::{AppData, Config, Q3Browser};


#[tauri::command(async)]
pub async fn save_config(app: AppHandle, updated_config: Config) -> Result<(), tauri::Error> {
	let state = app.state::<Mutex<Q3Browser>>();
	let mut config = app.path().app_config_dir()?;

	if !config.exists() {
		create_dir(&config)?;
	}

	config.push("config.json");
	updated_config.write_to_file(&config)?;
	state.lock().unwrap().config.lock().unwrap().replace(updated_config);

	Ok(())
}

#[tauri::command(async)]
pub async fn save_app_data(app: AppHandle, updated_data: AppData) -> Result<(), tauri::Error> {
	let state = app.state::<Mutex<Q3Browser>>();
	let mut app_data_dir = app.path().app_data_dir()?;

	if !app_data_dir.exists() {
		create_dir(&app_data_dir)?;
	}

	app_data_dir.push("appdata.json");
    updated_data.write_to_file(&app_data_dir)?;
    state.lock().unwrap().app_data.lock().unwrap().replace(updated_data);

	Ok(())
}

#[tauri::command(async)]
pub fn get_config(app: AppHandle) -> Result<Config, tauri::Error> {
	let state = app.state::<Mutex<Q3Browser>>();
	let mut config_dir = app.path().app_config_dir()?;

	if !config_dir.exists() {
		create_dir(&config_dir)?;
	}

	config_dir.push("config.json");

	if !config_dir.exists() {
		let config = Config::new(config_dir.to_str().unwrap().to_string());
		config.write_to_file(&config_dir)?;
	}

	let config_json = Config::read_from_file(&config_dir)?;
	state.lock().unwrap().config.lock().unwrap().replace(config_json.to_owned());

	Ok(config_json)
}

#[tauri::command(async)]
pub fn get_appdata(app: AppHandle) -> Result<AppData, tauri::Error> {
	let state = app.state::<Mutex<Q3Browser>>();
	let mut app_data_dir = app.path().app_data_dir()?;

	if !app_data_dir.exists() {
		create_dir(&app_data_dir)?;
	}

	app_data_dir.push("appdata.json");

	if !app_data_dir.exists() {
		let app_data = AppData::new(app_data_dir.to_str().unwrap().to_string());
		app_data.write_to_file(&app_data_dir)?;
	}

	let app_data_json = AppData::read_from_file(&app_data_dir)?;
	state.lock().unwrap().app_data.lock().unwrap().replace(app_data_json.to_owned());

	Ok(app_data_json)
}
