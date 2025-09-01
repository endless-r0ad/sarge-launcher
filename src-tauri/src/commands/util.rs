use tauri::AppHandle;

#[tauri::command]
pub fn exit_app(app: AppHandle) -> Result<(), String> {
	app.exit(1);
	Ok(())
}
