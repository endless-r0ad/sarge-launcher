use image::DynamicImage;
use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{collections::HashMap, path::PathBuf};
use std::fs::{create_dir, File};
use std::path::Path;
use tauri::{AppHandle, Manager};
use zip::ZipArchive;
use std::io::Read;
use std::sync::atomic::{AtomicUsize, Ordering};


use crate::level::Level;

#[tauri::command(async)]
pub async fn get_cached_levelshots(app: AppHandle) -> Result<HashMap<String, String>, tauri::Error> {
	let mut levelshots: HashMap<String, String> = HashMap::new();
	let mut cache_dir = app.path().app_cache_dir()?;
	cache_dir.push("levelshots");

	if !cache_dir.exists() {
		create_dir(&cache_dir)?;
	}

	for entry in std::fs::read_dir(cache_dir)? {
		let entry = entry?;
		let path = entry.path();
		let name = path.file_stem().unwrap().to_str().unwrap().to_string();
		levelshots.entry(name.to_lowercase()).or_insert(path.to_str().unwrap().to_string());
	}

	Ok(levelshots)
}

#[tauri::command(async)]
pub async fn get_levels(search_paths: Vec<String>, get_all_data: bool) -> Result<Vec<Level>, tauri::Error> {
	let mut levels: Vec<Level> = vec![];

	for p in search_paths {
		let path = Path::new(&p);
		levels.append(&mut Level::get_q3_levels(path, get_all_data).await?);
	}

	Ok(levels)
}

#[tauri::command(async)]
pub async fn extract_levelshots_to_cache(app: AppHandle, search_paths: Vec<String>) -> Result<usize, tauri::Error> {
    let mut all_pk3s: Vec<PathBuf> = vec![];
    let num_extracted = AtomicUsize::new(0);
    let mut cache_dir = app.path().app_cache_dir().unwrap();
	cache_dir.push("levelshots");

    if !cache_dir.exists() {
        create_dir(&cache_dir).unwrap();
    }

    for p in search_paths {
        let path = Path::new(&p);
        if path.is_dir() {
            all_pk3s.append(&mut get_pk3s(path).await?);
        }
    }

    all_pk3s.par_iter().for_each(|p| {

        let mut levelshots: Vec<(String, String)> = vec![];
        let mut archive: ZipArchive<File>;

        match std::fs::File::open(&p) {
            Ok(f) => {
                match zip::ZipArchive::new(f) {
                    Ok(z) => archive = z,
                    Err(e) => {
                        log::error!("Could not read pk3, skipping: {}\n{}", &p.to_string_lossy(), e);
                        return;
                    }
                }
            },
            Err(e) => {
                log::error!("Could not open pk3, skipping: {}\n{}", &p.to_string_lossy(), e);
                return;
            }
        }

        for name in archive.file_names() {
            match name {
                x if x.starts_with("levelshots") && x.ends_with(".jpg") => {
                    levelshots.push((name.to_string(), String::from(".jpg")));
                }
                x if x.starts_with("levelshots") && x.ends_with(".tga") => {
                    levelshots.push((name.to_string(), String::from(".tga")));
                }
                _ => continue
            }
        }
        for lshot in &levelshots {
            let level_name = lshot.0[11..lshot.0.len() - 4].to_string();
            let outpath = Path::new(cache_dir.to_str().unwrap()).join(level_name.to_lowercase() + ".jpg");

            match outpath.try_exists() {
                Ok(exists) => if exists { continue; },
                Err(e) => {
                    log::error!("error on path exist check: {}\n{}", &level_name.to_lowercase(), e);
                    continue;
                }
            }

            let mut buf: Vec<u8> = vec![];
            let mut img_b: zip::read::ZipFile<'_, File>;

            match archive.by_name(&lshot.0) {
                Ok(z) => img_b = z,
                Err(e) => {
                    log::error!("error getting zip: {}\n{}", &level_name.to_lowercase(), e);
                    continue;
                }
            }

            let _ = img_b.read_to_end(&mut buf);

            let reader = std::io::Cursor::new(buf);

            let loaded_img: DynamicImage;

            match image::load(reader, if lshot.1 == ".jpg" { image::ImageFormat::Jpeg } else { image::ImageFormat::Tga }) {
                Ok(i) => loaded_img = i,
                Err(e) => {
                    log::error!("error loading image: {}\n{}", &level_name.to_lowercase(), e);
                    continue;
                }
            }

            let resized = loaded_img.resize_exact(480, 360, image::imageops::FilterType::Lanczos3);

            if resized.color().has_alpha() {
                let rgb = resized.to_rgb8();
                match rgb.save_with_format(&outpath, image::ImageFormat::Jpeg) {
                    Ok(_a) => _ = num_extracted.fetch_add(1, Ordering::Relaxed),
                    Err(e) => {
                        log::error!("error saving image: {}\n{}", &level_name.to_lowercase(), e);
                        continue;
                    }
                }

                continue;
            }
            match resized.save_with_format(&outpath, image::ImageFormat::Jpeg) {
                Ok(_a) => _ = num_extracted.fetch_add(1, Ordering::Relaxed),
                Err(e) => {
                    log::error!("error saving image: {}\n{}", &level_name.to_lowercase(), e);
                    continue;
                }
            }
        }

    });

	Ok(num_extracted.load(Ordering::Relaxed))
}


async fn get_pk3s(path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut pk3_paths: Vec<PathBuf> = vec![];
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            pk3_paths.append(&mut Box::pin(get_pk3s(&path)).await?);
        }

        if let Some(ext) = path.extension() {
            if ext == "pk3" {
                pk3_paths.push(path);
            }
        }
    }
    Ok(pk3_paths)
}
