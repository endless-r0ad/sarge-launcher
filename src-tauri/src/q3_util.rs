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
