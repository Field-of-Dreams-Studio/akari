use crate::object::Value;

pub trait ValueSerializer<O: ?Sized> {
    /// Serialization error type
    type Error;

    /// Owned serialized output (e.g. `String` for `str`, `Vec<u8>` for `[u8]`)
    type Output: AsRef<O>;

    /// Serialize a single Value to owned output.
    fn serialize_one(value: &Value) -> Result<Self::Output, Self::Error>
    where
        Self: Sized;

    /// Serialize a single Value directly to any writer.
    fn serialize_to<W: std::io::Write + ?Sized>(value: &Value, writer: &mut W) -> Result<(), Self::Error>
    where
        Self: Sized;
}
