//! JSON serialization implementation
//!
//! Simple JSON serializer starting with string serialization.

use super::writer::BinWriter;
use super::error::SerializeError;

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
}
