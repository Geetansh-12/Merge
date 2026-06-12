use crate::escape::{decode_entities, encode_href, escape_html};
use crate::options::Options;
use crate::token::{Alignment, InlineToken, ListItem, Token};

pub trait Renderer {
    fn render_tokens(&mut self, tokens: &[Token]) -> String;
    fn render_inline(&mut self, tokens: &[InlineToken]) -> String;
}

pub struct HtmlRenderer {
    options: Options,
}

impl HtmlRenderer {
    pub fn new(options: Options) -> Self {
        Self { options }
    }
}

impl Renderer for HtmlRenderer {
    fn render_tokens(&mut self, tokens: &[Token]) -> String {
        let mut out = String::new();
        let mut i = 0;
        while i < tokens.len() {
            match &tokens[i] {
                Token::Space => {
                    i += 1;
                }
                Token::LinkDef { .. } => {
                    i += 1;
                }
                _ => {
                    out.push_str(&self.render_block(&tokens[i]));
                    i += 1;
                }
            }
        }
        out
    }

    fn render_inline(&mut self, tokens: &[InlineToken]) -> String {
        let mut out = String::new();
        for tok in tokens {
            out.push_str(&self.render_inline_token(tok));
        }
        out
    }
}

impl HtmlRenderer {
    fn render_block(&mut self, token: &Token) -> String {
        match token {
            Token::Heading { depth, tokens, .. } => {
                format!(
                    "<h{depth}>{}</h{depth}>\n",
                    self.render_inline(tokens)
                )
            }
            Token::Paragraph { tokens, .. } => {
                format!("<p>{}</p>\n", self.render_inline(tokens))
            }
            Token::Blockquote { tokens, .. } => {
                format!("<blockquote>\n{}</blockquote>\n", self.render_tokens(tokens))
            }
            Token::List { ordered, start, loose, items } => {
                self.render_list(*ordered, *start, *loose, items)
            }
            Token::CodeBlock { lang, text } => {
                if let Some(l) = lang {
                    if !l.is_empty() {
                        return format!(
                            "<pre><code class=\"language-{}\">{}</code></pre>\n",
                            escape_html(&decode_entities(l)),
                            escape_html(text)
                        );
                    }
                }
                format!("<pre><code>{}</code></pre>\n", escape_html(text))
            }
            Token::Table { align, header, rows } => self.render_table(align, header, rows),
            Token::HtmlBlock { text } => text.clone(), // clone needed because HtmlBlock owns String

            Token::HorizontalRule => "<hr />\n".to_string(),
            Token::Space => "\n".to_string(),
            Token::LinkDef { .. } => String::new(),
        }
    }

    fn render_list(&mut self, ordered: bool, start: u64, loose: bool, items: &[ListItem]) -> String {
        let tag = if ordered { "ol" } else { "ul" };
        let mut out = String::new();
        if ordered && start != 1 {
            out.push_str(&format!("<{tag} start=\"{start}\">\n"));
        } else {
            out.push_str(&format!("<{tag}>\n"));
        }
        for item in items {
            out.push_str("<li");
            if item.task {
                out.push_str(" class=\"task-list-item\"");
            }
            out.push('>');
            if item.task {
                let checked = if item.checked { " checked=\"\"" } else { "" };
                out.push_str(&format!("<input type=\"checkbox\" disabled=\"\"{checked}>"));
            }
            let all_space = item.tokens.iter().all(|t| matches!(t, Token::Space));
            if all_space {
                out.push_str("</li>\n");
            } else {
                let item_loose = loose || item.loose;
                if item_loose {
                    out.push('\n');
                    out.push_str(&self.render_tokens(&item.tokens));
                } else {
                    out.push_str(&self.render_list_item_tight(&item.tokens));
                }
                out.push_str("</li>\n");
            }
        }
        out.push_str(&format!("</{tag}>\n"));
        out
    }

    fn render_list_item_tight(&mut self, tokens: &[Token]) -> String {
        let mut out = String::new();
        for tok in tokens {
            match tok {
                Token::Paragraph { tokens: inline, .. } => {
                    out.push_str(&self.render_inline(inline));
                }
                Token::CodeBlock { text, .. } => {
                    if out.is_empty() {
                        out.push('\n');
                    }
                    out.push_str(&format!("<pre><code>{}</code></pre>\n", escape_html(text)));
                }
                Token::Blockquote { tokens: inner, .. } => {
                    if out.is_empty() {
                        out.push('\n');
                    }
                    out.push_str(&format!("<blockquote>\n{}</blockquote>\n", self.render_tokens(inner)));
                }
                other => {
                    if out.is_empty() {
                        out.push('\n');
                    }
                    out.push_str(&self.render_block(other));
                }
            }
        }
        out
    }

