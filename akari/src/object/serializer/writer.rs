use std::io::Write;

/// Binary writer - format-agnostic output buffer management
///
/// This struct provides low-level buffer management for serialization
/// that can be reused across different data formats (JSON, TOML, MessagePack, etc.).
///
/// # Design Philosophy
/// - **Format-agnostic**: No JSON/TOML/etc-specific logic here
/// - **Just buffer management**: Core output handling only
/// - **Reusable**: Any byte-based serializer can use this
///
/// # What goes here
/// - Buffer management: write_byte, write_bytes, write_str
/// - Capacity management: reserve, clear
/// - Output abstraction: works with any `Write` implementor
///
/// # What does NOT go here
/// - Format-specific serialization (serialize_string, serialize_number, etc.)
/// - Those belong in format-specific modules (e.g., json::primitive_serializing)
///
/// # Note
/// Using `BinWriter` is **optional** for implementing `ValueSerializer`.
/// It's provided as a convenience utility, but implementations can use
/// their own buffering strategy if preferred.
///
/// # Example
/// ```
/// use akari::object::serializer::BinWriter;
///
/// let mut writer = BinWriter::new();
/// writer.write_str("Hello");
/// writer.write_byte(b' ');
/// writer.write_str("World");
///
/// let output = writer.into_string().unwrap();
/// assert_eq!(output, "Hello World");
/// ```
#[derive(Debug, Clone)]
pub struct BinWriter {
    buffer: Vec<u8>,
}

impl BinWriter {
    /// Create a new BinWriter with empty buffer
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
        }
    }

    /// Create a new BinWriter with specified capacity
    ///
    /// Pre-allocating capacity is useful when you know the approximate output size.
    ///
    /// # Example
    /// ```ignore
    /// // For typical JSON objects, 4KB is a good starting point
    /// let writer = BinWriter::with_capacity(4096);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
        }
    }

    /// Get current buffer size
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Get buffer capacity
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Reserve additional capacity
    ///
    /// This can prevent multiple reallocations when writing large amounts of data.
    pub fn reserve(&mut self, additional: usize) {
        self.buffer.reserve(additional);
    }

    /// Clear buffer (keeps capacity)
    ///
    /// Useful for reusing the same writer for multiple serializations
    /// without deallocating the buffer.
    ///
    /// # Example
    /// ```ignore
    /// for value in values {
    ///     serialize_value(&mut writer, value)?;
    ///     let output = writer.as_bytes();
    ///     send_to_network(output)?;
    ///     writer.clear(); // Reuse buffer
    /// }
    /// ```
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    // ===== Write operations =====

    /// Write a single byte
    #[inline]
    pub fn write_byte(&mut self, byte: u8) {
        self.buffer.push(byte);
    }

    /// Write multiple bytes
    #[inline]
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    /// Write a string as UTF-8 bytes
    #[inline]
    pub fn write_str(&mut self, s: &str) {
        self.buffer.extend_from_slice(s.as_bytes());
    }

    // ===== Access operations =====

    /// Get immutable reference to internal buffer
    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer
    }

    /// Get mutable reference to internal buffer
    ///
    /// Use with caution - direct manipulation can break assumptions.
    pub fn as_bytes_mut(&mut self) -> &mut Vec<u8> {
        &mut self.buffer
    }

    /// Convert buffer to string slice (assumes valid UTF-8)
    ///
    /// # Panics
    /// Panics if buffer contains invalid UTF-8
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.buffer).expect("BinWriter buffer contains invalid UTF-8")
    }

    // ===== Conversion operations =====

    /// Consume writer and return buffer as Vec<u8>
    pub fn into_vec(self) -> Vec<u8> {
        self.buffer
    }

    /// Consume writer and return buffer as String
    ///
    /// Returns Err if buffer contains invalid UTF-8
    ///
    /// # Example
    /// ```ignore
    /// let mut writer = BinWriter::new();
    /// writer.write_str("{\"name\":\"Alice\"}");
    /// let json = writer.into_string()?;
    /// ```
    pub fn into_string(self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.buffer)
    }

    // ===== I/O operations =====

    /// Flush buffer to writer and clear
    ///
    /// This writes all buffered data to the writer and then clears the buffer,
    /// allowing it to be reused.
    ///
    /// # Example
    /// ```ignore
    /// let mut writer = BinWriter::new();
    /// let mut file = File::create("output.json")?;
    ///
    /// for value in large_dataset {
    ///     serialize_value(&mut writer, value)?;
    ///     if writer.len() > 4096 {
    ///         writer.flush(&mut file)?; // Write and clear
    ///     }
    /// }
    /// writer.flush(&mut file)?; // Final flush
    /// ```
    pub fn flush<W: Write>(&mut self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.buffer)?;
        self.buffer.clear();
        Ok(())
    }

    /// Write buffer to writer without clearing
    ///
    /// Useful when you want to write the buffer but keep it for inspection.
    pub fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.buffer)
    }
}

impl Default for BinWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl From<BinWriter> for Vec<u8> {
    fn from(writer: BinWriter) -> Self {
        writer.into_vec()
    }
}

impl AsRef<[u8]> for BinWriter {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}
