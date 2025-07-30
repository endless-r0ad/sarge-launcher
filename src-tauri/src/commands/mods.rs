use std::path::Path;

#[tauri::command(async)]
pub async fn get_q3_mods(fs_homepath: Option<&Path>) -> Result<Vec<String>, tauri::Error> {
	let mut mods: Vec<String> = vec![];

	if !fs_homepath.is_some() {
		return Ok(mods);
	}

	let homepath = fs_homepath.unwrap();

    for entry in std::fs::read_dir(homepath)? {
		let entry = entry?;
		let path = entry.path();
		let name = path.file_stem().unwrap().to_str().unwrap().to_string();
		mods.push(name); //.entry(name.to_lowercase()).or_insert(path.to_str().unwrap().to_string());
	}

	Ok(mods)
}
