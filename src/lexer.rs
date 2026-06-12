use crate::options::Options;
use crate::token::{Alignment, ListItem, TableCell, Token};
use regex::Regex;
use std::sync::OnceLock;

pub struct Lexer<'a> {
    options: &'a Options,
    lines: Vec<String>,
    line_idx: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str, options: &'a Options) -> Self {
        let normalized_src = src.replace("\r\n", "\n").replace('\r', "\n");
        let lines: Vec<String> = normalized_src.split('\n').map(|s| s.to_string()).collect();
        Self {
            options,
            lines,
            line_idx: 0,
        }
    }

    pub fn tokenize(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while self.line_idx < self.lines.len() {
            let line = &self.lines[self.line_idx];
            if line.trim().is_empty() {
                tokens.push(Token::Space);
                self.line_idx += 1;
                continue;
            }
            if let Some(tok) = self.try_parse_block() {
                tokens.push(tok);
            } else {
                self.line_idx += 1;
            }
        }
        tokens
    }

    fn try_parse_block(&mut self) -> Option<Token> {
        if let Some(tok) = self.parse_html_block() {
            return Some(tok);
        }
        if let Some(tok) = self.parse_link_def() {
            return Some(tok);
        }
        if let Some(tok) = self.parse_atx_heading() {
            return Some(tok);
        }
        if let Some(tok) = self.parse_fenced_code() {
            return Some(tok);
        }
        if let Some(tok) = self.parse_indented_code() {
            return Some(tok);
        }
        if let Some(tok) = self.parse_hr() {
            return Some(tok);
        }
        if self.options.gfm {
            if let Some(tok) = self.parse_table() {
                return Some(tok);
            }
        }
        if let Some(tok) = self.parse_blockquote() {
            return Some(tok);
        }
        if let Some(tok) = self.parse_list() {
            return Some(tok);
        }
        if let Some(tok) = self.parse_setext_heading() {
            return Some(tok);
        }
        self.parse_paragraph()
    }

    fn parse_atx_heading(&mut self) -> Option<Token> {
        let line = &self.lines[self.line_idx];
        let re = atx_heading_regex();
        let caps = re.captures(line)?;
        let hashes = caps.get(1)?.as_str();
        let depth = hashes.len().min(6) as u8;
        let raw = caps
            .get(2)
            .map(|m| strip_atx_closing_hashes(m.as_str()).to_string())
            .unwrap_or_default();
        self.line_idx += 1;
        Some(Token::Heading {
            depth,
            raw,
            tokens: Vec::new(),
        })
    }

    fn parse_setext_heading(&mut self) -> Option<Token> {
        if self.line_idx + 1 >= self.lines.len() {
            return None;
        }
        let first_line = &self.lines[self.line_idx];
        if leading_spaces(first_line) > 3 || first_line.trim().is_empty() {
            return None;
        }
        let mut text_lines = Vec::new();
        let mut i = self.line_idx;
        let depth;
        loop {
            if i >= self.lines.len() {
                return None;
            }
            let line = &self.lines[i];
            if line.trim().is_empty()
                || list_marker_can_interrupt(line)
                || atx_heading_regex().is_match(line)
                || line.trim_start().starts_with('>')
                || (line.trim().starts_with('<') && html_block_start_regex().is_match(line.trim()))
            {
                return None;
            }
            text_lines.push(line.trim().to_string());
            if i + 1 >= self.lines.len() {
                return None;
            }
            let underline = &self.lines[i + 1];
            if setext_h1_regex().is_match(underline) {
                depth = 1u8;
                break;
            }
            if setext_h2_regex().is_match(underline) {
                depth = 2u8;
                break;
            }
            i += 1;
        }
        let raw = text_lines.join("\n");
        self.line_idx = i + 2;
        Some(Token::Heading {
            depth,
            raw,
            tokens: Vec::new(),
        })
    }

    fn parse_fenced_code(&mut self) -> Option<Token> {
        let line = &self.lines[self.line_idx];
        let indent = leading_spaces(line);
        if indent > 3 {
            return None;
        }
        let opener = &line[indent..];
        if !opener.starts_with("```") && !opener.starts_with("~~~") {
            return None;
        }
        let fence_char = opener.chars().next()?;
        let fence_len = opener.chars().take_while(|c| *c == fence_char).count();
        if fence_len < 3 {
            return None;
        }
        let info = opener[fence_len..].trim();
        if fence_char == '`' && info.contains('`') {
            return None;
        }
        let lang = if info.is_empty() {
            None
        } else {
            Some(unescape_punctuation(info.split_whitespace().next().unwrap_or("")))
        };
        self.line_idx += 1;
        let mut code_lines = Vec::new();
        while self.line_idx < self.lines.len() {
            let l = &self.lines[self.line_idx];
            let close_indent = leading_spaces(l);
            let close_line = if close_indent <= 3 {
                &l[close_indent..]
            } else {
                l.as_str()
            };
            let close_count = close_line.chars().take_while(|c| *c == fence_char).count();
            if close_indent <= 3
                && close_count >= fence_len
                && close_line[close_count..].trim().is_empty()
            {
                self.line_idx += 1;
                break;
            }
            code_lines.push(strip_up_to_n_spaces(l, indent).to_string());
            self.line_idx += 1;
        }
        let text = if code_lines.is_empty() {
            String::new()
        } else {
            code_lines.join("\n") + "\n"
        };
        Some(Token::CodeBlock { lang, text })
    }

    fn parse_indented_code(&mut self) -> Option<Token> {
        let line = &self.lines[self.line_idx];
        if leading_indent_columns(line) < 4 {
            return None;
        }
        let mut code_lines = Vec::new();
        while self.line_idx < self.lines.len() {
            let l = &self.lines[self.line_idx];
            if l.trim().is_empty() {
                let mut next_idx = self.line_idx + 1;
                while next_idx < self.lines.len() && self.lines[next_idx].trim().is_empty() {
                    next_idx += 1;
                }
                if next_idx < self.lines.len() && leading_indent_columns(&self.lines[next_idx]) >= 4 {
                    code_lines.push(strip_indent_columns(l, 4));
                    self.line_idx += 1;
                    continue;
                }
                break;
            }
            if leading_indent_columns(l) >= 4 {
                code_lines.push(strip_indent_columns(l, 4));
            } else {
                break;
            }
            self.line_idx += 1;
        }
        let text = code_lines.join("\n");
        let text = if text.is_empty() { text } else { text + "\n" };
        Some(Token::CodeBlock {
            lang: None,
            text,
        })
    }

    fn parse_blockquote(&mut self) -> Option<Token> {
        let line = &self.lines[self.line_idx];
        if !line.starts_with('>') && !line.starts_with(" >") {
            let trimmed = line.trim_start();
            if !trimmed.starts_with('>') {
                return None;
            }
        }
        let mut bq_lines = Vec::new();
        let mut allow_lazy_continuation = false;
        while self.line_idx < self.lines.len() {
            let l = &self.lines[self.line_idx];
            if l.trim().is_empty() {
                break;
            }
            if let Some(stripped) = strip_blockquote_prefix(l) {
                bq_lines.push(stripped.to_string());
                allow_lazy_continuation = blockquote_line_allows_lazy_continuation(stripped);
                self.line_idx += 1;
            } else {
                if allow_lazy_continuation
                    && (!self.looks_like_block_start(l) || leading_indent_columns(l) >= 4)
                {
                    let continuation = if leading_indent_columns(l) >= 4 {
                        protect_lazy_continuation_marker(&strip_indent_columns(l, 4))
                    } else {
                        protect_lazy_continuation_marker(l)
                    };
                    bq_lines.push(continuation);
                    allow_lazy_continuation = true;
                    self.line_idx += 1;
                    continue;
                }
                break;
            }
        }
        let raw = bq_lines.join("\n");
        let inner = Lexer::new(&raw, self.options).tokenize();
        Some(Token::Blockquote {
            raw,
            tokens: inner,
        })
    }

    fn parse_list(&mut self) -> Option<Token> {
        let line = &self.lines[self.line_idx];
        let (ordered, marker_len, start) = parse_list_marker(line)?;
        let first_bullet = if ordered { None } else { unordered_marker_char(line) };
        let first_ordered_delim = if ordered { ordered_marker_delim(line) } else { None };
        let mut items: Vec<ListItem> = Vec::new();
        let first_idx = self.line_idx;
        let mut any_loose = false;

        while self.line_idx < self.lines.len() {
            let l = &self.lines[self.line_idx];
            if l.trim().is_empty() {
                let mut next_idx = self.line_idx + 1;
                while next_idx < self.lines.len() && self.lines[next_idx].trim().is_empty() {
                    next_idx += 1;
                }
                if next_idx < self.lines.len() {
                    let next = &self.lines[next_idx];
                    if parse_list_marker(next).is_some() || is_list_continuation(next, marker_len) {
                        any_loose = true;
                        self.line_idx += 1;
                        continue;
                    }
                }
                break;
            }
            if leading_indent_columns(l) <= 3 && hr_regex().is_match(l.trim()) {
                break;
            }
            if let Some((ord, ml, st)) = parse_list_marker(l) {
                if ord != ordered {
                    break;
                }
                if !ordered && unordered_marker_char(l) != first_bullet {
                    break;
                }
                if ordered && ordered_marker_delim(l) != first_ordered_delim {
                    break;
                }
                let item_start = self.line_idx;
                let (item_lines, loose) = self.collect_list_item_lines(ml, ord);
                if loose {
                    any_loose = true;
                }
                let (task, checked) = parse_task_item(&self.lines[item_start]);
                let raw = item_lines.join("\n");
                let item_tokens = Lexer::new(&raw, self.options).tokenize();
                items.push(ListItem {
                    task,
                    checked,
                    loose,
                    tokens: item_tokens,
                });
                let _ = st;
            } else if is_list_continuation(l, marker_len) {
                self.line_idx += 1;
                if let Some(last) = items.last_mut() {
                    let cont = strip_indent_columns(l, marker_len);
                    if let Some(tok) = last.tokens.last_mut() {
                        append_to_last_paragraph(tok, &cont);
                    }
                }
            } else {
                break;
            }
        }

        if items.is_empty() {
            self.line_idx = first_idx;
            return None;
        }
        Some(Token::List {
            ordered,
            start,
            loose: any_loose || items.iter().any(|i| i.loose),
            items,
        })
    }

    fn collect_list_item_lines(&mut self, marker_len: usize, _ordered: bool) -> (Vec<String>, bool) {
        let mut lines = Vec::new();
        let mut loose = false;
        if self.line_idx < self.lines.len() {
            let l = &self.lines[self.line_idx];
            let content = strip_indent_columns(l, marker_len);
            lines.push(content);
            self.line_idx += 1;
        }
        while self.line_idx < self.lines.len() {
            let l = &self.lines[self.line_idx];
            if l.trim().is_empty() {
                let mut next_idx = self.line_idx + 1;
                while next_idx < self.lines.len() && self.lines[next_idx].trim().is_empty() {
                    next_idx += 1;
                }
                if next_idx < self.lines.len() {
                    let next = &self.lines[next_idx];
                    if is_list_continuation(next, marker_len) {
                        loose = true;
                        lines.push(String::new());
                        self.line_idx += 1;
                        continue;
                    }
                }
                break;
            }
            let trimmed = l.trim_start();
            if leading_indent_columns(l) <= 3 && hr_regex().is_match(trimmed) {
                break;
            }
            if leading_indent_columns(l) <= 3 && trimmed.starts_with('#') && atx_heading_regex().is_match(trimmed) {
                break;
            }
            if is_list_continuation(l, marker_len) {
                let stripped = strip_indent_columns(l, marker_len);
                lines.push(stripped);
                self.line_idx += 1;
            } else if parse_list_marker(l).is_some() {
                break;
            } else {
                lines.push(l.to_string());
                self.line_idx += 1;
            }
        }
        (lines, loose)
    }

    fn parse_hr(&mut self) -> Option<Token> {
        let line = self.lines[self.line_idx].trim();
        if hr_regex().is_match(line) {
            self.line_idx += 1;
            return Some(Token::HorizontalRule);
        }
        None
    }

    fn parse_html_block(&mut self) -> Option<Token> {
        let line = &self.lines[self.line_idx];
        let indent = leading_spaces(line);
        if indent > 3 {
            return None;
        }
        let trimmed = line[indent..].trim_end();
        if !trimmed.starts_with('<') {
            return None;
        }
        let close_pat = html_block_close_pattern(trimmed);
        let is_type7 = html_block_type7_regex().is_match(trimmed);
        let is_block_html = html_block_start_regex().is_match(trimmed);
        if close_pat.is_none()
            && !is_block_html
            && !is_type7
            && !trimmed.starts_with("<!")
        {
            return None;
        }
        if close_pat.is_none() && is_type7 && !trimmed.ends_with('>') {
            return None;
        }
        if close_pat.is_none()
            && is_type7
            && !is_block_html
            && !is_standalone_html_tag(trimmed)
        {
            return None;
        }
        if close_pat.is_none()
            && is_type7
            && !is_block_html
            && has_same_line_paired_html_tag(trimmed)
        {
            return None;
        }
        let mut html_lines = vec![line.to_string()];
        self.line_idx += 1;

        if let Some(close) = close_pat {
            while self.line_idx < self.lines.len()
                && !html_lines.join("\n").to_ascii_lowercase().contains(close)
            {
                html_lines.push(self.lines[self.line_idx].to_string());
                self.line_idx += 1;
            }
        } else {
            while self.line_idx < self.lines.len() {
                let next = &self.lines[self.line_idx];
                if next.trim().is_empty() {
                    break;
                }
                html_lines.push(next.to_string());
                self.line_idx += 1;
            }
        }

        if self.line_idx < self.lines.len()
            && !html_lines.is_empty()
            && close_pat
                .map(|close| !html_lines.join("\n").to_ascii_lowercase().contains(close))
                .unwrap_or(false)
        {
            html_lines.push(self.lines[self.line_idx].to_string());
            self.line_idx += 1;
        }
        Some(Token::HtmlBlock {
            text: html_lines.join("\n") + "\n",
        })
    }

    fn parse_link_def(&mut self) -> Option<Token> {
        let mut line_buf = self.lines[self.line_idx].clone();
        if leading_spaces(&line_buf) > 3 {
            return None;
        }
        let mut consumed_label_lines = 1usize;
        let mut trimmed = line_buf.trim_start();
        if !trimmed.starts_with('[') {
            return None;
        }
        let mut close = find_link_label_end(trimmed);
        while close.is_none()
            && trimmed.starts_with('[')
            && self.line_idx + consumed_label_lines < self.lines.len()
        {
            line_buf.push('\n');
            line_buf.push_str(self.lines[self.line_idx + consumed_label_lines].trim_start());
            consumed_label_lines += 1;
            trimmed = line_buf.trim_start();
            close = find_link_label_end(trimmed);
        }
        let close = close?;
        if !trimmed[close..].starts_with("]:") {
            return None;
        }
        if trimmed[1..close].trim().is_empty() {
            return None;
        }
        if invalid_link_def_label(&trimmed[1..close]) {
            return None;
        }
        let label = normalize_link_label(&trimmed[1..close]);
        let mut rest = trimmed[close + 2..].trim_start().to_string();
        let mut consumed = consumed_label_lines;
        if rest.is_empty() && self.line_idx + consumed < self.lines.len() {
            rest = self.lines[self.line_idx + consumed].trim().to_string();
            consumed += 1;
        }
        let (href, remaining) = parse_link_def_destination(&rest)?;
        let mut title = None;
        if !remaining.is_empty() && !remaining.starts_with(char::is_whitespace) {
            return None;
        }
        let remaining = remaining.trim_start();
        if !remaining.is_empty() {
            if let Some((parsed_title, after_title)) = parse_link_def_title_with_rest(remaining) {
                if !after_title.trim().is_empty() {
                    return None;
                }
                title = Some(parsed_title);
            } else if let Some((parsed_title, extra_consumed)) =
                self.parse_multiline_link_def_title(remaining, consumed)
            {
                title = Some(parsed_title);
                consumed += extra_consumed;
            } else {
                return None;
            }
        }
        if title.is_none() && self.line_idx + consumed < self.lines.len() {
            let candidate = self.lines[self.line_idx + consumed].trim();
            if !candidate.is_empty() {
                if let Some((parsed_title, after_title)) = parse_link_def_title_with_rest(candidate) {
                    if after_title.trim().is_empty() {
                        title = Some(parsed_title);
                        consumed += 1;
                    }
                }
            }
        }
        self.line_idx += consumed;
        Some(Token::LinkDef {
            label,
            href,
            title,
        })
    }

    fn parse_multiline_link_def_title(
        &self,
        first_fragment: &str,
        consumed_before_title: usize,
    ) -> Option<(String, usize)> {
        let opener = first_fragment.chars().next()?;
        let close = match opener {
            '"' => '"',
            '\'' => '\'',
            '(' => ')',
            _ => return None,
        };
        if first_fragment[opener.len_utf8()..].contains(close) {
            return None;
        }
        let mut title = first_fragment[opener.len_utf8()..].to_string();
        let mut extra_consumed = 0usize;
        while self.line_idx + consumed_before_title + extra_consumed < self.lines.len() {
            let line = &self.lines[self.line_idx + consumed_before_title + extra_consumed];
            if line.trim().is_empty() {
                return None;
            }
            title.push('\n');
            if let Some(close_idx) = find_unescaped_char(line, close) {
                let after = &line[close_idx + close.len_utf8()..];
                if !after.trim().is_empty() {
                    return None;
                }
                title.push_str(&line[..close_idx]);
                return Some((unescape_punctuation(&title), extra_consumed + 1));
            }
            title.push_str(line);
            extra_consumed += 1;
        }
        None
    }

    fn parse_table(&mut self) -> Option<Token> {
        let line = &self.lines[self.line_idx];
        if !line.contains('|') {
            return None;
        }
        if self.line_idx + 1 >= self.lines.len() {
            return None;
        }
        let divider = &self.lines[self.line_idx + 1];
        if !table_divider_regex().is_match(divider.trim()) {
            return None;
        }
        let header_cells = parse_table_row(line);
        let align = parse_table_align(divider);
        let col_count = header_cells.len().max(align.len());
        let header: Vec<TableCell> = pad_cells(header_cells, col_count)
            .into_iter()
            .map(|raw| TableCell {
                raw: raw.clone(),
                tokens: Vec::new(),
            })
            .collect();
        self.line_idx += 2;
        let mut rows = Vec::new();
        while self.line_idx < self.lines.len() {
            let l = &self.lines[self.line_idx];
            if l.trim().is_empty() || !l.contains('|') {
                break;
            }
            let cells = pad_cells(parse_table_row(l), col_count);
            rows.push(
                cells
                    .into_iter()
                    .map(|raw| TableCell {
                        raw: raw.clone(),
                        tokens: Vec::new(),
                    })
                    .collect(),
            );
            self.line_idx += 1;
        }
        Some(Token::Table {
            align: pad_align(align, col_count),
            header,
            rows,
        })
    }

    fn parse_paragraph(&mut self) -> Option<Token> {
        let mut para_lines = vec![self.lines[self.line_idx].trim_start().to_string()];
        self.line_idx += 1;
        while self.line_idx < self.lines.len() {
            let l = &self.lines[self.line_idx];
            if l.trim().is_empty() {
                break;
            }
            if self.looks_like_block_start(l) {
                break;
            }
            para_lines.push(l.trim_start().to_string());
            self.line_idx += 1;
        }

        let raw = para_lines.join("\n");
        Some(Token::Paragraph {
            raw,
            tokens: Vec::new(),
        })
    }

    fn looks_like_block_start(&self, line: &str) -> bool {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            return true;
        }
        if leading_indent_columns(line) <= 3 && trimmed.starts_with('#') && atx_heading_regex().is_match(trimmed) {
            return true;
        }
        if leading_indent_columns(line) <= 3 && hr_regex().is_match(trimmed) {
            return true;
        }
        if list_marker_can_interrupt(line) {
            return true;
        }
        if strip_blockquote_prefix(line).is_some() {
            return true;
        }
        if line.trim().starts_with('<') && html_block_start_regex().is_match(line.trim()) {
            return !html_block_type7_regex().is_match(line.trim());
        }
        if self.options.gfm
            && line.contains('|')
            && self.line_idx + 1 < self.lines.len()
            && table_divider_regex().is_match(self.lines[self.line_idx + 1].trim())
        {
            return true;
        }
        false
    }
}

