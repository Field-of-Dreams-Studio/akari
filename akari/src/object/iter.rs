use super::value::Value;

/// Key-Value Pair returned by iterators.
/// 
/// This struct holds either owned or borrowed key-value pairs depending on
/// whether it was created from an owned or borrowed iterator.
pub enum KVP<'a> {
    /// An owned key-value pair where both key and value are owned `Value` instances
    Owned {
        /// The key of the key-value pair
        key: Value,
        /// The value of the key-value pair
        value: Value
    },
    /// A borrowed key-value pair where the value is a reference to an existing `Value`
    Borrowed {
        /// The key of the key-value pair (owned since keys are often computed)
        key: Value,
        /// Reference to the value in the key-value pair
        value: &'a Value
    }
}

impl<'a> KVP<'a> {
    /// Returns the key of this key-value pair.
    ///
    /// # Examples
    ///
    /// ```
    /// use akari::Value; 
    /// use akari::KVP; 
    /// 
    /// let kvp = KVP::Owned { key: Value::Numerical(0.0), value: Value::Boolean(true) };
    /// assert!(matches!(kvp.key(), Value::Numerical(0.0)));
    /// ```
    pub fn key(&self) -> &Value {
        match self {
            KVP::Owned { key, .. } => key,
            KVP::Borrowed { key, .. } => key,
        }
    }

    /// Returns the value of this key-value pair.
    ///
    /// # Examples
    ///
    /// ```
    /// use akari::Value; 
    /// use akari::KVP; 
    /// 
    /// let kvp = KVP::Owned { key: Value::Numerical(0.0), value: Value::Boolean(true) };
    /// assert!(matches!(kvp.value(), Value::Boolean(true)));
    /// ```
    pub fn value(&self) -> &Value {
        match self {
            KVP::Owned { value, .. } => value,
            KVP::Borrowed { value, .. } => value,
        }
    }

    /// Consumes this key-value pair and returns owned key and value.
    ///
    /// # Examples
    ///
    /// ```
    /// use akari::Value; 
    /// use akari::KVP; 
    /// 
    /// let kvp = KVP::Borrowed { 
    ///     key: Value::Numerical(0.0), 
    ///     value: &Value::Boolean(true) 
    /// };
    /// let (key, value) = kvp.into_owned();
    /// assert!(matches!(key, Value::Numerical(0.0)));
    /// assert!(matches!(value, Value::Boolean(true)));
    /// ```
    pub fn into_owned(self) -> (Value, Value) {
        match self {
            KVP::Owned { key, value } => (key, value),
            KVP::Borrowed { key, value } => (key, value.clone()),
        }
    }
}

/// A borrowed iterator for `Value` instances.
///
/// This iterator yields key-value pairs from a `Value` without taking ownership.
/// It produces `KVP::Borrowed` instances to minimize cloning of values.
///
/// # Examples
///
/// ```
/// use akari::Value; 
/// use akari::KVP; 
/// 
/// let list = Value::List(vec![Value::Numerical(1.0), Value::Boolean(true)]);
/// let mut iter = list.iter();
/// 
/// if let Some(KVP::Borrowed { key, value }) = iter.next() {
///     assert!(matches!(key, Value::Numerical(0.0)));
///     assert!(matches!(value, Value::Numerical(1.0)));
/// }
/// ```
pub struct IterBorrowed<'a> {
    /// Source data
    source: &'a Value,
    /// Current position for iteration
    pos: usize,
    /// Optional dictionary keys (cached for iteration)
    dict_keys: Option<Vec<&'a String>>,
}

/// An owned iterator for `Value` instances.
///
/// This iterator consumes the original `Value` and yields owned key-value pairs.
/// It produces `KVP::Owned` instances and transfers ownership of the original values.
///
/// # Examples
///
/// ```
/// use akari::Value; 
/// use akari::KVP; 
/// 
/// let list = Value::List(vec![Value::Numerical(1.0), Value::Boolean(true)]);
/// let mut iter = list.into_iter();
/// 
/// if let Some(KVP::Owned { key, value }) = iter.next() {
///     assert!(matches!(key, Value::Numerical(0.0)));
///     assert!(matches!(value, Value::Numerical(1.0)));
/// }
/// ```
pub struct IterOwned {
    /// Owned value being iterated
    source: Value,
    /// Current position for iteration
    pos: usize,
    /// Cached dictionary keys for iteration
    dict_keys: Option<Vec<String>>,
}

