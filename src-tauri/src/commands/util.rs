use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
pub fn exit_app(app: AppHandle) -> Result<(), String> {
	app.exit(1);
	Ok(())
}

#[tauri::command]
pub fn reveal_item_in_dir(app: AppHandle, path: String) -> Result<(), tauri_plugin_opener::Error> {

    app.opener().reveal_item_in_dir(path)?;

	Ok(())
}

#[tauri::command]
pub fn reveal_log(app: AppHandle) -> Result<(), tauri_plugin_opener::Error> {

    let path = app.path().app_log_dir()?;
    app.opener().reveal_item_in_dir(path)?;

	Ok(())
}