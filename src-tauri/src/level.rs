use image::EncodableLayout;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::create_dir;
use std::io::Read;
use std::path::Path;
use tauri::AppHandle;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Level {
	pub pk3_name: String,
	pub level_name: String,
	pub path: String,
	pub parent_path: String,
}

impl Level {
	pub async fn get_q3_levels(dir: &Path) -> Result<Vec<Level>, std::io::Error> {
		let mut maps: Vec<Level> = vec![];

		for entry in std::fs::read_dir(dir)? {
			let entry = entry?;
			let path = entry.path();

			if path.is_dir() {
				maps.append(&mut Box::pin(Self::get_q3_levels(&path)).await?);
			}

			if let Some(ext) = path.extension() {
				if ext == "pk3" {
					let file = std::fs::File::open(path.to_str().unwrap().to_string()).unwrap();
					let archive = zip::ZipArchive::new(file).unwrap();

					for name in archive.file_names() {
						if name.to_lowercase().starts_with("maps") && name.to_lowercase().ends_with(".bsp") {
							let level_name = &name[5..name.len() - 4];

							maps.push(Level {
								pk3_name: path.file_stem().unwrap().to_str().unwrap().to_string(),
								level_name: level_name.to_string(),
								path: path.to_str().unwrap().to_string(),
								parent_path: path.parent().unwrap().to_str().unwrap().to_string(),
							});
						}
					}
				}
			}
		}
		Ok(maps)
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
