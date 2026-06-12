use crate::options::Options;
use crate::token::InlineToken;
use crate::escape::decode_entities;
use std::collections::HashMap;
use std::sync::OnceLock;
use regex::Regex;

const MAX_BRACKET_DEPTH: usize = 32;
const HARDBREAK_SENTINEL: char = '\u{e000}';
const PLACEHOLDER_BASE: u32 = 0xe100;

/// Inline parser with delimiter stack emphasis matching.
pub struct InlineParser<'a> {
    src: &'a str,
    options: &'a Options,
    link_defs: &'a HashMap<String, (String, Option<String>)>,
    pos: usize,
}

impl<'a> InlineParser<'a> {
    pub fn new(
        src: &'a str,
        options: &'a Options,
        link_defs: &'a HashMap<String, (String, Option<String>)>,
    ) -> Self {
        Self {
            src,
            options,
            link_defs,
            pos: 0,
        }
    }

    pub fn parse<'b>(
        src: &'b str,
        options: &'b Options,
        link_defs: &'b HashMap<String, (String, Option<String>)>,
    ) -> Vec<InlineToken> {
        let mut parser = InlineParser::<'b>::new(src, options, link_defs);
        parser.parse_all()
    }

    fn parse_all(&mut self) -> Vec<InlineToken> {
        let mut tokens = Vec::new();
        while self.pos < self.src.len() {
            if let Some(tok) = self.try_parse_special() {
                tokens.push(tok);
            } else {
                self.parse_text_run(&mut tokens);
            }
        }
        self.process_emphasis(tokens)
    }

    fn try_parse_special(&mut self) -> Option<InlineToken> {
        let rest = &self.src[self.pos..];
        if rest.starts_with('\\') {
            return self.parse_escape();
        }
        if rest.starts_with('`') {
            return self.parse_code_span();
        }
        if rest.starts_with("![") {
            return self.parse_link_or_image();
        }
        if rest.starts_with('[') {
            return self.parse_link_or_image();
        }
        if rest.starts_with('<') {
            if let Some(tok) = self.parse_autolink_or_html() {
                return Some(tok);
            }
        }
        if self.options.gfm && rest.starts_with("~~") {
            return self.parse_strikethrough();
        }
        if rest.starts_with('\n') {
            return self.parse_line_break();
        }
        None
    }

    fn parse_escape(&mut self) -> Option<InlineToken> {
        let bytes = self.src.as_bytes();
        if self.pos + 1 >= bytes.len() {
            return None;
        }
        let next = self.src[self.pos + 1..].chars().next()?;
        if next == '\n' {
            self.pos += 1;
            if self.pos < self.src.len() {
                self.pos += next.len_utf8();
            }
            return Some(InlineToken::HardBreak);
        }
        if next.is_ascii_punctuation() {
            self.pos += 1;
            self.pos += next.len_utf8();
            return Some(InlineToken::Escape(next));
        }
        None
    }

    fn parse_code_span(&mut self) -> Option<InlineToken> {
        let start = self.pos;
        let mut ticks = 0;
        let mut i = self.pos;
        while i < self.src.len() && self.src[i..].starts_with('`') {
            ticks += 1;
            i += 1;
        }
        if ticks == 0 {
            return None;
        }
        let content_start = i;
        let mut found = None;
        let mut search = content_start;
        while search < self.src.len() {
            if !self.src[search..].starts_with('`') {
                let ch = self.src[search..].chars().next()?;
                search += ch.len_utf8();
                continue;
            }
            let mut close_ticks = 0;
            let mut j = search;
            while j < self.src.len() && self.src[j..].starts_with('`') {
                close_ticks += 1;
                j += 1;
            }
            if close_ticks == ticks {
                let content = &self.src[content_start..search];
                let trimmed = trim_code_span(content);
                found = Some((trimmed, j));
                break;
            }
            search = j;
        }
        if let Some((content, end)) = found {
            self.pos = end;
            return Some(InlineToken::CodeSpan(content));
        }
        self.pos = start + ticks;
        Some(InlineToken::Text("`".repeat(ticks)))
    }

    fn parse_link_or_image(&mut self) -> Option<InlineToken> {
        let is_image = self.src[self.pos..].starts_with("![");
        let label_start = self.pos + if is_image { 2 } else { 1 };
        let (label_text, label_end) = self.find_balanced_brackets(label_start)?;
        let parsed_label = Self::parse(&label_text, self.options, self.link_defs);
        let after = &self.src[label_end..];
        if after.starts_with('(') {
            if let Some(tok) = self.parse_inline_link(is_image, parsed_label.clone(), label_end) {
                return Some(tok);
            }
            return self.parse_shortcut_ref(is_image, &label_text, parsed_label, label_end);
        }
        if after.starts_with('[') {
            return self.parse_ref_link(is_image, &label_text, parsed_label, label_end);
        }
        self.parse_shortcut_ref(is_image, &label_text, parsed_label, label_end)
    }

    fn find_balanced_brackets(&self, start: usize) -> Option<(String, usize)> {
        let mut depth = 0;
        let mut i = start;
        let bytes = self.src.as_bytes();
        while i < bytes.len() {
            let ch = self.src[i..].chars().next()?;
            let len = ch.len_utf8();
            if ch == '[' {
                depth += 1;
                if depth > MAX_BRACKET_DEPTH {
                    return None;
                }
            } else if ch == ']' {
                if depth == 0 {
                    let text = self.src[start..i].to_string();
                    return Some((text, i + len));
                }
                depth -= 1;
            } else if ch == '\\' && i + len < bytes.len() {
                i += len;
                let _ = self.src[i..].chars().next()?;
                i += self.src[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(0);
                continue;
            } else if ch == '`' {
                let mut ticks = 0usize;
                let mut tick_end = i;
                while tick_end < bytes.len() && self.src[tick_end..].starts_with('`') {
                    ticks += 1;
                    tick_end += 1;
                }
                if let Some(code_end) = find_matching_backtick_run(self.src, tick_end, ticks) {
                    i = code_end;
                    continue;
                }
            } else if ch == '<' {
                if let Some(angle_end) = find_angle_construct_end(self.src, i + len) {
                    i = angle_end;
                    continue;
                }
            }
            i += len;
        }
        None
    }

    fn parse_inline_link(&mut self, is_image: bool, parsed_label: Vec<InlineToken>, label_end: usize) -> Option<InlineToken> {
        let (href, title, end) = self.parse_link_destination(label_end)?;
        if is_image {
            self.pos = end;
            return Some(InlineToken::Image {
                href,
                title,
                alt: plain_alt_text(&parsed_label),
            });
        }
        if contains_link(&parsed_label) {
            return None;
        }
        self.pos = end;
        Some(InlineToken::Link {
            href,
            title,
            tokens: parsed_label,
        })
    }

    fn parse_shortcut_ref(
        &mut self,
        is_image: bool,
        label_text: &str,
        parsed_label: Vec<InlineToken>,
        label_end: usize,
    ) -> Option<InlineToken> {
        let ref_label = normalize_label(label_text);
        if let Some((href, title)) = self.link_defs.get(&ref_label) {
            if is_image {
                self.pos = label_end;
                return Some(InlineToken::Image {
                    href: href.clone(),
                    title: title.clone(),
                    alt: plain_alt_text(&parsed_label),
                });
            }
            if contains_link(&parsed_label) {
                return None;
            }
            self.pos = label_end;
            return Some(InlineToken::Link {
                href: href.clone(), // clone needed because HashMap returns ref
                title: title.clone(), // clone needed because HashMap returns ref
                tokens: parsed_label,
            });
        }
        None
    }

    fn parse_ref_link(&mut self, is_image: bool, label_text: &str, parsed_label: Vec<InlineToken>, label_end: usize) -> Option<InlineToken> {
        let mut i = label_end;
        if i >= self.src.len() || self.src.as_bytes()[i] != b'[' {
            return None;
        }
        i += 1;
        let ref_start = i;
        while i < self.src.len() {
            let ch = self.src[i..].chars().next()?;
            if ch == ']' {
                break;
            }
            i += ch.len_utf8();
        }
        if i >= self.src.len() {
            return None;
        }
        let ref_label = if i == ref_start {
            normalize_label(label_text)
        } else {
            let explicit = &self.src[ref_start..i];
            if invalid_explicit_ref_label(explicit) {
                self.pos = i + 1;
                let bang = if is_image { "!" } else { "" };
                return Some(InlineToken::Text(format!(
                    "{bang}[{label_text}][{}]",
                    unescape_punctuation(explicit)
                )));
            }
            normalize_label(explicit)
        };
        // collapsed reference link [text][]
        if let Some((href, title)) = self.link_defs.get(&ref_label) {
            if is_image {
                self.pos = i + 1;
                return Some(InlineToken::Image {
                    href: href.clone(), // clone needed because HashMap returns ref
                    title: title.clone(), // clone needed because HashMap returns ref
                    alt: plain_alt_text(&parsed_label),
                });
            }
            if contains_link(&parsed_label) {
                return None;
            }
            self.pos = i + 1;
            return Some(InlineToken::Link {
                href: href.clone(), // clone needed because HashMap returns ref
                title: title.clone(), // clone needed because HashMap returns ref
                tokens: parsed_label,
            });
        }
        None
    }

    fn parse_link_destination(&self, start: usize) -> Option<(String, Option<String>, usize)> {
        let mut i = start;
        if i >= self.src.len() || self.src.as_bytes()[i] != b'(' {
            return None;
        }
        i += 1;
        while i < self.src.len() {
            let ch = self.src[i..].chars().next()?;
            if !ch.is_ascii_whitespace() {
                break;
            }
            i += ch.len_utf8();
        }
        let (href, mut i) = if i < self.src.len() && self.src[i..].starts_with(')') {
            (String::new(), i)
        } else {
            self.parse_link_href(i)?
        };
        let mut title = None;
        while i < self.src.len() {
            let ch = self.src[i..].chars().next()?;
            if ch.is_ascii_whitespace() {
                i += ch.len_utf8();
                if let Some((t, ni)) = self.parse_link_title(i) {
                    title = Some(t);
                    i = ni;
                }
                continue;
            }
            if ch == ')' {
                return Some((href, title, i + ch.len_utf8()));
            }
            break;
        }
        None
    }

    fn parse_link_href(&self, start: usize) -> Option<(String, usize)> {
        let mut i = start;
        if i >= self.src.len() {
            return Some((String::new(), i));
        }
        let first = self.src[i..].chars().next()?;
        if first == '<' {
            i += 1;
            let href_start = i;
            while i < self.src.len() {
                let ch = self.src[i..].chars().next()?;
                if ch == '>' {
                    let href = unescape_punctuation(&decode_entities(&self.src[href_start..i]));
                    return Some((href, i + ch.len_utf8()));
                }
                if ch == '\n' || ch == '\r' || ch == '\\' {
                    return None;
                }
                i += ch.len_utf8();
            }
            return None;
        }

        let href_start = i;
        let mut in_parens = false;
        while i < self.src.len() {
            let ch = self.src[i..].chars().next().expect("checked string bounds");
            if ch.is_ascii_whitespace() || ch.is_ascii_control() {
                break;
            }
            if ch == '\\' {
                i += ch.len_utf8();
                if i < self.src.len() {
                    let next_ch = self.src[i..].chars().next().expect("checked string bounds");
                    i += next_ch.len_utf8();
                }
                continue;
            }
            if ch == '(' {
                if in_parens {
                    break;
                }
                in_parens = true;
                i += ch.len_utf8();
                continue;
            }
            if ch == ')' {
                if in_parens {
                    in_parens = false;
                    i += ch.len_utf8();
                    continue;
                } else {
                    break;
                }
            }
            i += ch.len_utf8();
        }

        let href = unescape_punctuation(&decode_entities(&self.src[href_start..i]));
        Some((href, i))
    }

    fn parse_link_title(&self, start: usize) -> Option<(String, usize)> {
        if start >= self.src.len() {
            return None;
        }
        let first = self.src[start..].chars().next()?;
        let close = match first {
            '"' => ('"', '"'),
            '\'' => ('\'', '\''),
            '(' => ('(', ')'),
            _ => return None,
        }
        .1;
        let mut i = start + first.len_utf8();
        let content_start = i;
        while i < self.src.len() {
            let ch = self.src[i..].chars().next()?;
            if ch == close {
                let title = unescape_punctuation(&decode_entities(&self.src[content_start..i]));
                return Some((title, i + ch.len_utf8()));
            }
            if ch == '\\' {
                i += ch.len_utf8();
                if i < self.src.len() {
                    i += self.src[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(0);
                }
                continue;
            }
            i += ch.len_utf8();
        }
        None
    }

    fn parse_autolink_or_html(&mut self) -> Option<InlineToken> {
        let rest = &self.src[self.pos..];
        if self.looks_like_uri_autolink() {
            return self.parse_uri_autolink();
        }
        if rest.starts_with('<') {
            let inner: String = rest.chars().skip(1).take_while(|c| *c != '>' && !c.is_whitespace()).collect();
            if inner.contains('@') && inner.contains('.') {
                if let Some(tok) = self.parse_email_autolink() {
                    return Some(tok);
                }
            }
        }
        if self.looks_like_html_tag() {
            return self.parse_inline_html();
        }
        None
    }

    fn parse_uri_autolink(&mut self) -> Option<InlineToken> {
        let start = self.pos + 1;
        let mut i = start;
        while i < self.src.len() {
            let ch = self.src[i..].chars().next()?;
            if ch == '>' {
                let href = self.src[start..i].to_string();
                if href.contains(char::is_whitespace) {
                    return None;
                }
                self.pos = i + ch.len_utf8();
                return Some(InlineToken::Autolink {
                    href: href.clone(), // clone needed because text field also needs href
                    text: href,
                    is_email: false,
                });
            }
            i += ch.len_utf8();
        }
        None
    }

    fn parse_email_autolink(&mut self) -> Option<InlineToken> {
        let start = self.pos + 1;
        let mut i = start;
        while i < self.src.len() {
            let ch = self.src[i..].chars().next()?;
            if ch == '>' {
                let email = self.src[start..i].to_string();
                if email.contains('\\') {
                    return None;
                }
                self.pos = i + ch.len_utf8();
                return Some(InlineToken::Autolink {
                    href: format!("mailto:{email}"),
                    text: email,
                    is_email: true,
                });
            }
            i += ch.len_utf8();
        }
        None
    }

    fn looks_like_html_tag(&self) -> bool {
        let rest = &self.src[self.pos..];
        Self::inline_html_regex().is_match(rest)
    }



    fn parse_inline_html(&mut self) -> Option<InlineToken> {
        let rest = &self.src[self.pos..];
        if let Some(m) = Self::inline_html_regex().find(rest) {
            self.pos += m.end();
            return Some(InlineToken::RawHtml(m.as_str().to_string()));
        }
        None
    }

fn inline_html_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(
        r"^(?:(?:<!---->|<!--(?:[^>-]|-[^>])(?s:.*?)-->)|</[a-zA-Z][a-zA-Z0-9:-]*\s*>|<[a-zA-Z][a-zA-Z0-9-]*(?:\s+[a-zA-Z_:][a-zA-Z0-9:._-]*(?:\s*=\s*(?:[^ \x22'=<>`]+|'[^']*'|\x22[^\x22]*\x22))?)*\s*/?>|<\?(?s:.*?)\?>|<![a-zA-Z]+\s+(?s:.*?)>|<!\[CDATA\[(?s:.*?)\]\]>)"
    ).expect("valid regex"))
}

    fn parse_strikethrough(&mut self) -> Option<InlineToken> {
        let start = self.pos + 2;
        let mut i = start;
        while i < self.src.len() {
            if self.src[i..].starts_with("~~") {
                let content = &self.src[start..i];
                let inner = Self::parse(content, self.options, self.link_defs);
                self.pos = i + 2;
                return Some(InlineToken::Del(inner));
            }
            let ch = self.src[i..].chars().next()?;
            i += ch.len_utf8();
        }
        None
    }

    fn parse_line_break(&mut self) -> Option<InlineToken> {
        if self.options.breaks {
            self.pos += 1;
            return Some(InlineToken::HardBreak);
        }
        let before = &self.src[..self.pos];
        if before.ends_with("  ") {
            self.pos += 1;
            return Some(InlineToken::HardBreak);
        }
        self.pos += 1;
        Some(InlineToken::SoftBreak)
    }

    fn parse_text_run(&mut self, tokens: &mut Vec<InlineToken>) {
        let start = self.pos;
        while self.pos < self.src.len() {
            let ch = self.src[self.pos..].chars().next().expect("valid char in string");
            if (matches!(ch, '\\' | '`' | '[' | '<' | '\n')
                || ch == '*'
                || ch == '_'
                || (ch == '!' && self.src[self.pos..].starts_with("![")))
                && self.pos > start
            {
                break;
            }
            if self.options.gfm && self.src[self.pos..].starts_with("~~") && self.pos > start {
                break;
            }
            self.pos += ch.len_utf8();
        }
        if self.pos > start {
            tokens.push(InlineToken::Text(self.src[start..self.pos].to_string()));
        } else if self.pos < self.src.len() {
            let ch = self.src[self.pos..].chars().next().expect("valid char in string");
            self.pos += ch.len_utf8();
            tokens.push(InlineToken::Text(ch.to_string()));
        }
    }

    fn process_emphasis(&self, tokens: Vec<InlineToken>) -> Vec<InlineToken> {
        let mut result = Vec::new();
        let mut text_buf = String::new();
        let mut placeholders = Vec::new();
        for tok in tokens {
            match tok {
                InlineToken::Text(s) => text_buf.push_str(&s),
                InlineToken::HardBreak => {
                    while text_buf.ends_with(' ') {
                        text_buf.pop();
                    }
                    text_buf.push(HARDBREAK_SENTINEL);
                }
                InlineToken::SoftBreak => {
                    text_buf.push('\n');
                }
                other => {
                    if let Some(ch) = placeholder_char(placeholders.len()) {
                        placeholders.push(other);
                        text_buf.push(ch);
                    } else {
                        if !text_buf.is_empty() {
                            result.extend(parse_emphasis_only(&text_buf));
                            text_buf.clear();
                        }
                        result.push(other);
                    }
                }
            }
        }
        if !text_buf.is_empty() {
            result.extend(parse_emphasis_only(&text_buf));
        }
        expand_special_sentinels(result, &placeholders)
    }

    fn looks_like_uri_autolink(&self) -> bool {
        let rest = &self.src[self.pos..];
        let Some(end) = rest.find('>') else {
            return false;
        };
        let inner = &rest[1..end];
        let Some(colon) = inner.find(':') else {
            return false;
        };
        let scheme = &inner[..colon];
        let mut chars = scheme.chars();
        let Some(first) = chars.next() else {
            return false;
        };
        first.is_ascii_alphabetic()
            && (2..=32).contains(&scheme.len())
            && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-'))
    }
}

