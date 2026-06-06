use crate::object::Value;
use super::BinWriter;

pub trait ValueSerializer<O: ?Sized> {
    /// Serialization error type
    type Error;

    /// Owned serialized output (e.g. `String` for `str`, `Vec<u8>` for `[u8]`)
    type Output: AsRef<O>;

    /// Serialize a single Value to owned output.
    fn serialize_one(value: &Value) -> Result<Self::Output, Self::Error>
    where
        Self: Sized;

    /// Serialize a single Value into a caller-provided [`BinWriter`].
    ///
    /// Useful for reusing buffer allocations across multiple serializations.
    /// To stream the result to an `io::Write` sink (file, socket), call
    /// [`BinWriter::flush`] or [`BinWriter::write_to`] afterward.
    fn serialize_buf(value: &Value, writer: &mut BinWriter) -> Result<(), Self::Error>
    where
        Self: Sized;
}
