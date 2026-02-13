use std::collections::HashMap;
use crate::object::Value;
use crate::object::parser::FrameState;
use crate::object::parser::stack::ValueStack;
use super::{ValueParser, Next, ParseError, BinInner};
use super::error::ParseErrorKind;

/// Primitive parsing functions for JSON format
///
/// These functions operate on BinInner and handle JSON-specific syntax.
/// Different formats (TOML, MessagePack, etc.) would have their own primitive modules.
mod primitive_parsing {
    use super::*;

    /// Parse null or none.
    ///
    /// Returns Incomplete if fewer than 4 bytes remain in the buffer.
    pub fn parse_null(inner: &mut BinInner) -> Result<Value, ParseError> {
        let pos = inner.pos();
        let remaining = inner.buffer().len().saturating_sub(pos);
        if remaining < 4 {
            inner.set_pos(pos);
            return Err(ParseError::new(ParseErrorKind::Incomplete, pos));
        }
        if inner.buffer()[pos..].starts_with(b"null") {
            // Manually advance position
            for _ in 0..4 {
                inner.next_byte();
            }
            return Ok(Value::None);
        }

        if inner.buffer()[pos..].starts_with(b"none") {
            for _ in 0..4 {
                inner.next_byte();
            }
            return Ok(Value::None);
        }

        Err(ParseError::new(
            ParseErrorKind::UnexpectedToken {
                found: String::from_utf8_lossy(
                    &inner.buffer()[pos..pos.saturating_add(4).min(inner.buffer().len())]
                ).to_string(),
                expected: &["null", "none"],
            },
            pos,
        ))
    }

    /// Parse a JSON boolean (true or false).
    ///
    /// Returns Incomplete if fewer than 4 bytes remain, or fewer than 5 bytes
    /// when checking for "false".
    pub fn parse_boolean(inner: &mut BinInner) -> Result<bool, ParseError> {
        let pos = inner.pos();
        let remaining = inner.buffer().len().saturating_sub(pos);
        if remaining < 4 {
            inner.set_pos(pos);
            return Err(ParseError::new(ParseErrorKind::Incomplete, pos));
        }
        if inner.buffer()[pos..].starts_with(b"true") {
            for _ in 0..4 {
                inner.next_byte();
            }
            return Ok(true);
        }

        if remaining < 5 {
            inner.set_pos(pos);
            return Err(ParseError::new(ParseErrorKind::Incomplete, pos));
        }
        if inner.buffer()[pos..].starts_with(b"false") {
            for _ in 0..5 {
                inner.next_byte();
            }
            return Ok(false);
        }

        Err(ParseError::new(
            ParseErrorKind::UnexpectedToken {
                found: String::from_utf8_lossy(
                    &inner.buffer()[pos..pos.saturating_add(5).min(inner.buffer().len())]
                ).to_string(),
                expected: &["true", "false"],
            },
            pos,
        ))
    }

    /// Parse a JSON-like number (integer or floating-point) into f64.
    ///
    /// Completion rules:
    /// - The number ends when a non-number delimiter is encountered
    ///   (whitespace, ',', ']', or '}').
    /// - If the buffer ends and `eof` is false, returns Incomplete because more
    ///   digits may still arrive.
    /// - If `eof` is true, buffer end is treated as a valid termination.
    ///
    /// Accepted syntax:
    /// - Leading '+' is allowed.
    /// - A trailing '.' is allowed (e.g. "1.").
    /// - A trailing 'e' or 'E' is allowed (e.g. "1e", "1e+").
    pub fn parse_number(inner: &mut BinInner, eof: bool) -> Result<f64, ParseError> {
        let start = inner.pos();
        let bytes = inner.buffer();
        let len = bytes.len();
        let mut i = start;

        if i >= len {
            inner.set_pos(start); 
            return Err(ParseError::new(ParseErrorKind::Incomplete, start));
        }

        let mut sign = 1.0f64;
        if bytes[i] == b'+' || bytes[i] == b'-' {
            if bytes[i] == b'-' {
                sign = -1.0;
            }
            i += 1;
            if i >= len && !eof {
                inner.set_pos(start);
                return Err(ParseError::new(ParseErrorKind::Incomplete, start));
            }
        }

        // Integer part
        let mut int_val = 0.0f64;
        let mut int_digits = 0usize;
        while i < len && bytes[i].is_ascii_digit() {
            int_val = int_val * 10.0 + (bytes[i] - b'0') as f64;
            i += 1;
            int_digits += 1;
        }

        if int_digits == 0 {
            return Err(ParseError::new(ParseErrorKind::InvalidNumber, start));
        }

        // Fractional part
        let mut value = int_val;
        if i < len && bytes[i] == b'.' {
            i += 1;
            let mut frac = 0.0f64;
            let mut scale = 1.0f64;
            let mut frac_digits = 0usize;
            while i < len && bytes[i].is_ascii_digit() {
                frac = frac * 10.0 + (bytes[i] - b'0') as f64;
                scale *= 10.0;
                i += 1;
                frac_digits += 1;
            }
            if frac_digits > 0 {
                value += frac / scale;
            }
        }

        // Exponent part
        if i < len && (bytes[i] == b'e' || bytes[i] == b'E') {
            i += 1;
            let mut exp_sign = 1i32;
            if i < len && (bytes[i] == b'+' || bytes[i] == b'-') {
                if bytes[i] == b'-' {
                    exp_sign = -1;
                }
                i += 1;
            }
            let mut exp_val: i32 = 0;
            let mut exp_digits = 0usize;
            while i < len && bytes[i].is_ascii_digit() {
                exp_val = exp_val.saturating_mul(10)
                    .saturating_add((bytes[i] - b'0') as i32);
                i += 1;
                exp_digits += 1;
            }
            if exp_digits > 0 {
                value *= 10f64.powi(exp_sign * exp_val);
            }
        }

        if i == len && !eof {
            inner.set_pos(start);
            return Err(ParseError::new(ParseErrorKind::Incomplete, start));
        }

        if i < len {
            let b = bytes[i];
            let is_delim = b == b' ' || b == b'\t' || b == b'\r' || b == b'\n'
                || b == b',' || b == b']' || b == b'}';
            if !is_delim {
                return Err(ParseError::new(ParseErrorKind::InvalidNumber, start));
            }
        }

        inner.set_pos(i);
        Ok(sign * value)
    }

