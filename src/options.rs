/// Parser and renderer options matching marked v13 defaults.
#[derive(Debug, Clone)]
pub struct Options {
    /// Enable GFM extensions (tables, strikethrough, task lists).
    pub gfm: bool,
    /// Enable line breaks on single newlines (GFM breaks).
    pub breaks: bool,
    /// Enable pedantic mode (stricter CommonMark).
    pub pedantic: bool,
    /// Smart typography (quotes, dashes).
    pub smartypants: bool,
    /// Maximum nesting depth for block structures.
    pub max_nesting: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            gfm: true,
            breaks: false,
            pedantic: false,
            smartypants: false,
            max_nesting: 100,
        }
    }
}

impl Options {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn without_gfm() -> Self {
        Self {
            gfm: false,
            ..Self::default()
        }
    }
}
