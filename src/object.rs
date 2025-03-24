use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Numerical(f64),
    Boolean(bool), 
    Str(String),
    List(Vec<Object>),
    Dictionary(HashMap<String, Object>),  
    None, 
} 

impl Object { 
    /// Creates a new Object from a value. 
    /// This function will convert the value into an Object.
    /// # Example 
    /// ```rust 
    /// use akari::Object; 
    /// use std::collections::HashMap; 
    /// let obj = Object::new(42); 
    /// assert_eq!(obj, Object::Numerical(42.0)); 
    /// let obj = Object::new("hello"); 
    /// assert_eq!(obj, Object::Str("hello".to_string())); 
    /// let obj = Object::new(true); 
    /// assert_eq!(obj, Object::Boolean(true)); 
    /// let obj = Object::new(vec![1, 2, 3]);
    /// assert_eq!(obj, Object::List(vec![Object::Numerical(1.0), Object::Numerical(2.0), Object::Numerical(3.0)])); 
    /// let obj = Object::new(HashMap::from([("key", "value")])); 
    /// assert_eq!(obj, Object::Dictionary(HashMap::from([("key".to_string(), Object::Str("value".to_string()))]))); 
    /// ``` 
    /// 
    /// # Old grammar 
    /// ```rust
    /// use akari::Object; 
    /// use std::collections::HashMap; 
    /// let obj = Object::new(vec![Object::new(1), Object::new(2), Object::new(3)]);
    /// assert_eq!(obj, Object::List(vec![Object::Numerical(1.0), Object::Numerical(2.0), Object::Numerical(3.0)])); 
    /// let obj = Object::new(HashMap::from([("key".to_string(), Object::Str("value".to_string()))])); 
    /// assert_eq!(obj, Object::Dictionary(HashMap::from([("key".to_string(), Object::Str("value".to_string()))]))); 
    /// ``` 
    pub fn new<T: Into<Object>>(value: T) -> Self {
        value.into()
    }
    
    pub fn type_of(&self) -> String {
        match self {
            Object::Numerical(_) => "num".to_string(),
            Object::Boolean(_) => "bool".to_string(),
            Object::Str(_) => "str".to_string(),
            Object::List(_) => "vec".to_string(),
            Object::Dictionary(_) => "dict".to_string(),
            Object::None => "none".to_string(), 
        }
    }
    
    /// Converts the Object into a JSON string representation. 
    /// This function will return a string that is a valid JSON representation of the Object. 
    /// # Example 
    /// ```rust 
    /// use akari::object::Object; 
    /// let obj = Object::Dictionary(HashMap::from([ 
    ///    ("key".to_string(), Object::Str("value".to_string())), 
    ///    ("number".to_string(), Object::Numerical(42.0)), 
    ///    ("list".to_string(), Object::List(vec![Object::Numerical(1.0), Object::Numerical(2.0), Object::Numerical(3.0)])), 
    /// ])); 
    /// let json = obj.into_json(); 
    /// println!(json); // Output: {"key": "value", "number": 42, "list": [1, 2, 3]} 
    /// ``` 
    pub fn into_json(&self) -> String {
        match self {
            Object::None => "none".to_string(), 
            Object::Numerical(n) => n.to_string(),
            Object::Boolean(b) => b.to_string(),
            Object::Str(s) => format!("\"{}\"", s),
            Object::List(l) => {
                let mut result = String::new();
                for item in l {
                    result.push_str(&format!("{}, ", item.into_json()));
                }
                if result.len() >= 2 { result.truncate(result.len() - 2); }
                format!("[{}]", result)
            }
            Object::Dictionary(d) => {
                let mut result = String::new();
                for (key, value) in d {
                    result.push_str(&format!("\"{}\": {}, ", key, value.into_json()));
                }
                if result.len() >= 2 { result.truncate(result.len() - 2); }
                format!("{{{}}}", result)
            }
        }
    }
    
