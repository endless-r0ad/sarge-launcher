use crate::client::Q3Config;
use std::collections::HashMap;
use std::path::Path;

pub async fn get_q3_configs(dir: &Path) -> Result<Vec<Q3Config>, std::io::Error> {
	let mut q3_configs: Vec<Q3Config> = vec![];

	for entry in std::fs::read_dir(dir)? {
		let entry = entry?;
		let path = entry.path();

		if path.is_dir() {
			q3_configs.append(&mut Box::pin(get_q3_configs(&path)).await?);
		}

		if let Some(ext) = path.extension() {
			let ext_s = ext.to_string_lossy().to_string();

			let name = path.file_name();

			if ext_s != "cfg" || name.is_none() {
				continue;
			}

			let config_p = path.to_str().unwrap();

			q3_configs.push(Q3Config {
				name: name.unwrap().to_str().unwrap().to_string(),
				path: config_p.to_string(),
			})
		}
	}
	Ok(q3_configs)
}

pub fn parse_colorstring(q3_string: String) -> (String, String) {
	let mut byte_pos: usize = 0;
	let mut vhtml_s: String = String::new(); // for v-html frontend
	let mut current_color: String = String::from("7"); // default white
	let mut span_text: String = String::new();
	let mut unstyled_s: String = String::new();

	let trimmed = q3_string.trim().to_string();

	if trimmed.len() == 0 {
		return (unstyled_s, vhtml_s);
	}

	loop {
		let char_ = trimmed.as_bytes()[byte_pos] as char;
        let mut char_s = char_.to_string();
        
        if char_ == '<' { // escape for v-html
            char_s = String::from("&lt;");
        }

        if char_ == '>' {
            char_s = String::from("&gt;");
        }

		if byte_pos == trimmed.len() - 1 {
			span_text.push_str(&char_s);
			vhtml_s.push_str(format!("<span class=\"q3c-{}\">{}</span>", current_color, span_text).as_str());
			unstyled_s.push_str(&span_text);

			break;
		}

		let next_char = trimmed.as_bytes()[byte_pos + 1] as char;

		if char_ == '^' && next_char.is_digit(36) {
            vhtml_s.push_str(format!("<span class=\"q3c-{}\">{}</span>", current_color, span_text).as_str());
			unstyled_s.push_str(&span_text);

			if byte_pos == trimmed.len() - 2 {
				break;
			}

			current_color = next_char.to_lowercase().to_string();

			span_text.clear();
			byte_pos += 2;
			continue;
		}
		span_text.push_str(&char_s);

		byte_pos += 1;
	}

	return (unstyled_s, vhtml_s);
}

pub async fn get_defrag_recs(dir: &Path) -> Result<HashMap<String, Vec<Vec<String>>>, std::io::Error> {
	let mut defrag_recs: HashMap<String, Vec<Vec<String>>> = HashMap::new();

	for entry in std::fs::read_dir(dir)? {
		let entry = entry?;
		let path = entry.path();

		if let Some(ext) = path.extension() {
			let ext_s = ext.to_string_lossy().to_string();

			let name = path.file_name();

			if ext_s != "rec" || name.is_none() {
				continue;
			}

			let rec_file_name = path.file_stem().unwrap().to_str().unwrap().to_string();
			let mut rec_name_parts: Vec<&str> = rec_file_name.split('_').collect();
			let parts_len = rec_name_parts.len();
			let real_map: String;

			if parts_len > 3 {
				real_map = rec_name_parts[0..rec_name_parts.len() - 2].join("_");
				rec_name_parts[parts_len - 3] = real_map.as_str();
				rec_name_parts = rec_name_parts[parts_len - 3..parts_len].to_vec();
			}

			let mut rec_bytes = std::fs::read(Path::new(&path))?;
			rec_bytes.truncate(rec_bytes.len() - 4);
			rec_bytes.reverse();

			let last_finish_byte = rec_bytes.iter().position(|&r| r != 0x00).unwrap();
			let u32_start = 4 - (last_finish_byte % 4); // 4 - since its reversed
			let finish_byte_start = last_finish_byte + u32_start;

			let finish_bytes: [u8; 4] = [
				rec_bytes[finish_byte_start - 1],
				rec_bytes[finish_byte_start - 2],
				rec_bytes[finish_byte_start - 3],
				rec_bytes[finish_byte_start - 4],
			];

			let finish_time = u32::from_le_bytes(finish_bytes);
			let finish_parsed = get_df_time_from_u32(finish_time);

			if let Some(v) = defrag_recs.get_mut(rec_name_parts[0]) {
				v.push(vec![rec_name_parts[1].to_string(), rec_name_parts[2].to_string(), finish_parsed]);
			} else {
				defrag_recs.entry(rec_name_parts[0].to_string()).or_insert(vec![vec![
					rec_name_parts[1].to_string(),
					rec_name_parts[2].to_string(),
					finish_parsed,
				]]);
			}
		}
	}
	Ok(defrag_recs)
}

fn get_df_time_from_u32(rec_time: u32) -> String {
	let mut res = String::from("");
	let msec = rec_time % 1000;
	let mut sec = rec_time / 1000;
	let mins = sec / 60;

	if mins > 0 {
		sec = sec % 60;
		res.push_str(mins.to_string().as_str());
		res.push_str(":");
	}
	if sec > 0 {
		if sec < 10 && mins > 0 {
			res.push_str("0");
		}
		res.push_str(sec.to_string().as_str());
	}

	res.push_str(".");

	if msec < 10 {
		res.push_str("00");
	}
	if msec > 9 && msec < 100 {
		res.push_str("0");
	}

	res.push_str(msec.to_string().as_str());

	res
}
