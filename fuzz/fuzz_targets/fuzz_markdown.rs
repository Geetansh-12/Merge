#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(s) = std::str::from_utf8(data) else {
        return;
    };

    let result = marked_rs::parse(s);

    assert!(
        std::str::from_utf8(result.as_bytes()).is_ok(),
        "Output is not valid UTF-8"
    );

    assert_balanced_html(&result);

    assert!(
        result.len() <= s.len() * 100 + 1024,
        "Output {} bytes is suspiciously large for {} byte input",
        result.len(),
        s.len()
    );
});

fn assert_balanced_html(html: &str) {
    let void_elements = [
        "br", "hr", "img", "input", "area", "base", "col", "embed", "link", "meta", "param",
        "source", "track", "wbr",
    ];
    let mut stack: Vec<String> = Vec::new();
    let mut i = 0;
    let bytes = html.as_bytes();

    while i < bytes.len() {
        if bytes[i] == b'<' {
            if let Some(end) = html[i..].find('>') {
                let tag_content = &html[i + 1..i + end];
                if tag_content.starts_with('/') {
                    let name = tag_content[1..]
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_lowercase();
                    if !void_elements.contains(&name.as_str()) {
                        stack.pop();
                    }
                } else if !tag_content.ends_with('/') {
                    let name = tag_content
                        .split(|c: char| c.is_whitespace() || c == '>')
                        .next()
                        .unwrap_or("")
                        .to_lowercase();
                    if !name.is_empty()
                        && !name.starts_with('!')
                        && !void_elements.contains(&name.as_str())
                    {
                        stack.push(name);
                    }
                }
                i += end + 1;
                continue;
            }
        }
        i += 1;
    }

    assert!(
        stack.len() < 100,
        "Suspiciously deep unclosed tag stack: {} deep in output: {:?}",
        stack.len(),
        &html[..html.len().min(200)]
    );
}