fn atx_heading_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^ {0,3}(#{1,6})(?:[ \t]+(.*)|[ \t]*)$").expect("valid regex"))
}

fn setext_h1_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^ {0,3}=+\s*$").expect("valid regex"))
}

fn setext_h2_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^ {0,3}-+\s*$").expect("valid regex"))
}

fn hr_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^ {0,3}(\*[\s\*]*){3,}$|^ {0,3}(-[\s-]*){3,}$|^ {0,3}(_[\s_]*){3,}$").expect("valid regex"))
}

fn html_block_start_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)^</?(address|article|aside|base|basefont|blockquote|body|caption|center|col|colgroup|dd|details|dialog|dir|div|dl|dt|fieldset|figcaption|figure|footer|form|frame|frameset|h[1-6]|head|header|hr|html|iframe|legend|li|link|main|menu|menuitem|nav|noframes|ol|optgroup|option|p|param|section|source|summary|table|tbody|td|tfoot|th|thead|title|tr|track|ul)(?:\s|>|/)")
            .expect("valid regex")
    })
}

fn html_block_type7_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(
        r"^(?:<[a-zA-Z][a-zA-Z0-9-]*(?:\s+[a-zA-Z_:][a-zA-Z0-9:._-]*(?:\s*=\s*(?:[^ \x22'=<>`]+|'[^']*'|\x22[^\x22]*\x22))?)*\s*/?>|</[a-zA-Z][a-zA-Z0-9-]*\s*>)[ \t]*$"
    ).unwrap())
}

