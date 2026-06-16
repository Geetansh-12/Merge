#[test]
fn deeply_nested_blockquotes_no_panic() {
    let input: String = (0..100).map(|_| "> ").collect::<String>() + "text";
    let _ = marked_rs::parse(&input);
}

#[test]
fn degenerate_link_definition_no_url() {
    let got = marked_rs::parse("[label]:\n");
    assert!(!got.contains("<a"));
}

#[test]
fn table_mismatched_columns() {
    let input = "| a | b | c |\n|---|---|\n| only | two | cols | here |\n| 1 |\n";
    let got = marked_rs::parse(input);
    let td_count = got.matches("<td>").count();
    assert_eq!(td_count % 3, 0);
}

#[test]
fn fenced_code_backticks_in_info_string() {
    let input = "```rust `code`\nfn main() {}\n```";
    let got = marked_rs::parse(input);
    assert!(got.contains("fn main()"));
    assert!(!got.contains("language-rust"));
}

#[test]
fn setext_heading_after_list_item() {
    let input = "- item\nheading\n=======\n";
    let got = marked_rs::parse(input);
    assert!(got.contains("<h1>item\nheading</h1>"));
}

#[test]
fn link_titles_all_delimiters() {
    let d = marked_rs::parse("[text](href \"double quoted title\")");
    assert!(d.contains("title=\"double quoted title\""));
    let s = marked_rs::parse("[text](href 'single quoted title')");
    assert!(s.contains("title=\"single quoted title\""));
    let p = marked_rs::parse("[text](href (paren title))");
    assert!(p.contains("title=\"paren title\""));
}

#[test]
fn autolink_unicode() {
    let got = marked_rs::parse("<https://例え.jp>");
    assert!(got.contains("<a href="));
    assert!(got.contains("例え"));
}

#[test]
fn hard_break_after_emphasis() {
    let got = marked_rs::parse("*em*  \nnext line");
    assert!(got.contains("<em>em</em>"));
    assert!(got.contains("<br />"));
}

#[test]
fn backslash_escapes_in_code_spans() {
    let got = marked_rs::parse(r"`\*not escaped\*`");
    assert_eq!(got.trim(), "<p><code>\\*not escaped\\*</code></p>");
}

#[test]
fn emphasis_inside_link_labels() {
    let got = marked_rs::parse("[**bold link**](href)");
    assert!(got.contains("<strong>bold link</strong>"));
}

#[test]
fn inline_link_after_text_is_not_dropped() {
    let got = marked_rs::parse("before [text](href) after");
    assert!(got.contains("before <a href=\"href\">text</a> after"));
}

#[test]
fn image_syntax_renders_image() {
    let got = marked_rs::parse("![alt text](img.png \"title\")");
    assert_eq!(
        got.trim(),
        "<p><img src=\"img.png\" alt=\"alt text\" title=\"title\" /></p>"
    );
}

#[test]
fn reference_link_titles_all_delimiters() {
    let double = marked_rs::parse("[text][ref]\n\n[ref]: href \"double\"");
    assert!(double.contains("title=\"double\""));

    let single = marked_rs::parse("[text][ref]\n\n[ref]: href 'single'");
    assert!(single.contains("title=\"single\""));

    let paren = marked_rs::parse("[text][ref]\n\n[ref]: href (paren)");
    assert!(paren.contains("title=\"paren\""));
}

#[test]
fn reference_definition_inside_blockquote_is_visible() {
    let input = "[foo]\n\n> [foo]: /url\n";
    let got = marked_rs::parse(input);
    assert!(got.contains("<a href=\"/url\">foo</a>"));
    assert!(got.contains("<blockquote>"));
}

#[test]
fn blockquote_lazy_continuation_stays_inside_quote() {
    let got = marked_rs::parse("> foo\nbar\n");
    assert_eq!(got.trim(), "<blockquote>\n<p>foo\nbar</p>\n</blockquote>");
}

#[test]
fn indented_blockquote_lazy_continuation_stays_paragraph_text() {
    let got = marked_rs::parse("> foo\n    - bar\n");
    assert_eq!(got.trim(), "<blockquote>\n<p>foo\n- bar</p>\n</blockquote>");
}

#[test]
fn paragraph_breaks_before_blockquote() {
    let got = marked_rs::parse("foo\n> bar\n");
    assert_eq!(got.trim(), "<p>foo</p>\n<blockquote>\n<p>bar</p>\n</blockquote>");
}

#[test]
fn blank_line_separates_blockquotes() {
    let got = marked_rs::parse("> foo\n\n> bar\n");
    assert_eq!(
        got.trim(),
        "<blockquote>\n<p>foo</p>\n</blockquote>\n<blockquote>\n<p>bar</p>\n</blockquote>"
    );
}

#[test]
fn indented_thematic_break_after_paragraph_is_text() {
    let got = marked_rs::parse("Foo\n    ***\n");
    assert_eq!(got.trim(), "<p>Foo\n***</p>");
}

#[test]
fn indented_code_keeps_blank_lines_between_chunks() {
    let got = marked_rs::parse("    chunk1\n\n    chunk2\n  \n \n \n    chunk3\n");
    assert_eq!(
        got.trim(),
        "<pre><code>chunk1\n\nchunk2\n\n\n\nchunk3\n</code></pre>"
    );
}

