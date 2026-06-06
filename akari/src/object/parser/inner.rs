#[cfg(feature = "no_std")]
use crate::prelude::*;

/// Binary inner buffer - format-agnostic buffer and position management
///
/// This struct provides low-level buffer management and byte navigation primitives
/// that can be reused across different data formats (JSON, TOML, MessagePack, etc.).
///
/// # Design Philosophy
/// - **Format-agnostic**: No JSON/TOML/etc-specific logic here
/// - **Just buffer + position**: Core state management only
/// - **Reusable**: Any byte-based parser can use this
///
/// # What goes here
/// - Buffer management: feed, compact
/// - Byte navigation: peek_byte, next_byte
/// - Whitespace skipping: skip_whitespace (ASCII whitespace is universal)
///
/// # What does NOT go here
/// - Format-specific parsing (parse_string, parse_number, etc.)
/// - Those belong in format-specific modules (e.g., json::primitive_parsing)
#[derive(Debug)]
pub struct BinInner {
    buffer: Vec<u8>,
    pos: usize,
}

impl BinInner {
    /// Create a new BinInner with empty buffer
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            pos: 0,
        }
    }

    /// Get current position
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Set current position (for backtracking/error recovery)
    ///
    /// Use this to restore position after failed parsing attempts.
    /// See format-specific parsing modules for usage patterns.
    pub fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
    }

    /// Get reference to buffer
    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    /// Feed more data into the buffer
    pub fn feed(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    /// Clear consumed data from buffer (keep only unparsed portion)
    pub fn compact(&mut self) {
        if self.pos > 0 {
            self.buffer.drain(0..self.pos);
            self.pos = 0;
        }
    }

    // ===== Byte navigation =====

    /// Peek at the current byte without consuming it
    pub fn peek_byte(&self) -> Option<u8> {
        self.buffer.get(self.pos).copied()
    }

    /// Consume and return the current byte
    pub fn next_byte(&mut self) -> Option<u8> {
        if let Some(&b) = self.buffer.get(self.pos) {
            self.pos += 1;
            Some(b)
        } else {
            None
        }
    }

    // ===== Whitespace handling =====

    /// Skip ASCII whitespace (space, tab, newline, carriage return)
    pub fn skip_whitespace(&mut self) {
        while let Some(&b) = self.buffer.get(self.pos) {
            if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }
}

impl Default for BinInner {
    fn default() -> Self {
        Self::new()
    }
}