#[derive(Clone)]
enum EmphResult {
    Text(String),
    Em(Vec<EmphResult>),
    Strong(Vec<EmphResult>),
}

pub fn parse_emphasis_only(src: &str) -> Vec<InlineToken> {
    let results = parse_emphasis_delimiters_str(src);
    results
        .iter()
        .map(|r| match r {
            EmphResult::Text(s) => InlineToken::Text(s.clone()),
            EmphResult::Em(v) => {
                InlineToken::Em(v.iter().map(emph_result_to_simple).collect())
            }
            EmphResult::Strong(v) => {
                InlineToken::Strong(v.iter().map(emph_result_to_simple).collect())
            }
        })
        .collect()
}

fn emph_result_to_simple(r: &EmphResult) -> InlineToken {
    match r {
        EmphResult::Text(s) => InlineToken::Text(s.clone()),
        EmphResult::Em(v) => InlineToken::Em(v.iter().map(emph_result_to_simple).collect()),
        EmphResult::Strong(v) => InlineToken::Strong(v.iter().map(emph_result_to_simple).collect()),
    }
}

fn expand_special_sentinels(
    tokens: Vec<InlineToken>,
    placeholders: &[InlineToken],
) -> Vec<InlineToken> {
    let mut out = Vec::new();
    for token in tokens {
        match token {
            InlineToken::Text(s) => push_text_with_sentinels(&mut out, &s, placeholders),
            InlineToken::Em(inner) => {
                out.push(InlineToken::Em(expand_special_sentinels(inner, placeholders)))
            }
            InlineToken::Strong(inner) => {
                out.push(InlineToken::Strong(expand_special_sentinels(inner, placeholders)))
            }
            InlineToken::StrongEm(inner) => {
                out.push(InlineToken::StrongEm(expand_special_sentinels(inner, placeholders)))
            }
            InlineToken::Del(inner) => {
                out.push(InlineToken::Del(expand_special_sentinels(inner, placeholders)))
            }
            other => out.push(other),
        }
    }
    out
}