    /// Parses a JSON string and returns an Object. 
    /// This function will return an error if the JSON is invalid or if there are extra characters after the JSON value. 
    /// # Example 
    /// ```rust 
    /// use akari::object::Object; 
    /// let json = r#"{"key": "value", "number": 42, "list": [1, 2, 3]}"#; 
    /// let obj = Object::from_json(json).expect("Failed to parse JSON"); 
    /// assert_eq!(obj, Object::Dictionary(HashMap::from([
    ///     ("key".to_string(), Object::Str("value".to_string())),
    ///     ("number".to_string(), Object::Numerical(42.0)),
    ///     ("list".to_string(), Object::List(vec![Object::Numerical(1.0), Object::Numerical(2.0), Object::Numerical(3.0)])), 
    ///     ]))); 
    /// ``` 
    /// # Errors 
    /// This function will return an error if the JSON is invalid or if there are extra characters after the JSON value. 
    pub fn from_json(json: &str) -> Result<Self, String> {
        let mut parser = Parser::new(json);
        let value = parser.parse_value()?;
        parser.skip_whitespace();
        if parser.pos != json.len() {
            return Err("Extra characters after JSON value".to_string());
        }
        Ok(value)
    } 

    /// Retrieves a value from the dictionary by path. 
    /// This function will return None if the path is invalid or if the key does not exist. 
    /// # Example 
    /// ```rust 
    /// use akari::object::Object; 
    /// use std::collections::HashMap; 
    /// let mut map = HashMap::new(); 
    /// map.insert("key".to_string(), Object::Str("value".to_string())); 
    /// let obj = Object::Dictionary(map); 
    /// let value = obj.get_path("key"); 
    /// assert_eq!(value, Some(&Object::Str("value".to_string()))); 
    /// ``` 
    pub fn get_path(&self, path: &str) -> Option<&Object> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = self;
        