fn table_divider_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\|?(\s*:?-+:?\s*\|)+\s*:?-+:?\s*\|?\s*$").unwrap())
}

fn strip_blockquote_prefix(line: &str) -> Option<&str> {
    if leading_indent_columns(line) > 3 {
        return None;
    }
    let trimmed = line.trim_start();
    if !trimmed.starts_with('>') {
        return None;
    }
    let after = trimmed[1..]
        .strip_prefix(' ')
        .or_else(|| trimmed[1..].strip_prefix('\t'))
        .unwrap_or(&trimmed[1..]);
    Some(after)
}

fn protect_lazy_continuation_marker(line: &str) -> String {
    if list_marker_can_interrupt(line)
        || hr_regex().is_match(line.trim())
        || setext_h1_regex().is_match(line)
        || setext_h2_regex().is_match(line)
    {
        let mut protected = String::with_capacity(line.len() + 1);
        protected.push('\\');
        protected.push_str(line);
        protected
    } else {
        line.to_string()
    }
}

fn blockquote_line_allows_lazy_continuation(line: &str) -> bool {
    if line.trim().is_empty() || leading_indent_columns(line) >= 4 {
        return false;
    }
    let trimmed = line.trim_start();
    if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
        return false;
    }
    if atx_heading_regex().is_match(line)
        || hr_regex().is_match(trimmed)
        || list_marker_can_interrupt(line)
    {
        return false;
    }
    if trimmed.starts_with('<') && html_block_start_regex().is_match(trimmed) {
        return false;
    }
    true
}

