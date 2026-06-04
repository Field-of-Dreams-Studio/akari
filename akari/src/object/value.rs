use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use super::parser::JsonParser;
use super::error::ValueError; 

#[derive(Debug, Clone)]
pub enum Value {
    Numerical(f64),
    Boolean(bool), 
    Str(String),
    List(Vec<Value>),
    Dict(HashMap<String, Value>),  
    None, 
}   

impl Value { 
    /// Creates a new Value from a value. 
    /// This function will convert the value into an Value.
    /// # Example 
    /// ```rust 
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// let obj = Value::new(42); 
    /// assert_eq!(obj, Value::Numerical(42.0)); 
    /// let obj = Value::new("hello"); 
    /// assert_eq!(obj, Value::Str("hello".to_string())); 
    /// let obj = Value::new(true); 
    /// assert_eq!(obj, Value::Boolean(true)); 
    /// let obj = Value::new(vec![1, 2, 3]);
    /// assert_eq!(obj, Value::List(vec![Value::Numerical(1.0), Value::Numerical(2.0), Value::Numerical(3.0)])); 
    /// let obj = Value::new(HashMap::from([("key", "value")])); 
    /// assert_eq!(obj, Value::Dict(HashMap::from([("key".to_string(), Value::Str("value".to_string()))]))); 
    /// ``` 
    /// 
    /// # Old grammar 
    /// ```rust
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// let obj = Value::new(vec![Value::new(1), Value::new(2), Value::new(3)]);
    /// assert_eq!(obj, Value::List(vec![Value::Numerical(1.0), Value::Numerical(2.0), Value::Numerical(3.0)])); 
    /// let obj = Value::new(HashMap::from([("key".to_string(), Value::Str("value".to_string()))])); 
    /// assert_eq!(obj, Value::Dict(HashMap::from([("key".to_string(), Value::Str("value".to_string()))]))); 
    /// ``` 
    pub fn new<T: Into<Value>>(value: T) -> Self {
        value.into()
    } 
    
    pub fn type_of(&self) -> String {
        match self {
            Value::Numerical(_) => "num".to_string(),
            Value::Boolean(_) => "bool".to_string(),
            Value::Str(_) => "str".to_string(),
            Value::List(_) => "vec".to_string(),
            Value::Dict(_) => "dict".to_string(),
            Value::None => "none".to_string(), 
        }
    } 

    /// Creates a default numerical value, aka 0.0 
    /// No parameters needs to be pass in 
    /// ```rust 
    /// use akari::Value; 
    /// assert_eq!(Value::new_numerical(), Value::Numerical(0.0))
    /// ```
    pub fn new_numerical() -> Self { 
        return Self::Numerical(0f64) 
    } 

    /// Creates a default boolean value, aka True 
    /// No parameters needed to be pass in 
    /// ```rust 
    /// use akari::Value; 
    /// assert_eq!(Value::new_boolean(), Value::Boolean(true))
    /// ```
    pub fn new_boolean() -> Self { 
        return Self::Boolean(true) 
    } 

    /// Creates a default String value, aka True 
    /// No parameters needed to be pass in 
    /// ```rust 
    /// use akari::Value; 
    /// assert_eq!(Value::new_str(), Value::Str(String::new()))
    /// ```
    pub fn new_str() -> Self { 
        return Self::Str("".to_string())
    } 

    /// Creates a default List, aka True 
    /// No parameters needed to be pass in 
    /// ```rust 
    /// use akari::Value; 
    /// assert_eq!(Value::new_list(), Value::List(Vec::new())) 
    /// ```
    pub fn new_list() -> Self { 
        return Self::List(Vec::new()) 
    } 

    /// Creates a Dictionary value, aka True 
    /// No parameters needed to be pass in 
    /// ```rust 
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// assert_eq!(Value::new_dict(), Value::Dict(HashMap::new())) 
    /// ``` 
    pub fn new_dict() -> Self { 
        return Self::Dict(HashMap::new()) 
    } 

    /// Converts the Value into a numerical value. 
    /// This function will return a numerical value based on the type of the Value. 
    /// If the Value is not a number, it will return 0.0. 
    /// If the object is a boolean, it will return 1.0 for true and 0.0 for false. 
    pub fn numerical(&self) -> f64 {
        match self {
            Value::Numerical(n) => *n,
            Value::Boolean(b) => if *b { 1.0 } else { 0.0 },
            Value::Str(s) => s.parse::<f64>().unwrap_or(0.0),
            Value::List(l) => l.len() as f64,
            Value::Dict(d) => d.len() as f64,
            Value::None => 0.0, 
        }
    } 

    /// Checks if the Value is a numerical value. 
    /// This function will return true if the Value is a numerical value, 
    /// false otherwise. 
    /// # Example 
    /// ```rust 
    /// use akari::Value; 
    /// let obj = Value::Numerical(42.0); 
    /// assert!(obj.is_numerical());
    /// let obj = Value::Str("hello".to_string());
    /// assert!(!obj.is_numerical());
    /// let obj = Value::Boolean(true);
    /// assert!(!obj.is_numerical());
    /// ``` 
    pub fn is_numerical(&self) -> bool {
        match self {
            Value::Numerical(_) => true,
            _ => false, 
        }
    } 

    /// Checks if the Value is a boolean value. 
    /// This function will return true if the Value is a boolean value, 
    /// false otherwise. 
    /// # Example
    /// ```rust 
    /// use akari::Value; 
    /// let obj = Value::Boolean(true);
    /// assert!(obj.is_boolean());
    /// let obj = Value::Str("hello".to_string());
    /// assert!(!obj.is_boolean());
    /// let obj = Value::Numerical(42.0);
    /// assert!(!obj.is_boolean()); 
    /// ```
    pub fn is_boolean(&self) -> bool {
        match self {
            Value::Boolean(_) => true,
            _ => false, 
        }
    } 

