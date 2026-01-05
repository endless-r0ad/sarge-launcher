use std::sync::Mutex;

mod client;
mod commands;
mod config;
mod demo;
mod huffman_node;
mod level;
mod master;
mod server;
mod q3_util;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
		.plugin(tauri_plugin_dialog::init())
		.plugin(tauri_plugin_log::Builder::new().build())
		.plugin(tauri_plugin_shell::init())
		.manage(Mutex::new(config::SargeLauncher::default()))
		.invoke_handler(tauri::generate_handler![
			commands::level::get_levels,
			commands::level::get_cached_levelshots,
			commands::level::extract_levelshots_to_cache,
			commands::master::get_q3_server_ips,
			commands::server::refresh_all_servers,
			commands::server::refresh_single_server,
			commands::config::get_config,
			commands::config::get_appdata,
			commands::config::save_config,
			commands::config::save_app_data,
			commands::client::spawn_client,
			commands::client::pick_client,
			commands::client::kill_q3_client,
            commands::client::get_client_paths,
            commands::client::get_client_configs,
            commands::client::get_defrag_rec_files,
			commands::demo::get_demos,
            commands::demo::create_demo_script,
            commands::demo::delete_temp_script,
			commands::util::exit_app,
            commands::util::reveal_item_in_dir,
            commands::util::reveal_log
		])
        .setup(move |_app| {
            #[cfg(not(debug_assertions))]
            std::panic::set_hook(Box::new(move |info| {
                log::error!("Panic! {:?}", info);
            }));

            Ok(())
        })
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
