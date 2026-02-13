use std::io::Write;

use crate::object::Value;

pub trait ValueSerializer<O: ?Sized> {
    /// Output type: String for str, Vec<u8> for [u8]
    type Output;

    /// Serialization error type
    type Error;

    /// Serialize a Value to owned output (String or Vec<u8>)
    fn serialize_one(value: &Value) -> Result<Self::Output, Self::Error>
    where
        Self: Sized;

    /// Serialize a Value to any Writer (TcpStream, File, Vec, etc.)
    fn serialize_to<W: Write>(value: &Value, writer: &mut W) -> Result<(), Self::Error>
    where
        Self: Sized;
}