fn push_text_with_sentinels(
    out: &mut Vec<InlineToken>,
    text: &str,
    placeholders: &[InlineToken],
) {
    let mut start = 0usize;
    for (idx, ch) in text.char_indices() {
        if ch != HARDBREAK_SENTINEL && placeholder_index(ch).is_none() {
            continue;
        }
        if start < idx {
            out.push(InlineToken::Text(text[start..idx].to_string()));
        }
        if ch == HARDBREAK_SENTINEL {
            out.push(InlineToken::HardBreak);
        } else if let Some(index) = placeholder_index(ch) {
            if let Some(token) = placeholders.get(index) {
                out.push(token.clone());
            }
        }
        start = idx + ch.len_utf8();
    }
    if start < text.len() {
        out.push(InlineToken::Text(text[start..].to_string()));
    }
}

fn placeholder_char(index: usize) -> Option<char> {
    char::from_u32(PLACEHOLDER_BASE + index as u32)
}

fn placeholder_index(ch: char) -> Option<usize> {
    let value = ch as u32;
    if (PLACEHOLDER_BASE..=0xf8ff).contains(&value) {
        Some((value - PLACEHOLDER_BASE) as usize)
} else {
        None
    }
}

#[derive(Clone, Debug)]
enum Node {
    Text(String),
    Delim {
        ch: char,
        count: usize,
        can_open: bool,
        can_close: bool,
    },
    Em(Vec<Node>),
    Strong(Vec<Node>),
}

