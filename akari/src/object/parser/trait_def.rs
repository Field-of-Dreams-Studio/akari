use crate::object::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Next<T> {
    Value(T),
    NeedMore,
    Eof,
}

/// Trait for parsing Akari `Value`s from text or binary input.
///
/// This trait is designed for both:
/// - **Non-streaming** parsing (one-shot parsing from a complete buffer), and
/// - **Streaming** parsing (incremental parsing with `feed()` as data arrives).
///
/// Implementations typically choose the input slice type `I`:
/// - Text formats: `I = str`
/// - Binary formats: `I = [u8]`
pub trait ValueParser<I: ?Sized> {
    type Error;

    // ===== Initialization & Input Management =====

    /// Create a new, empty parser instance.
    fn new() -> Self;

    /// Append additional input to the parser's internal buffer.
    ///
    /// This method enables incremental/streaming parsing. The parser should retain any
    /// incomplete state internally and continue parsing when more data is fed.
    fn feed(&mut self, input: &I) -> Result<(), Self::Error>;

    /// Signal that no more input will be provided.
    ///
    /// After calling this method, the parser must treat "need more data" as a definitive
    /// `Incomplete` error (because no more bytes/chars will arrive).
    fn end_of_input(&mut self);

    // ===== Parsing Methods =====

    /// One-shot parse: parse exactly ONE complete `Value` from the given input.
    ///
    /// Equivalent to: `new()` + `feed()` + `end_of_input()` + `parse_full()`.
    fn parse_one(input: &I) -> Result<Value, Self::Error>
    where
        Self: Sized,
    {
        let mut p = Self::new();
        p.feed(input)?;
        p.end_of_input();
        p.parse_full()
    }

    /// Strict parse: parse exactly one complete `Value` and reject trailing non-ignorable data.
    fn parse_full(&mut self) -> Result<Value, Self::Error>;

    /// Alias for `parse_full()`.
    fn fparse(&mut self) -> Result<Value, Self::Error> {
        self.parse_full()
    }

    /// Streaming parse step: attempt to parse the next `Value`.
    ///
    /// Returns:
    /// - `Ok(Next::Value(v))` if a full value was parsed and the internal cursor advanced.
    /// - `Ok(Next::NeedMore)` if more input is required to complete the next value.
    /// - `Ok(Next::Eof)` if there is no more value available in the current buffer.
    ///
    /// After `end_of_input()` has been called, implementations should prefer returning an
    /// `Incomplete` error instead of `NeedMore` when the remaining buffered input cannot
    /// form a complete value.
    fn parse_next(&mut self) -> Result<Next<Value>, Self::Error>;

    /// Alias for `parse_next()`.
    fn pparse(&mut self) -> Result<Next<Value>, Self::Error> {
        self.parse_next()
    }

    /// Current parsing position (0-based; recommended: byte offset in the logical stream).
    fn pos(&self) -> usize {
        0
    }
}