fn parse_list_marker(line: &str) -> Option<(bool, usize, u64)> {
    let expanded = expand_leading_tabs(line);
    let indent = leading_indent_columns(&expanded);
    if indent > 3 {
        return None;
    }
    let trimmed = &expanded[indent..];
    let mut chars = trimmed.chars();

    if let Some(_marker @ ('-' | '*' | '+')) = chars.next() {
        let after = &trimmed[1..];
        if after.is_empty() {
            return Some((false, indent + 2, 1));
        }
        let spaces: usize = after.chars().take_while(|&c| c == ' ').count();
        if spaces > 0 {
            let marker_len = if spaces >= 5 {
                indent + 2
            } else {
                indent + 1 + spaces
            };
            return Some((false, marker_len, 1));
        }
    }

    let mut chars = trimmed.chars();
    if let Some(first_digit) = chars.next() {
        if first_digit.is_ascii_digit() {
            let mut num_str = String::new();
            num_str.push(first_digit);
            let mut marker_char = None;
            for ch in chars {
                if ch.is_ascii_digit() {
                    if num_str.len() >= 9 {
                        return None;
                    }
                    num_str.push(ch);
                } else if ch == '.' || ch == ')' {
                    marker_char = Some(ch);
                    break;
                } else {
                    return None;
                }
            }
            if let Some(_m) = marker_char {
                let after = &trimmed[num_str.len() + 1..];
                let num = num_str.parse::<u64>().unwrap_or(1);
                if after.is_empty() {
                    return Some((true, indent + num_str.len() + 2, num));
                }
                let spaces: usize = after.chars().take_while(|&c| c == ' ').count();
                if spaces > 0 {
                    let marker_len = if spaces >= 5 {
                        indent + num_str.len() + 2
                    } else {
                        indent + num_str.len() + 1 + spaces
                    };
                    return Some((true, marker_len, num));
                }
            }
        }
    }
    None
}

