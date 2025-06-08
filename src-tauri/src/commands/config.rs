// #[macro_use] -- needs to be in root, main.rs
// extern crate structure;

//use std::str;
use std::sync::Mutex;

//use serde::{Deserialize, Serialize};

use std::fs::{create_dir, read_to_string, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::config::{AppData, Config, Q3Browser};

fn read_config_json(config_path: &PathBuf) -> Result<Config, String> {
    
    //println!("config_dir is - {:?}", config_path);
    let config_str = read_to_string(config_path).unwrap();
    //println!("config_str is - {}", config_str);

    let config_json = serde_json::from_str(&config_str);

    let config: Config = match config_json {
        Ok(config) => config,
        Err(_e) => {
            //println!("error during read_config_json - {:?}", e);
            std::fs::remove_file(config_path).unwrap();
            //println!("file should have deleted");
            let new_config = Config::new(config_path.to_str().unwrap().to_string());
            let file = File::create(&config_path).unwrap();
            let mut writer = BufWriter::new(file);
            serde_json::to_writer(&mut writer, &new_config).unwrap();
            writer.flush().unwrap();
            return Ok(new_config);
        }
    };
    return Ok(config);
}

fn read_app_data_json(data_dir: &PathBuf) -> Result<AppData, String> {
    let app_data_str = read_to_string(data_dir).unwrap();

    let app_data_json = serde_json::from_str(&app_data_str);

    let app_data: AppData = match app_data_json {
        Ok(app_data) => app_data,
        Err(e) => {
            println!("error during read_app_data_json - {:?}", e);
            std::fs::remove_file(data_dir).unwrap();
            //println!("file should have deleted");
            let new_app_data = AppData::new(data_dir.to_str().unwrap().to_string());
            let file = File::create(&data_dir).unwrap();
            let mut writer = BufWriter::new(file);
            serde_json::to_writer(&mut writer, &new_app_data).unwrap();
            writer.flush().unwrap();
            return Ok(new_app_data);
        }
    };
    return Ok(app_data);
}

#[tauri::command(async)]
pub async fn save_app_data(app: AppHandle, app_data: AppData) -> Result<(), String> {
    
    let state = app.state::<Mutex<Q3Browser>>();
    let mut app_data_dir = app.path().app_data_dir().unwrap();

    // create the dir if its not there for some reason
    if !app_data_dir.exists() {
        create_dir(&app_data_dir).unwrap();
    }

    app_data_dir.push("sarge-launcher-legacy-data.json");

    // create default config

    //println!("SAVING THE APP DATA");
    //println!("new data is - {:?}", app_data);

    //let config = Config::new("sarge-launcher-legacy.json".to_string(), config_dir.to_str().unwrap().to_string());
    let file = File::create(&app_data_dir).unwrap();
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &app_data).unwrap();
    writer.flush().unwrap();

    let state = state.lock().unwrap();
    let mut state_data = state.app_data.lock().unwrap();

    if let Some(_old_state) = &mut *state_data {
        state_data.replace(app_data);
    } else {
        state_data.replace(app_data);
    }

    drop(state_data);

    //println!("new STATE data is - {:?}", state.app_data);

    Ok(())
}

#[tauri::command(async)]
pub async fn save_config(app: AppHandle, updated_config: Config) -> Result<(), String> {
    
    //confirm_config;
    let state = app.state::<Mutex<Q3Browser>>();
    let mut config_dir = app.path().app_config_dir().unwrap();

    // create dir if not there
    if !config_dir.exists() {
        create_dir(&config_dir).unwrap();
    }

    config_dir.push("sarge-launcher-legacy.json");

    //println!("updated config is - {:?}", updated_config);

    // create default config
    //let config = Config::new("sarge-launcher-legacy.json".to_string(), config_dir.to_str().unwrap().to_string());
    let file = File::create(&config_dir).unwrap();
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &updated_config).unwrap();
    writer.flush().unwrap();

    let state = state.lock().unwrap();
    let mut state_config = state.config.lock().unwrap();

    if let Some(_old_state) = &mut *state_config {
        state_config.replace(updated_config);
    } else {
        state_config.replace(updated_config);
    }
    drop(state_config);

    Ok(())
}

#[tauri::command(async)]
pub fn get_config(app: AppHandle) -> Result<Config, String> {
    
    //confirm_config;
    let state = app.state::<Mutex<Q3Browser>>();
    let mut config_dir = app.path().app_config_dir().unwrap();

    // create dirs if not there
    if !config_dir.exists() {
        create_dir(&config_dir).unwrap();
    }

    config_dir.push("sarge-launcher-legacy.json");

    if !config_dir.exists() {
        // create default config
        //println!("the file DOES NOT EXIST");
        let config = Config::new(config_dir.to_str().unwrap().to_string());
        let file = File::create(&config_dir).unwrap();
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &config).unwrap();
        writer.flush().unwrap();
    }

    //validate the config here
    let config_json = read_config_json(&config_dir).unwrap();

    let state = state.lock().unwrap();
    let mut state_config = state.config.lock().unwrap();

    if let Some(_old_state) = &mut *state_config {
        state_config.replace(config_json.to_owned());
        drop(state_config);
    } else {
        state_config.replace(config_json.to_owned());
        drop(state_config);
    }

    Ok(config_json)
}

#[tauri::command(async)]
pub fn get_appdata(app: AppHandle) -> Result<AppData, String> {
    
    //confirm_config;
    let state = app.state::<Mutex<Q3Browser>>();
    let mut app_data_dir = app.path().app_data_dir().unwrap();

    // create dirs if not there
    if !app_data_dir.exists() {
        create_dir(&app_data_dir).unwrap();
    }

    app_data_dir.push("sarge-launcher-legacy-data.json");


    if !app_data_dir.exists() {
        // create default config
        //println!("the file DOES NOT EXIST");
        let app_data = AppData::new(app_data_dir.to_str().unwrap().to_string());
        let file = File::create(&app_data_dir).unwrap();
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &app_data).unwrap();
        writer.flush().unwrap();
    }

    //validate the config here
    let app_data_json = read_app_data_json(&app_data_dir).unwrap();
    
    let state = state.lock().unwrap();
    let mut state_data = state.app_data.lock().unwrap();

    if let Some(_old_state) = &mut *state_data {
        state_data.replace(app_data_json.to_owned());
        drop(state_data);
    } else {
        state_data.replace(app_data_json.to_owned());
        drop(state_data);
    }

    Ok(app_data_json)

}