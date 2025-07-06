use tauri::AppHandle;

#[tauri::command]
pub fn exit_app(app: AppHandle) -> Result<(), String> {
	app.exit(1);
	Ok(())
}

#[tauri::command]
pub fn print_frontend_message(msg: String) -> Result<(), String> {
	println!("Frontend message: {}", msg);
	Ok(())
}