        for part in parts {
            // Handle array indexing
            if let Some(idx_part) = part.strip_suffix(']') {
                let parts: Vec<&str> = idx_part.split('[').collect();
                if parts.len() != 2 {
                    return None;
                }
                
                let obj_key = parts[0];
                let idx_str = parts[1];
                
                // Get object by key first
                if !obj_key.is_empty() {
                    if let Some(obj) = current.get(obj_key) {
                        current = obj;
                    } else {
                        return None;
                    }
                }
                
                // Then get by index
                if let Ok(idx) = idx_str.parse::<usize>() {
                    if let Some(obj) = current.idx(idx) {
                        current = obj;
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else {
                // Regular dictionary access
                if let Some(obj) = current.get(part) {
                    current = obj;
                } else {
                    return None;
                }
            }
        }
        
        Some(current)
    } 

    /// Retrieves a value from the dictionary by key. 
    /// # Example 
    /// ```rust 
    /// use akari::object::Object; 
    /// use std::collections::HashMap; 
    /// let mut map = HashMap::new(); 
    /// map.insert("key".to_string(), Object::Str("value".to_string())); 
    /// let obj = Object::Dictionary(map); 
    /// let value = obj.get("key"); 
    /// assert_eq!(value, Some(&Object::Str("value".to_string()))); 
    /// ``` 
    pub fn get(&self, key: &str) -> Option<&Object> {
        if let Object::Dictionary(map) = self {
            map.get(key)
        } else {
            None
        }
    } 

    /// Sets a value in the dictionary by key. 
    /// # Example 
    /// ```rust 
    /// use akari::object::Object; 
    /// use std::collections::HashMap; 
    /// let mut map = HashMap::new(); 
    /// map.insert("key".to_string(), Object::Str("value".to_string())); 
    /// let mut obj = Object::Dictionary(map); 
    /// obj.set("key".to_string(), Object::Str("new_value".to_string())); 
    /// let value = obj.get("key"); 
    /// assert_eq!(value, Some(&Object::Str("new_value".to_string()))); 
    /// ``` 
    pub fn set(&mut self, key: String, value: Object) {
        if let Object::Dictionary(map) = self {
            map.insert(key, value);
        }
    } 

    /// Deletes a value from the dictionary by key. 
    /// # Example 
    /// ```rust 
    /// use akari::object::Object;
    /// use std::collections::HashMap; 
    /// let mut map = HashMap::new(); 
    /// map.insert("key".to_string(), Object::Str("value".to_string())); 
    /// let mut obj = Object::Dictionary(map); 
    /// let value = obj.delete("key"); 
    /// assert_eq!(value, Some(Object::Str("value".to_string()))); 
    /// ``` 
    /// This function will return None if the key does not exist. 
    pub fn delete(&mut self, key: &str) -> Option<Object> {
        if let Object::Dictionary(map) = self {
            map.remove(key)
        } else {
            None
        }
    } 

    /// Retrieves a value from the list by index.
    /// # Example
    /// ```rust 
    /// use akari::object::Object; 
    /// use std::collections::HashMap; 
    /// let list = Object::List(vec![Object::Str("value1".to_string()), Object::Str("value2".to_string())]); 
    /// let value = list.idx(1); 
    /// assert_eq!(value, Some(&Object::Str("value2".to_string()))); 
    /// ``` 
    pub fn idx(&self, index: usize) -> Option<&Object> {
        if let Object::List(vec) = self {
            vec.get(index)
        } else {
            None
        }
    } 

    /// Sets a value in the list by index. 
    /// # Example 
    /// ```rust 
    /// use akari::object::Object; 
    /// use std::collections::HashMap; 
    /// let mut list = Object::List(vec![Object::Str("value1".to_string()), Object::Str("value2".to_string())]); 
    /// list.insert(1, Object::Str("new_value".to_string())); 
    /// let value = list.idx(1); 
    /// assert_eq!(value, Some(&Object::Str("new_value".to_string()))); 
    /// ``` 
    /// This function will push the value to the end of the list if the index is out of bounds. 
    pub fn insert(&mut self, index: usize, value: Object) {
        if let Object::List(vec) = self {
            if index < vec.len() {
                vec[index] = value;
            } else {
                vec.push(value);
            }
        }
    } 

    /// Pushes a value to the end of the list. 
    /// # Example 
    /// ```rust 
    /// use akari::object::Object; 
    /// use std::collections::HashMap; 
    /// let mut list = Object::List(vec![Object::Str("value1".to_string()), Object::Str("value2".to_string())]); 
    /// list.push(Object::Str("new_value".to_string())); 
    /// let value = list.idx(2); 
    /// assert_eq!(value, Some(&Object::Str("new_value".to_string()))); 
    /// ``` 
    /// This function will push the value to the end of the list. 
    pub fn push(&mut self, value: Object) {
        if let Object::List(vec) = self {
            vec.push(value);
        }
    } 

    /// Pops a value from the end of the list. 
    /// # Example 
    /// ```rust 
    /// use akari::object::Object; 
    /// use std::collections::HashMap; 
    /// let mut list = Object::List(vec![Object::Str("value1".to_string()), Object::Str("value2".to_string())]); 
    /// let value = list.pop(); 
    /// assert_eq!(value, Some(Object::Str("value2".to_string()))); 
    /// ``` 
    pub fn pop(&mut self) -> Option<Object> {
        if let Object::List(vec) = self {
            vec.pop()
        } else {
            None
        }
    } 

    /// Removes a value from the list by index. 
    /// # Example 
    /// ```rust 
    /// use akari::object::Object; 
    /// use std::collections::HashMap; 
    /// let mut list = Object::List(vec![Object::Str("value1".to_string()), Object::Str("value2".to_string())]); 
    /// let value = list.remove(1); 
    /// assert_eq!(value, Some(Object::Str("value2".to_string()))); 
    /// ``` 
    pub fn remove(&mut self, index: usize) -> Option<Object> {
        if let Object::List(vec) = self {
            if index < vec.len() {
                Some(vec.remove(index))
            } else {
                None
            }
        } else {
            None
        }
    } 

    /// Returns the length of the Object. 
    /// # Example 
    /// ```rust 
    /// use akari::object::Object; 
    /// use std::collections::HashMap; 
    /// let list = Object::List(vec![Object::Str("value1".to_string()), Object::Str("value2".to_string())]); 
    /// let length = list.len(); 
    /// assert_eq!(length, 2); 
    /// let dict = Object::Dictionary(HashMap::from([ 
    ///    ("key".to_string(), Object::Str("value".to_string())), 
    /// ])); 
    /// let length = dict.len(); 
    /// assert_eq!(length, 1); 
    /// ``` 
    /// This function will return the length of the Object. 
    pub fn len(&self) -> usize {
        match self {
            Object::List(vec) => vec.len(),
            Object::Dictionary(map) => map.len(),
            _ => 1,
        }
    } 

    pub fn interal_value_as_string(&self) -> String {
        match self {
            Object::Str(s) => s.clone(),
            Object::Numerical(n) => n.to_string(),
            Object::Boolean(b) => b.to_string(),
            _ => "".to_string(),
        }
    } 

    pub fn format(&self) -> String {
        match self {
            Object::None => "none".to_string(),
            Object::Numerical(n) => format!("{}", n),
            Object::Boolean(b) => format!("{}", b),
            Object::Str(s) => format!("\"{}\"", s),
            Object::List(l) => {
                let mut result = String::new();
                for item in l {
                    result.push_str(&format!("{}, ", item));
                }
                if result.len() >= 2 {
                    result.truncate(result.len() - 2);
                }
                format!("[{}]", result)
            }
            Object::Dictionary(d) => {
                let mut result = String::new();
                for (key, value) in d {
                    result.push_str(&format!("{} {} = {}, ", value.type_of(), key, value));
                }
                if result.len() >= 2 {
                    result.truncate(result.len() - 2);
                }
                format!("{{{}}}", result)
            }
        }
    } 
}

impl std::fmt::Display for Object { 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Object::format(self)) 
    }
} 

