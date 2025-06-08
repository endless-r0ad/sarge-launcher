
pub fn parse_q3_colorstring(q3_string: String) -> (String, String) {
    let mut byte_pos: usize = 0;
    let mut styled_string: String = String::new();
    let mut current_color: String = String::from("\"q3c-7\">"); // default white
    let mut span_text: String = String::new();
    let mut host_text: String = String::new();
    let element_start: String = String::from("<span class=");
    let element_end: String = String::from("</span>");

    let trimmed = q3_string.trim().to_string();

    //println!("q3_string is - {} with len - {}", q3_string, q3_string.len());
    if trimmed.len() == 0 {
        return (host_text, styled_string);
    }

    loop {
        let char_ = trimmed.as_bytes()[byte_pos] as char;

        if byte_pos == trimmed.len() - 1 {
            span_text.push_str(&char_.to_string());

            styled_string.push_str(&element_start);
            styled_string.push_str(&current_color);
            styled_string.push_str(&span_text);
            styled_string.push_str(&element_end);
            host_text.push_str(&span_text);

            break;
        }

        let next_char = trimmed.as_bytes()[byte_pos + 1] as char;

        if char_ == '^' && next_char.is_digit(36) {
            // add previous span element with current color
            styled_string.push_str(&element_start);
            styled_string.push_str(&current_color);
            styled_string.push_str(&span_text);
            styled_string.push_str(&element_end);
            host_text.push_str(&span_text);

            if byte_pos == trimmed.len() - 2 {
                break;
            }

            current_color = format!("\"q3c-{}\">", next_char.to_lowercase());

            span_text.clear();
            byte_pos += 2;
            continue;
        }
        span_text.push_str(&char_.to_string());

        byte_pos += 1;
    }

    return (host_text, styled_string);
}
