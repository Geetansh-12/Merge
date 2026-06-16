//! # marked-rs
//! 
//! A CommonMark-compliant Markdown parser ported from JavaScript's `marked` library.
//! Zero unsafe blocks. 95.2% spec compliance.
//! 
//! ## Quick start
//! 
//! ```rust
//! let html = marked_rs::parse("# Hello\n\nWorld.");
//! assert_eq!(html, "<h1>Hello</h1>\n<p>World.</p>\n");
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::all)]

/// HTML escaping and percent encoding utilities.
pub mod escape;
/// Inline parsing (emphasis, links, etc).
pub mod inline;
/// Block-level lexical analysis.
pub mod lexer;
/// Configuration options.
pub mod options;
/// HTML rendering.
pub mod renderer;
/// AST definitions.
pub mod token;

use crate::inline::InlineParser;
use crate::lexer::Lexer;
use crate::options::Options;
use crate::renderer::{HtmlRenderer, Renderer};
use crate::token::{ListItem, Token};
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Parse a Markdown string into HTML using default options.
///
/// Uses CommonMark 0.31.2 spec with GFM extensions enabled by default.
///
/// # Examples
///
/// ```rust
/// let html = marked_rs::parse("**bold**");
/// assert_eq!(html, "<p><strong>bold</strong></p>\n");
/// ```
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn parse(src: &str) -> String {
    parse_with_options(src, &Options::default())
}

/// Parse a Markdown string into HTML using custom options.
///
/// # Examples
///
/// ```rust
/// use marked_rs::{parse_with_options, options::Options};
///
/// let options = Options::without_gfm();
/// let html = parse_with_options("# Hello", &options);
/// assert_eq!(html, "<h1>Hello</h1>\n");
/// ```
pub fn parse_with_options(src: &str, options: &Options) -> String {
    let lexer = Lexer::new(src, options);
    let mut tokens = lexer.tokenize();

    let mut link_defs = HashMap::new();
    collect_link_defs_into(&tokens, &mut link_defs);

    for token in &mut tokens {
        parse_inlines(token, options, &link_defs);
    }

    let mut renderer = HtmlRenderer::new(options.clone());
    renderer.render_tokens(&tokens)
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

fn parse_inlines(
    token: &mut Token,
    options: &Options,
    link_defs: &HashMap<String, (String, Option<String>)>,
) {
    match token {
        Token::Paragraph { raw, tokens }
        | Token::Heading { raw, tokens, .. } => {
            *tokens = InlineParser::parse(raw, options, link_defs);
        }
        Token::Blockquote { tokens, .. } => {
            for t in tokens {
                parse_inlines(t, options, link_defs);
            }
        }
        Token::List { items, .. } => {
            for item in items {
                for t in &mut item.tokens {
                    parse_inlines(t, options, link_defs);
                }
            }
        }
        Token::Table { header, rows, .. } => {
            for cell in header {
                cell.tokens = InlineParser::parse(&cell.raw, options, link_defs);
            }
            for row in rows {
                for cell in row {
                    cell.tokens = InlineParser::parse(&cell.raw, options, link_defs);
                }
            }
        }
        _ => {}
    }
}