// Implement Hash trait
impl Hash for Object {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Object::None => 0.hash(state),
            Object::Boolean(b) => {
                0.hash(state);
                b.hash(state);
            },
            Object::Numerical(n) => {
                1.hash(state);
                // Convert f64 to a bitwise representation for hashing
                n.to_bits().hash(state);
            },
            Object::Str(s) => {
                2.hash(state);
                s.hash(state);
            },
            Object::List(items) => {
                3.hash(state);
                // Hash the length and each element
                items.len().hash(state);
                for item in items {
                    item.hash(state);
                }
            },
            Object::Dictionary(dict) => {
                4.hash(state);
                // For dictionaries, hash the number of entries
                // We can't reliably hash the entries themselves as HashMap doesn't implement Hash
                dict.len().hash(state);
                // Note: This means dictionaries with the same length but different contents
                // will have the same hash, which is not ideal but prevents infinite recursion
            },
        }
    }
}

// Implement Eq trait (required since we already have PartialEq)
impl Eq for Object {} 

// Implement Into<String> trait 
impl Into<String> for Object {
    fn into(self) -> String {
        self.to_string() 
    } 
} 

// From implementations
impl From<i8> for Object { fn from(n: i8) -> Self { Object::Numerical(n as f64) } }
impl From<i16> for Object { fn from(n: i16) -> Self { Object::Numerical(n as f64) } }
impl From<i32> for Object { fn from(n: i32) -> Self { Object::Numerical(n as f64) } }
impl From<i64> for Object { fn from(n: i64) -> Self { Object::Numerical(n as f64) } }
impl From<i128> for Object { fn from(n: i128) -> Self { Object::Numerical(n as f64) } }
impl From<isize> for Object { fn from(n: isize) -> Self { Object::Numerical(n as f64) } }
impl From<u8> for Object { fn from(n: u8) -> Self { Object::Numerical(n as f64) } }
impl From<u16> for Object { fn from(n: u16) -> Self { Object::Numerical(n as f64) } }
impl From<u32> for Object { fn from(n: u32) -> Self { Object::Numerical(n as f64) } }
impl From<u64> for Object { fn from(n: u64) -> Self { Object::Numerical(n as f64) } }
impl From<u128> for Object { fn from(n: u128) -> Self { Object::Numerical(n as f64) } }
impl From<usize> for Object { fn from(n: usize) -> Self { Object::Numerical(n as f64) } }
impl From<f32> for Object { fn from(n: f32) -> Self { Object::Numerical(n as f64) } }
impl From<f64> for Object { fn from(n: f64) -> Self { Object::Numerical(n) } }
impl From<char> for Object { fn from(c: char) -> Self { Object::Str(c.to_string()) } }
impl From<bool> for Object { fn from(b: bool) -> Self { Object::Boolean(b) } }
impl From<&str> for Object { fn from(s: &str) -> Self { Object::Str(s.to_string()) } }
impl From<String> for Object { fn from(s: String) -> Self { Object::Str(s) } }
impl From<&String> for Object { fn from(s: &String) -> Self { Object::Str(s.clone()) } } 

