/// A block-level token produced by the Lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// ATX heading (# h1) or setext heading (underline style)
    Heading {
        depth: u8,
        raw: String,
        tokens: Vec<InlineToken>,
    },
    /// Paragraph — catch-all block
    Paragraph {
        raw: String,
        tokens: Vec<InlineToken>,
    },
    /// > blockquote — recursively contains block tokens
    Blockquote {
        raw: String,
        tokens: Vec<Token>,
    },
    /// Ordered or unordered list
    List {
        ordered: bool,
        start: u64,
        loose: bool,
        items: Vec<ListItem>,
    },
    /// Fenced or indented code block
    CodeBlock {
        lang: Option<String>,
        text: String,
    },
    /// GFM table
    Table {
        align: Vec<Option<Alignment>>,
        header: Vec<TableCell>,
        rows: Vec<Vec<TableCell>>,
    },
    /// Raw HTML block (CommonMark types 1–7)
    HtmlBlock { text: String },
    /// Thematic break (---, ***, ___)
    HorizontalRule,
    /// Link reference definition [label]: url "title"
    LinkDef {
        label: String,
        href: String,
        title: Option<String>,
    },
    /// Blank lines between blocks
    Space,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    pub task: bool,
    pub checked: bool,
    pub loose: bool,
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableCell {
    pub raw: String,
    pub tokens: Vec<InlineToken>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

/// An inline-level token produced by the InlineParser.
#[derive(Debug, Clone, PartialEq)]
pub enum InlineToken {
    /// Plain text run
    Text(String),
    /// Backslash-escaped character: \* → literal *
    Escape(char),
    /// `code span`
    CodeSpan(String),
    /// *emphasis* or _emphasis_
    Em(Vec<InlineToken>),
    /// **strong** or __strong__
    Strong(Vec<InlineToken>),
    /// ***strong emphasis*** or ___strong emphasis___
    StrongEm(Vec<InlineToken>),
    /// [text](href "title") or [text][ref]
    Link {
        href: String,
        title: Option<String>,
        tokens: Vec<InlineToken>,
    },
    /// ![alt](href "title")
    Image {
        href: String,
        title: Option<String>,
        alt: String,
    },
    /// <https://example.com> autolink
    Autolink {
        href: String,
        text: String,
        is_email: bool,
    },
    /// Inline raw HTML: <span class="x">
    RawHtml(String),
    /// Hard line break (two spaces + newline, or backslash + newline)
    HardBreak,
    /// Soft line break (single newline in source)
    SoftBreak,
    /// ~~GFM strikethrough~~
    Del(Vec<InlineToken>),
}