fn unordered_marker_char(line: &str) -> Option<char> {
    let trimmed = line.trim_start();
    if (trimmed.starts_with('*') || trimmed.starts_with('-') || trimmed.starts_with('+'))
        && (trimmed.len() == 1 || trimmed[1..].starts_with([' ', '\t']))
    {
        return Some(trimmed.chars().next().expect("checked starts_with"));
    }
    None
}

fn ordered_marker_delim(line: &str) -> Option<char> {
    line.trim_start()
        .chars()
        .find(|ch| matches!(ch, '.' | ')'))
}

fn is_list_continuation(line: &str, marker_len: usize) -> bool {
    let trimmed = line.trim_start();
    let indent = leading_indent_columns(line) - leading_indent_columns(trimmed);
    indent >= marker_len
}

fn list_marker_can_interrupt(line: &str) -> bool {
    let Some((ordered, marker_len, start)) = parse_list_marker(line) else {
        return false;
    };
    if ordered && start != 1 {
        return false;
    }
    line.get(marker_len..)
        .map(|rest| !rest.trim().is_empty())
        .unwrap_or(false)
}

fn parse_task_item(line: &str) -> (bool, bool) {
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
        let _ = rest;
        return (true, false);
    }
    if trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ") {
        return (true, true);
    }
    (false, false)
}

