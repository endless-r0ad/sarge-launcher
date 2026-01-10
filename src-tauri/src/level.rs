use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

use crate::q3_util::parse_colorstring;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Level {
	pub pk3_name: String,
	pub level_name: String,
    pub long_name: String,
    pub gametype: Vec<String>,
    pub author: String,
    pub author_vhtml: String,
	pub path: String,
	pub parent_path: String,
    pub is_defrag: bool,
    pub year_created: u16
}

impl Level {
	pub async fn get_q3_levels(dir: &Path, get_all_data: bool) -> Result<Vec<Level>, std::io::Error> {
		let mut maps: Vec<Level> = vec![];

		for entry in std::fs::read_dir(dir)? {
			let entry = entry?;
			let path = entry.path();

			if path.is_dir() {
				maps.append(&mut Box::pin(Self::get_q3_levels(&path, get_all_data)).await?);
			}

			if let Some(ext) = path.extension() {
				if ext == "pk3" {
                    let mut pk3_maps: Vec<Level> = vec![];
                    let pk3_name = path.file_stem().unwrap().to_str().unwrap().to_string();
                    let mut arena_files: Vec<String> = vec![];
                    let mut defi_files: Vec<String> = vec![];

                    let mut archive: ZipArchive<File>;

					match std::fs::File::open(&path) {
                        Ok(f) => {
                            match zip::ZipArchive::new(f) {
                                Ok(z) => archive = z,
                                Err(e) => {
                                    log::error!("Could not read pk3, skipping: {}\n{}", &path.to_string_lossy(), e);
                                    continue;
                                }
                            }
                        },
                        Err(e) => {
                            log::error!("Could not open pk3, skipping: {}\n{}", &path.to_string_lossy(), e);
                            continue;
                        }
                    }

					for name in archive.file_names() {
                        let name_lowered = name.to_lowercase();
                        match name_lowered {
                            x if x.starts_with("maps") && x.ends_with(".bsp") => {
                                pk3_maps.push(
                                    Level {
                                        pk3_name: pk3_name.clone(),
                                        level_name: name[5..name.len() - 4].to_string(),
                                        long_name: String::from(""),
                                        gametype: vec![],
                                        author: String::from(""),
                                        author_vhtml: String::from(""),
                                        path: path.to_str().unwrap().to_string(),
                                        parent_path: path.parent().unwrap().to_str().unwrap().to_string(),
                                        is_defrag: false,
                                        year_created: 1901
                                    }
                                );
                            }
                            x if x.starts_with("scripts") && x.ends_with("arenas.txt") => {
                                arena_files.push(name.to_string());
                            }
                            x if x.starts_with("scripts") && x.ends_with(".arena") => {
                                arena_files.push(name.to_string());
                            }
                            x if x.starts_with("scripts") && x.ends_with(".defi") => {
                                defi_files.push(name.to_string());
                            }
                            _ => continue
                        }
					}
                    if get_all_data {
                        Level::get_remaining_data(&mut pk3_maps, arena_files, defi_files, &mut archive);
                    }                 
                    maps.append(&mut pk3_maps);          
				}
			}
		}
		Ok(maps)
	}

    pub fn get_remaining_data(pk3_maps: &mut Vec<Level>, arena_files: Vec<String>, defi_files: Vec<String>, archive: &mut ZipArchive<File>) {
        
        let mut arena_data: Vec<HashMap<String, String>> = vec![];
        let mut defi_data: Vec<HashMap<String, String>> = vec![];

        for arena in arena_files {
            let arena_file = archive.by_name(&arena);
            if let Ok(mut file) = arena_file {
                let mut contents = String::from("");
                let x = file.read_to_string(&mut contents);
                if let Ok(_good_read) = x {
                    arena_data.extend(Self::parse_arena_data(contents.clone()));
                }
            }
        }

        for defi in defi_files {
            let defi_file = archive.by_name(&defi);
            if let Ok(mut file) = defi_file {
                let mut contents = String::from("");
                let x = file.read_to_string(&mut contents);
                if let Ok(_good_read) = x {
                    defi_data.extend(Self::parse_arena_data(contents.clone()));
                }
            }
        }

        for m in pk3_maps.iter_mut() {
            if m.pk3_name == "pak0" && m.parent_path.contains("baseq3") {
                m.author = String::from("id Software");
                m.author_vhtml = String::from("id Software");
            }
            let bsp = archive.by_name(format!("maps/{}.bsp", m.level_name).as_str());
            if let Ok(bsp_map) = bsp {
                let last_date = bsp_map.last_modified();
                if let Some(date) = last_date {
                    m.year_created = date.year();
                }
            }
            for d in &arena_data {
                if *d.get("map").unwrap() == m.level_name.to_lowercase() {
                    if d.contains_key("longname") {
                        m.long_name = d.get("longname").unwrap().to_owned();
                    }
                    if d.contains_key("author") {
                        m.author = d.get("author").unwrap().to_owned();
                    }
                    if d.contains_key("gametype") {
                        let gametypes: Vec<String> = d.get("gametype").unwrap().to_owned().split(" ").map(String::from).collect();
                        m.gametype = gametypes.into_iter().filter(|t| !t.trim().is_empty()).collect();
                    }
                    break
                }
            }
            for d in &defi_data {
                if *d.get("map").unwrap() == m.level_name.to_lowercase() {
                    m.is_defrag = true;
                    m.gametype = vec![String::from("race")];
                    if d.contains_key("longname") {
                        m.long_name = d.get("longname").unwrap().to_owned();
                    }
                    if d.contains_key("author") {
                        m.author = d.get("author").unwrap().to_owned();
                        m.author_vhtml = d.get("author_vhtml").unwrap().to_owned();
                    }
                    if d.contains_key("gametype") {
                        let gametypes: Vec<String> = d.get("gametype").unwrap().to_owned().split(" ").map(String::from).collect();
                        m.gametype = gametypes.into_iter().filter(|t| !t.trim().is_empty()).collect();
                    }
                    break
                }
            }
        }
    }

    pub fn parse_arena_data(contents: String) -> Vec<HashMap<String, String>> {
        let mut arenas: Vec<HashMap<String, String>> = vec![];
        let mut current_arena_data: HashMap<String, String> = HashMap::new();

        for l in contents.lines() {
            let new_l = l.replace("\t", "");

            if new_l.len() == 0 || new_l.starts_with("//") {
                continue;
            }

            if new_l == "{" {
                current_arena_data.clear();
            }

            if new_l == "}" {
                if current_arena_data.contains_key("map") {
                    arenas.push(current_arena_data.clone());
                }
                continue;
            }

            let key_val: Vec<&str> = new_l.split("\"").collect();

            if key_val.len() > 1 {
                match key_val[0].to_lowercase().trim() {
                    "map" => current_arena_data.entry(String::from("map")).or_insert(key_val[1].trim().to_lowercase().to_string()),
                    "type" => current_arena_data.entry(String::from("gametype")).or_insert(key_val[1].trim().to_string()),
                    "longname" => current_arena_data.entry(String::from("longname")).or_insert(parse_colorstring(key_val[1].trim()).1),
                    "author" => {
                        let parsed = parse_colorstring(key_val[1].trim());
                        current_arena_data.entry(String::from("author")).or_insert(parsed.0);
                        current_arena_data.entry(String::from("author_vhtml")).or_insert(parsed.1)
                    }
                    _ => continue
                };
            }      
        }
        arenas
    }
}