#[test]
fn indented_code_preserves_extra_spaces_on_blank_lines() {
    let got = marked_rs::parse("    chunk1\n      \n      chunk2\n");
    assert_eq!(got.trim(), "<pre><code>chunk1\n  \n  chunk2\n</code></pre>");
}

#[test]
fn thematic_break_between_list_items_ends_list() {
    let got = marked_rs::parse("* Foo\n* * *\n* Bar\n");
    assert_eq!(
        got.trim(),
        "<ul>\n<li>Foo</li>\n</ul>\n<hr />\n<ul>\n<li>Bar</li>\n</ul>"
    );
}

#[test]
fn thematic_break_as_list_item_renders_on_own_line() {
    let got = marked_rs::parse("- Foo\n- * * *\n");
    assert_eq!(got.trim(), "<ul>\n<li>Foo</li>\n<li>\n<hr />\n</li>\n</ul>");
}

#[test]
fn lazy_setext_underline_in_blockquote_stays_text() {
    let got = marked_rs::parse("> foo\nbar\n===\n");
    assert_eq!(got.trim(), "<blockquote>\n<p>foo\nbar\n===</p>\n</blockquote>");
}

#[test]
fn reference_labels_use_unicode_case_folding_for_sharp_s() {
    let got = marked_rs::parse("[ẞ]\n\n[SS]: /url\n");
    assert_eq!(got.trim(), "<p><a href=\"/url\">ẞ</a></p>");
}

#[test]
fn single_trailing_space_before_softbreak_is_trimmed() {
    let got = marked_rs::parse("[foo] \n[]\n\n[foo]: /url \"title\"\n");
    assert_eq!(
        got.trim(),
        "<p><a href=\"/url\" title=\"title\">foo</a>\n[]</p>"
    );
}

#[test]
fn outer_inline_link_with_inner_link_stays_literal() {
    let got = marked_rs::parse("[foo [bar](/uri)](/uri)\n");
    assert_eq!(got.trim(), "<p>[foo <a href=\"/uri\">bar</a>](/uri)</p>");
}

#[test]
fn outer_reference_link_with_inner_link_stays_literal() {
    let got = marked_rs::parse("[foo [bar](/uri)][ref]\n\n[ref]: /uri\n");
    assert_eq!(got.trim(), "<p>[foo <a href=\"/uri\">bar</a>]<a href=\"/uri\">ref</a></p>");
}

#[test]
fn link_label_does_not_close_inside_code_span() {
    let got = marked_rs::parse("[not a `link](/foo`)\n");
    assert_eq!(got.trim(), "<p>[not a <code>link](/foo</code>)</p>");
}

#[test]
fn link_label_does_not_close_inside_inline_html() {
    let got = marked_rs::parse("[foo <bar attr=\"](baz)\">\n");
    assert_eq!(got.trim(), "<p>[foo <bar attr=\"](baz)\"></p>");
}

#[test]
fn link_label_does_not_close_inside_autolink() {
    let got = marked_rs::parse("[foo<https://example.com/?search=](uri)>\n");
    assert_eq!(
        got.trim(),
        "<p>[foo<a href=\"https://example.com/?search=%5D(uri)\">https://example.com/?search=](uri)</a></p>"
    );
}

#[test]
fn image_alt_text_preserves_nested_link_visible_text() {
    let got = marked_rs::parse("![[[foo](uri1)](uri2)](uri3)\n");
    assert_eq!(got.trim(), "<p><img src=\"uri3\" alt=\"[foo](uri2)\" /></p>");
}

#[test]
fn escaped_explicit_reference_label_does_not_match() {
    let got = marked_rs::parse("[bar][foo\\!]\n\n[foo!]: /url\n");
    assert_eq!(got.trim(), "<p>[bar][foo!]</p>");
}

#[test]
fn reference_definition_label_with_bracket_is_paragraph() {
    let got = marked_rs::parse("[foo][ref[]\n\n[ref[]: /uri\n");
    assert_eq!(got.trim(), "<p>[foo][ref[]</p>\n<p>[ref[]: /uri</p>");
}

#[test]
fn escaped_bracket_reference_label_matches() {
    let got = marked_rs::parse("[foo][ref\\[]\n\n[ref\\[]: /uri\n");
    assert_eq!(got.trim(), "<p><a href=\"/uri\">foo</a></p>");
}

#[test]
fn escaped_closing_bracket_reference_definition_matches() {
    let got = marked_rs::parse("[Foo*bar\\]]:my_(url) 'title (with parens)'\n\n[Foo*bar\\]]\n");
    assert_eq!(
        got.trim(),
        "<p><a href=\"my_(url)\" title=\"title (with parens)\">Foo*bar]</a></p>"
    );
}

#[test]
fn undefined_reference_link_literal() {
    let got = marked_rs::parse("[text][undefined-ref]");
    assert!(got.contains("[text][undefined-ref]") || got.contains("[undefined-ref]"));
}

#[test]
fn exponential_blowup_protection() {
    let input = "[".repeat(1000) + "a" + &"]".repeat(1000);
    let start = std::time::Instant::now();
    let _ = marked_rs::parse(&input);
    assert!(start.elapsed().as_secs() < 1);
}