fn append_to_last_paragraph(tok: &mut Token, text: &str) {
    if let Token::Paragraph { raw, .. } = tok {
        raw.push('\n');
        raw.push_str(text);
    }
}

fn html_block_close_pattern(line: &str) -> Option<&'static str> {
    let lower = line.to_ascii_lowercase();
    if lower.starts_with("<script") {
        Some("</script>")
    } else if lower.starts_with("<pre") {
        Some("</pre>")
    } else if lower.starts_with("<style") {
        Some("</style>")
    } else if lower.starts_with("<textarea") {
        Some("</textarea>")
    } else if lower.starts_with("<!--") {
        Some("-->")
    } else if lower.starts_with("<?") {
        Some("?>")
    } else if lower.starts_with("<![cdata[") {
        Some("]]>")
    } else {
        None
    }
}

fn has_same_line_paired_html_tag(line: &str) -> bool {
    if line.starts_with("</") {
        return false;
    }
    let Some(name_end) = line[1..]
        .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '-'))
        .map(|idx| idx + 1)
    else {
        return false;
    };
    let name = &line[1..name_end];
    let close = format!("</{name}>");
    line[name_end..].to_ascii_lowercase().contains(&close.to_ascii_lowercase())
}

fn is_standalone_html_tag(line: &str) -> bool {
    let mut in_quote = None;
    for (idx, ch) in line.char_indices() {
        match in_quote {
            Some(q) if ch == q => in_quote = None,
            None => {
                if ch == '"' || ch == '\'' {
                    in_quote = Some(ch);
                } else if ch == '>' {
                    return line[idx + ch.len_utf8()..].trim().is_empty();
                }
            }
            _ => {}
        }
    }
    false
}

