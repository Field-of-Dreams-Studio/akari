use core::fmt;
use std::io;

/// The category of a serialization failure.
///
/// `SerializeErrorKind` is intentionally structured (instead of a free-form `String`) so callers
/// can reliably distinguish conditions such as *I/O errors* vs *invalid values* vs *depth limits*,
/// and implement robust error handling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SerializeErrorKind {
    /// An I/O error occurred while writing output.
    ///
    /// This wraps the `io::ErrorKind` since `io::Error` is not Clone.
    /// The original error message is preserved in the `SerializeError.context` field.
    IoError(io::ErrorKind),

    /// The Value contains data that cannot be serialized in the target format.
    ///
    /// Examples:
    /// - NaN or Infinity in JSON (not allowed by spec)
    /// - Keys that are not strings in JSON objects
    /// - Unsupported Value variants for specific formats
    InvalidValue(&'static str),

    /// The Value structure exceeds the configured maximum nesting depth.
    ///
    /// This is a security measure to prevent stack overflow from malicious
    /// or accidentally deeply-nested data structures.
    DepthLimit,

    /// The output buffer or writer is full and cannot accept more data.
    ///
    /// This is rare but can occur with fixed-size buffers or quota-limited writers.
    BufferFull,

    /// A generic static message for implementation-specific errors that do not fit other kinds.
    ///
    /// Prefer more specific variants when possible.
    Message(&'static str),
}

/// A structured serialization error with optional context.
///
/// Unlike `ParseError` which tracks position in input, `SerializeError` focuses on:
/// - What kind of error occurred
/// - Optional context (like the problematic value or field name)
#[derive(Debug, Clone)]
pub struct SerializeError {
    /// The structured error category.
    pub kind: SerializeErrorKind,

    /// Optional short, human-readable context.
    ///
    /// Examples:
    /// - Field name where error occurred: `"field 'user.age'"`
    /// - Value that caused error: `"NaN"`
    /// - Original I/O error message
    pub context: Option<String>,
}

impl SerializeError {
    /// Create a new `SerializeError` with the given kind.
    pub fn new(kind: SerializeErrorKind) -> Self {
        Self {
            kind,
            context: None,
        }
    }

    /// Attach short human-readable context to the error.
    pub fn with_context(mut self, ctx: impl Into<String>) -> Self {
        self.context = Some(ctx.into());
        self
    }

    /// Create an I/O error from an `io::Error`.
    pub fn from_io_error(err: io::Error) -> Self {
        Self {
            kind: SerializeErrorKind::IoError(err.kind()),
            context: Some(err.to_string()),
        }
    }

    /// Create an InvalidValue error with a message.
    pub fn invalid_value(msg: &'static str) -> Self {
        Self::new(SerializeErrorKind::InvalidValue(msg))
    }

    /// Create a depth limit error.
    pub fn depth_limit() -> Self {
        Self::new(SerializeErrorKind::DepthLimit)
    }

    /// Create a message error.
    pub fn message(msg: &'static str) -> Self {
        Self::new(SerializeErrorKind::Message(msg))
    }
}

impl fmt::Display for SerializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            SerializeErrorKind::IoError(kind) => {
                write!(f, "I/O error: {:?}", kind)?;
            }
            SerializeErrorKind::InvalidValue(msg) => {
                write!(f, "invalid value: {}", msg)?;
            }
            SerializeErrorKind::DepthLimit => {
                write!(f, "depth limit exceeded")?;
            }
            SerializeErrorKind::BufferFull => {
                write!(f, "buffer full")?;
            }
            SerializeErrorKind::Message(msg) => {
                write!(f, "{}", msg)?;
            }
        }

        if let Some(ctx) = &self.context {
            write!(f, " - {}", ctx)?;
        }

        Ok(())
    }
}

impl std::error::Error for SerializeError {}

impl From<io::Error> for SerializeError {
    fn from(err: io::Error) -> Self {
        Self::from_io_error(err)
    }
}