fn parse_emphasis_delimiters_str(src: &str) -> Vec<EmphResult> {
    let mut nodes = Vec::new();
    let chars: Vec<(usize, char)> = src.char_indices().collect();
    let mut i = 0;
    while i < chars.len() {
        let (pos, ch) = chars[i];
        if ch != '*' && ch != '_' {
            let start = pos;
            while i < chars.len() && chars[i].1 != '*' && chars[i].1 != '_' {
                i += 1;
            }
            let end = if i < chars.len() { chars[i].0 } else { src.len() };
            nodes.push(Node::Text(src[start..end].to_string()));
            continue;
        }
        let mut count = 0;
        let mut j = i;
        while j < chars.len() && chars[j].1 == ch {
            count += 1;
            j += 1;
        }
        let before = if i > 0 { Some(chars[i - 1].1) } else { None };
        let after = if j < chars.len() { Some(chars[j].1) } else { None };
        let (can_open, can_close) = delimiter_flanking(ch, before, after);
        nodes.push(Node::Delim { ch, count, can_open, can_close });
        i = j;
    }

    let processed = process_emphasis(nodes);
    processed.into_iter().map(node_to_emph_result).collect()
}

fn process_emphasis(mut nodes: Vec<Node>) -> Vec<Node> {
    let mut i = 0;
    while i < nodes.len() {
        let (ch, mut count, can_open, can_close) = match nodes[i] {
            Node::Delim { ch, count, can_open, can_close } => (ch, count, can_open, can_close),
            _ => {
                i += 1;
                continue;
            }
        };
        if !can_close {
            i += 1;
            continue;
        }
        let mut match_idx = None;
        let mut j = i;
        while j > 0 {
            j -= 1;
            if let Node::Delim { ch: o_ch, can_open: o_open, can_close: o_close, count: o_count } = nodes[j] {
                if o_ch == ch && o_open {
                    let rule9 = (o_close || can_open) 
                        && (o_count + count) % 3 == 0 
                        && (o_count % 3 != 0 || count % 3 != 0);
                    if !rule9 {
                        match_idx = Some(j);
                        break;
                    }
                }
            }
        }
        if let Some(opener_idx) = match_idx {
            let mut opener_count = 0;
            if let Node::Delim { count: o_count, .. } = nodes[opener_idx] {
                opener_count = o_count;
            }
            let use_count = if opener_count >= 2 && count >= 2 {
                2
            } else {
                1
            };
            
            let inner: Vec<Node> = nodes.drain(opener_idx + 1 .. i).collect();
            
            let result_node = match use_count {
                1 => Node::Em(inner),
                2 => Node::Strong(inner),
                _ => unreachable!(),
            };
            
            if opener_count > use_count {
                if let Node::Delim { ref mut count, .. } = nodes[opener_idx] {
                    *count -= use_count;
                }
                nodes.insert(opener_idx + 1, result_node);
                if count > use_count {
                    count -= use_count;
                    if let Node::Delim { count: ref mut c, .. } = nodes[opener_idx + 2] {
                        *c = count;
                    }
                    i = opener_idx + 2;
                } else {
                    nodes.remove(opener_idx + 2);
                    i = opener_idx + 1;
                }
            } else {
                nodes.remove(opener_idx);
                nodes.insert(opener_idx, result_node);
                if count > use_count {
                    count -= use_count;
                    if let Node::Delim { count: ref mut c, .. } = nodes[opener_idx + 1] {
                        *c = count;
                    }
                    i = opener_idx + 1;
                } else {
                    nodes.remove(opener_idx + 1);
                    i = opener_idx + 1;
                }
            }
        } else {
            i += 1;
        }
    }
    
    for node in &mut nodes {
        if let Node::Delim { ch, count, .. } = node {
            *node = Node::Text(std::iter::repeat(*ch).take(*count).collect());
        }
    }
    nodes
}