fn parse_table_row(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let inner = trimmed.strip_prefix('|').unwrap_or(trimmed);
    let inner = inner.strip_suffix('|').unwrap_or(inner);
    inner.split('|').map(|c| c.trim().to_string()).collect()
}

fn parse_table_align(divider: &str) -> Vec<Option<Alignment>> {
    let trimmed = divider.trim();
    let inner = trimmed.strip_prefix('|').unwrap_or(trimmed);
    let inner = inner.strip_suffix('|').unwrap_or(inner);
    inner
        .split('|')
        .map(|cell| {
            let c = cell.trim();
            let left = c.starts_with(':');
            let right = c.ends_with(':');
            match (left, right) {
                (true, true) => Some(Alignment::Center),
                (false, true) => Some(Alignment::Right),
                (true, false) => Some(Alignment::Left),
                _ => None,
            }
        })
        .collect()
}

fn pad_cells(mut cells: Vec<String>, count: usize) -> Vec<String> {
    while cells.len() < count {
        cells.push(String::new());
    }
    cells.truncate(count);
    cells
}

fn pad_align(mut align: Vec<Option<Alignment>>, count: usize) -> Vec<Option<Alignment>> {
    while align.len() < count {
        align.push(None);
    }
    align.truncate(count);
    align
}

fn leading_spaces(line: &str) -> usize {
    line.as_bytes().iter().take_while(|b| **b == b' ').count()
}

