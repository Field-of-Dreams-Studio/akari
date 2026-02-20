//! JSON serialization implementation
//!
//! Compact JSON serializer for Akari `Value`.

use crate::object::Value;

use super::error::SerializeError;
use super::writer::BinWriter;
use super::ValueSerializer;

/// JSON serializer for Akari values.
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonSerializer;

impl JsonSerializer {
    const MAX_DEPTH: usize = 512;

    fn serialize_value(writer: &mut BinWriter, value: &Value, depth: usize) -> Result<(), SerializeError> {
        match value {
            Value::None => {
                writer.write_str("null");
                Ok(())
            }
            Value::Boolean(b) => {
                if *b {
                    writer.write_str("true");
                } else {
                    writer.write_str("false");
                }
                Ok(())
            }
            Value::Numerical(n) => {
                if !n.is_finite() {
                    return Err(
                        SerializeError::invalid_value("NaN or Infinity is not valid JSON number")
                            .with_context(n.to_string()),
                    );
                }
                writer.write_str(&n.to_string());
                Ok(())
            }
            Value::Str(s) => serialize_string(writer, s),
            Value::List(items) => {
                if depth >= Self::MAX_DEPTH {
                    return Err(SerializeError::depth_limit());
                }

                writer.write_byte(b'[');
                for (idx, item) in items.iter().enumerate() {
                    if idx > 0 {
                        writer.write_byte(b',');
                    }
                    Self::serialize_value(writer, item, depth + 1)?;
                }
                writer.write_byte(b']');
                Ok(())
            }
            Value::Dict(map) => {
                if depth >= Self::MAX_DEPTH {
                    return Err(SerializeError::depth_limit());
                }

                writer.write_byte(b'{');
                for (idx, (k, v)) in map.iter().enumerate() {
                    if idx > 0 {
                        writer.write_byte(b',');
                    }
                    serialize_string(writer, k)?;
                    writer.write_byte(b':');
                    Self::serialize_value(writer, v, depth + 1)?;
                }
                writer.write_byte(b'}');
                Ok(())
            }
        }
    }
}

impl ValueSerializer<str> for JsonSerializer {
    type Error = SerializeError;
    type Output = String;

    fn serialize_one(value: &Value) -> Result<Self::Output, Self::Error> {
        let mut writer = BinWriter::new();
        Self::serialize_value(&mut writer, value, 0)?;
        writer
            .into_string()
            .map_err(|e| SerializeError::message("serializer produced invalid UTF-8").with_context(e.to_string()))
    }

    fn serialize_to<W: std::io::Write + ?Sized>(value: &Value, writer: &mut W) -> Result<(), Self::Error> {
        let mut bin_writer = BinWriter::new();
        Self::serialize_value(&mut bin_writer, value, 0)?;
        std::io::Write::write_all(writer, bin_writer.as_bytes()).map_err(SerializeError::from)
    }
}

/// Serialize a string value with proper JSON escaping
///
/// This function properly escapes:
/// - `\"` (quotation mark)
/// - `\\` (backslash)
/// - `\n` (newline)
/// - `\r` (carriage return)
/// - `\t` (tab)
/// - `\b` (backspace)
/// - `\f` (form feed)
/// - `\uXXXX` (unicode escape for other control characters)
pub fn serialize_string(writer: &mut BinWriter, s: &str) -> Result<(), SerializeError> {
    writer.write_byte(b'"');

    for c in s.chars() {
        match c {
            '"' => writer.write_str("\\\""),  // Escape double quotes
            '\\' => writer.write_str("\\\\"), // Escape backslashes
            '\n' => writer.write_str("\\n"),  // Escape newlines
            '\r' => writer.write_str("\\r"),  // Escape carriage returns
            '\t' => writer.write_str("\\t"),  // Escape tabs
            '\u{0008}' => writer.write_str("\\b"), // Escape backspace
            '\u{000C}' => writer.write_str("\\f"), // Escape form feed
            _ if c.is_control() => {
                // Escape other control characters as unicode
                write_unicode_escape(writer, c as u32);
            }
            _ => {
                // Regular characters - write as UTF-8
                let mut buf = [0u8; 4];
                let encoded = c.encode_utf8(&mut buf);
                writer.write_str(encoded);
            }
        }
    }

    writer.write_byte(b'"');
    Ok(())
}

