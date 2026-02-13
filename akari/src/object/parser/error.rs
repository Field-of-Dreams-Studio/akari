use core::fmt;

/// The category of a parse failure.
///
/// `ParseErrorKind` is intentionally structured (instead of a free-form `String`) so callers
/// can reliably distinguish conditions such as *incomplete input* vs *syntax errors* vs
/// *trailing garbage*, and implement robust recovery / retries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    /// The parser reached the end of the currently buffered input while still needing more
    /// bytes/chars to complete a valid `Value`.
    ///
    /// In streaming mode, this usually corresponds to "need more data" (e.g. `next()` may
    /// return `NeedMore`). After `end_of_input()` is signaled, this becomes a definitive
    /// error because no more input will arrive.
    Incomplete,

    /// A complete `Value` was parsed, but additional non-ignorable input remained afterwards.
    ///
    /// This is typically raised by strict APIs such as `parse_full()` / `parse_one()`.
    TrailingData,

    /// The parser encountered an unexpected token/byte sequence.
    ///
    /// `found` is a short description of what was seen (e.g. `"i"`, `"0xFF"`, `"end of input"`).
    /// `expected` is a static list of expected token descriptions (e.g. `["number", "\"", "{"]`).
    UnexpectedToken {
        found: String,
        expected: &'static [&'static str],
    },

    /// The input encoding is invalid for the parser (e.g. non-UTF-8 in a text parser).
    InvalidEncoding(&'static str),

    /// A number literal is malformed or out of range for the target representation.
    InvalidNumber,

    /// The input exceeded a configured nesting/recursion limit (DoS protection).
    DepthLimit,

    /// A generic static message for implementation-specific errors that do not fit other kinds.
    ///
    /// Prefer more specific variants when possible.
    Message(&'static str),
}

/// Optional line/column location information for text inputs.
///
/// - `line` is 1-based (first line is 1)
/// - `col`  is 1-based (first column is 1)
///
/// The meaning of "column" is parser-defined (byte column vs unicode scalar column); pick one
/// and document it consistently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineCol {
    pub line: u32,
    pub col: u32,
}

/// A structured parse error with position information.
///
/// `pos` is a 0-based offset into the *logical input stream* (usually a byte offset).
/// For streaming inputs, `pos` should be monotonic across successive `feed()` calls (i.e.
/// relative to the total stream, not just the current chunk).
///
/// `line_col` is optional and typically only provided by text parsers.
///
/// `context` is optional, short extra information intended for humans (e.g. a snippet preview,
/// or additional explanation). Avoid storing large strings here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// The structured error category.
    pub kind: ParseErrorKind,
    /// 0-based position where the error occurred (recommended: byte offset).
    pub pos: usize,
    /// Optional line/column location for text inputs.
    pub line_col: Option<LineCol>,
    /// Optional short, human-readable context.
    pub context: Option<String>,
}

impl ParseError {
    /// Create a new `ParseError` at the given position.
    pub fn new(kind: ParseErrorKind, pos: usize) -> Self {
        Self {
            kind,
            pos,
            line_col: None,
            context: None,
        }
    }

    /// Attach short human-readable context to the error.
    pub fn with_context(mut self, ctx: impl Into<String>) -> Self {
        self.context = Some(ctx.into());
        self
    }

    /// Attach line/column information (typically produced by text parsers).
    pub fn with_line_col(mut self, lc: LineCol) -> Self {
        self.line_col = Some(lc);
        self
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ParseErrorKind::Incomplete => write!(f, "incomplete input at pos {}", self.pos)?,
            ParseErrorKind::TrailingData => write!(f, "trailing data at pos {}", self.pos)?,
            ParseErrorKind::UnexpectedToken { found, expected } => {
                write!(f, "unexpected token {found:?} at pos {}", self.pos)?;
                if !expected.is_empty() {
                    write!(f, ", expected one of: {:?}", expected)?;
                }
                Ok(())
            }?,
            ParseErrorKind::InvalidEncoding(msg) => write!(f, "invalid encoding ({msg}) at pos {}", self.pos)?,
            ParseErrorKind::InvalidNumber => write!(f, "invalid number at pos {}", self.pos)?,
            ParseErrorKind::DepthLimit => write!(f, "depth limit exceeded at pos {}", self.pos)?,
            ParseErrorKind::Message(msg) => write!(f, "{msg} at pos {}", self.pos)?,
        }

        if let Some(lc) = self.line_col {
            write!(f, " (line {}, col {})", lc.line, lc.col)?;
        }
        if let Some(ctx) = &self.context {
            write!(f, " - {}", ctx)?;
        }
        Ok(())
    }
}

impl std::error::Error for ParseError {}