fn leading_indent_columns(line: &str) -> usize {
    let mut col = 0usize;
    for ch in line.chars() {
        match ch {
            ' ' => col += 1,
            '\t' => col += 4 - (col % 4),
            _ => break,
        }
    }
    col
}

fn expand_leading_tabs(line: &str) -> String {
    let mut out = String::with_capacity(line.len() + 8);
    let mut col = 0;
    for (idx, ch) in line.char_indices() {
        if ch == ' ' {
            out.push(' ');
            col += 1;
        } else if ch == '\t' {
            let tab_stop = col + 4 - (col % 4);
            let spaces = tab_stop - col;
            for _ in 0..spaces {
                out.push(' ');
            }
            col = tab_stop;
        } else {
            out.push_str(&line[idx..]);
            return out;
        }
    }
    out
}

fn strip_indent_columns(line: &str, columns: usize) -> String {
    let expanded = expand_leading_tabs(line);
    for (col, (idx, _ch)) in expanded.char_indices().enumerate() {
        if col >= columns {
            return expanded[idx..].to_string();
        }
    }
    String::new()
}

fn strip_up_to_n_spaces(line: &str, n: usize) -> &str {
    let strip = leading_spaces(line).min(n);
    &line[strip..]
}

fn strip_atx_closing_hashes(raw: &str) -> &str {
    let trimmed = raw.trim();
    let without_hashes = trimmed.trim_end_matches('#');
    if without_hashes == trimmed {
        return trimmed;
    }
    if without_hashes.is_empty() {
        return "";
    }
    if without_hashes.ends_with(' ') || without_hashes.ends_with('\t') {
        without_hashes.trim_end()
    } else {
        trimmed
    }
}



fn find_link_label_end(s: &str) -> Option<usize> {
    let mut escaped = false;
    for (idx, ch) in s.char_indices().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
        } else if ch == ']' {
            return Some(idx);
        }
    }
    None
}

fn normalize_link_label(s: &str) -> String {
    let lowered = unescape_punctuation(s)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    lowered.replace('ß', "ss")
}

fn invalid_link_def_label(label: &str) -> bool {
    let mut chars = label.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            let _ = chars.next();
        } else if matches!(ch, '[' | ']') {
            return true;
        }
    }
    false
}

fn parse_link_def_destination(rest: &str) -> Option<(String, &str)> {
    let rest = rest.trim_start();
    if let Some(after_open) = rest.strip_prefix('<') {
        if let Some(end) = after_open.find('>') {
            return Some((unescape_punctuation(&after_open[..end]), &after_open[end + 1..]));
        }
        return None;
    }
    if rest.is_empty() {
        return None;
    }
    let end = rest
        .char_indices()
        .find(|(_, ch)| ch.is_whitespace())
        .map(|(idx, _)| idx)
        .unwrap_or(rest.len());
    Some((unescape_punctuation(&rest[..end]), &rest[end..]))
}

fn parse_link_def_title_with_rest(rest: &str) -> Option<(String, &str)> {
    let first = rest.chars().next()?;
    let close = match first {
        '"' => '"',
        '\'' => '\'',
        '(' => ')',
        _ => return None,
    };
    let start = first.len_utf8();
    let mut escaped = false;
    for (idx, ch) in rest[start..].char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
        } else if ch == close {
            let end = start + idx + ch.len_utf8();
            return Some((unescape_punctuation(&rest[start..start + idx]), &rest[end..]));
        }
    }
    None
}

fn find_unescaped_char(s: &str, needle: char) -> Option<usize> {
    let mut escaped = false;
    for (idx, ch) in s.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
        } else if ch == needle {
            return Some(idx);
        }
    }
    None
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


