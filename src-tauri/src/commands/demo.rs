use crate::huffman_node::Node;
use rayon::prelude::*;
use std::path::Path;

use crate::demo::Demo;

#[tauri::command(async)]
pub async fn get_demos_rayon(search_paths: Vec<String>) -> Result<Vec<Demo>, tauri::Error> {
	let mut demos: Vec<Demo> = vec![];
	const Q3_HUFFMAN_TREE: [Node; 514] = Node::create_tree();

    for p in search_paths {
        let path = Path::new(&p).join("demos");
        let demo_path = path.as_path();
        println!("demo_path is {:#?}", demo_path);
        if demo_path.is_dir() {
            demos.append(&mut Demo::get_q3_demos(demo_path).await?);
        }
    }

	demos.par_iter_mut().for_each(|re| {
		re.parse_demo(Q3_HUFFMAN_TREE).unwrap_or_else(|error| re.issue = Some(error.to_string()));
	});

	Ok(demos)
}
