#![forbid(unsafe_code)]
pub mod escape;
pub mod inline;
pub mod lexer;
pub mod options;
pub mod renderer;
pub mod token;

use crate::inline::InlineParser;
use crate::lexer::Lexer;
use crate::options::Options;
use crate::renderer::{HtmlRenderer, Renderer};
use crate::token::{ListItem, Token};
use std::collections::HashMap;

/// Parse markdown to HTML using default options.
pub fn parse(src: &str) -> String {
    parse_with_options(src, &Options::default())
}

/// Parse markdown to HTML with custom options.
pub fn parse_with_options(src: &str, options: &Options) -> String {
    let block_tokens = Lexer::new(src, options).tokenize();
    let link_defs = collect_link_defs(&block_tokens);
    let resolved = resolve_inline(block_tokens, options, &link_defs);
    let mut renderer = HtmlRenderer::new(options.clone());
    renderer.render_tokens(&resolved)
}

fn collect_link_defs(tokens: &[Token]) -> HashMap<String, (String, Option<String>)> {
    let mut defs = HashMap::new();
    collect_link_defs_into(tokens, &mut defs);
    defs
}

fn collect_link_defs_into(
    tokens: &[Token],
    defs: &mut HashMap<String, (String, Option<String>)>,
) {
    for tok in tokens {
        match tok {
            Token::LinkDef { label, href, title } => {
                defs.entry(label.clone())
                    .or_insert_with(|| (href.clone(), title.clone()));
            }
            Token::Blockquote { tokens, .. } => collect_link_defs_into(tokens, defs),
            Token::List { items, .. } => {
                for item in items {
                    collect_link_defs_into(&item.tokens, defs);
                }
            }
            _ => {}
        }
    }
}

fn resolve_inline(
    tokens: Vec<Token>,
    options: &Options,
    link_defs: &HashMap<String, (String, Option<String>)>,
) -> Vec<Token> {
    tokens
        .into_iter()
        .map(|tok| resolve_token(tok, options, link_defs))
        .collect()
}

fn resolve_token(
    tok: Token,
    options: &Options,
    link_defs: &HashMap<String, (String, Option<String>)>,
) -> Token {
    match tok {
        Token::Heading { depth, raw, .. } => Token::Heading {
            depth,
            tokens: InlineParser::parse(&raw, options, link_defs),
            raw,
        },
        Token::Paragraph { raw, .. } => Token::Paragraph {
            tokens: InlineParser::parse(&raw, options, link_defs),
            raw,
        },
        Token::Blockquote { raw, tokens } => Token::Blockquote {
            raw,
            tokens: resolve_inline(tokens, options, link_defs),
        },
        Token::List {
            ordered,
            start,
            loose,
            items,
        } => Token::List {
            ordered,
            start,
            loose,
            items: items
                .into_iter()
                .map(|item| ListItem {
                    task: item.task,
                    checked: item.checked,
                    loose: item.loose,
                    tokens: resolve_inline(item.tokens, options, link_defs),
                })
                .collect(),
        },
        Token::Table {
            align,
            header,
            rows,
        } => Token::Table {
            align,
            header: header
                .into_iter()
                .map(|cell| crate::token::TableCell {
                    tokens: InlineParser::parse(&cell.raw, options, link_defs),
                    raw: cell.raw,
                })
                .collect(),
            rows: rows
                .into_iter()
                .map(|row| {
                    row.into_iter()
                        .map(|cell| crate::token::TableCell {
                            tokens: InlineParser::parse(&cell.raw, options, link_defs),
                            raw: cell.raw,
                        })
                        .collect()
                })
                .collect(),
        },
        other => other,
    }
}
