use std::sync::Mutex;

mod client;
mod commands;
mod config;
mod demo;
mod huffman_node;
mod level;
mod master;
mod server;
mod shared;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()
		.plugin(tauri_plugin_dialog::init())
		.plugin(tauri_plugin_log::Builder::new().build())
		.plugin(tauri_plugin_shell::init())
		.manage(Mutex::new(config::Q3Browser::default()))
		.invoke_handler(tauri::generate_handler![
			commands::level::get_levels,
			commands::level::get_cached_levelshots,
			commands::level::extract_levelshots_to_cache,
			commands::level::pick_fs_homepath,
			commands::master::get_q3_server_ips,
			commands::server::refresh_all_servers,
			commands::server::refresh_single_server,
			commands::config::get_config,
			commands::config::get_appdata,
			commands::config::save_config,
			commands::config::save_app_data,
			commands::client::spawn_quake,
			commands::client::pick_client,
			commands::client::kill_q3_client,
			commands::demo::get_demos_rayon,
			commands::demo::pick_demo_path,
			commands::util::exit_app,
			commands::util::print_frontend_message
		])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
