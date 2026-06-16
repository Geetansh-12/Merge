/// A block-level token produced by the Lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// ATX heading (# h1) or setext heading (underline style)
    Heading {
        /// Heading depth (1-6)
        depth: u8,
        /// Raw string representation
        raw: String,
        /// Parsed inline tokens
        tokens: Vec<InlineToken>,
    },
    /// Paragraph - catch-all block
    Paragraph {
        /// Raw paragraph text
        raw: String,
        /// Parsed inline tokens
        tokens: Vec<InlineToken>,
    },
    /// > blockquote - recursively contains block tokens
    Blockquote {
        /// Raw blockquote text
        raw: String,
        /// Child block tokens
        tokens: Vec<Token>,
    },
    /// Ordered or unordered list
    List {
        /// True if ordered (1. 2. 3.), false if unordered (- * +)
        ordered: bool,
        /// Start number for ordered lists
        start: u64,
        /// True if list items are separated by blank lines
        loose: bool,
        /// Items contained in this list
        items: Vec<ListItem>,
    },
    /// Fenced or indented code block
    CodeBlock {
        /// Language identifier for fenced blocks
        lang: Option<String>,
        /// Literal code content
        text: String,
    },
    /// GFM table
    Table {
        /// Column alignments (Left, Center, Right, None)
        align: Vec<Option<Alignment>>,
        /// Header cells
        header: Vec<TableCell>,
        /// Body rows containing cells
        rows: Vec<Vec<TableCell>>,
    },
    /// Raw HTML block (CommonMark types 1-7)
    HtmlBlock { 
        /// Literal HTML text
        text: String 
    },
    /// Thematic break (---, ***, ___)
    HorizontalRule,
    /// Link reference definition `[label]: url "title"`
    LinkDef {
        /// Link label
        label: String,
        /// Destination URL
        href: String,
        /// Optional title
        title: Option<String>,
    },
    /// Blank lines between blocks
    Space,
}

/// Represents an item within a list.
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    /// True if this is a task list item `[ ]` or `[x]`
    pub task: bool,
    /// True if the task list item is checked `[x]`
    pub checked: bool,
    /// True if the item is loose
    pub loose: bool,
    /// Block tokens contained in the item
    pub tokens: Vec<Token>,
}

/// Represents a cell within a GFM table.
#[derive(Debug, Clone, PartialEq)]
pub struct TableCell {
    /// Raw unparsed cell text
    pub raw: String,
    /// Parsed inline content
    pub tokens: Vec<InlineToken>,
}

/// Text alignment within a table column.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Alignment {
    /// Left aligned
    Left,
    /// Center aligned
    Center,
    /// Right aligned
    Right,
}

/// An inline-level token produced by the InlineParser.
#[derive(Debug, Clone, PartialEq)]
pub enum InlineToken {
    /// Plain text run
    Text(String),
    /// Backslash-escaped character: \* -> literal *
    Escape(char),
    /// `code span`
    CodeSpan(String),
    /// *emphasis* or _emphasis_
    Em(Vec<InlineToken>),
    /// **strong** or __strong__
    Strong(Vec<InlineToken>),
    /// ***strong emphasis*** or ___strong emphasis___
    StrongEm(Vec<InlineToken>),
    /// `[text](href "title")` or `[text][ref]`
    Link {
        /// Destination URL
        href: String,
        /// Optional title
        title: Option<String>,
        /// Link text as parsed inline tokens
        tokens: Vec<InlineToken>,
    },
    /// `![alt](href "title")`
    Image {
        /// Destination URL
        href: String,
        /// Optional title
        title: Option<String>,
        /// Plain alt text string
        alt: String,
    },
    /// `<https://example.com>` autolink
    Autolink {
        /// Destination URL
        href: String,
        /// Visible text
        text: String,
        /// True if this is an email autolink
        is_email: bool,
    },
    /// Inline raw HTML: `<span class="x">`
    RawHtml(String),
    /// Hard line break (two spaces + newline, or backslash + newline)
    HardBreak,
    /// Soft line break (single newline in source)
    SoftBreak,
    /// ~~GFM strikethrough~~
    Del(Vec<InlineToken>),
}