    /// Checks if the Value is a string value. 
    /// This function will return true if the Value is a string value, 
    /// false otherwise. 
    /// # Example 
    /// ```rust
    /// use akari::Value;
    /// let obj = Value::Str("hello".to_string());
    /// assert!(obj.is_str());
    /// let obj = Value::Numerical(42.0);
    /// assert!(!obj.is_str());
    /// let obj = Value::Boolean(true);
    /// assert!(!obj.is_str());
    /// ```
    pub fn is_str(&self) -> bool {
        match self {
            Value::Str(_) => true,
            _ => false, 
        }
    } 

    /// Checks if the Value is a list value. 
    /// This function will return true if the Value is a list value, 
    /// false otherwise. 
    /// # Example 
    /// ```rust
    /// use akari::Value;
    /// let obj = Value::List(vec![Value::Str("hello".to_string()), Value::Numerical(42.0)]);
    /// assert!(obj.is_list());
    /// let obj = Value::Numerical(42.0);
    /// assert!(!obj.is_list());
    /// let obj = Value::Boolean(true);
    /// assert!(!obj.is_list());
    /// ``` 
    pub fn is_list(&self) -> bool {
        match self {
            Value::List(_) => true,
            _ => false, 
        }
    } 

    /// Checks if the Value is a dictionary value. 
    /// This function will return true if the Value is a dictionary value, 
    /// false otherwise. 
    /// # Example 
    /// ```rust 
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// let obj = Value::Dict(HashMap::from([("key".to_string(), Value::Str("value".to_string()))])); 
    /// assert!(obj.is_dict()); 
    /// let obj = Value::Numerical(42.0); 
    /// assert!(!obj.is_dict()); 
    /// let obj = Value::Boolean(true); 
    /// assert!(!obj.is_dict()); 
    /// ``` 
    pub fn is_dict(&self) -> bool {
        match self {
            Value::Dict(_) => true,
            _ => false, 
        }
    } 

    /// Checks if the Value is None. 
    /// This function will return true if the Value is None, 
    /// false otherwise. 
    /// # Example 
    /// ```rust
    /// use akari::Value;
    /// let obj = Value::None;
    /// assert!(obj.is_none());
    /// let obj = Value::Numerical(42.0);
    /// assert!(!obj.is_none());
    /// let obj = Value::Boolean(true);
    /// assert!(!obj.is_none());
    /// ```
    pub fn is_none(&self) -> bool {
        match self {
            Value::None => true,
            _ => false, 
        }
    } 

    /// Converts the Value into an integ·er value. 
    /// The rule is the same as numerical, but it will return an i64 value. 
    pub fn integer(&self) -> i64 {
        match self {
            Value::Numerical(n) => *n as i64,
            Value::Boolean(b) => if *b { 1 } else { 0 },
            Value::Str(s) => s.parse::<i64>().unwrap_or(0),
            Value::List(l) => l.len() as i64,
            Value::Dict(d) => d.len() as i64,
            Value::None => 0, 
        }
    } 

    /// Converts the Value into a boolean value. 
    pub fn boolean(&self) -> bool {
        match self {
            Value::Numerical(n) => *n != 0.0,
            Value::Boolean(b) => *b,
            Value::Str(s) => !s.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::Dict(d) => !d.is_empty(),
            Value::None => false, 
        }
    } 

    /// Converts the Value into a string representation. 
    pub fn string(&self) -> String {
        match self {
            Value::None => "".to_string(), 
            Value::Numerical(n) => n.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Str(s) => s.clone(),
            _ => self.format(), // Use the format method for other types 
        }
    } 

    /// Converts the Value into a list of Values. 
    pub fn list(&self) -> Vec<Value> {
        match self {
            Value::List(l) => l.clone(),
            Value::Dict(d) => d.values().cloned().collect(),
            _ => vec![self.clone()],
        }
    } 
    
    /// Converts the Value into a JSON string representation. 
    /// This function will return a string that is a valid JSON representation of the Value. 
    /// # Example 
    /// ```rust 
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// let obj = Value::Dict(HashMap::from([ 
    ///    ("key".to_string(), Value::Str("value".to_string())), 
    ///    ("number".to_string(), Value::Numerical(42.0)), 
    ///    ("list".to_string(), Value::List(vec![Value::Numerical(1.0), Value::Numerical(2.0), Value::Numerical(3.0)])), 
    /// ])); 
    /// let json = obj.into_json(); 
    /// println!("{}", json); // Output: {"key": "value", "number": 42, "list": [1, 2, 3]} 
    /// ``` 
    /// 
    /// This function will automatically escape special characters in strings and format the output as a valid JSON string. 
    /// # Example 
    /// ```rust 
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// let obj = Value::Str("Hello, \"world\"! \\".to_string()); 
    /// let json = obj.into_json(); 
    /// println!("{}", json); // Output: "\"Hello, \\\"world\\\"! \\\"\"" 
    /// ``` 
    pub fn into_json(&self) -> String {
        match <super::JsonSerializer as super::ValueSerializer<str>>::serialize_one(self) {
            Ok(json) => json,
            Err(_) => "null".to_string(),
        }
    } 

    /// Converts the Value into a JSON string representation and writes it to a file. 
    /// This function will return an error if the file cannot be written. 
    /// # Example 
    /// ```rust 
    /// use akari::Value; 
    /// 
    /// # #[cfg(feature = "object_macro")]
    /// # {
    /// use akari::object;
    /// // Write a JSON file using object macro
    /// object!({ 
    ///    key: "value", 
    ///    number: 42, 
    ///    list: [1, 2, 3], 
    /// }).into_jsonf("test_temp/write_test.json").expect("Failed to write JSON file");
    /// # }
    /// 
    /// # #[cfg(not(feature = "object_macro"))]
    /// # {
    /// // Alternative approach without the object macro
    /// use std::collections::HashMap;
    /// let mut dict = HashMap::new();
    /// dict.insert("key".to_string(), Value::Str("value".to_string()));
    /// dict.insert("number".to_string(), Value::Numerical(42.0));
    /// 
    /// let mut list = Vec::new();
    /// list.push(Value::Numerical(1.0));
    /// list.push(Value::Numerical(2.0));
    /// list.push(Value::Numerical(3.0));
    /// dict.insert("list".to_string(), Value::List(list));
    /// 
    /// Value::Dict(dict).into_jsonf("test_temp/write_test.json").expect("Failed to write JSON file");
    /// # }
    /// ``` 
    pub fn into_jsonf(&self, file_path: &str) -> Result<(), String> {
        use std::fs::File;
        let mut file = File::create(file_path)
            .map_err(|e| format!("Failed to write JSON file: {}", e))?;
        <super::JsonSerializer as super::ValueSerializer<str>>::serialize_to(self, &mut file)
            .map_err(|e| format!("Failed to write JSON file: {}", e))?;
        Ok(())
    } 
    