impl<'a> Iterator for IterBorrowed<'a> {
    type Item = KVP<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.source {
            Value::Boolean(_) | Value::Numerical(_) | Value::Str(_) => {
                if self.pos == 0 {
                    self.pos += 1; // Mark as done
                    Some(KVP::Borrowed {
                        key: Value::Numerical(0.0),
                        value: self.source,
                    })
                } else {
                    None
                }
            }, 
            Value::List(values) => {
                if self.pos < values.len() {
                    let key = Value::Numerical(self.pos as f64);
                    let value = &values[self.pos];
                    self.pos += 1;
                    Some(KVP::Borrowed { key, value })
                } else {
                    None
                }
            },
            Value::Dict(map) => {
                // Initialize dict_keys if needed
                if self.dict_keys.is_none() {
                    self.dict_keys = Some(map.keys().collect());
                }
                
                if let Some(keys) = &self.dict_keys {
                    if self.pos < keys.len() {
                        let key = keys[self.pos];
                        // This unwrap is safe because we got the key from the map's keys
                        let value = map.get(key).unwrap();
                        let kvp = KVP::Borrowed {
                            key: Value::Str(key.clone()),
                            value,
                        };
                        self.pos += 1;
                        Some(kvp)
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            Value::None => None,
        }
    }
}

impl Iterator for IterOwned {
    type Item = KVP<'static>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.source {
            Value::Boolean(_) | Value::Numerical(_) | Value::Str(_) => {
                if self.pos == 0 {
                    self.pos += 1; // Mark as done
                    Some(KVP::Owned {
                        key: Value::Numerical(0.0),
                        value: std::mem::replace(&mut self.source, Value::None),
                    })
                } else {
                    None
                }
            }, 
            Value::List(values) => {
                if self.pos < values.len() {
                    let key = Value::Numerical(self.pos as f64);
                    // Take ownership to avoid cloning
                    let value = std::mem::replace(&mut values[self.pos], Value::None);
                    self.pos += 1;
                    Some(KVP::Owned { key, value })
                } else {
                    None
                }
            },
            Value::Dict(map) => {
                // Initialize dict_keys if needed
                if self.dict_keys.is_none() {
                    self.dict_keys = Some(map.keys().cloned().collect());
                }
                
                if let Some(keys) = &self.dict_keys {
                    if self.pos < keys.len() {
                        let key = keys[self.pos].clone();
                        // Remove the value from the map
                        let value = map.remove(&key).unwrap_or(Value::None);
                        self.pos += 1;
                        Some(KVP::Owned {
                            key: Value::Str(key),
                            value,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            Value::None => None,
        }
    }
}

impl Value {
    /// Creates a borrowed iterator over this `Value`.
    ///
    /// This method returns an iterator that yields borrowed key-value pairs
    /// without taking ownership of the original value.
    ///
    /// # Returns
    ///
    /// A new `IterBorrowed` instance that yields `KVP::Borrowed` items.
    ///
    /// # Examples
    ///
    /// Iterating over a list:
    /// ```
    /// use akari::Value; 
    /// 
    /// let list = Value::List(vec![
    ///     Value::Numerical(1.0),
    ///     Value::Numerical(2.0)
    /// ]);
    /// 
    /// for pair in list.iter() {
    ///     println!("Key: {:?}, Value: {:?}", pair.key(), pair.value());
    /// }
    /// // Prints:
    /// // Key: Numerical(0.0), Value: Numerical(1.0)
    /// // Key: Numerical(1.0), Value: Numerical(2.0)
    /// ```
    ///
    /// Iterating over a dictionary:
    /// ```
    /// use akari::Value;
    /// use std::collections::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("key".to_string(), Value::Boolean(true));
    /// let dict = Value::Dict(map);
    /// 
    /// for pair in dict.iter() {
    ///     println!("Key: {:?}, Value: {:?}", pair.key(), pair.value());
    /// }
    /// // Prints:
    /// // Key: Str("key"), Value: Boolean(true)
    /// ```
    pub fn iter(&self) -> IterBorrowed<'_> {
        IterBorrowed {
            source: self,
            pos: 0,
            dict_keys: None,
        }
    } 

    /// Creates an owned iterator that consumes this `Value`.
    ///
    /// This method returns an iterator that takes ownership of the original value
    /// and yields owned key-value pairs.
    ///
    /// # Returns
    ///
    /// A new `IterOwned` instance that yields `KVP::Owned` items.
    ///
    /// # Examples
    ///
    /// Consuming a list:
    /// ```
    /// use akari::Value; 
    /// 
    /// let list = Value::List(vec![
    ///     Value::Numerical(1.0),
    ///     Value::Numerical(2.0)
    /// ]);
    /// 
    /// for pair in list.iter_owned() {
    ///     println!("Key: {:?}, Value: {:?}", pair.key(), pair.value());
    /// }
    /// // The original list has been consumed
    /// ```
    ///
    /// Consuming a dictionary:
    /// ```
    /// use akari::Value; 
    /// use std::collections::HashMap; 
    /// 
    /// let mut map = HashMap::new();
    /// map.insert("key".to_string(), Value::Boolean(true));
    /// let dict = Value::Dict(map);
    /// 
    /// for pair in dict.iter_owned() {
    ///     println!("Key: {:?}, Value: {:?}", pair.key(), pair.value());
    /// }
    /// // The original dictionary has been consumed
    /// ```
    pub fn iter_owned(self) -> IterOwned {
        IterOwned {
            source: self,
            pos: 0,
            dict_keys: None,
        }
    } 
} 

// Implement IntoIterator for &Value
impl<'a> IntoIterator for &'a Value {
    type Item = KVP<'a>;
    type IntoIter = IterBorrowed<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// Implement IntoIterator for Value
impl IntoIterator for Value {
    type Item = KVP<'static>;
    type IntoIter = IterOwned;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_owned() 
    }
} 
