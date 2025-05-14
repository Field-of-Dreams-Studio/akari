use std::collections::HashMap;

use super::value::Value; 

// Recursive-descent JSON parser 
pub(super) struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Parser { input, pos: 0 }
    }
    
    pub fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    } 

    pub fn get_pos(&self) -> usize { 
        self.pos 
    }
    
    pub fn next(&mut self) -> Option<char> {
        if let Some(ch) = self.peek() {
            self.pos += ch.len_utf8();
            Some(ch)
        } else {
            None
        }
    }
    
    pub fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() { self.next(); } else { break; }
        }
    }
    
    pub fn parse_value(&mut self) -> Result<Value, String> {
        self.skip_whitespace();
        match self.peek() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => self.parse_string().map(Value::Str),
            Some(ch) if ch == 't' || ch == 'f' => self.parse_boolean().map(Value::Boolean),
            Some(ch) if ch.is_digit(10) || ch == '-' => self.parse_number().map(Value::Numerical),
            _ => Err(format!("Unexpected character at position {}: {:?}", self.pos, self.peek())),
        }
    }
    
    fn parse_object(&mut self) -> Result<Value, String> {
        let mut map = HashMap::new();
        if self.next() != Some('{') {
            return Err(format!("Expected '{{' at position {}", self.pos));
        }
        self.skip_whitespace();
        if let Some('}') = self.peek() {
            self.next();
            return Ok(Value::Dictionary(map));
        }
        loop {
            self.skip_whitespace();
            if self.peek() != Some('"') {
                return Err(format!("Expected '\"' at position {} for object key", self.pos));
            }
            let key = self.parse_string()?;
            self.skip_whitespace();
            if self.next() != Some(':') {
                return Err(format!("Expected ':' after key at position {}", self.pos));
            }
            self.skip_whitespace();
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();
            match self.peek() {
                Some(',') => { self.next(); },
                Some('}') => { self.next(); break; },
                _ => return Err(format!("Expected ',' or '}}' at position {}", self.pos)),
            }
        }
        Ok(Value::Dictionary(map))
    }
    
    fn parse_array(&mut self) -> Result<Value, String> {
        let mut vec = Vec::new();
        if self.next() != Some('[') {
            return Err(format!("Expected '[' at position {}", self.pos));
        }
        self.skip_whitespace();
        if let Some(']') = self.peek() {
            self.next();
            return Ok(Value::List(vec));
        }
        loop {
            self.skip_whitespace();
            let value = self.parse_value()?;
            vec.push(value);
            self.skip_whitespace();
            match self.peek() {
                Some(',') => { self.next(); },
                Some(']') => { self.next(); break; },
                _ => return Err(format!("Expected ',' or ']' at position {}", self.pos)),
            }
        }
        Ok(Value::List(vec))
    }
    
    fn parse_string(&mut self) -> Result<String, String> {
        let mut result = String::new();
        if self.next() != Some('"') {
            return Err("Expected '\"' at beginning of string".to_string());
        }
        while let Some(ch) = self.next() {
            if ch == '"' { return Ok(result); }
            if ch == '\\' {
                if let Some(esc) = self.next() {
                    match esc {
                        '"'  => result.push('"'),
                        '\\' => result.push('\\'),
                        '/'  => result.push('/'),
                        'b'  => result.push('\x08'),
                        'f'  => result.push('\x0C'),
                        'n'  => result.push('\n'),
                        'r'  => result.push('\r'),
                        't'  => result.push('\t'),
                        _    => return Err(format!("Invalid escape sequence: \\{}", esc)),
                    }
                } else {
                    return Err("Incomplete escape sequence".to_string());
                }
            } else {
                result.push(ch);
            }
        }
        Err("Unterminated string literal".to_string())
    }
    
    fn parse_number(&mut self) -> Result<f64, String> {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_digit(10) || ch == '.' || ch == '-' || ch == 'e' || ch == 'E' || ch == '+' {
                self.next();
            } else {
                break;
            }
        }
        let number_str = &self.input[start..self.pos];
        number_str.parse::<f64>().map_err(|_| format!("Invalid number: {}", number_str))
    }
    
    fn parse_boolean(&mut self) -> Result<bool, String> {
        if self.input[self.pos..].starts_with("true") {
            self.pos += 4;
            Ok(true)
        } else if self.input[self.pos..].starts_with("false") {
            self.pos += 5;
            Ok(false)
        } else {
            Err(format!("Invalid boolean value at position {}", self.pos))
        }
    }
} 