    /// Parses a JSON string and returns an Value. 
    /// This function will return an error if the JSON is invalid or if there are extra characters after the JSON value. 
    /// # Example 
    /// ```rust 
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// let json = r#"{"key": "value", "number": 42, "list": [1, 2, 3]}"#; 
    /// let obj = Value::from_json(json).expect("Failed to parse JSON"); 
    /// assert_eq!(obj, Value::Dict(HashMap::from([
    ///     ("key".to_string(), Value::Str("value".to_string())),
    ///     ("number".to_string(), Value::Numerical(42.0)),
    ///     ("list".to_string(), Value::List(vec![Value::Numerical(1.0), Value::Numerical(2.0), Value::Numerical(3.0)])), 
    ///     ]))); 
    /// ``` 
    /// # Errors
    /// This function will return an error if the JSON is invalid or if there are extra characters after the JSON value.
    pub fn from_json(json: &str) -> Result<Self, String> {
        JsonParser::parse_one(json).map_err(|e| e.to_string())
    } 

    /// Parses a JSON file and returns an Value. 
    /// This function will return an error if the file cannot be read or if the JSON is invalid. 
    /// # Example 
    /// ```rust 
    /// use akari::Value; 
    /// use std::fs; 
    /// // Create a JSON file for testing
    /// fs::write("test_temp/test_read.json", r#"{"key": "value", "number": 42, "list": [1, 2, 3]}"#).unwrap(); 
    /// 
    /// # #[cfg(feature = "object_macro")]
    /// # {
    /// use akari::object;
    /// // Read the JSON file and parse it into an Value 
    /// let obj = Value::from_jsonf("test_temp/test_read.json").expect("Failed to parse JSON file"); 
    /// assert_eq!(obj, object!({
    ///    key: "value",
    ///    number: 42, 
    ///    list: [1, 2, 3], 
    /// })); 
    /// # }
    /// 
    /// # #[cfg(not(feature = "object_macro"))]
    /// # {
    /// // Alternative approach without the object macro
    /// use std::collections::HashMap;
    /// let obj = Value::from_jsonf("test_temp/test_read.json").expect("Failed to parse JSON file");
    /// 
    /// if let Value::Dict(map) = &obj {
    ///     assert_eq!(map.get("key"), Some(&Value::Str("value".to_string())));
    ///     assert_eq!(map.get("number"), Some(&Value::Numerical(42.0)));
    ///     if let Some(Value::List(list)) = map.get("list") {
    ///         assert_eq!(list.len(), 3);
    ///         assert_eq!(list[0], Value::Numerical(1.0));
    ///         assert_eq!(list[1], Value::Numerical(2.0));
    ///         assert_eq!(list[2], Value::Numerical(3.0));
    ///     } else {
    ///         panic!("Expected 'list' to be a List");
    ///     }
    /// } else {
    ///     panic!("Expected a Dict");
    /// }
    /// # }
    /// ``` 
    pub fn from_jsonf<T: AsRef<str>>(file_path: T) -> Result<Self, String> {
        use std::fs;
        let content = fs::read_to_string(file_path.as_ref())
            .map_err(|e| format!("Failed to read JSON file: {}", e))?;
        Self::from_json(&content) 
    }

    // /// Retrieves a value from the dictionary by path. 
    // /// This function will return None if the path is invalid or if the key does not exist. 
    // /// # Example 
    // /// ```rust 
    // /// use akari::object::Value; 
    // /// use std::collections::HashMap; 
    // /// let mut map = HashMap::new(); 
    // /// map.insert("key".to_string(), Value::Str("value".to_string())); 
    // /// let obj = Value::Dict(map); 
    // /// let value = obj.get_path("key"); 
    // /// assert_eq!(value, Some(&Value::Str("value".to_string()))); 
    // /// ``` 
    // pub fn get_path<T: AsRef<str>>(&self, path: T) -> Option<&Value> {
    //     let parts: Vec<&str> = path.as_ref().split('.').collect();
    //     let mut current = self;
        
    //     for part in parts {
    //         // Handle array indexing
    //         if let Some(idx_part) = part.strip_suffix(']') {
    //             let parts: Vec<&str> = idx_part.split('[').collect();
    //             if parts.len() != 2 {
    //                 return None;
    //             }
                
    //             let obj_key = parts[0];
    //             let idx_str = parts[1];
                
    //             // Get object by key first
    //             if !obj_key.is_empty() {
    //                 if let Some(obj) = current.get(obj_key) {
    //                     current = obj;
    //                 } else {
    //                     return None;
    //                 }
    //             }
                
    //             // Then get by index
    //             if let Ok(idx) = idx_str.parse::<usize>() {
    //                 if let Some(obj) = current.idx(idx) {
    //                     current = obj;
    //                 } else {
    //                     return None;
    //                 }
    //             } else {
    //                 return None;
    //             }
    //         } else {
    //             // Regular dictionary access
    //             if let Some(obj) = current.get(part) {
    //                 current = obj;
    //             } else {
    //                 return None;
    //             }
    //         }
    //     }
        
    //     Some(current)
    // } 