    /// Parse a JSON string with escape sequences.
    ///
    /// Returns Incomplete if the buffer ends mid-string, mid-escape, or mid-\uXXXX.
    pub fn parse_string(inner: &mut BinInner) -> Result<String, ParseError> {
        let pos = inner.pos();
        // Consume opening '"'
        if inner.next_byte() != Some(b'"') {
            return Err(ParseError::new(ParseErrorKind::Message("Expected '\"'"), pos));
        }

        let mut bytes = Vec::new();

        loop {
            match inner.next_byte() {
                None => {
                    inner.set_pos(pos);
                    return Err(ParseError::new(
                        ParseErrorKind::Incomplete,
                        inner.pos(),
                    ));
                }
                Some(b'"') => break,  // End of string
                Some(b'\\') => {
                    // Handle escape sequences
                    match inner.next_byte() {
                        Some(b'"') => bytes.push(b'"'),
                        Some(b'\\') => bytes.push(b'\\'),
                        Some(b'/') => bytes.push(b'/'),
                        Some(b'b') => bytes.push(0x08),
                        Some(b'f') => bytes.push(0x0C),
                        Some(b'n') => bytes.push(b'\n'),
                        Some(b'r') => bytes.push(b'\r'),
                        Some(b't') => bytes.push(b'\t'),
                        Some(b'u') => {
                            // Parse \uXXXX Unicode escape
                            let codepoint = parse_unicode_escape(inner)?;
                            let mut buf = [0u8; 4];
                            let ch = char::from_u32(codepoint)
                                .ok_or_else(|| ParseError::new(
                                    ParseErrorKind::Message("Invalid Unicode codepoint"),
                                    inner.pos(),
                                ))?;
                            let len = ch.encode_utf8(&mut buf).len();
                            bytes.extend_from_slice(&buf[..len]);
                        }
                        None => {
                            inner.set_pos(pos);
                            return Err(ParseError::new(
                                ParseErrorKind::Incomplete,
                                inner.pos(),
                            ));
                        }
                        _ => return Err(ParseError::new(
                            ParseErrorKind::Message("Invalid escape sequence"),
                            inner.pos(),
                        )),
                    }
                }
                Some(b) => bytes.push(b),  // Regular byte (including multi-byte UTF-8)
            }
        }

        // Validate UTF-8 once at the end
        String::from_utf8(bytes).map_err(|_| {
            ParseError::new(
                ParseErrorKind::InvalidEncoding("Invalid UTF-8 in string"),
                inner.pos(),
            )
        })
    }

    /// Parse \uXXXX Unicode escape sequence
    fn parse_unicode_escape(inner: &mut BinInner) -> Result<u32, ParseError> {
        let start = inner.pos(); // Save position for backtracking
        let mut codepoint = 0u32;
        for _ in 0..4 {
            let b = inner.next_byte()
                .ok_or_else(|| {
                    inner.set_pos(start);
                    ParseError::new(
                        ParseErrorKind::Incomplete,
                        inner.pos(),
                    )
                })?;

            let digit = match b {
                b'0'..=b'9' => (b - b'0') as u32,
                b'a'..=b'f' => (b - b'a' + 10) as u32,
                b'A'..=b'F' => (b - b'A' + 10) as u32,
                _ => return Err(ParseError::new(
                    ParseErrorKind::Message("Invalid hex digit in Unicode escape"),
                    inner.pos(),
                )),
            };

            codepoint = codepoint * 16 + digit;
        }
        Ok(codepoint)
    }
}