// impl From<Vec<Object>> for Object { fn from(vec: Vec<Object>) -> Self { Object::List(vec) } }
// impl From<HashMap<String, Object>> for Object { fn from(map: HashMap<String, Object>) -> Self { Object::Dictionary(map) } }

// Implement From trait for Vec<T>
impl<T> From<Vec<T>> for Object 
where
    T: Into<Object>,
{
    fn from(vec: Vec<T>) -> Self {
        Object::List(vec.into_iter().map(Into::into).collect())
    }
}

// Implement From trait for HashMap<String, T>
impl<S, T> From<HashMap<S, T>> for Object 
where
    S: Into<String> + Hash + Eq,  
    T: Into<Object>, 
{
    fn from(map: HashMap<S, T>) -> Self {
        Object::Dictionary(map.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
    }
} 

// Recursive-descent JSON parser
struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser { input, pos: 0 }
    }
    
    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }
    
    fn next(&mut self) -> Option<char> {
        if let Some(ch) = self.peek() {
            self.pos += ch.len_utf8();
            Some(ch)
        } else {
            None
        }
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() { self.next(); } else { break; }
        }
    }
    
    fn parse_value(&mut self) -> Result<Object, String> {
        self.skip_whitespace();
        match self.peek() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => self.parse_string().map(Object::Str),
            Some(ch) if ch == 't' || ch == 'f' => self.parse_boolean().map(Object::Boolean),
            Some(ch) if ch.is_digit(10) || ch == '-' => self.parse_number().map(Object::Numerical),
            _ => Err(format!("Unexpected character at position {}: {:?}", self.pos, self.peek())),
        }
    }
    
    fn parse_object(&mut self) -> Result<Object, String> {
        let mut map = HashMap::new();
        if self.next() != Some('{') {
            return Err(format!("Expected '{{' at position {}", self.pos));
        }
        self.skip_whitespace();
        if let Some('}') = self.peek() {
            self.next();
            return Ok(Object::Dictionary(map));
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
        Ok(Object::Dictionary(map))
    }
    
    fn parse_array(&mut self) -> Result<Object, String> {
        let mut vec = Vec::new();
        if self.next() != Some('[') {
            return Err(format!("Expected '[' at position {}", self.pos));
        }
        self.skip_whitespace();
        if let Some(']') = self.peek() {
            self.next();
            return Ok(Object::List(vec));
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
        Ok(Object::List(vec))
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

/// A macro to create an Object from a literal or expression. 
/// It can handle dictionaries, lists, booleans, strings, and numeric values. 
/// # Example 
/// ```rust 
/// use akari::object::Object; 
/// use akari::object; 
/// let num_obj = object!(3); 
/// assert_eq!(num_obj, Object::Numerical(3.0)); 
/// ```
/// ```rust 
/// use akari::object::Object;  
/// use std::collections::HashMap; 
/// use akari::object; 
/// let list_obj = object!(["aaa", "bbb"]); 
/// assert_eq!(list_obj, Object::List(vec![Object::Str("aaa".to_string()), Object::Str("bbb".to_string())]));  
/// ``` 
/// ```rust 
/// use akari::object::Object; 
/// use std::collections::HashMap; 
/// use akari::object; 
/// let obj_obj = object!({c: "p", b: ["aaa", "bbb"], u: 32});  
/// assert_eq!(obj_obj, Object::Dictionary(HashMap::from([
///     ("c".to_string(), Object::Str("p".to_string())),
///     ("b".to_string(), Object::List(vec![Object::Str("aaa".to_string()), Object::Str("bbb".to_string())])), 
///     ("u".to_string(), Object::Numerical(32.0)),
/// ]))); 
/// ```
 
#[macro_export]
macro_rules! object {
    // Dictionary: keys become Strings now.
    ({ $( $key:ident : $value:tt ),* $(,)? }) => {{
        let mut map = ::std::collections::HashMap::new();
        $(
            map.insert(stringify!($key).to_string(), object!($value));
        )*
        Object::Dictionary(map)
    }};
    // List
    ([ $( $elem:tt ),* $(,)? ]) => {{
        let mut vec = Vec::new();
        $(
            vec.push(object!($elem));
        )*
        Object::List(vec)
    }};
    // Booleans
    (true) => {
        Object::new(true)
    };
    (false) => {
        Object::new(false)
    };
    // String literals
    ($e:literal) => {
        Object::new($e)
    };
    // Fallback for expressions (like numbers)
    ($e:expr) => {
        Object::new($e)
    };
}  

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_from_json_object() {
        let json = r#"{"a": 1, "b": true, "c": "hello"}"#;
        let obj = Object::from_json(json).expect("Failed to parse JSON");
        let mut expected_map = HashMap::new();
        expected_map.insert("a".to_string(), Object::Numerical(1.0));
        expected_map.insert("b".to_string(), Object::Boolean(true));
        expected_map.insert("c".to_string(), Object::Str("hello".to_string()));
        assert_eq!(obj, Object::Dictionary(expected_map));
    }
    
    #[test]
    fn test_from_json_array() {
        let json = r#"[1, 2, 3]"#;
        let obj = Object::from_json(json).expect("Failed to parse JSON");
        assert_eq!(obj, Object::List(vec![
            Object::Numerical(1.0),
            Object::Numerical(2.0),
            Object::Numerical(3.0)
        ]));
    }
    
    #[test]
    fn test_from_json_nested() {
        let json = r#"{"a": [true, false], "b": {"nested": "value"}}"#;
        let obj = Object::from_json(json).expect("Failed to parse JSON");
        // Further assertions can be added here.
    } 

    #[test] 
    fn test_object_macro() {
        let obj = object!({a: 1, b: true, c: "hello"});
        let mut expected_map = HashMap::new();
        expected_map.insert("a".to_string(), Object::Numerical(1.0));
        expected_map.insert("b".to_string(), Object::Boolean(true));
        expected_map.insert("c".to_string(), Object::Str("hello".to_string()));
        assert_eq!(obj, Object::Dictionary(expected_map));
    } 

    #[test] 
    fn test_object_macro_expr() { 
        let a = 1; 
        let obj = object!({a: a, b: [1, 2, 3], c: {
            hello: "world"
        }});
        let mut expected_map = HashMap::new();
        expected_map.insert("a".to_string(), Object::Numerical(1.0));
        expected_map.insert("b".to_string(), Object::List(vec![
            Object::Numerical(1.0),
            Object::Numerical(2.0),
            Object::Numerical(3.0)
        ]));
        expected_map.insert("c".to_string(), Object::Dictionary({
            let mut inner_map = HashMap::new();
            inner_map.insert("hello".to_string(), Object::Str("world".to_string()));
            inner_map
        })); 
        // assert_eq!(obj, Object::Dictionary(expected_map)); 
        println!("{:?}", obj.format()); 
        println!("{:?}", obj.into_json()); 
    } 
}