    /// Retrieves a value from the dictionary by key. 
    /// If there does not have a vaild value, it will return Value::None as default. 
    /// If you want to return a Option<Value>, please use try_get 
    /// If you want to set your own default value, please use get_or 
    /// # Example 
    /// ```rust 
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// let mut map = HashMap::new(); 
    /// map.insert("key".to_string(), Value::Str("value".to_string())); 
    /// let obj = Value::Dict(map); 
    /// let value = obj.get("key"); 
    /// assert_eq!(value, &Value::Str("value".to_string())); 
    /// ``` 
    pub fn get<T: AsRef<str>>(&self, key: T) -> &Value {
        self.try_get(key).unwrap_or(&Value::None)
    } 

    /// Retrieves a value from the dictionary by key, with a specified default value 
    /// # Example 
    /// ```rust 
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// let mut map = HashMap::new(); 
    /// map.insert("key".to_string(), Value::Str("value".to_string())); 
    /// let obj = Value::Dict(map); 
    /// let default = Value::Str("default".to_string()); 
    /// let value = obj.get_or("no_way", &default); 
    /// assert_eq!(value, &default); 
    /// ``` 
    pub fn get_or<'a, T: AsRef<str>>(&'a self, key: T, default: &'a Value) -> &'a Value { 
        self.try_get(key).unwrap_or(default)
    }

    /// Retrieves a value from the dictionary by key. 
    /// It will returns a Result<&Value, ValueError> 
    /// # Example 
    /// ```rust 
    /// use akari::Value; 
    ///  
    /// use std::collections::HashMap; 
    /// let mut map = HashMap::new(); 
    /// map.insert("key".to_string(), Value::Str("value".to_string())); 
    /// let obj = Value::Dict(map); 
    /// let value = obj.try_get("key"); 
    /// assert_eq!(value, Ok(&Value::Str("value".to_string()))); 
    /// ``` 
    pub fn try_get<T: AsRef<str>>(&self, key: T) -> Result<&Value, ValueError> {
        if let Value::Dict(map) = self {
            match map.get(key.as_ref()) { 
                Some(value) => Ok(value), 
                None => Err(ValueError::KeyNotFoundError)
            }
        } else {
            Err(ValueError::TypeError)
        }
    } 

    /// Retrieves a value from the dictionary by key. 
    /// Panics when error 
    /// # Panics 
    /// When there is no value correspond to the key, 
    /// OR the value is not a dictionary 
    /// # Example 
    /// ```rust 
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// let mut map = HashMap::new(); 
    /// map.insert("key".to_string(), Value::Str("value".to_string())); 
    /// let obj = Value::Dict(map); 
    /// let value = obj.get_unchecked("key"); 
    /// assert_eq!(value, &Value::Str("value".to_string())); 
    /// ``` 
    pub fn get_unchecked<T: AsRef<str>>(&self, key: T) -> &Value {  
        self.try_get(key).unwrap() 
    } 

    /// Sets a value in the dictionary by key. 
    /// The key should be convertable into a String  
    /// Value should be convertable into an Value. The value don't necessarily be an Value 
    /// # Example 
    /// ```rust 
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// let mut map = HashMap::new(); 
    /// let mut obj = Value::Dict(map); 
    /// obj.set("key".to_string(), Value::Str("new_value".to_string())); 
    /// let value = obj.get("key"); 
    /// assert_eq!(value, &Value::Str("new_value".to_string())); 
    /// ``` 
    /// 
    /// ```rust 
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// let mut map = HashMap::new(); 
    /// map.insert("key".to_string(), Value::Str("value".to_string())); 
    /// let mut obj = Value::Dict(map); 
    /// obj.set("key".to_string(), Value::Str("new_value".to_string())); 
    /// let value = obj.get("key"); 
    /// assert_eq!(value, &Value::Str("new_value".to_string())); 
    /// ``` 
    pub fn set<T: Into<String>, O: Into<Value>>(&mut self, key: T, value: O) {
        if let Value::Dict(map) = self {
            map.insert(key.into(), value.into());
        }
    } 

    /// Take a value from the dictionary by key.
    /// If there does not have a valid value, it will return Value::None as default.
    /// If you want to return a Result<Value, ValueError>, please use try_take
    /// If you want to set your own default value, please use take_or
    /// # Example
    /// ```rust
    /// use akari::Value;
    /// use std::collections::HashMap;
    /// let mut map = HashMap::new();
    /// map.insert("key".to_string(), Value::Str("value".to_string()));
    /// let mut obj = Value::Dict(map);
    /// let value = obj.take("key");
    /// assert_eq!(value, Value::Str("value".to_string()));
    /// ``` 
    pub fn take<T: AsRef<str>>(&mut self, key: T) -> Value {
        self.try_take(key).unwrap_or(Value::None)
    }

    /// Take a value from the dictionary by key, with a specified default value
    /// # Example
    /// ```rust
    /// use akari::Value;
    /// use std::collections::HashMap;
    /// let mut map = HashMap::new();
    /// map.insert("key".to_string(), Value::Str("value".to_string()));
    /// let mut obj = Value::Dict(map);
    /// let default = Value::Str("default".to_string());
    /// let value = obj.take_or("no_way", default.clone());
    /// assert_eq!(value, default);
    /// ``` 
    pub fn take_or<T: AsRef<str>>(&mut self, key: T, default: Value) -> Value {
        self.try_take(key).unwrap_or(default)
    }

    /// Take a value from the dictionary by key.
    /// This function will remove the value from the dictionary and return it.
    /// If the key does not exist, it will return an error.
    /// 
    /// # Example
    /// ```rust
    /// use akari::Value;
    /// use std::collections::HashMap;
    /// let mut map = HashMap::new();
    /// map.insert("key".to_string(), Value::Str("value".to_string()));
    /// let mut obj = Value::Dict(map);
    /// let value = obj.try_take("key");
    /// assert_eq!(value, Ok(Value::Str("value".to_string())));
    /// ``` 
    pub fn try_take<T: AsRef<str>>(&mut self, key: T) -> Result<Value, ValueError> {
        if let Value::Dict(map) = self {
            match map.remove(key.as_ref()) {
                Some(value) => Ok(value),
                None => Err(ValueError::KeyNotFoundError),
            }
        } else {
            Err(ValueError::TypeError)
        }
    }

