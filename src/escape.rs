/// Escape HTML special characters for text content.
pub fn escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

/// Percent-encode a URL for use in href attributes.
pub fn encode_href(url: &str) -> String {
    let mut out = String::with_capacity(url.len());
    for ch in url.chars() {
        if ch.is_ascii() {
            let byte = ch as u8;
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~'
                | b':' | b'/' | b'?' | b'#' | b'@' | b'!' | b'$' | b'&' | b'\'' | b'('
                | b')' | b'*' | b'+' | b',' | b';' | b'=' | b'%' => out.push(ch),
                _ => push_percent_encoded_byte(&mut out, byte),
            }
        } else {
            let mut buf = [0u8; 4];
            for byte in ch.encode_utf8(&mut buf).bytes() {
                push_percent_encoded_byte(&mut out, byte);
            }
        }
    }
    out
}

pub fn decode_entities(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(pos) = rest.find('&') {
        out.push_str(&rest[..pos]);
        rest = &rest[pos..];
        if let Some(end) = rest.find(';') {
            let entity = &rest[1..end];
            if let Some(decoded) = decode_entity(entity) {
                out.push_str(&decoded);
                rest = &rest[end + 1..];
                continue;
            }
        }
        out.push('&');
        rest = &rest[1..];
    }
    out.push_str(rest);
    out
}

fn decode_entity(entity: &str) -> Option<String> {
    if let Some(num) = entity.strip_prefix("#x").or_else(|| entity.strip_prefix("#X")) {
        return u32::from_str_radix(num, 16)
            .ok()
            .and_then(valid_entity_char)
            .map(|c| c.to_string());
    }
    if let Some(num) = entity.strip_prefix('#') {
        return num
            .parse::<u32>()
            .ok()
            .and_then(valid_entity_char)
            .map(|c| c.to_string());
    }
    let decoded = match entity {
        "amp" => Some("&"),
        "lt" => Some("<"),
        "gt" => Some(">"),
        "quot" => Some("\""),
        "apos" => Some("'"),
        "nbsp" => Some("\u{00a0}"),
        "copy" => Some("\u{00a9}"),
        "AElig" => Some("\u{00c6}"),
        "auml" => Some("\u{00e4}"),
        "Dcaron" => Some("\u{010e}"),
        "frac34" => Some("\u{00be}"),
        "HilbertSpace" => Some("\u{210b}"),
        "DifferentialD" => Some("\u{2146}"),
        "ClockwiseContourIntegral" => Some("\u{2232}"),
        "ngE" => Some("\u{2267}\u{0338}"),
        "ouml" => Some("\u{00f6}"),
        _ => None,
    }?;
    Some(decoded.to_string())
}

fn valid_entity_char(value: u32) -> Option<char> {
    if value == 0 {
        return Some('\u{fffd}');
    }
    char::from_u32(value)
}

fn hex_digit(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        10..=15 => (b'A' + (n - 10)) as char,
        _ => '0',
    }
}

fn push_percent_encoded_byte(out: &mut String, byte: u8) {
    out.push('%');
    out.push(hex_digit(byte >> 4));
    out.push(hex_digit(byte & 0xf));
}
