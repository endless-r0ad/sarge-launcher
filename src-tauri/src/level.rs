use image::EncodableLayout;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use zip::ZipArchive;
use std::fs::{create_dir, File};
use std::io::Read;
use std::path::Path;
use tauri::AppHandle;
use tauri::Manager;

use crate::shared::parse_q3_colorstring;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Level {
	pub pk3_name: String,
	pub level_name: String,
    pub long_name: String,
    pub gametype: Vec<String>,
    pub author: String,
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

					let file = std::fs::File::open(&path).unwrap();
					let mut archive: ZipArchive<File> = zip::ZipArchive::new(file).unwrap();

					for name in archive.file_names() {
						if name.to_lowercase().starts_with("maps") && name.to_lowercase().ends_with(".bsp") {
		
							pk3_maps.push(
                                Level {
                                    pk3_name: pk3_name.clone(),
                                    level_name: name[5..name.len() - 4].to_string(),
                                    long_name: String::from(""),
                                    gametype: vec![],
                                    author: String::from(""),
                                    path: path.to_str().unwrap().to_string(),
                                    parent_path: path.parent().unwrap().to_str().unwrap().to_string(),
                                    is_defrag: false,
                                    year_created: 1901
							    }
                            );
						}
					}
                    if get_all_data {
                        Level::get_remaining_data(&mut pk3_maps, pk3_name, &mut archive);
                    }                 
                    maps.append(&mut pk3_maps);          
				}
			}
		}
		Ok(maps)
	}

    pub fn get_remaining_data(pk3_maps: &mut Vec<Level>, pk3_name: String, archive: &mut ZipArchive<File>) {
        if pk3_name == "pak0" {
            let arena_file = archive.by_name("scripts/arenas.txt");
            if let Ok(mut file) = arena_file {
                let mut contents = String::from("");
                let x = file.read_to_string(&mut contents);
                if let Ok(_good_read) = x {
                    for i in 0..pk3_maps.len() {
                        pk3_maps[i].author = String::from("id Software");
                        pk3_maps[i].parse_arena_data(contents.clone());
                    }
                }
            } 
        }

        for m in pk3_maps.iter_mut() {
            let bsp = archive.by_name(format!("maps/{}.bsp", m.level_name).as_str());
            if let Ok(bsp_map) = bsp {
                let last_date = bsp_map.last_modified();
                if let Some(date) = last_date {
                    m.year_created = date.year();
                }
            }
        }

        {
            let arena_file = archive.by_name(format!("scripts/{}.arena", pk3_name).as_str());
            if let Ok(mut file) = arena_file {
                let mut contents = String::from("");
                let x = file.read_to_string(&mut contents);
                if let Ok(_good_read) = x {
                    for i in 0..pk3_maps.len() {
                        pk3_maps[i].parse_arena_data(contents.clone());
                    }
                }
            } 
        }
        {
            let defi_file = archive.by_name(format!("scripts/{}.defi", pk3_name).as_str());
            if let Ok(mut file) = defi_file {
                pk3_maps.iter_mut().for_each(|m| { m.is_defrag = true; });
                let mut contents = String::from("");
                let x = file.read_to_string(&mut contents);
                if let Ok(_good_read) = x {
                    for i in 0..pk3_maps.len() {
                        pk3_maps[i].parse_arena_data(contents.clone());
                    }
                    pk3_maps.iter_mut().for_each(|m| { m.gametype = vec![String::from("race")]; });
                }
            }
        }
        // if the data file wasnt found by pk3_name, try each level_name
        for i in 0..pk3_maps.len() {
            if pk3_maps[i].long_name != "" {
                continue;
            }

            {
                let arena_file = archive.by_name(format!("scripts/{}.arena", pk3_maps[i].level_name).as_str());
                if let Ok(mut file) = arena_file {
                    let mut contents = String::from("");
                    let x = file.read_to_string(&mut contents);
                    if let Ok(_good_read) = x {
                        for i in 0..pk3_maps.len() {
                            pk3_maps[i].parse_arena_data(contents.clone());
                        }
                    }
                } 
            }
            {
                let defi_file = archive.by_name(format!("scripts/{}.defi", pk3_maps[i].level_name).as_str());
                if let Ok(mut file) = defi_file {
                    pk3_maps.iter_mut().for_each(|m| { m.is_defrag = true; });
                    let mut contents = String::from("");
                    let x = file.read_to_string(&mut contents);
                    if let Ok(_good_read) = x {
                        for i in 0..pk3_maps.len() {
                            pk3_maps[i].parse_arena_data(contents.clone());
                        }
                        pk3_maps.iter_mut().for_each(|m| { m.gametype = vec![String::from("race")]; });
                    }
                }
            }
        }
    }

    pub fn parse_arena_data(&mut self, contents: String) {
        let mut contents_map = self.pk3_name.clone();

        for l in contents.lines() {
            let new_l = l.replace("\t", "");

            if new_l.len() == 0 || new_l == "{" || new_l == "}" || new_l.starts_with("//") {
                continue;
            }

            let key_val: Vec<&str> = new_l.split("\"").collect();

            if key_val.len() > 1 {
                match key_val[0].to_lowercase().trim() {
                    "map" => contents_map = key_val[1].trim().to_lowercase().to_string(),
                    "type" => {
                        if contents_map == self.level_name.to_lowercase() {
                            let gametypes: Vec<String> = key_val[1].trim().split(" ").map(String::from).collect();
                            self.gametype = gametypes.into_iter().filter(|t| !t.trim().is_empty()).collect();
                        }
                    },
                    "longname" => {
                        if contents_map == self.level_name.to_lowercase() {
                            self.long_name = parse_q3_colorstring(key_val[1].trim().to_string()).1
                        }
                    },
                    "author" => {
                        if contents_map == self.level_name.to_lowercase() {
                            self.author = parse_q3_colorstring(key_val[1].trim().to_string()).1
                        }
                    },
                    _ => continue
                }
            }
            
        }

    }

	pub async fn extract_levelshots(app: &AppHandle, mut levels: Vec<Level>) -> Result<(), std::io::Error> {
		let mut cache_dir = app.path().app_cache_dir().unwrap();
		cache_dir.push("levelshots");

		if !cache_dir.exists() {
			create_dir(&cache_dir).unwrap();
		}

		levels.retain(|l| {
			let outpath_est = Path::new(cache_dir.to_str().unwrap()).join(l.level_name.to_lowercase().to_string() + ".jpg");
			return !outpath_est.try_exists().unwrap();
		});

		levels.par_iter_mut().for_each(|level| {
			let mut levelshots: Vec<(String, String)> = vec![];

			let file = std::fs::File::open(&level.path).unwrap();
			let mut archive = zip::ZipArchive::new(file).unwrap();

			for name in archive.file_names() {
				if name.to_lowercase().starts_with("levelshots") {
					if name.to_lowercase().ends_with(".jpg") {
						levelshots.push((name.to_string(), String::from(".jpg")));
					}
					if name.to_lowercase().ends_with(".tga") {
						levelshots.push((name.to_string(), String::from(".tga")));
					}
				}
			}

			for lshot in &levelshots {
				if lshot.0.to_lowercase().contains(&level.level_name.to_lowercase()) {
					let outpath = Path::new(cache_dir.to_str().unwrap()).join(level.level_name.to_lowercase().to_string() + ".jpg");
					let mut buf = vec![];

					let mut img_b = archive.by_name(&lshot.0).unwrap();
					let _ = img_b.read_to_end(&mut buf);

					let reader = std::io::Cursor::new(buf.as_bytes());

					let loaded_img = image::load(
						reader,
						if lshot.1 == ".jpg" {
							image::ImageFormat::Jpeg
						} else {
							image::ImageFormat::Tga
						},
					)
					.unwrap();

					let resized = loaded_img.resize_exact(640, 480, image::imageops::FilterType::Lanczos3);

					if resized.color().has_alpha() {
						let rgb = resized.to_rgb8();
						rgb.save_with_format(&outpath, image::ImageFormat::Jpeg).unwrap();
						break;
					}
					resized.save_with_format(&outpath, image::ImageFormat::Jpeg).unwrap();

					break;
				}
			}
		});

		Ok(())
	}
}