    /// Take a value from the dictionary by key.
    /// Panics when error
    /// # Panics
    /// When there is no value correspond to the key,
    /// OR the value is not a dictionary
    /// # Example
    /// ```rust
    /// use akari::Value;
    /// use std::collections::HashMap;
    /// let mut map = HashMap::new();
    /// map.insert("key".to_string(), Value::Str("value".to_string()));
    /// let mut obj = Value::Dict(map);
    /// let value = obj.take_unchecked("key");
    /// assert_eq!(value, Value::Str("value".to_string()));
    /// ``` 
    pub fn take_unchecked<T: AsRef<str>>(&mut self, key: T) -> Value {
        self.try_take(key).unwrap()
    } 

    /// Deletes a value from the dictionary by key. 
    /// # Example 
    /// ```rust 
    /// use akari::Value;
    /// use std::collections::HashMap; 
    /// let mut map = HashMap::new(); 
    /// map.insert("key".to_string(), Value::Str("value".to_string())); 
    /// let mut obj = Value::Dict(map); 
    /// let value = obj.delete("key"); 
    /// assert_eq!(value, Some(Value::Str("value".to_string()))); 
    /// ``` 
    /// This function will return None if the key does not exist. 
    pub fn delete(&mut self, key: &str) -> Option<Value> {
        if let Value::Dict(map) = self {
            map.remove(key)
        } else {
            None
        }
    } 

    /// Retrieves a value from the list by index.
    /// # Example
    /// ```rust 
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// let list = Value::List(vec![Value::Str("value1".to_string()), Value::Str("value2".to_string())]); 
    /// let value = list.idx(1); 
    /// assert_eq!(value, &Value::Str("value2".to_string())); 
    /// ``` 
    pub fn idx(&self, index: usize) -> &Value {
        self.try_idx(index).unwrap_or(&Value::None) 
    } 

    /// Retrieves a value from the list by index. 
    /// This function will return a default value if the index is out of bounds. 
    /// # Example 
    /// ```rust 
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// let list = Value::List(vec![Value::Str("value1".to_string()), Value::Str("value2".to_string())]); 
    /// let default = Value::Str("default".to_string()); 
    /// let value = list.idx_or(1, &default); 
    /// assert_eq!(value, &Value::Str("value2".to_string())); 
    /// ``` 
    pub fn idx_or<'a>(&'a self, index: usize, default: &'a Value) -> &'a Value {
        self.try_idx(index).unwrap_or(default)
    } 

    /// Retrieves a value from the list by index. 
    /// This function will return an error if the index is out of bounds.   
    /// # Example 
    ///  ```rust 
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// use akari::ValueError; 
    /// let list = Value::List(vec![Value::Str("value1".to_string()), Value::Str("value2".to_string())]); 
    /// let value = list.try_idx(1); 
    /// assert_eq!(value, Ok(&Value::Str("value2".to_string()))); 
    /// let value = list.try_idx(5); 
    /// assert_eq!(value, Err(ValueError::IndexOutOfBoundError));  
    /// ``` 
    pub fn try_idx(&self, index: usize) -> Result<&Value, ValueError> {
        if let Value::List(vec) = self {
            if index < vec.len() {
                Ok(&vec[index])
            } else {
                Err(ValueError::IndexOutOfBoundError)
            }
        } else {
            Err(ValueError::TypeError)
        }
    } 

    /// Retrieves a value from the list by index. 
    /// This function will panic if the index is out of bounds. 
    /// # Example  
    /// ```rust 
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// let list = Value::List(vec![Value::Str("value1".to_string()), Value::Str("value2".to_string())]); 
    /// let value = list.idx_unchecked(1); 
    /// assert_eq!(value, &Value::Str("value2".to_string())); 
    /// ``` 
    pub fn idx_unchecked(&self, index: usize) -> &Value {
        if let Value::List(vec) = self {
            &vec[index]
        } else {
            panic!("Value is not a list")
        }
    } 

