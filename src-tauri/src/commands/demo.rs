use crate::huffman_node::Node;
use rayon::prelude::*;
use std::path::Path;
use tauri::AppHandle;

use crate::demo::Demo;
use tauri_plugin_dialog::{DialogExt, FilePath};

#[tauri::command(async)]
pub async fn pick_demo_path(app: AppHandle) -> Result<Option<FilePath>, String> {
    let file_dialog = app.dialog().file().set_title("Select your Quake 3 demo folder");

    let file = file_dialog.blocking_pick_folder();

    Ok(file)
}

#[tauri::command(async)]
pub async fn get_demos_rayon(demo_path: &Path) -> Result<Vec<Demo>, String> {

    let mut demos: Vec<Demo>;
    const Q3_HUFFMAN_TREE: [Node; 514] = Node::create_tree();

    if demo_path.is_dir() {
        demos = Demo::get_q3_demos(demo_path).await.map_err(|e| format!("failed to parse demos in {} - {}", demo_path.display(), e))?;
    } else {
        return Err(format!("The specified path does not exist, or is not a directory - {}", demo_path.display()));
    }

    demos.par_iter_mut().for_each(|re| {
        re.parse_demo(Q3_HUFFMAN_TREE).unwrap_or_else(|error| re.issue = error.to_string());
    });

    Ok(demos)
}