fn node_to_emph_result(node: Node) -> EmphResult {
    match node {
        Node::Text(s) => EmphResult::Text(s),
        Node::Delim { ch, count, .. } => EmphResult::Text(std::iter::repeat(ch).take(count).collect()),
        Node::Em(inner) => EmphResult::Em(inner.into_iter().map(node_to_emph_result).collect()),
        Node::Strong(inner) => EmphResult::Strong(inner.into_iter().map(node_to_emph_result).collect()),

    }
}

fn delimiter_flanking(ch: char, before: Option<char>, after: Option<char>) -> (bool, bool) {
    let left_flanking = is_left_flanking(before, after);
    let right_flanking = is_right_flanking(before, after);
    if ch == '*' {
        (left_flanking, right_flanking)
    } else {
        let can_open =
            left_flanking && (!right_flanking || before.map(is_unicode_punct).unwrap_or(false));
        let can_close =
            right_flanking && (!left_flanking || before.map(is_unicode_punct).unwrap_or(false));
        (can_open, can_close)
    }
}

fn is_left_flanking(before: Option<char>, after: Option<char>) -> bool {
    after.map(|c| !c.is_whitespace()).unwrap_or(false)
        && (after.map(|c| !is_unicode_punct(c)).unwrap_or(true)
            || before.map(|c| c.is_whitespace()).unwrap_or(true)
            || before.map(is_unicode_punct).unwrap_or(false))
}