/// Write a unicode escape sequence \uXXXX
fn write_unicode_escape(writer: &mut BinWriter, code_point: u32) {
    const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

    writer.write_str("\\u");
    writer.write_byte(HEX_DIGITS[((code_point >> 12) & 0xF) as usize]);
    writer.write_byte(HEX_DIGITS[((code_point >> 8) & 0xF) as usize]);
    writer.write_byte(HEX_DIGITS[((code_point >> 4) & 0xF) as usize]);
    writer.write_byte(HEX_DIGITS[(code_point & 0xF) as usize]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::object::parser::BsonParser;

    #[test]
    fn test_serialize_simple_string() {
        let mut writer = BinWriter::new();
        serialize_string(&mut writer, "hello").unwrap();
        assert_eq!(writer.as_str(), r#""hello""#);
    }

    #[test]
    fn test_serialize_string_with_escapes() {
        let mut writer = BinWriter::new();
        serialize_string(&mut writer, "hello\nworld").unwrap();
        assert_eq!(writer.as_str(), r#""hello\nworld""#);
    }

    #[test]
    fn test_serialize_string_with_quotes() {
        let mut writer = BinWriter::new();
        serialize_string(&mut writer, r#"say "hi""#).unwrap();
        assert_eq!(writer.as_str(), r#""say \"hi\"""#);
    }

    #[test]
    fn test_serialize_string_with_backslash() {
        let mut writer = BinWriter::new();
        serialize_string(&mut writer, r"C:\path\file").unwrap();
        assert_eq!(writer.as_str(), r#""C:\\path\\file""#);
    }

    #[test]
    fn test_serialize_string_with_control_chars() {
        let mut writer = BinWriter::new();
        serialize_string(&mut writer, "\x00\x01\x1F").unwrap();
        assert_eq!(writer.as_str(), r#""\u0000\u0001\u001f""#);
    }

    #[test]
    fn test_serialize_string_unicode() {
        let mut writer = BinWriter::new();
        serialize_string(&mut writer, "Hello 世界 🌍").unwrap();
        assert_eq!(writer.as_str(), r#""Hello 世界 🌍""#);
    }

    #[test]
    fn test_json_serialize_primitives() {
        assert_eq!(
            JsonSerializer::serialize_one(&Value::Numerical(42.0)).unwrap(),
            "42"
        );
        assert_eq!(
            JsonSerializer::serialize_one(&Value::Boolean(false)).unwrap(),
            "false"
        );
        assert_eq!(
            JsonSerializer::serialize_one(&Value::None).unwrap(),
            "null"
        );
    }

    #[test]
    fn test_json_serialize_nested_and_parse_back() {
        let mut nested = HashMap::new();
        nested.insert("ok".to_string(), Value::Boolean(true));
        nested.insert("msg".to_string(), Value::Str("hello\nworld".to_string()));

        let value = Value::List(vec![
            Value::Numerical(1.0),
            Value::Dict(nested),
            Value::None,
        ]);

        let out = JsonSerializer::serialize_one(&value).unwrap();
        let reparsed = BsonParser::parse_one(out.as_bytes()).unwrap();
        assert_eq!(reparsed, value);
    }

    #[test]
    fn test_json_serialize_to_writer() {
        let value = Value::Str("abc".to_string());
        let mut out = Vec::new();
        JsonSerializer::serialize_to(&value, &mut out).unwrap();
        assert_eq!(out, br#""abc""#);
    }

    #[test]
    fn test_json_serialize_nan_error() {
        let err = JsonSerializer::serialize_one(&Value::Numerical(f64::NAN)).unwrap_err();
        assert!(matches!(
            err.kind,
            super::super::error::SerializeErrorKind::InvalidValue(_)
        ));
    }

    #[test]
    fn test_value_into_json_delegates_to_json_serializer() {
        let value = Value::List(vec![
            Value::Numerical(1.0),
            Value::Str("x".to_string()),
            Value::Boolean(true),
        ]);
        let via_value = value.into_json();
        let via_serializer = JsonSerializer::serialize_one(&value).unwrap();
        assert_eq!(via_value, via_serializer);
    }
}
