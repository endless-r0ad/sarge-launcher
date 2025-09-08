use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
pub fn exit_app(app: AppHandle) -> Result<(), String> {
	app.exit(1);
	Ok(())
}

#[tauri::command]
pub fn open_folder(app: AppHandle, path: String) -> Result<(), tauri_plugin_opener::Error> {

    app.opener().reveal_item_in_dir(path)?;

	Ok(())
}