fn is_right_flanking(before: Option<char>, after: Option<char>) -> bool {
    before.map(|c| !c.is_whitespace()).unwrap_or(false)
        && (before.map(|c| !is_unicode_punct(c)).unwrap_or(true)
            || after.map(|c| c.is_whitespace()).unwrap_or(true)
            || after.map(is_unicode_punct).unwrap_or(false))
}

fn is_unicode_punct(c: char) -> bool {
    c.is_ascii_punctuation() || (!c.is_alphanumeric() && !c.is_whitespace())
}

fn trim_code_span(s: &str) -> String {
    let normalized = s
        .chars()
        .map(|ch| if ch == '\n' || ch == '\r' { ' ' } else { ch })
        .collect::<String>();
    if normalized.starts_with(' ')
        && normalized.ends_with(' ')
        && normalized.chars().any(|ch| ch != ' ')
    {
        normalized[1..normalized.len() - 1].to_string()
    } else {
        normalized
    }
}

fn normalize_label(s: &str) -> String {
    let lowered = unescape_punctuation(s)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    lowered.replace('ß', "ss")
}

fn contains_link(tokens: &[InlineToken]) -> bool {
    tokens.iter().any(|tok| match tok {
        InlineToken::Link { .. } => true,
        InlineToken::Em(inner)
        | InlineToken::Strong(inner)
        | InlineToken::StrongEm(inner)
        | InlineToken::Del(inner) => contains_link(inner),
        _ => false,
    })
}