    fn render_table(
        &mut self,
        align: &[Option<Alignment>],
        header: &[crate::token::TableCell],
        rows: &[Vec<crate::token::TableCell>],
    ) -> String {
        let mut out = String::from("<table>\n<thead>\n<tr>\n");
        for (i, cell) in header.iter().enumerate() {
            out.push_str(&self.render_table_cell("th", cell, align.get(i).copied().flatten()));
        }
        out.push_str("</tr>\n</thead>\n");
        if !rows.is_empty() {
            out.push_str("<tbody><tr>\n");
            for (r, row) in rows.iter().enumerate() {
                if r > 0 {
                    out.push_str("<tr>\n");
                }
                for (i, cell) in row.iter().enumerate() {
                    out.push_str(&self.render_table_cell("td", cell, align.get(i).copied().flatten()));
                }
                out.push_str("</tr>\n");
            }
            out.push_str("</tbody></table>\n");
        } else {
            out.push_str("</table>\n");
        }
        out
    }

    fn render_table_cell(
        &mut self,
        tag: &str,
        cell: &crate::token::TableCell,
        align: Option<Alignment>,
    ) -> String {
        let align_attr = match align {
            Some(Alignment::Left) => " align=\"left\"",
            Some(Alignment::Center) => " align=\"center\"",
            Some(Alignment::Right) => " align=\"right\"",
            None => "",
        };
        format!(
            "<{tag}{align_attr}>{}</{tag}>\n",
            self.render_inline(&cell.tokens)
        )
    }

    fn render_inline_token(&mut self, token: &InlineToken) -> String {
        match token {
            InlineToken::Text(s) => escape_html(&decode_entities(s)),
            InlineToken::Escape(c) => escape_html(&c.to_string()),
            InlineToken::CodeSpan(s) => format!("<code>{}</code>", escape_html(s)),
            InlineToken::Em(tokens) => {
                format!("<em>{}</em>", self.render_inline(tokens))
            }
            InlineToken::Strong(tokens) => {
                format!("<strong>{}</strong>", self.render_inline(tokens))
            }
            InlineToken::StrongEm(tokens) => {
                format!("<em><strong>{}</strong></em>", self.render_inline(tokens))
            }
            InlineToken::Link { href, title, tokens } => {
                let decoded_href = decode_entities(href);
                let href_esc = encode_href(&decoded_href);
                let title_attr = title
                    .as_ref()
                    .map(|t| format!(" title=\"{}\"", escape_html(&decode_entities(t))))
                    .unwrap_or_default();
                format!(
                    "<a href=\"{}\"{title_attr}>{}</a>",
                    escape_html(&href_esc),
                    self.render_inline(tokens)
                )
            }
            InlineToken::Image { href, title, alt } => {
                let decoded_href = decode_entities(href);
                let href_esc = encode_href(&decoded_href);
                let title_attr = title
                    .as_ref()
                    .map(|t| format!(" title=\"{}\"", escape_html(&decode_entities(t))))
                    .unwrap_or_default();
                format!(
                    "<img src=\"{}\" alt=\"{}\"{title_attr} />",
                    escape_html(&href_esc),
                    escape_html(alt)
                )
            }
            InlineToken::Autolink { href, text, is_email } => {
                let href_esc = if *is_email {
                    format!("mailto:{text}")
                } else {
                    encode_href(href)
                };
                format!(
                    "<a href=\"{}\">{}</a>",
                    escape_html(&href_esc),
                    escape_html(text)
                )
            }
            InlineToken::RawHtml(s) => s.clone(), // clone needed because RawHtml owns String
            InlineToken::HardBreak => "<br />\n".to_string(),
            InlineToken::SoftBreak => {
                if self.options.breaks {
                    "<br />\n".to_string()
                } else {
                    "\n".to_string()
                }
            }
            InlineToken::Del(tokens) => {
                format!("<del>{}</del>", self.render_inline(tokens))
            }
        }
    }
}