    /// Checks if the Value contains a specific value. 
    /// This function will return true if the Value contains the value, 
    /// false otherwise. 
    /// By meaning of "contains", it means that the value is equal to the Value for the following types: 
    /// - Numerical: The numerical value is equal to the value. 
    /// - Boolean: The boolean value is equal to the value. 
    /// - String: The string value is equal to the value. 
    /// For List and Dict, it will check if the value is contained in the list or dict. 
    /// For None it will always return false. 
    /// # Example 
    /// ```rust 
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// let list = Value::List(vec![Value::Str("value1".to_string()), Value::Str("value2".to_string())]); 
    /// assert!(list.value_contains(&Value::Str("value1".to_string()))); 
    /// assert!(!list.value_contains(&Value::Str("value3".to_string()))); 
    /// ``` 
    /// 
    /// ```rust 
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// let mut map = HashMap::new(); 
    /// map.insert("key".to_string(), Value::Str("value".to_string())); 
    /// let obj = Value::Dict(map); 
    /// assert!(obj.value_contains(&Value::Str("value".to_string()))); 
    /// assert!(!obj.value_contains(&Value::Str("other_value".to_string()))); 
    /// ```
    /// 
    /// ```rust 
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// let obj = Value::Numerical(42.0); 
    /// assert!(obj.value_contains(&Value::Numerical(42.0))); 
    /// assert!(!obj.value_contains(&Value::Numerical(43.0))); 
    /// ``` 
    ///  
    /// ```rust 
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// let obj = Value::Boolean(true); 
    /// assert!(obj.value_contains(&Value::Boolean(true))); 
    /// assert!(!obj.value_contains(&Value::Boolean(false))); 
    /// ``` 
    ///  
    /// ```rust 
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// let obj = Value::Str("value".to_string()); 
    /// assert!(obj.value_contains(&Value::Str("value".to_string()))); 
    /// assert!(!obj.value_contains(&Value::Str("other_value".to_string()))); 
    /// ``` 
    pub fn value_contains<'a, T>(&self, value: &'a T) -> bool 
        where &'a Value: From<&'a T> + Sized { 
        match self {
            Value::Dict(map) => map.values().any(|v| { 
                let value: &Value = value.into(); 
                v == value 
            }),
            Value::List(vec) => vec.contains(value.into()),
            Value::None => false, 
            _ => { 
                let value: &Value = value.into(); 
                self == value 
            } 
        } 
    } 

    /// Replaces the value at `index` in the list.
    /// # Example
    /// ```rust
    /// use akari::Value;
    /// let mut list = Value::List(vec![Value::Str("value1".to_string()), Value::Str("value2".to_string())]);
    /// list.set_idx(1, Value::Str("new_value".to_string()));
    /// assert_eq!(list.idx(1), &Value::Str("new_value".to_string()));
    /// ```
    /// Silently no-ops if `index` is out of bounds or the value is not a list.
    /// Use [`Value::try_set_idx`] for explicit error handling or
    /// [`Value::set_idx_unchecked`] to panic on failure.
    pub fn set_idx(&mut self, index: usize, value: Value) {
        if let Value::List(vec) = self {
            if index < vec.len() {
                vec[index] = value;
            }
        }
    }

    /// Replaces the value at `index`, returning an error if the index is out
    /// of bounds or the value is not a list.
    /// # Example
    /// ```rust
    /// use akari::Value;
    /// use akari::ValueError;
    /// let mut list = Value::List(vec![Value::Str("value1".to_string())]);
    /// assert!(list.try_set_idx(0, Value::Str("new".to_string())).is_ok());
    /// assert_eq!(list.try_set_idx(5, Value::None), Err(ValueError::IndexOutOfBoundError));
    /// ```
    pub fn try_set_idx(&mut self, index: usize, value: Value) -> Result<(), ValueError> {
        if let Value::List(vec) = self {
            if index < vec.len() {
                vec[index] = value;
                Ok(())
            } else {
                Err(ValueError::IndexOutOfBoundError)
            }
        } else {
            Err(ValueError::TypeError)
        }
    }

    /// Replaces the value at `index`, panicking if the index is out of bounds
    /// or the value is not a list.
    /// # Panics
    /// Panics if `self` is not a `Value::List` or `index >= len`.
    /// # Example
    /// ```rust
    /// use akari::Value;
    /// let mut list = Value::List(vec![Value::Str("value1".to_string())]);
    /// list.set_idx_unchecked(0, Value::Str("new".to_string()));
    /// assert_eq!(list.idx(0), &Value::Str("new".to_string()));
    /// ```
    pub fn set_idx_unchecked(&mut self, index: usize, value: Value) {
        if let Value::List(vec) = self {
            vec[index] = value;
        } else {
            panic!("Value is not a list")
        }
    }

    /// Inserts a value at `index`, shifting all later elements to the right
    /// (matches `Vec::insert` semantics).
    /// # Example
    /// ```rust
    /// use akari::Value;
    /// let mut list = Value::List(vec![Value::Str("a".to_string()), Value::Str("c".to_string())]);
    /// list.insert(1, Value::Str("b".to_string()));
    /// assert_eq!(list.idx(1), &Value::Str("b".to_string()));
    /// assert_eq!(list.idx(2), &Value::Str("c".to_string()));
    /// ```
    /// Accepts `index == len` (equivalent to push). Silently no-ops if
    /// `index > len` or the value is not a list. Use [`Value::try_insert`] for
    /// explicit error handling or [`Value::insert_unchecked`] to panic on failure.
    pub fn insert(&mut self, index: usize, value: Value) {
        if let Value::List(vec) = self {
            if index <= vec.len() {
                vec.insert(index, value);
            }
        }
    }

    /// Inserts a value at `index` with shift-right semantics, returning an
    /// error if the index is out of bounds or the value is not a list.
    /// # Example
    /// ```rust
    /// use akari::Value;
    /// use akari::ValueError;
    /// let mut list = Value::List(vec![Value::Str("a".to_string())]);
    /// assert!(list.try_insert(0, Value::Str("z".to_string())).is_ok());
    /// assert_eq!(list.try_insert(99, Value::None), Err(ValueError::IndexOutOfBoundError));
    /// ```
    pub fn try_insert(&mut self, index: usize, value: Value) -> Result<(), ValueError> {
        if let Value::List(vec) = self {
            if index <= vec.len() {
                vec.insert(index, value);
                Ok(())
            } else {
                Err(ValueError::IndexOutOfBoundError)
            }
        } else {
            Err(ValueError::TypeError)
        }
    }

    /// Inserts a value at `index` with shift-right semantics, panicking if
    /// the index is out of bounds or the value is not a list.
    /// # Panics
    /// Panics if `self` is not a `Value::List` or `index > len`.
    /// # Example
    /// ```rust
    /// use akari::Value;
    /// let mut list = Value::List(vec![Value::Str("a".to_string())]);
    /// list.insert_unchecked(0, Value::Str("z".to_string()));
    /// assert_eq!(list.idx(0), &Value::Str("z".to_string()));
    /// ```
    pub fn insert_unchecked(&mut self, index: usize, value: Value) {
        if let Value::List(vec) = self {
            vec.insert(index, value);
        } else {
            panic!("Value is not a list")
        }
    }

    

    /// Pushes a value to the end of the list. 
    /// # Example 
    /// ```rust 
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// let mut list = Value::List(vec![Value::Str("value1".to_string()), Value::Str("value2".to_string())]); 
    /// list.push(Value::Str("new_value".to_string())); 
    /// let value = list.idx(2); 
    /// assert_eq!(value, &Value::Str("new_value".to_string())); 
    /// ``` 
    /// This function will push the value to the end of the list. 
    pub fn push(&mut self, value: Value) {
        if let Value::List(vec) = self {
            vec.push(value);
        }
    } 

    /// Pops a value from the end of the list. 
    /// # Example 
    /// ```rust 
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// let mut list = Value::List(vec![Value::Str("value1".to_string()), Value::Str("value2".to_string())]); 
    /// let value = list.pop(); 
    /// assert_eq!(value, Some(Value::Str("value2".to_string()))); 
    /// ``` 
    pub fn pop(&mut self) -> Option<Value> {
        if let Value::List(vec) = self {
            vec.pop()
        } else {
            None
        }
    } 

    /// Removes a value from the list by index. 
    /// # Example 
    /// ```rust 
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// let mut list = Value::List(vec![Value::Str("value1".to_string()), Value::Str("value2".to_string())]); 
    /// let value = list.remove(1); 
    /// assert_eq!(value, Some(Value::Str("value2".to_string()))); 
    /// ``` 
    pub fn remove(&mut self, index: usize) -> Option<Value> {
        if let Value::List(vec) = self {
            if index < vec.len() {
                Some(vec.remove(index))
            } else {
                None
            }
        } else {
            None
        }
    } 

    /// Returns the length of the Value. 
    /// # Example 
    /// ```rust 
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// let list = Value::List(vec![Value::Str("value1".to_string()), Value::Str("value2".to_string())]); 
    /// let length = list.len(); 
    /// assert_eq!(length, 2); 
    /// let dict = Value::Dict(HashMap::from([ 
    ///    ("key".to_string(), Value::Str("value".to_string())), 
    /// ])); 
    /// let length = dict.len(); 
    /// assert_eq!(length, 1); 
    /// ``` 
    /// This function will return the length of the Value. 
    pub fn len(&self) -> usize {
        match self {
            Value::List(vec) => vec.len(),
            Value::Dict(map) => map.len(),
            Value::None => 0,
            _ => 1,
        }
    } 

    /// Checks if this value is empty.
    ///
    /// A value is considered empty if:
    /// - It's a `List` or `Dict` with no elements
    /// - It's `None`
    /// - Otherwise (for `Boolean`, `Numerical`, and `Str`), it's never empty
    ///
    /// # Returns
    ///
    /// `true` if the value is empty, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use akari::Value; 
    /// 
    /// let list = Value::List(vec![]);
    /// assert!(list.is_empty());
    ///
    /// let single = Value::Numerical(42.0);
    /// assert!(!single.is_empty());
    ///
    /// let none = Value::None;
    /// assert!(none.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    } 

    /// Checks if the Values are the same 
    pub fn equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Numerical(n1), Value::Numerical(n2)) => n1 == n2,
            (Value::Boolean(b1), Value::Boolean(b2)) => b1 == b2,
            (Value::Str(s1), Value::Str(s2)) => s1 == s2,
            (Value::List(l1), Value::List(l2)) => l1 == l2,
            (Value::Dict(d1), Value::Dict(d2)) => d1 == d2,
            (Value::None, Value::None) => true,
            _ => false,
        }
    } 

    /// Checks if the Value is greater than another Value. 
    pub fn greater_than(&self, other: &Value) -> bool {
        self.numerical() > other.numerical() 
    } 

    /// Checks if the Value is less than another Value. 
    pub fn less_than(&self, other: &Value) -> bool {
        self.numerical() < other.numerical() 
    } 

    /// Checks if the Value contains another Value. 
    /// 
    /// This function will return true if the Value is a List or a Dict and contains the other Value. 
    /// 
    /// # Example 
    /// 
    /// ```rust 
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// let list = Value::List(vec![Value::Str("value1".to_string()), Value::Str("value2".to_string())]); 
    /// assert!(list.contains(&Value::Str("value1".to_string()))); 
    /// let dict = Value::Dict(HashMap::from([ 
    ///    ("key1".to_string(), Value::Str("value1".to_string())), 
    ///   ("key2".to_string(), Value::Str("value2".to_string())),
    ///  ])); 
    /// assert!(dict.contains(&Value::Str("key1".to_string()))); 
    /// ``` 
    pub fn contains(&self, other: &Value) -> bool { 
        match self { 
            Value::Str(s) => s.contains(&other.string()), 
            Value::List(l) => l.contains(other),
            Value::Dict(d) => d.get(&other.string()).is_some(), 
            _ => false,
        } 
    }

    pub fn interal_value_as_string(&self) -> String {
        match self {
            Value::Str(s) => s.clone(),
            Value::Numerical(n) => n.to_string(),
            Value::Boolean(b) => b.to_string(),
            _ => "".to_string(),
        }
    } 
 
    /// Returns a display-safe string representation of a `Value::Str`.
    ///
    /// **Not JSON** — this is for human-readable repr used by [`format`](Value::format) and
    /// [`Display`](std::fmt::Display). Uses JSON-style escape sequences purely for readability,
    /// not for interchange. For actual JSON output use [`JsonSerializer`](super::JsonSerializer) instead.
    ///
    /// Returns an empty string for non-`Str` variants.
    pub fn string_repr_safely(&self) -> String {
        match self {
            Value::Str(s) => {
                // Properly escape the string content
                let mut result = String::with_capacity(s.len() + 2);
                result.push('"');
                for c in s.chars() {
                    match c {
                        '"' => result.push_str("\\\""),  // Escape double quotes
                        '\\' => result.push_str("\\\\"), // Escape backslashes
                        '\n' => result.push_str("\\n"),  // Escape newlines
                        '\r' => result.push_str("\\r"),  // Escape carriage returns
                        '\t' => result.push_str("\\t"),  // Escape tabs
                        '\u{0008}' => result.push_str("\\b"), // Escape backspace
                        '\u{000C}' => result.push_str("\\f"), // Escape form feed
                        _ if c.is_control() => {
                            // Escape other control characters as unicode
                            result.push_str(&format!("\\u{:04x}", c as u32));
                        }
                        _ => result.push(c), // Regular characters pass through unchanged
                    }
                }
                result.push('"');
                result
            }, 
            _ => "".to_string(), 
        }
    } 

    /// Returns a human-readable representation of the `Value`.
    ///
    /// **Not JSON** — Dict uses `{type key = value}` syntax and is not valid JSON.
    /// This is the backing implementation for [`Display`](std::fmt::Display).
    /// For JSON output use [`JsonSerializer`](super::JsonSerializer) instead.
    pub fn format(&self) -> String {
        match self {
            Value::None => "null".to_string(),
            Value::Numerical(n) => format!("{}", n),
            Value::Boolean(b) => format!("{}", b),
            Value::Str(_) => format!("{}", self.string_repr_safely()), 
            Value::List(l) => {
                let mut result = String::new();
                for item in l {
                    result.push_str(&format!("{}, ", item));
                }
                if result.len() >= 2 {
                    result.truncate(result.len() - 2);
                }
                format!("[{}]", result)
            }
            Value::Dict(d) => {
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

impl std::fmt::Display for Value { 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Value::format(self)) 
    }
} 

// Implement Hash trait
impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::None => 0.hash(state),
            Value::Boolean(b) => {
                0.hash(state);
                b.hash(state);
            },
            Value::Numerical(n) => {
                1.hash(state);
                // Convert f64 to a bitwise representation for hashing
                n.to_bits().hash(state);
            },
            Value::Str(s) => {
                2.hash(state);
                s.hash(state);
            },
            Value::List(items) => {
                3.hash(state);
                // Hash the length and each element
                items.len().hash(state);
                for item in items {
                    item.hash(state);
                }
            },
            Value::Dict(dict) => {
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
impl Eq for Value {} 

// Implement Into<String> trait 
impl Into<String> for Value {
    fn into(self) -> String {
        self.to_string() 
    } 
} 

// From implementations
impl From<i8> for Value { fn from(n: i8) -> Self { Value::Numerical(n as f64) } }
impl From<i16> for Value { fn from(n: i16) -> Self { Value::Numerical(n as f64) } }
impl From<i32> for Value { fn from(n: i32) -> Self { Value::Numerical(n as f64) } }
impl From<i64> for Value { fn from(n: i64) -> Self { Value::Numerical(n as f64) } }
impl From<i128> for Value { fn from(n: i128) -> Self { Value::Numerical(n as f64) } }
impl From<isize> for Value { fn from(n: isize) -> Self { Value::Numerical(n as f64) } }
impl From<u8> for Value { fn from(n: u8) -> Self { Value::Numerical(n as f64) } }
impl From<u16> for Value { fn from(n: u16) -> Self { Value::Numerical(n as f64) } }
impl From<u32> for Value { fn from(n: u32) -> Self { Value::Numerical(n as f64) } }
impl From<u64> for Value { fn from(n: u64) -> Self { Value::Numerical(n as f64) } }
impl From<u128> for Value { fn from(n: u128) -> Self { Value::Numerical(n as f64) } }
impl From<usize> for Value { fn from(n: usize) -> Self { Value::Numerical(n as f64) } }
impl From<f32> for Value { fn from(n: f32) -> Self { Value::Numerical(n as f64) } }
impl From<f64> for Value { fn from(n: f64) -> Self { Value::Numerical(n) } }
impl From<char> for Value { fn from(c: char) -> Self { Value::Str(c.to_string()) } }
impl From<bool> for Value { fn from(b: bool) -> Self { Value::Boolean(b) } }
impl From<&str> for Value { fn from(s: &str) -> Self { Value::Str(s.to_string()) } }
impl From<String> for Value { fn from(s: String) -> Self { Value::Str(s) } }
impl From<&String> for Value { fn from(s: &String) -> Self { Value::Str(s.clone()) } } 

// impl From<Vec<Value>> for Value { fn from(vec: Vec<Value>) -> Self { Value::List(vec) } }
// impl From<HashMap<String, Value>> for Value { fn from(map: HashMap<String, Value>) -> Self { Value::Dict(map) } }

// Implement From trait for Vec<T>
impl<T> From<Vec<T>> for Value 
where
    T: Into<Value>,
{
    fn from(vec: Vec<T>) -> Self {
        Value::List(vec.into_iter().map(Into::into).collect())
    }
}

// Implement From trait for HashMap<String, T>
impl<S, T> From<HashMap<S, T>> for Value 
where
    S: Into<String> + Hash + Eq,  
    T: Into<Value>, 
{
    fn from(map: HashMap<S, T>) -> Self {
        Value::Dict(map.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
    }
} 

impl Into<i8> for Value { fn into(self) -> i8 { self.integer() as i8 } } 
impl Into<i16> for Value { fn into(self) -> i16 { self.integer() as i16 } } 
impl Into<i32> for Value { fn into(self) -> i32 { self.integer() as i32 } } 
impl Into<i64> for Value { fn into(self) -> i64 { self.integer() as i64 } } 
impl Into<i128> for Value { fn into(self) -> i128 { self.integer() as i128 } } 
impl Into<isize> for Value { fn into(self) -> isize { self.integer() as isize } } 
impl Into<u8> for Value { fn into(self) -> u8 { self.integer() as u8 } } 
impl Into<u16> for Value { fn into(self) -> u16 { self.integer() as u16 } } 
impl Into<u32> for Value { fn into(self) -> u32 { self.integer() as u32 } } 
impl Into<u64> for Value { fn into(self) -> u64 { self.integer() as u64 } } 
impl Into<u128> for Value { fn into(self) -> u128 { self.integer() as u128 } } 
impl Into<usize> for Value { fn into(self) -> usize { self.integer() as usize } } 
impl Into<f32> for Value { fn into(self) -> f32 { self.numerical() as f32 } } 
impl Into<f64> for Value { fn into(self) -> f64 { self.numerical() } } 
impl Into<char> for Value { fn into(self) -> char { self.string().chars().next().unwrap_or('\0') } } 
impl Into<bool> for Value { fn into(self) -> bool { self.boolean() } } 