fn find_matching_backtick_run(src: &str, mut search: usize, ticks: usize) -> Option<usize> {
    while search < src.len() {
        if !src[search..].starts_with('`') {
            search += src[search..].chars().next()?.len_utf8();
            continue;
        }
        let mut count = 0usize;
        let mut end = search;
        while end < src.len() && src[end..].starts_with('`') {
            count += 1;
            end += 1;
        }
        if count == ticks {
            return Some(end);
        }
        search = end;
    }
    None
}

fn find_angle_construct_end(src: &str, mut search: usize) -> Option<usize> {
    while search < src.len() {
        let ch = src[search..].chars().next()?;
        if ch == '\n' || ch == '\r' || ch == '<' {
            return None;
        }
        search += ch.len_utf8();
        if ch == '>' {
            return Some(search);
        }
    }
    None
}

fn invalid_explicit_ref_label(label: &str) -> bool {
    invalid_reference_label(label)
}

fn invalid_reference_label(label: &str) -> bool {
    let mut chars = label.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next) = chars.next() {
                if next.is_ascii_punctuation() && !matches!(next, '[' | ']') {
                    return true;
                }
            }
        } else if matches!(ch, '[' | ']') {
            return true;
        }
    }
    false
}

fn unescape_punctuation(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next) = chars.next() {
                if next.is_ascii_punctuation() {
                    out.push(next);
                } else {
                    out.push(ch);
                    out.push(next);
                }
            } else {
                out.push(ch);
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn plain_alt_text(tokens: &[InlineToken]) -> String {
    let mut out = String::new();
    append_plain_alt_text(tokens, &mut out);
    out
}

fn append_plain_alt_text(tokens: &[InlineToken], out: &mut String) {
    for tok in tokens {
        match tok {
            InlineToken::Text(s) | InlineToken::CodeSpan(s) | InlineToken::RawHtml(s) => {
                out.push_str(s);
            }
            InlineToken::Escape(ch) => out.push(*ch),
            InlineToken::HardBreak | InlineToken::SoftBreak => out.push('\n'),
            InlineToken::Em(inner)
            | InlineToken::Strong(inner)
            | InlineToken::StrongEm(inner)
            | InlineToken::Del(inner) => append_plain_alt_text(inner, out),
            InlineToken::Link { tokens, .. } => append_plain_alt_text(tokens, out),
            InlineToken::Image { alt, .. } => out.push_str(alt),
            InlineToken::Autolink { text, .. } => out.push_str(text),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has_strong_em(tokens: &[InlineToken]) -> bool {
        tokens.iter().any(|t| match t {
            InlineToken::Em(inner) => has_strong(inner) || has_strong_em(inner),
            InlineToken::Strong(inner) => has_em(inner) || has_strong_em(inner),
            InlineToken::Del(inner) => has_strong_em(inner),
            _ => false,
        })
    }

    fn has_strong(tokens: &[InlineToken]) -> bool {
        tokens.iter().any(|t| match t {
            InlineToken::Strong(_) => true,
            InlineToken::Em(inner)
            | InlineToken::Del(inner) => has_strong(inner),
            _ => false,
        })
    }

    fn has_em(tokens: &[InlineToken]) -> bool {
        tokens.iter().any(|t| match t {
            InlineToken::Em(_) => true,
            InlineToken::Strong(inner)
            | InlineToken::Del(inner) => has_em(inner),
            _ => false,
        })
    }

    #[test]
    fn emphasis_triple_both() {
        let tokens = parse_emphasis_only("***strong em***");
        assert!(has_strong_em(&tokens));
        if let InlineToken::Em(inner1) = &tokens[0] {
            if let InlineToken::Strong(inner2) = &inner1[0] {
                assert_eq!(inner2.len(), 1);
                assert_eq!(inner2[0], InlineToken::Text("strong em".to_string()));
            } else { panic!("expected Strong"); }
        } else if let InlineToken::Strong(inner1) = &tokens[0] {
            if let InlineToken::Em(inner2) = &inner1[0] {
                assert_eq!(inner2.len(), 1);
                assert_eq!(inner2[0], InlineToken::Text("strong em".to_string()));
            } else { panic!("expected Em"); }
        } else {
            panic!("expected nested Em/Strong");
        }
    }

    #[test]
    fn emphasis_triple_split() {
        let tokens = parse_emphasis_only("***strong** em*");
        assert!(has_strong(&tokens));
        assert!(has_em(&tokens));
    }

    #[test]
    fn emphasis_nested_inside_strong() {
        let tokens = parse_emphasis_only("**strong *with em* inside**");
        assert!(has_strong(&tokens));
    }

    #[test]
    fn emphasis_underscore_not_nested() {
        let tokens = parse_emphasis_only("foo_bar_baz");
        assert!(!has_em(&tokens));
    }

    #[test]
    fn emphasis_mid_word_star() {
        let tokens = parse_emphasis_only("foo*bar*baz");
        assert!(has_em(&tokens));
    }

    #[test]
    fn emphasis_spaced_underscore() {
        let tokens = parse_emphasis_only("foo _bar_ baz");
        assert!(has_em(&tokens));
    }

    #[test]
    fn emphasis_mid_word_underscore_no_em() {
        let tokens = parse_emphasis_only("foo_bar_baz");
        assert!(!has_em(&tokens));
        assert!(tokens.iter().all(|t| matches!(t, InlineToken::Text(_))));
    }
}