/// Macro to generate type-safe wrapper types for BinJsonParser
macro_rules! impl_typed_parser {
    ($name:ident, $input_ty:ty, $trait_bound:ty) => {
        /// Type-safe JSON parser wrapper
        #[derive(Debug)]
        pub struct $name(BinJsonParser);

        impl $name {
            /// Create a new parser instance
            pub fn new() -> Self {
                Self(<BinJsonParser as ValueParser<$trait_bound>>::new())
            }

            /// Parse one complete JSON value (one-shot)
            pub fn parse_one(input: $input_ty) -> Result<Value, ParseError> {
                <BinJsonParser as ValueParser<$trait_bound>>::parse_one(input)
            }

            /// Feed input to the parser
            pub fn feed(&mut self, input: $input_ty) -> Result<(), ParseError> {
                <BinJsonParser as ValueParser<$trait_bound>>::feed(&mut self.0, input)
            }

            /// Signal end of input
            pub fn end_of_input(&mut self) {
                <BinJsonParser as ValueParser<$trait_bound>>::end_of_input(&mut self.0)
            }

            /// Parse exactly one complete value and reject trailing data
            pub fn parse_full(&mut self) -> Result<Value, ParseError> {
                <BinJsonParser as ValueParser<$trait_bound>>::parse_full(&mut self.0)
            }

            /// Attempt to parse the next value (streaming)
            pub fn parse_next(&mut self) -> Result<Next<Value>, ParseError> {
                <BinJsonParser as ValueParser<$trait_bound>>::parse_next(&mut self.0)
            }

            /// Get current parsing position
            pub fn pos(&self) -> usize {
                <BinJsonParser as ValueParser<$trait_bound>>::pos(&self.0)
            }

            /// Configure maximum nesting depth (default: 512)
            ///
            /// Returns `&mut self` for method chaining.
            pub fn max_depth(&mut self, depth: usize) -> &mut Self {
                self.0.max_depth = depth;
                self
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

// Generate the two typed parser wrappers
impl_typed_parser!(BsonParser, &[u8], [u8]);
impl_typed_parser!(JsonParser, &str, str);

/// Binary JSON parser with streaming support
///
/// This parser operates on raw bytes (`Vec<u8>`) and validates UTF-8 only when
/// constructing string values. All structural parsing uses byte comparisons for speed.
///
/// # Performance
/// - All JSON control characters are ASCII (single bytes)
/// - Structure parsing uses direct byte comparisons (no UTF-8 overhead)
/// - String content is validated as UTF-8 only when complete
/// - Optimized for typical use cases: 10-100MB JSON documents
///
/// # Design
/// - Uses `BinInner` for format-agnostic buffer management
/// - JSON-specific parsing logic in `primitive_parsing` module
///
/// # Security
/// - `max_depth` prevents stack overflow from deeply nested structures
#[derive(Debug)]
pub struct BinJsonParser {
    inner: BinInner,
    eof: bool,
    max_depth: usize,     // Maximum nesting depth (default: 512)
    current_depth: usize, // Current nesting level
}

impl BinJsonParser {
    // ===== Internal helper methods =====

    /// Check if we need more input to continue parsing
    fn needs_more(&self) -> bool {
        !self.eof && self.inner.pos() >= self.inner.buffer().len()
    }

    /// Create a ParseError at the current position
    fn error(&self, kind: ParseErrorKind) -> ParseError {
        ParseError::new(kind, self.inner.pos())
    }

    /// Create a ParseError with a message
    fn error_msg(&self, msg: &'static str) -> ParseError {
        ParseError::new(ParseErrorKind::Message(msg), self.inner.pos())
    }

    /// Parse a JSON value (entry point for recursive parsing)
    fn parse_value(&mut self) -> Result<Value, ParseError> {
        self.inner.skip_whitespace();

        if self.needs_more() {
            return Err(self.error(ParseErrorKind::Incomplete));
        }

        // Read byte once, match on all possibilities
        match self.inner.peek_byte() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'"') => primitive_parsing::parse_string(&mut self.inner).map(Value::Str),
            Some(b't') | Some(b'f') => primitive_parsing::parse_boolean(&mut self.inner).map(Value::Boolean),
            Some(b'n') => primitive_parsing::parse_null(&mut self.inner),
            Some(b'-') | Some(b'0'..=b'9') => {
                primitive_parsing::parse_number(&mut self.inner, self.eof).map(Value::Numerical)
            }
            Some(b) => Err(self.error(ParseErrorKind::UnexpectedToken {
                found: format!("0x{:02X} ('{}')", b, (b as char).escape_default()),
                expected: &["{", "[", "\"", "number", "true", "false", "null", "none"],
            })),
            None => Err(self.error(ParseErrorKind::Incomplete)),
        }
    }

    /// Parse a JSON object: { "key": value, ... }
    fn parse_object(&mut self) -> Result<Value, ParseError> {
        // Check depth limit before entering
        self.current_depth += 1;
        if self.current_depth > self.max_depth {
            self.current_depth -= 1;
            return Err(self.error(ParseErrorKind::DepthLimit));
        }

        let mut map = HashMap::new();

        // Consume '{'
        if self.inner.next_byte() != Some(b'{') {
            self.current_depth -= 1;
            return Err(self.error_msg("Expected '{'"));
        }

        self.inner.skip_whitespace();
        if self.needs_more() {
            self.current_depth -= 1;
            return Err(self.error(ParseErrorKind::Incomplete));
        }

        // Empty object
        if self.inner.peek_byte() == Some(b'}') {
            self.inner.next_byte();
            self.current_depth -= 1;
            return Ok(Value::Dict(map));
        }

        loop {
            self.inner.skip_whitespace();
            if self.needs_more() {
                self.current_depth -= 1;
                return Err(self.error(ParseErrorKind::Incomplete));
            }

            // Parse key (must be string)
            if self.inner.peek_byte() != Some(b'"') {
                self.current_depth -= 1;
                return Err(self.error(ParseErrorKind::UnexpectedToken {
                    found: format!("{:?}", self.inner.peek_byte()),
                    expected: &["\""],
                }));
            }

            let key = match primitive_parsing::parse_string(&mut self.inner) {
                Ok(k) => k,
                Err(e) => {
                    self.current_depth -= 1;
                    return Err(e);
                }
            };
            self.inner.skip_whitespace();

            if self.needs_more() {
                self.current_depth -= 1;
                return Err(self.error(ParseErrorKind::Incomplete));
            }

            // Expect ':'
            if self.inner.next_byte() != Some(b':') {
                self.current_depth -= 1;
                return Err(self.error(ParseErrorKind::UnexpectedToken {
                    found: format!("{:?}", self.inner.peek_byte()),
                    expected: &[":"],
                }));
            }

            // Parse value
            self.inner.skip_whitespace();
            let value = match self.parse_value() {
                Ok(v) => v,
                Err(e) => {
                    self.current_depth -= 1;
                    return Err(e);
                }
            };
            map.insert(key, value);
            self.inner.skip_whitespace();

            if self.needs_more() {
                self.current_depth -= 1;
                return Err(self.error(ParseErrorKind::Incomplete));
            }

            // Check for ',' or '}'
            match self.inner.next_byte() {
                Some(b',') => continue,
                Some(b'}') => break,
                _ => {
                    self.current_depth -= 1;
                    return Err(self.error(ParseErrorKind::UnexpectedToken {
                        found: format!("{:?}", self.inner.peek_byte()),
                        expected: &[",", "}"],
                    }))
                }
            }
        }
        self.current_depth -= 1;
        Ok(Value::Dict(map))
    }

    /// Parse a JSON array: [ value1, value2, ... ]
    fn parse_array(&mut self) -> Result<Value, ParseError> {
        // Check depth limit before entering
        self.current_depth += 1;
        if self.current_depth > self.max_depth {
            self.current_depth -= 1;
            return Err(self.error(ParseErrorKind::DepthLimit));
        }

        let mut vec = Vec::new();

        // Consume '['
        if self.inner.next_byte() != Some(b'[') {
            self.current_depth -= 1;
            return Err(self.error_msg("Expected '['"));
        }

        self.inner.skip_whitespace();
        if self.needs_more() {
            self.current_depth -= 1;
            return Err(self.error(ParseErrorKind::Incomplete));
        }

        // Empty array
        if self.inner.peek_byte() == Some(b']') {
            self.inner.next_byte();
            self.current_depth -= 1;
            return Ok(Value::List(vec));
        }

        loop {
            self.inner.skip_whitespace();
            let value = match self.parse_value() {
                Ok(v) => v,
                Err(e) => {
                    self.current_depth -= 1;
                    return Err(e);
                }
            };
            vec.push(value);
            self.inner.skip_whitespace();

            if self.needs_more() {
                self.current_depth -= 1;
                return Err(self.error(ParseErrorKind::Incomplete));
            }

            // Check for ',' or ']'
            match self.inner.next_byte() {
                Some(b',') => continue,
                Some(b']') => break,
                _ => {
                    self.current_depth -= 1;
                    return Err(self.error(ParseErrorKind::UnexpectedToken {
                        found: format!("{:?}", self.inner.peek_byte()),
                        expected: &[",", "]"],
                    }))
                }
            }
        }
        self.current_depth -= 1;
        Ok(Value::List(vec))
    }

}

/// Implementation of the ValueParser trait for binary JSON format
impl ValueParser<[u8]> for BinJsonParser {
    type Error = ParseError;

    fn new() -> Self {
        BinJsonParser {
            inner: BinInner::new(),
            eof: false,
            max_depth: 512,      // Default: prevent deeply nested structures
            current_depth: 0,    // Start at depth 0
        }
    }

    fn feed(&mut self, input: &[u8]) -> Result<(), Self::Error> {
        if self.eof {
            return Err(ParseError::new(
                ParseErrorKind::Message("Cannot feed after end_of_input"),
                self.inner.pos(),
            ));
        }
        self.inner.feed(input);
        Ok(())
    }

    fn end_of_input(&mut self) {
        self.eof = true;
    }

    fn parse_full(&mut self) -> Result<Value, Self::Error> {
        let value = self.parse_value()?;

        // Verify no trailing data (skip whitespace first)
        self.inner.skip_whitespace();
        if self.inner.pos() != self.inner.buffer().len() {
            return Err(self.error(ParseErrorKind::TrailingData));
        }

        Ok(value)
    }

    fn parse_next(&mut self) -> Result<Next<Value>, Self::Error> {
        self.inner.skip_whitespace();

        // If we're at the end of buffer
        if self.inner.pos() >= self.inner.buffer().len() {
            if self.eof {
                return Ok(Next::Eof);
            } else {
                return Ok(Next::NeedMore);
            }
        }

        // Try to parse a value
        match self.parse_value() {
            Ok(value) => Ok(Next::Value(value)),
            Err(e) if matches!(e.kind, ParseErrorKind::Incomplete) => {
                if self.eof {
                    Err(e) // Incomplete is an error after EOF
                } else {
                    Ok(Next::NeedMore)
                }
            }
            Err(e) => Err(e),
        }
    }

    fn pos(&self) -> usize {
        self.inner.pos()
    }
}

/// Implementation of the ValueParser trait for string input (converts to bytes)
impl ValueParser<str> for BinJsonParser {
    type Error = ParseError;

    fn new() -> Self {
        BinJsonParser {
            inner: BinInner::new(),
            eof: false,
            max_depth: 512,      // Default: prevent deeply nested structures
            current_depth: 0,    // Start at depth 0
        }
    }

    fn feed(&mut self, input: &str) -> Result<(), Self::Error> {
        // Convert string to bytes and delegate to [u8] implementation
        if self.eof {
            return Err(ParseError::new(
                ParseErrorKind::Message("Cannot feed after end_of_input"),
                self.inner.pos(),
            ));
        }
        self.inner.feed(input.as_bytes());
        Ok(())
    }

    fn end_of_input(&mut self) {
        self.eof = true;
    }

    fn parse_full(&mut self) -> Result<Value, Self::Error> {
        let value = self.parse_value()?;

        // Verify no trailing data (skip whitespace first)
        self.inner.skip_whitespace();
        if self.inner.pos() != self.inner.buffer().len() {
            return Err(self.error(ParseErrorKind::TrailingData));
        }

        Ok(value)
    }

    fn parse_next(&mut self) -> Result<Next<Value>, Self::Error> {
        self.inner.skip_whitespace();

        // If we're at the end of buffer
        if self.inner.pos() >= self.inner.buffer().len() {
            if self.eof {
                return Ok(Next::Eof);
            } else {
                return Ok(Next::NeedMore);
            }
        }

        // Try to parse a value
        match self.parse_value() {
            Ok(value) => Ok(Next::Value(value)),
            Err(e) if matches!(e.kind, ParseErrorKind::Incomplete) => {
                if self.eof {
                    Err(e) // Incomplete is an error after EOF
                } else {
                    Ok(Next::NeedMore)
                }
            }
            Err(e) => Err(e),
        }
    }

    fn pos(&self) -> usize {
        self.inner.pos()
    }
}

/// Resumable stack-based JSON parser
///
/// This parser can pause and resume parsing at any point, making it ideal
/// for streaming scenarios where data arrives in small chunks.
///
/// # Performance
/// - **No re-parsing**: Saves state when incomplete, resumes from exact position
/// - **Memory efficient**: Only stores partial containers, not entire input
/// - **I/O optimized**: Can parse while waiting for more data
///
/// # Use Cases
/// - Network streaming (data arrives in small packets)
/// - Large file streaming (read in chunks to avoid loading everything)
/// - Interactive parsing (process data as it arrives)
///
/// # Example
/// ```ignore
/// let mut parser = StackParser::new();
///
/// // Chunk 1: Incomplete object
/// parser.feed(br#"{"users": [{"name": "Alice", "age":"#)?;
/// assert!(matches!(parser.parse_next()?, Next::NeedMore));
/// // State saved! Parser remembers:
/// // - We're in an object with key "users"
/// // - Inside an array
/// // - Inside another object with "name": "Alice" and pending "age" key
///
/// // Chunk 2: Complete the data
/// parser.feed(br#"30}]}"#)?;
/// parser.end_of_input();
/// let value = parser.parse_next()?;  // Resumes from saved state!
/// ```
pub struct StackParser {
    inner: BinInner,
    eof: bool,
    stack: ValueStack,
    max_depth: usize,  // Maximum nesting depth (default: 512)
    // TODO: Add current_depth tracking when implementing depth checking
}

impl StackParser { 
    /// Check if we need more input
    fn needs_more(&self) -> bool {
        !self.eof && self.inner.pos() >= self.inner.buffer().len()
    }

    /// Create a ParseError at the current position
    fn error(&self, kind: ParseErrorKind) -> ParseError {
        ParseError::new(kind, self.inner.pos())
    }

    /// Create a ParseError with a message
    fn _error_msg(&self, msg: &'static str) -> ParseError {
        ParseError::new(ParseErrorKind::Message(msg), self.inner.pos())
    }

    /// Compact internal buffer to free consumed data 
    pub fn compact(&mut self) {
        self.inner.compact();
    } 

    /// Parse a primitive value (anything except '{' or '[')
    ///
    /// Handles: strings, numbers, booleans, null
    /// Uses: primitive_parsing module
    /// Note: Position restoration is handled by primitive_parsing functions themselves
    fn parse_primitive(&mut self) -> Result<Value, ParseError> {
        self.inner.skip_whitespace();

        if self.needs_more() {
            return Err(self.error(ParseErrorKind::Incomplete));
        }

        // Determine primitive type and parse
        // Each primitive function handles its own position restoration on Incomplete
        match self.inner.peek_byte() {
            Some(b'"') => primitive_parsing::parse_string(&mut self.inner)
                .map(Value::Str),
            Some(b't') | Some(b'f') => primitive_parsing::parse_boolean(&mut self.inner)
                .map(Value::Boolean),
            Some(b'n') => primitive_parsing::parse_null(&mut self.inner),
            Some(b'-') | Some(b'0'..=b'9') => {
                primitive_parsing::parse_number(&mut self.inner, self.eof)
                    .map(Value::Numerical)
            }
            Some(b) => Err(self.error(ParseErrorKind::UnexpectedToken {
                found: format!("0x{:02X}", b),
                expected: &["string", "number", "boolean", "null"],
            })),
            None => Err(self.error(ParseErrorKind::Incomplete)),
        }
    }

    /// Main parsing loop (stack-based, resumable)
    ///
    fn parse_loop(&mut self) -> Result<Value, ParseError> {
        loop { 
            self.inner.skip_whitespace();

            if self.needs_more() {
                return Err(self.error(ParseErrorKind::Incomplete));
            }

            match self.stack.current_frame_state() { 
                Some(FrameState::Array) => {
                    match self.inner.peek_byte() {
                        Some(b']') => {
                            match self.end_array() { 
                                Ok(Some(v)) => return Ok(v),
                                Ok(None) => continue, // Still more to parse
                                Err(e) => return Err(ParseError::new(e, self.inner.pos())),
                            }
                        } 
                        _ => {
                            // Expecting a value in the array
                            match self.inner.peek_byte() {
                                Some(b',') => {
                                    self.inner.next_byte(); // Consume ':' if present (error otherwise)
                                }
                                Some(b'{') => self.parse_object()?,
                                Some(b'[') => self.parse_array()?,
                                _ => {
                                    let r = self.parse_primitive()?;
                                    self.stack.push(r).map_err(|e| ParseError::new(e, self.inner.pos()))?;
                                }
                            } 
                        }
                    } 
                }, 
                Some(FrameState::ObjectWaitingForKey) => {
                    match self.inner.peek_byte() {
                        Some(b',') => {
                            self.inner.next_byte(); // Consume ':' if present (error otherwise) 
                        } 
                        Some(b'"') => { 
                            let key = primitive_parsing::parse_string(&mut self.inner)?;
                            self.stack.push_new_key(key).map_err(|e| ParseError::new(e, self.inner.pos()))?;
                        }
                        Some(b'}') => {
                            match self.end_object() { 
                                Ok(Some(v)) => return Ok(v),
                                Ok(None) => continue, // Still more to parse
                                Err(e) => return Err(ParseError::new(e, self.inner.pos())),
                            }
                        } 
                        _ => {
                            return Err(self.error(ParseErrorKind::UnexpectedToken {
                                found: format!("{:?}", self.inner.peek_byte()),
                                expected: &["\"", "}"],
                            }))
                        }
                    } 
                },
                Some(FrameState::ObjectWaitingForValue) => {
                    // Expecting a value for the last key
                    match self.inner.peek_byte() {
                        Some(b':') => {
                            self.inner.next_byte(); // Consume ':' if present (error otherwise)
                        }
                        Some(b'{') => self.parse_object()?,
                        Some(b'[') => self.parse_array()?,
                        _ => {
                            let r = self.parse_primitive()?;
                            self.stack.push(r).map_err(|e| ParseError::new(e, self.inner.pos()))?;
                        }
                    }
                }, 
                None => {
                    match self.inner.peek_byte() {
                        Some(b'{') => self.parse_object()?,
                        Some(b'[') => self.parse_array()?,
                        _ => return self.parse_primitive(),
                    }
                }, // Top-level value 
            } 
        } 
    }

    fn parse_array(&mut self) -> Result<(), ParseError> {
        // Check depth limit before entering new nesting level
        if self.stack.len() >= self.max_depth {
            return Err(self.error(ParseErrorKind::DepthLimit));
        }

        self.inner.next_byte(); // Consume '['
        self.stack.push_new_array(); // Push new array frame onto stack
        Ok(())
    }

    fn parse_object(&mut self) -> Result<(), ParseError> {
        // Check depth limit before entering new nesting level
        if self.stack.len() >= self.max_depth {
            return Err(self.error(ParseErrorKind::DepthLimit));
        }

        self.inner.next_byte(); // Consume '{'
        self.stack.push_new_object(); // Push new object frame onto stack
        Ok(())
    } 

    fn end_array(&mut self) -> Result<Option<Value>, ParseErrorKind> {
        self.inner.next_byte(); // Consume ']' 
        self.stack.push_to_parent() 
    }

    fn end_object(&mut self) -> Result<Option<Value>, ParseErrorKind> {
        self.inner.next_byte(); // Consume '}' 
        self.stack.push_to_parent()  
    } 
}

/// Implementation of ValueParser trait for stack-based parser
impl ValueParser<[u8]> for StackParser {
    type Error = ParseError;

    fn new() -> Self {
        StackParser {
            inner: BinInner::new(), 
            eof: false,
            stack: ValueStack::new(), 
            max_depth: 512,  // Default: prevent deeply nested structures
        }
    }

    fn feed(&mut self, input: &[u8]) -> Result<(), Self::Error> {
        if self.eof {
            return Err(ParseError::new(
                ParseErrorKind::Message("Cannot feed after end_of_input"),
                self.inner.pos(),
            ));
        }
        self.inner.feed(input);
        Ok(())
    }

    fn end_of_input(&mut self) {
        self.eof = true;
    }

    fn parse_full(&mut self) -> Result<Value, Self::Error> {
        let value = self.parse_loop()?;

        // Verify no trailing data
        self.inner.skip_whitespace();
        if self.inner.pos() != self.inner.buffer().len() {
            return Err(self.error(ParseErrorKind::TrailingData));
        }

        Ok(value)
    }

    fn parse_next(&mut self) -> Result<Next<Value>, Self::Error> {
        self.inner.skip_whitespace();

        // Check if at end of buffer
        if self.inner.pos() >= self.inner.buffer().len() {
            if self.eof {
                return Ok(Next::Eof);
            } else {
                return Ok(Next::NeedMore);
            }
        }

        // Try to parse one value
        match self.parse_loop() {
            Ok(value) => Ok(Next::Value(value)),
            Err(e) if matches!(e.kind, ParseErrorKind::Incomplete) => {
                if self.eof {
                    Err(e) // Incomplete after EOF is an error
                } else {
                    Ok(Next::NeedMore)
                }
            }
            Err(e) => Err(e),
        }
    }

    fn pos(&self) -> usize {
        self.inner.pos()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // // Helper function to create a parser with byte input
    // fn parser_with(input: &[u8]) -> BsonParser {
    //     let mut p = BsonParser::new();
    //     p.feed(input).unwrap();
    //     p.end_of_input();
    //     p
    // }

    #[test]
    fn test_parse_string_simple() {
        let result = BsonParser::parse_one(br#""hello world""#).unwrap();
        assert_eq!(result, Value::Str("hello world".to_string()));
    }

    #[test]
    fn test_parse_number_integer() {
        let result = BsonParser::parse_one(b"42").unwrap();
        assert_eq!(result, Value::Numerical(42.0));
    }

    #[test]
    fn test_parse_boolean_true() {
        let result = BsonParser::parse_one(b"true").unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_parse_null() {
        let result = BsonParser::parse_one(b"null").unwrap();
        assert_eq!(result, Value::None);
    }

    #[test]
    fn test_parse_array() {
        let result = BsonParser::parse_one(br#"[1, "hello", true]"#).unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Numerical(1.0),
                Value::Str("hello".to_string()),
                Value::Boolean(true),
            ])
        );
    }

    #[test]
    fn test_parse_object() {
        let result = BsonParser::parse_one(br#"{"name": "Alice", "age": 30}"#).unwrap();

        match result {
            Value::Dict(map) => {
                assert_eq!(map.get("name"), Some(&Value::Str("Alice".to_string())));
                assert_eq!(map.get("age"), Some(&Value::Numerical(30.0)));
            }
            _ => panic!("Expected Dict"),
        }
    }

    #[test]
    fn test_trait_parse_one() {
        let result = BsonParser::parse_one(br#"{"key": "value"}"#).unwrap();

        match result {
            Value::Dict(map) => {
                assert_eq!(map.get("key"), Some(&Value::Str("value".to_string())));
            }
            _ => panic!("Expected Dict"),
        }
    }

    #[test]
    fn test_trait_streaming() {
        let mut parser = BsonParser::new();
        parser.feed(br#"{"a": 1} {"b": 2}"#).unwrap();
        parser.end_of_input();

        // Parse first value
        let result1 = parser.parse_next();
        assert!(matches!(result1, Ok(Next::Value(_))));

        // Parse second value
        let result2 = parser.parse_next();
        assert!(matches!(result2, Ok(Next::Value(_))));

        // No more values
        let result3 = parser.parse_next();
        assert!(matches!(result3, Ok(Next::Eof)));
    }

    #[test]
    fn test_chunked_feed() {
        let mut parser = BsonParser::new();
        parser.feed(br#"{"key": "#).unwrap();
        parser.feed(br#""value"}"#).unwrap();
        parser.end_of_input();

        let result = parser.parse_full().unwrap();
        match result {
            Value::Dict(map) => {
                assert_eq!(map.get("key"), Some(&Value::Str("value".to_string())));
            }
            _ => panic!("Expected Dict"),
        }
    }

    #[test]
    fn test_utf8_multibyte() {
        let input = r#""Hello 世界 🌍""#.as_bytes();
        let result = BsonParser::parse_one(input).unwrap();
        assert_eq!(result, Value::Str("Hello 世界 🌍".to_string()));
    }

    #[test]
    fn test_unicode_escape() {
        let result = BsonParser::parse_one(br#""\u0048\u0065\u006C\u006C\u006F""#).unwrap();
        assert_eq!(result, Value::Str("Hello".to_string()));
    }

    // ========== String Input Tests (ValueParser<str> impl) ==========

    #[test]
    fn test_str_parse_one() {
        let result = JsonParser::parse_one(r#"{"key": "value"}"#).unwrap();

        match result {
            Value::Dict(map) => {
                assert_eq!(map.get("key"), Some(&Value::Str("value".to_string())));
            }
            _ => panic!("Expected Dict"),
        }
    }

    #[test]
    fn test_str_feed_and_parse() {
        let mut parser = JsonParser::new();
        parser.feed(r#"{"name": "Alice", "age": 30}"#).unwrap();
        parser.end_of_input();
        let result = parser.parse_full().unwrap();

        match result {
            Value::Dict(map) => {
                assert_eq!(map.get("name"), Some(&Value::Str("Alice".to_string())));
                assert_eq!(map.get("age"), Some(&Value::Numerical(30.0)));
            }
            _ => panic!("Expected Dict"),
        }
    }

    #[test]
    fn test_str_chunked_feed() {
        let mut parser = JsonParser::new();
        parser.feed(r#"{"key": "#).unwrap();
        parser.feed(r#""value"}"#).unwrap();
        parser.end_of_input();

        let result = parser.parse_full().unwrap();
        match result {
            Value::Dict(map) => {
                assert_eq!(map.get("key"), Some(&Value::Str("value".to_string())));
            }
            _ => panic!("Expected Dict"),
        }
    }

    #[test]
    fn test_str_streaming() {
        let mut parser = JsonParser::new();
        parser.feed(r#"{"a": 1} {"b": 2}"#).unwrap();
        parser.end_of_input();

        // Parse first value
        let result1 = parser.parse_next();
        assert!(matches!(result1, Ok(Next::Value(_))));

        // Parse second value
        let result2 = parser.parse_next();
        assert!(matches!(result2, Ok(Next::Value(_))));

        // No more values
        let result3 = parser.parse_next();
        assert!(matches!(result3, Ok(Next::Eof)));
    }

    #[test]
    fn test_str_utf8_multibyte() {
        let result = JsonParser::parse_one(r#""Hello 世界 🌍""#).unwrap();
        assert_eq!(result, Value::Str("Hello 世界 🌍".to_string()));
    }

    #[test]
    fn test_str_need_more() {
        let mut parser = JsonParser::new();
        parser.feed(r#"{"key":"#).unwrap();

        // Not enough data yet
        let result = parser.parse_next();
        assert!(matches!(result, Ok(Next::NeedMore)));

        // Feed more data
        parser.feed(r#""value"}"#).unwrap();
        parser.end_of_input();

        // Now should succeed
        let result = parser.parse_next();
        assert!(matches!(result, Ok(Next::Value(_))));
    }

        #[test]
    fn test_simple_object() {
        let mut parser = StackParser::new();
        parser.feed(br#"{"key": "value"}"#).unwrap();
        parser.end_of_input();

        let result = parser.parse_full().unwrap();
        match result {
            Value::Dict(map) => {
                assert_eq!(map.get("key"), Some(&Value::Str("value".to_string())));
            }
            _ => panic!("Expected Dict"),
        }
    }

    #[test]
    fn test_stack_empty_containers() {
        let mut parser = StackParser::new();
        parser.feed(br#"{"obj": {}, "arr": []}"#).unwrap();
        parser.end_of_input();

        let result = parser.parse_full().unwrap();
        match result {
            Value::Dict(map) => {
                assert!(matches!(map.get("obj"), Some(Value::Dict(_))));
                assert!(matches!(map.get("arr"), Some(Value::List(_))));
            }
            _ => panic!("Expected Dict"),
        }
    }

    #[test]
    fn test_stack_multiple_keys_and_values() {
        let mut parser = StackParser::new();
        parser.feed(br#"{"a": 1, "b": 2, "c": 3}"#).unwrap();
        parser.end_of_input();

        let result = parser.parse_full().unwrap();
        match result {
            Value::Dict(map) => {
                assert_eq!(map.get("a"), Some(&Value::Numerical(1.0)));
                assert_eq!(map.get("b"), Some(&Value::Numerical(2.0)));
                assert_eq!(map.get("c"), Some(&Value::Numerical(3.0)));
            }
            _ => panic!("Expected Dict"),
        }
    }

    #[test]
    fn test_stack_nested_structures() {
        let mut parser = StackParser::new();
        parser.feed(br#"{"a":[1,{"b":[2,3]}],"c":{"d":4}}"#).unwrap();
        parser.end_of_input();

        let result = parser.parse_full().unwrap();
        assert!(matches!(result, Value::Dict(_)));
    }

    #[test]
    fn test_chunked_parsing() {
        let mut parser = StackParser::new();

        // Chunk 1: Incomplete
        parser.feed(br#"{"users": [{"name": "Alice", "age":"#).unwrap();
        let result1 = parser.parse_next().unwrap();
        assert!(matches!(result1, Next::NeedMore));

        // Chunk 2: Complete
        parser.feed(br#" 30}]}"#).unwrap();
        parser.end_of_input();

        let result2 = parser.parse_next().unwrap();
        assert!(matches!(result2, Next::Value(_)));
    }

    #[test]
    fn test_stack_chunked_commas_and_colons() {
        let mut parser = StackParser::new();
        parser.feed(br#"{"a": 1, "b": {"#).unwrap();
        let result1 = parser.parse_next().unwrap();
        assert!(matches!(result1, Next::NeedMore));

        parser.feed(br#""c": 2}, "d": ["#).unwrap();
        let result2 = parser.parse_next().unwrap();
        assert!(matches!(result2, Next::NeedMore));

        parser.feed(br#"3, 4]}"#).unwrap();
        parser.end_of_input();

        let result3 = parser.parse_next().unwrap();
        assert!(matches!(result3, Next::Value(_)));
    }

    #[test]
    fn test_multiple_values() {
        let mut parser = StackParser::new();
        parser.feed(br#"{"a": 1} {"b": 2}"#).unwrap();
        parser.end_of_input();

        let result1 = parser.parse_next();
        assert!(matches!(result1, Ok(Next::Value(_))));

        let result2 = parser.parse_next();
        assert!(matches!(result2, Ok(Next::Value(_))));

        let result3 = parser.parse_next();
        assert!(matches!(result3, Ok(Next::Eof)));
    }

    #[test]
    fn test_stack_deeply_nested_chunked() {
        let mut parser = StackParser::new();

        // Chunk 1: Start deeply nested structure
        parser.feed(br#"{"data": {"users": [{"id": 1, "name": ""#).unwrap();
        assert!(matches!(parser.parse_next(), Ok(Next::NeedMore)));

        // Chunk 2: Continue with name
        parser.feed(br#"Alice", "tags": ["admin""#).unwrap();
        assert!(matches!(parser.parse_next(), Ok(Next::NeedMore)));

        // Chunk 3: Complete the structure
        parser.feed(br#", "developer"]}]}}"#).unwrap();
        parser.end_of_input();

        let result = parser.parse_next().unwrap();
        if let Next::Value(Value::Dict(obj)) = result {
            assert!(obj.contains_key("data"));
        } else {
            panic!("Expected object");
        }

        assert!(matches!(parser.parse_next(), Ok(Next::Eof)));
    }

    #[test]
    fn test_stack_edge_cases() {
        let mut parser = StackParser::new();

        // Test: empty arrays and objects mixed with values
        parser.feed(br#"{"empty_obj": {}, "empty_arr": [], "nested": [[], {}], "val": 42}"#).unwrap();
        parser.end_of_input();

        let result = parser.parse_next().unwrap();
        if let Next::Value(Value::Dict(obj)) = result {
            assert!(obj.contains_key("empty_obj"));
            assert!(obj.contains_key("empty_arr"));
            assert!(obj.contains_key("nested"));
            assert!(obj.contains_key("val"));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_stack_malformed_should_fail() {
        // Note: This parser guarantees that CORRECT JSON will parse successfully.
        // Some "incorrect but somewhat sensible" inputs (e.g., trailing commas, [1,,2])
        // MAY be accepted as an implementation detail, but this is not guaranteed.
        // Only rely on correct JSON for portable behavior.

        // Test 1: Mismatched brackets [(}
        let mut parser1 = StackParser::new();
        parser1.feed(br#"[(}"#).unwrap();
        parser1.end_of_input();
        let result1 = parser1.parse_next();
        assert!(result1.is_err(), "Expected error for mismatched brackets [(}}");  // Double }} to escape

        // Test 2: Missing value after colon {"a":}
        let mut parser2 = StackParser::new();
        parser2.feed(br#"{"a":}"#).unwrap();
        parser2.end_of_input();
        let result2 = parser2.parse_next();
        assert!(result2.is_err(), "Expected error for missing value after :");

        // Test 3: Unclosed array [1, 2
        let mut parser3 = StackParser::new();
        parser3.feed(br#"[1, 2"#).unwrap();
        parser3.end_of_input();
        let result3 = parser3.parse_next();
        assert!(result3.is_err(), "Expected error for unclosed array");
    }

    #[test]
    fn test_stack_nested_arrays() {
        let mut parser = StackParser::new();
        parser.feed(br#"[1, [2, 3], [4, [5, 6]]]"#).unwrap();
        parser.end_of_input();

        let result = parser.parse_next().unwrap();
        if let Next::Value(Value::List(arr)) = result {
            assert_eq!(arr.len(), 3);
            // Check nested structure
            if let Value::List(inner) = &arr[1] {
                assert_eq!(inner.len(), 2);
            } else {
                panic!("Expected nested array");
            }
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_stack_mixed_types() {
        let mut parser = StackParser::new();
        parser.feed(br#"{"str": "hello", "num": 42, "bool": true, "null": null, "arr": [1, 2], "obj": {"nested": "value"}}"#).unwrap();
        parser.end_of_input();

        let result = parser.parse_next().unwrap();
        if let Next::Value(Value::Dict(obj)) = result {
            assert_eq!(obj.len(), 6);
            assert!(matches!(obj.get("str"), Some(Value::Str(_))));
            assert!(matches!(obj.get("num"), Some(Value::Numerical(_))));
            assert!(matches!(obj.get("bool"), Some(Value::Boolean(_))));
            assert!(matches!(obj.get("null"), Some(Value::None)));
            assert!(matches!(obj.get("arr"), Some(Value::List(_))));
            assert!(matches!(obj.get("obj"), Some(Value::Dict(_))));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_stack_resume_from_incomplete_string() {
        let mut parser = StackParser::new();

        // Chunk 1: Incomplete string (cuts in middle of string)
        parser.feed(br#"{"message": "Hello Wo"#).unwrap();
        assert!(matches!(parser.parse_next(), Ok(Next::NeedMore)));

        // Chunk 2: Complete the string and object
        parser.feed(br#"rld!"}"#).unwrap();
        parser.end_of_input();

        let result = parser.parse_next().unwrap();
        if let Next::Value(Value::Dict(obj)) = result {
            if let Some(Value::Str(s)) = obj.get("message") {
                assert_eq!(s, "Hello World!");
            } else {
                panic!("Expected string value");
            }
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_stack_very_deep_nesting() {
        let mut parser = StackParser::new();
        // Create deeply nested structure: [[[[[[1]]]]]]
        let depth = 20;
        let mut json = String::new();
        for _ in 0..depth {
            json.push('[');
        }
        json.push_str("42");
        for _ in 0..depth {
            json.push(']');
        }

        parser.feed(json.as_bytes()).unwrap();
        parser.end_of_input();

        let result = parser.parse_next().unwrap();
        // Verify it parsed successfully
        assert!(matches!(result, Next::Value(Value::List(_))));
    }

    #[test]
    fn test_stack_unicode_in_strings() {
        let mut parser = StackParser::new();
        parser.feed(r#"{"emoji": "🚀", "chinese": "你好", "escaped": "\u0048\u0065\u006C\u006C\u006F"}"#.as_bytes()).unwrap();
        parser.end_of_input();

        let result = parser.parse_next().unwrap();
        if let Next::Value(Value::Dict(obj)) = result {
            if let Some(Value::Str(s)) = obj.get("emoji") {
                assert_eq!(s, "🚀");
            }
            if let Some(Value::Str(s)) = obj.get("chinese") {
                assert_eq!(s, "你好");
            }
            if let Some(Value::Str(s)) = obj.get("escaped") {
                assert_eq!(s, "Hello");
            }
        } else {
            panic!("Expected object");
        }
    }

    // ===== Malformed input tests =====

    #[test]
    fn test_malformed_empty_input() {
        assert!(BsonParser::parse_one(b"").is_err());
        assert!(BsonParser::parse_one(b"   ").is_err());
    }

    #[test]
    fn test_malformed_random_bytes() {
        assert!(BsonParser::parse_one(b"@#$%^&").is_err());
        assert!(BsonParser::parse_one(b"\xff\xfe").is_err());
    }

    #[test]
    fn test_malformed_unterminated_string() {
        assert!(BsonParser::parse_one(br#""hello"#).is_err());
        assert!(BsonParser::parse_one(br#""hello\"#).is_err());
    }

    #[test]
    fn test_malformed_unterminated_object() {
        assert!(BsonParser::parse_one(br#"{"a": 1"#).is_err());
        assert!(BsonParser::parse_one(br#"{"a":"#).is_err());
        assert!(BsonParser::parse_one(br#"{"#).is_err());
    }

    #[test]
    fn test_malformed_unterminated_array() {
        assert!(BsonParser::parse_one(br#"[1, 2"#).is_err());
        assert!(BsonParser::parse_one(br#"["#).is_err());
    }

    #[test]
    fn test_malformed_mismatched_brackets() {
        assert!(BsonParser::parse_one(br#"[1, 2}"#).is_err());
        assert!(BsonParser::parse_one(br#"{"a": 1]"#).is_err());
    }

    #[test]
    fn test_malformed_colon_without_value() {
        let mut parser = StackParser::new();
        parser.feed(br#"{"a":}"#).unwrap();
        parser.end_of_input();
        assert!(parser.parse_next().is_err());
    }

    #[test]
    fn test_malformed_truncated_keywords() {
        assert!(BsonParser::parse_one(b"tru").is_err());
        assert!(BsonParser::parse_one(b"fals").is_err());
        assert!(BsonParser::parse_one(b"nul").is_err());
    }

    #[test]
    fn test_malformed_invalid_number_leading_zero() {
        // This parser is lenient about leading zeros — it parses them as valid numbers
        let result = BsonParser::parse_one(b"00");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Numerical(0.0));
    }

    #[test]
    fn test_malformed_lone_minus() {
        assert!(BsonParser::parse_one(b"-").is_err());
    }

    #[test]
    fn test_malformed_number_trailing_dot() {
        // This parser is lenient about trailing dots — "1." parses as 1.0
        let result = BsonParser::parse_one(b"1.");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Numerical(1.0));
    }

    #[test]
    fn test_malformed_number_leading_dot() {
        assert!(BsonParser::parse_one(b".5").is_err());
    }

    #[test]
    fn test_malformed_invalid_unicode_escape() {
        // Incomplete unicode escape
        assert!(BsonParser::parse_one(br#""\u00""#).is_err());
        // Non-hex in unicode escape
        assert!(BsonParser::parse_one(br#""\uGGGG""#).is_err());
    }

    #[test]
    fn test_malformed_stack_empty_input() {
        let mut parser = StackParser::new();
        parser.feed(b"").unwrap();
        parser.end_of_input();
        let result = parser.parse_next();
        // Should be Eof or error, not a value
        assert!(result.is_err() || matches!(result, Ok(Next::Eof)));
    }

    #[test]
    fn test_malformed_stack_unterminated_string() {
        let mut parser = StackParser::new();
        parser.feed(br#"{"key": "unterminated"#).unwrap();
        parser.end_of_input();
        assert!(parser.parse_next().is_err());
    }

    #[test]
    fn test_malformed_stack_mismatched_close() {
        let mut parser = StackParser::new();
        parser.feed(br#"[1, 2}"#).unwrap();
        parser.end_of_input();
        assert!(parser.parse_next().is_err());
    }

    // ===== ParseErrorKind variant coverage =====

    #[test]
    fn test_error_kind_trailing_data() {
        // parse_one / parse_full should reject trailing non-whitespace
        let result = BsonParser::parse_one(br#"42 "extra""#);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.kind, ParseErrorKind::TrailingData);
        }
    }

    #[test]
    fn test_error_kind_trailing_data_json() {
        let result = JsonParser::parse_one(r#"true false"#);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.kind, ParseErrorKind::TrailingData);
        }
    }

    #[test]
    fn test_error_kind_trailing_data_stack() {
        let mut parser = StackParser::new();
        parser.feed(br#"42 "extra""#).unwrap();
        parser.end_of_input();
        let result = parser.parse_full();
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.kind, ParseErrorKind::TrailingData);
        }
    }

    #[test]
    fn test_error_kind_incomplete() {
        let mut parser = StackParser::new();
        parser.feed(br#"[1, 2"#).unwrap();
        parser.end_of_input();
        let result = parser.parse_next();
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.kind, ParseErrorKind::Incomplete);
        }
    }

    #[test]
    fn test_error_kind_incomplete_bson() {
        let input = b"{\"key\":";
        let result = BsonParser::parse_one(input);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.kind, ParseErrorKind::Incomplete);
        }
    }

    #[test]
    fn test_error_kind_invalid_number() {
        let result = BsonParser::parse_one(b"1.2.3");
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(
                e.kind == ParseErrorKind::InvalidNumber || e.kind == ParseErrorKind::TrailingData,
                "Expected InvalidNumber or TrailingData, got {:?}", e.kind
            );
        }
    }

    #[test]
    fn test_error_kind_depth_limit_bson() {
        // Build deeply nested array that exceeds depth limit
        let mut json = String::new();
        for _ in 0..600 {
            json.push('[');
        }
        json.push('1');
        for _ in 0..600 {
            json.push(']');
        }
        let result = BsonParser::parse_one(json.as_bytes());
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.kind, ParseErrorKind::DepthLimit);
        }
    }

    #[test]
    fn test_error_kind_depth_limit_json() {
        let mut json = String::new();
        for _ in 0..600 {
            json.push('[');
        }
        json.push('1');
        for _ in 0..600 {
            json.push(']');
        }
        let result = JsonParser::parse_one(&json);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.kind, ParseErrorKind::DepthLimit);
        }
    }

    #[test]
    fn test_error_kind_depth_limit_stack() {
        let mut json = String::new();
        for _ in 0..600 {
            json.push('[');
        }
        json.push('1');
        for _ in 0..600 {
            json.push(']');
        }
        let mut parser = StackParser::new();
        parser.feed(json.as_bytes()).unwrap();
        parser.end_of_input();
        let result = parser.parse_next();
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.kind, ParseErrorKind::DepthLimit);
        }
    }

    #[test]
    fn test_error_kind_depth_limit_custom() {
        // Use a small custom depth limit
        let mut parser = BsonParser::new();
        parser.max_depth(2);
        parser.feed(b"[[[1]]]").unwrap();
        parser.end_of_input();
        let result = parser.parse_full();
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.kind, ParseErrorKind::DepthLimit);
        }
    }

    #[test]
    fn test_error_kind_unexpected_token() {
        let result = BsonParser::parse_one(b"@");
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e.kind, ParseErrorKind::UnexpectedToken { .. }));
        }
    }

    #[test]
    fn test_error_kind_unexpected_token_stack() {
        let mut parser = StackParser::new();
        parser.feed(b"@").unwrap();
        parser.end_of_input();
        let result = parser.parse_next();
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e.kind, ParseErrorKind::UnexpectedToken { .. }));
        }
    }

    #[test]
    fn test_error_kind_invalid_encoding() {
        // Feed raw invalid UTF-8 inside a string to trigger InvalidEncoding
        let input: &[u8] = b"\"hello \xff world\"";
        let result = BsonParser::parse_one(input);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(
                matches!(e.kind, ParseErrorKind::InvalidEncoding(_)),
                "Expected InvalidEncoding, got {:?}", e.kind
            );
        }
    }

    #[test]
    fn test_error_display_formatting() {
        // Verify ParseError Display impl works for each variant
        let e = ParseError::new(ParseErrorKind::Incomplete, 5);
        assert!(e.to_string().contains("incomplete"));

        let e = ParseError::new(ParseErrorKind::TrailingData, 10);
        assert!(e.to_string().contains("trailing"));

        let e = ParseError::new(ParseErrorKind::DepthLimit, 0);
        assert!(e.to_string().contains("depth"));

        let e = ParseError::new(ParseErrorKind::InvalidNumber, 3);
        assert!(e.to_string().contains("number"));

        let e = ParseError::new(ParseErrorKind::InvalidEncoding("bad utf8"), 0);
        assert!(e.to_string().contains("encoding"));

        let e = ParseError::new(ParseErrorKind::Message("custom"), 0);
        assert!(e.to_string().contains("custom"));

        let e = ParseError::new(ParseErrorKind::UnexpectedToken {
            found: "x".to_string(),
            expected: &["number", "string"],
        }, 0);
        let s = e.to_string();
        assert!(s.contains("unexpected"));
        assert!(s.contains("x"));
    }
}
