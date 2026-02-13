use std::collections::HashMap; 
use super::error::ValueError;
use core::ops::{Index, Range, RangeFrom, RangeTo, RangeFull};
use super::value::Value; 

// Existing Value enum and implementations...

impl Value {
    // --- ADDITION OPERATIONS ---
    
    /// Adds two values according to Python-like type rules.
    /// 
    /// This method will convert the values based on the following rules:
    /// - Numerical + Numerical = Numerical
    /// - Boolean + Boolean = Numerical (True=1, False=0)
    /// - Numerical + Boolean = Numerical (Converting Boolean to int)
    /// - Boolean + Numerical = Numerical (Converting Boolean to int)
    /// - Str + Str = Str (Concatenation)
    /// - List + List = List (Concatenation)
    /// - List + Any = List (Convert right to singleton list, then concatenate)
    /// - Any + List = List (Convert left to singleton list, then concatenate)
    /// - Dict + Dict = Dict (Merge, right values win on key conflict)
    /// - Value + None = Value (Identity operation)
    /// - None + Value = Value (Identity operation)
    /// - Otherwise, returns None
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let num1 = Value::Numerical(5.0);
    /// let num2 = Value::Numerical(3.0);
    /// assert_eq!(num1.add(&num2), Value::Numerical(8.0));
    /// 
    /// let list1 = Value::new(vec![1, 2]);
    /// let list2 = Value::new(vec![3, 4]);
    /// assert_eq!(list1.add(&list2), Value::new(vec![1, 2, 3, 4]));
    /// ```
    pub fn add(&self, rhs: &Value) -> Value {
        match (self, rhs) {
            // Numerical + Numerical
            (Value::Numerical(a), Value::Numerical(b)) => Value::Numerical(a + b),
            
            // Boolean conversions
            (Value::Boolean(a), Value::Boolean(b)) => Value::Numerical((*a as i64 + *b as i64) as f64),
            (Value::Numerical(a), Value::Boolean(b)) => Value::Numerical(a + *b as i64 as f64),
            (Value::Boolean(a), Value::Numerical(b)) => Value::Numerical(*a as i64 as f64 + b),
            
            // String concatenation
            (Value::Str(a), Value::Str(b)) => Value::Str(a.clone() + b),
            
            // List concatenation
            (Value::List(a), Value::List(b)) => {
                let mut result = a.clone();
                result.extend(b.clone());
                Value::List(result)
            },
            
            // Any value + List
            (a, Value::List(b)) if !matches!(a, Value::List(_)) => {
                let mut result = vec![self.clone()];
                result.extend(b.clone());
                Value::List(result)
            },
            
            // List + Any value
            (Value::List(a), b) if !matches!(b, Value::List(_)) => {
                let mut result = a.clone();
                result.push(rhs.clone());
                Value::List(result)
            },
            
            // Dict merging
            (Value::Dict(a), Value::Dict(b)) => {
                let mut result = a.clone();
                for (key, value) in b.iter() {
                    result.insert(key.clone(), value.clone());
                }
                Value::Dict(result)
            },
            
            // None handling
            (Value::None, _) => rhs.clone(),
            (_, Value::None) => self.clone(),
            
            // Error fallback
            _ => Value::None,
        }
    }

    /// Attempts to add two values according to Python-like type rules.
    ///
    /// This method returns a Result that is Ok(Value) on success or an Error on failure.
    /// It follows the same type rules as the `add` method but will return errors for
    /// incompatible types instead of defaulting to None.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let num = Value::Numerical(5.0);
    /// let boolean = Value::Boolean(true);
    /// assert_eq!(num.try_add(&boolean).unwrap(), Value::Numerical(6.0));
    /// 
    /// let num = Value::Numerical(5.0);
    /// let str_val = Value::Str("hello".to_string());
    /// assert!(num.try_add(&str_val).is_err());
    /// ```
    pub fn try_add(&self, rhs: &Value) -> Result<Value, ValueError> {
        match (self, rhs) {
            // Numerical + Numerical
            (Value::Numerical(a), Value::Numerical(b)) => Ok(Value::Numerical(a + b)),
            
            // Boolean conversions
            (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Numerical((*a as i64 + *b as i64) as f64)),
            (Value::Numerical(a), Value::Boolean(b)) => Ok(Value::Numerical(a + *b as i64 as f64)),
            (Value::Boolean(a), Value::Numerical(b)) => Ok(Value::Numerical(*a as i64 as f64 + b)),
            
            // String concatenation
            (Value::Str(a), Value::Str(b)) => Ok(Value::Str(a.clone() + b)),
            
            // List concatenation
            (Value::List(a), Value::List(b)) => {
                let mut result = a.clone();
                result.extend(b.clone());
                Ok(Value::List(result))
            },
            
            // Any value + List
            (a, Value::List(b)) if !matches!(a, Value::List(_)) => {
                let mut result = vec![self.clone()];
                result.extend(b.clone());
                Ok(Value::List(result))
            },
            
            // List + Any value
            (Value::List(a), b) if !matches!(b, Value::List(_)) => {
                let mut result = a.clone();
                result.push(rhs.clone());
                Ok(Value::List(result))
            },
            
            // Dict merging
            (Value::Dict(a), Value::Dict(b)) => {
                let mut result = a.clone();
                for (key, value) in b.iter() {
                    result.insert(key.clone(), value.clone());
                }
                Ok(Value::Dict(result))
            },
            
            // None handling
            (Value::None, _) => Ok(rhs.clone()),
            (_, Value::None) => Ok(self.clone()),
            
            // Error for incompatible types
            _ => Err(ValueError::TypeError),
        }
    }

    /// Adds two values with a custom default value if the operation would fail.
    ///
    /// This method follows the same rules as `add` but allows specifying a default value
    /// in case the operation cannot be performed with the given types.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let num = Value::Numerical(5.0);
    /// let str_val = Value::Str("hello".to_string());
    /// let default = Value::Numerical(0.0);
    /// 
    /// // This would normally fail, but we get the default instead
    /// assert_eq!(num.add_or(&str_val, &default), Value::Numerical(0.0));
    /// ```
    pub fn add_or<'a>(&self, rhs: &Value, default: &'a Value) -> Value {
        self.try_add(rhs).unwrap_or(default.clone())
    }

    /// Adds two values, panicking if the operation would fail.
    ///
    /// # Panics
    /// This method will panic if the operation cannot be performed with the given types.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let num = Value::Numerical(5.0);
    /// let boolean = Value::Boolean(true);
    /// 
    /// // This works fine
    /// assert_eq!(num.add_unchecked(&boolean), Value::Numerical(6.0));
    /// 
    /// // This would panic:
    /// // let str_val = Value::Str("hello".to_string());
    /// // num.add_unchecked(&str_val);
    /// ```
    pub fn add_unchecked(&self, rhs: &Value) -> Value {
        self.try_add(rhs).unwrap()
    }

    /// In-place addition. Equivalent to the += operator.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let mut num = Value::Numerical(5.0);
    /// num.add_assign(&Value::Numerical(3.0));
    /// assert_eq!(num, Value::Numerical(8.0));
    /// 
    /// let mut list = Value::new(vec![1, 2]);
    /// list.add_assign(&Value::new(vec![3, 4]));
    /// assert_eq!(list, Value::new(vec![1, 2, 3, 4]));
    /// ```
    pub fn add_assign(&mut self, rhs: &Value) {
        *self = self.add(rhs);
    }

    /// Tries to perform in-place addition. Returns an error if the operation fails.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let mut num = Value::Numerical(5.0);
    /// assert!(num.try_add_assign(&Value::Numerical(3.0)).is_ok());
    /// assert_eq!(num, Value::Numerical(8.0));
    /// 
    /// let mut num = Value::Numerical(5.0);
    /// assert!(num.try_add_assign(&Value::Str("hello".to_string())).is_err());
    /// // Value remains unchanged on error
    /// assert_eq!(num, Value::Numerical(5.0));
    /// ```
    pub fn try_add_assign(&mut self, rhs: &Value) -> Result<(), ValueError> {
        match self.try_add(rhs) {
            Ok(result) => {
                *self = result;
                Ok(())
            },
            Err(e) => Err(e),
        }
    }

    /// In-place addition with a default value if the operation would fail.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let mut num = Value::Numerical(5.0);
    /// let default = Value::Numerical(0.0);
    /// 
    /// // This would normally fail, but we get the default instead
    /// num.add_assign_or(&Value::Str("hello".to_string()), &default);
    /// assert_eq!(num, Value::Numerical(0.0));
    /// ```
    pub fn add_assign_or(&mut self, rhs: &Value, default: &Value) {
        *self = self.add_or(rhs, default);
    }

    /// In-place addition that panics if the operation would fail.
    ///
    /// # Panics
    /// This method will panic if the operation cannot be performed with the given types.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let mut num = Value::Numerical(5.0);
    /// 
    /// // This works fine
    /// num.add_assign_unchecked(&Value::Numerical(3.0));
    /// assert_eq!(num, Value::Numerical(8.0));
    /// ```
    pub fn add_assign_unchecked(&mut self, rhs: &Value) {
        *self = self.add_unchecked(rhs);
    } 
    
    // --- SUBTRACTION OPERATIONS ---
    
    /// Subtracts one value from another according to Python-like type rules.
    /// 
    /// This method will convert the values based on the following rules:
    /// - Numerical - Numerical = Numerical
    /// - Boolean - Boolean = Numerical (True=1, False=0)
    /// - Numerical - Boolean = Numerical (Converting Boolean to int)
    /// - Boolean - Numerical = Numerical (Converting Boolean to int)
    /// - Dict - Dict = Dict (Remove all keys from left that are present in right)
    /// - Value - None = Value (Identity operation)
    /// - None - Value = Value (Identity operation)
    /// - Otherwise, returns None
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let num1 = Value::Numerical(5.0);
    /// let num2 = Value::Numerical(3.0);
    /// assert_eq!(num1.sub(&num2), Value::Numerical(2.0));
    /// ```
    pub fn sub(&self, rhs: &Value) -> Value {
        match (self, rhs) {
            // Numerical - Numerical
            (Value::Numerical(a), Value::Numerical(b)) => Value::Numerical(a - b),
            
            // Boolean conversions
            (Value::Boolean(a), Value::Boolean(b)) => Value::Numerical((*a as i64 - *b as i64) as f64),
            (Value::Numerical(a), Value::Boolean(b)) => Value::Numerical(a - *b as i64 as f64),
            (Value::Boolean(a), Value::Numerical(b)) => Value::Numerical(*a as i64 as f64 - b),
            
            // Dict key removal
            (Value::Dict(a), Value::Dict(b)) => {
                let mut result = a.clone();
                for key in b.keys() {
                    result.remove(key);
                }
                Value::Dict(result)
            },
            
            // None handling
            (Value::None, _) => rhs.clone(),
            (_, Value::None) => self.clone(),
            
            // Error fallback
            _ => Value::None,
        }
    }

    /// Attempts to subtract one value from another according to Python-like type rules.
    ///
    /// This method returns a Result that is Ok(Value) on success or an Error on failure.
    /// It follows the same type rules as the `sub` method but will return errors for
    /// incompatible types instead of defaulting to None.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let num = Value::Numerical(5.0);
    /// let boolean = Value::Boolean(true);
    /// assert_eq!(num.try_sub(&boolean).unwrap(), Value::Numerical(4.0));
    /// 
    /// let str_val = Value::Str("hello".to_string());
    /// assert!(num.try_sub(&str_val).is_err());
    /// ```
    pub fn try_sub(&self, rhs: &Value) -> Result<Value, ValueError> {
        match (self, rhs) {
            // Numerical - Numerical
            (Value::Numerical(a), Value::Numerical(b)) => Ok(Value::Numerical(a - b)),
            
            // Boolean conversions
            (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Numerical((*a as i64 - *b as i64) as f64)),
            (Value::Numerical(a), Value::Boolean(b)) => Ok(Value::Numerical(a - *b as i64 as f64)),
            (Value::Boolean(a), Value::Numerical(b)) => Ok(Value::Numerical(*a as i64 as f64 - b)),
            
            // Dict key removal
            (Value::Dict(a), Value::Dict(b)) => {
                let mut result = a.clone();
                for key in b.keys() {
                    result.remove(key);
                }
                Ok(Value::Dict(result))
            },
            
            // None handling
            (Value::None, _) => Ok(rhs.clone()),
            (_, Value::None) => Ok(self.clone()),
            
            // Error for incompatible types
            _ => Err(ValueError::TypeError),
        }
    }

    /// Subtracts one value from another with a custom default value if the operation would fail.
    ///
    /// This method follows the same rules as `sub` but allows specifying a default value
    /// in case the operation cannot be performed with the given types.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let num = Value::Numerical(5.0);
    /// let str_val = Value::Str("hello".to_string());
    /// let default = Value::Numerical(0.0);
    /// 
    /// // This would normally fail, but we get the default instead
    /// assert_eq!(num.sub_or(&str_val, &default), Value::Numerical(0.0));
    /// ```
    pub fn sub_or<'a>(&self, rhs: &Value, default: &'a Value) -> Value {
        self.try_sub(rhs).unwrap_or(default.clone())
    }

    /// Subtracts one value from another, panicking if the operation would fail.
    ///
    /// # Panics
    /// This method will panic if the operation cannot be performed with the given types.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let num = Value::Numerical(5.0);
    /// let boolean = Value::Boolean(true);
    /// 
    /// // This works fine
    /// assert_eq!(num.sub_unchecked(&boolean), Value::Numerical(4.0));
    /// ```
    pub fn sub_unchecked(&self, rhs: &Value) -> Value {
        self.try_sub(rhs).unwrap()
    } 

    /// In-place subtraction. Equivalent to the -= operator.
    pub fn sub_assign(&mut self, rhs: &Value) {
        *self = self.sub(rhs);
    }

    /// Tries to perform in-place subtraction. Returns an error if the operation fails.
    pub fn try_sub_assign(&mut self, rhs: &Value) -> Result<(), ValueError> {
        match self.try_sub(rhs) {
            Ok(result) => {
                *self = result;
                Ok(())
            },
            Err(e) => Err(e),
        }
    }

    /// In-place subtraction with a default value if the operation would fail.
    pub fn sub_assign_or(&mut self, rhs: &Value, default: &Value) {
        *self = self.sub_or(rhs, default);
    }

    /// In-place subtraction that panics if the operation would fail.
    pub fn sub_assign_unchecked(&mut self, rhs: &Value) {
        *self = self.sub_unchecked(rhs);
    } 
    
    // --- MULTIPLICATION OPERATIONS ---
    
    /// Multiplies two values according to Python-like type rules.
    /// 
    /// This method will convert the values based on the following rules:
    /// - Numerical * Numerical = Numerical
    /// - Boolean * Boolean = Numerical (True=1, False=0)
    /// - Numerical * Boolean = Numerical (Converting Boolean to int)
    /// - Boolean * Numerical = Numerical (Converting Boolean to int)
    /// - Str * Numerical = Str (Repeat string N times)
    /// - Numerical * Str = Str (Repeat string N times)
    /// - List * Numerical = List (Repeat list N times)
    /// - Numerical * List = List (Repeat list N times)
    /// - Dict * Numerical = List (Repeat dict N times as list of dicts)
    /// - Numerical * Dict = List (Repeat dict N times as list of dicts)
    /// - Otherwise, returns None
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let num1 = Value::Numerical(5.0);
    /// let num2 = Value::Numerical(3.0);
    /// assert_eq!(num1.mul(&num2), Value::Numerical(15.0));
    /// 
    /// let str_val = Value::Str("abc".to_string());
    /// let num = Value::Numerical(3.0);
    /// assert_eq!(str_val.mul(&num), Value::Str("abcabcabc".to_string()));
    /// ```
    pub fn mul(&self, rhs: &Value) -> Value {
        match (self, rhs) {
            // Numerical * Numerical
            (Value::Numerical(a), Value::Numerical(b)) => Value::Numerical(a * b),
            
            // Boolean conversions
            (Value::Boolean(a), Value::Boolean(b)) => Value::Numerical((*a as i64 * *b as i64) as f64),
            (Value::Numerical(a), Value::Boolean(b)) => Value::Numerical(a * *b as i64 as f64),
            (Value::Boolean(a), Value::Numerical(b)) => Value::Numerical(*a as i64 as f64 * b),
            
            // String repetition
            (Value::Str(a), Value::Numerical(b)) => {
                if b.fract() == 0.0 && *b >= 0.0 {
                    let count = *b as i64;
                    Value::Str(a.repeat(count as usize))
                } else {
                    Value::None
                }
            },
            (Value::Numerical(a), Value::Str(b)) => {
                if a.fract() == 0.0 && *a >= 0.0 {
                    let count = *a as i64;
                    Value::Str(b.repeat(count as usize))
                } else {
                    Value::None
                }
            },
            
            // List repetition
            (Value::List(a), Value::Numerical(b)) => {
                if b.fract() == 0.0 && *b >= 0.0 {
                    let count = *b as i64;
                    let mut result = Vec::new();
                    for _ in 0..count {
                        result.extend(a.clone());
                    }
                    Value::List(result)
                } else {
                    Value::None
                }
            },
            (Value::Numerical(a), Value::List(b)) => {
                if a.fract() == 0.0 && *a >= 0.0 {
                    let count = *a as i64;
                    let mut result = Vec::new();
                    for _ in 0..count {
                        result.extend(b.clone());
                    }
                    Value::List(result)
                } else {
                    Value::None
                }
            },
            
            // Dict repetition (creates list of dicts)
            (Value::Dict(a), Value::Numerical(b)) => {
                if b.fract() == 0.0 && *b >= 0.0 {
                    let count = *b as i64;
                    let mut result = Vec::new();
                    for _ in 0..count {
                        result.push(Value::Dict(a.clone()));
                    }
                    Value::List(result)
                } else {
                    Value::None
                }
            },
            (Value::Numerical(a), Value::Dict(b)) => {
                if a.fract() == 0.0 && *a >= 0.0 {
                    let count = *a as i64;
                    let mut result = Vec::new();
                    for _ in 0..count {
                        result.push(Value::Dict(b.clone()));
                    }
                    Value::List(result)
                } else {
                    Value::None
                }
            },
            
            // Error fallback
            _ => Value::None,
        }
    }

    /// Attempts to multiply two values according to Python-like type rules.
    ///
    /// This method returns a Result that is Ok(Value) on success or an Error on failure.
    /// It follows the same type rules as the `mul` method but will return errors for
    /// incompatible types instead of defaulting to None.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let str_val = Value::Str("abc".to_string());
    /// let num = Value::Numerical(3.0);
    /// assert_eq!(str_val.try_mul(&num).unwrap(), Value::Str("abcabcabc".to_string()));
    /// 
    /// // Fractional repeats are not allowed
    /// let str_val = Value::Str("abc".to_string());
    /// let frac = Value::Numerical(2.5);
    /// assert!(str_val.try_mul(&frac).is_err());
    /// ```
    pub fn try_mul(&self, rhs: &Value) -> Result<Value, ValueError> {
        match (self, rhs) {
            // Numerical * Numerical
            (Value::Numerical(a), Value::Numerical(b)) => Ok(Value::Numerical(a * b)),
            
            // Boolean conversions
            (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Numerical((*a as i64 * *b as i64) as f64)),
            (Value::Numerical(a), Value::Boolean(b)) => Ok(Value::Numerical(a * *b as i64 as f64)),
            (Value::Boolean(a), Value::Numerical(b)) => Ok(Value::Numerical(*a as i64 as f64 * b)),
            
            // String repetition
            (Value::Str(a), Value::Numerical(b)) => {
                if b.fract() == 0.0 && *b >= 0.0 {
                    let count = *b as i64;
                    Ok(Value::Str(a.repeat(count as usize)))
                } else {
                    Err(ValueError::InvalidOperationError)
                }
            },
            (Value::Numerical(a), Value::Str(b)) => {
                if a.fract() == 0.0 && *a >= 0.0 {
                    let count = *a as i64;
                    Ok(Value::Str(b.repeat(count as usize)))
                } else {
                    Err(ValueError::InvalidOperationError)
                }
            },
            
            // List repetition
            (Value::List(a), Value::Numerical(b)) => {
                if b.fract() == 0.0 && *b >= 0.0 {
                    let count = *b as i64;
                    let mut result = Vec::new();
                    for _ in 0..count {
                        result.extend(a.clone());
                    }
                    Ok(Value::List(result))
                } else {
                    Err(ValueError::InvalidOperationError)
                }
            },
            (Value::Numerical(a), Value::List(b)) => {
                if a.fract() == 0.0 && *a >= 0.0 {
                    let count = *a as i64;
                    let mut result = Vec::new();
                    for _ in 0..count {
                        result.extend(b.clone());
                    }
                    Ok(Value::List(result))
                } else {
                    Err(ValueError::InvalidOperationError)
                }
            },
            
            // Dict repetition (creates list of dicts)
            (Value::Dict(a), Value::Numerical(b)) => {
                if b.fract() == 0.0 && *b >= 0.0 {
                    let count = *b as i64;
                    let mut result = Vec::new();
                    for _ in 0..count {
                        result.push(Value::Dict(a.clone()));
                    }
                    Ok(Value::List(result))
                } else {
                    Err(ValueError::InvalidOperationError)
                }
            },
            (Value::Numerical(a), Value::Dict(b)) => {
                if a.fract() == 0.0 && *a >= 0.0 {
                    let count = *a as i64;
                    let mut result = Vec::new();
                    for _ in 0..count {
                        result.push(Value::Dict(b.clone()));
                    }
                    Ok(Value::List(result))
                } else {
                    Err(ValueError::InvalidOperationError)
                }
            },
            
            // Error for incompatible types
            _ => Err(ValueError::TypeError),
        }
    }

    /// Multiplies two values with a custom default value if the operation would fail.
    ///
    /// This method follows the same rules as `mul` but allows specifying a default value
    /// in case the operation cannot be performed with the given types.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let str_val = Value::Str("abc".to_string());
    /// let frac = Value::Numerical(2.5);  // Fractions not allowed for repetition
    /// let default = Value::Str("default".to_string());
    /// 
    /// assert_eq!(str_val.mul_or(&frac, &default), Value::Str("default".to_string()));
    /// ```
    pub fn mul_or<'a>(&self, rhs: &Value, default: &'a Value) -> Value {
        self.try_mul(rhs).unwrap_or(default.clone())
    }

    /// Multiplies two values, panicking if the operation would fail.
    ///
    /// # Panics
    /// This method will panic if the operation cannot be performed with the given types.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let str_val = Value::Str("abc".to_string());
    /// let num = Value::Numerical(3.0);
    /// 
    /// // This works fine
    /// assert_eq!(str_val.mul_unchecked(&num), Value::Str("abcabcabc".to_string()));
    /// ```
    pub fn mul_unchecked(&self, rhs: &Value) -> Value {
        self.try_mul(rhs).unwrap()
    } 

    /// In-place multiplication. Equivalent to the *= operator.
    pub fn mul_assign(&mut self, rhs: &Value) {
        *self = self.mul(rhs);
    }

    /// Tries to perform in-place multiplication. Returns an error if the operation fails.
    pub fn try_mul_assign(&mut self, rhs: &Value) -> Result<(), ValueError> {
        match self.try_mul(rhs) {
            Ok(result) => {
                *self = result;
                Ok(())
            },
            Err(e) => Err(e),
        }
    }

    /// In-place multiplication with a default value if the operation would fail.
    pub fn mul_assign_or(&mut self, rhs: &Value, default: &Value) {
        *self = self.mul_or(rhs, default);
    }

    /// In-place multiplication that panics if the operation would fail.
    pub fn mul_assign_unchecked(&mut self, rhs: &Value) {
        *self = self.mul_unchecked(rhs);
    } 

    // --- DIVISION OPERATIONS ---
    
    /// Divides two values according to Python-like type rules.
    /// 
    /// This method will convert the values based on the following rules:
    /// - Numerical / Numerical = Numerical
    /// - Boolean / Boolean = Numerical (True=1, False=0)
    /// - Numerical / Boolean = Numerical (Converting Boolean to int)
    /// - Boolean / Numerical = Numerical (Converting Boolean to int)
    /// - List / Numerical = List of Lists (Splits list into N chunks)
    /// - Dict / Numerical = List of Dictionaries (Splits dict into N dicts)
    /// - Otherwise, returns None
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let num1 = Value::Numerical(10.0);
    /// let num2 = Value::Numerical(2.0);
    /// assert_eq!(num1.div(&num2), Value::Numerical(5.0));
    /// 
    /// // List chunking
    /// let list = Value::new(vec![1, 2, 3, 4, 5, 6]);
    /// let chunks = Value::Numerical(2.0);
    /// // Should split into [[1,2,3], [4,5,6]]
    /// ```
    pub fn div(&self, rhs: &Value) -> Value {
        match (self, rhs) {
            // Numerical / Numerical
            (Value::Numerical(a), Value::Numerical(b)) => {
                if *b == 0.0 {
                    Value::None
                } else {
                    Value::Numerical(a / b)
                }
            },
            
            // Boolean conversions
            (Value::Boolean(a), Value::Boolean(b)) => {
                if !*b {
                    Value::None
                } else {
                    Value::Numerical((*a as i64) as f64)
                }
            },
            (Value::Numerical(a), Value::Boolean(b)) => {
                if !*b {
                    Value::None
                } else {
                    Value::Numerical(a / 1.0)
                }
            },
            (Value::Boolean(a), Value::Numerical(b)) => {
                if *b == 0.0 {
                    Value::None
                } else {
                    Value::Numerical((*a as i64) as f64 / b)
                }
            },
            
            // List chunking
            (Value::List(a), Value::Numerical(b)) => {
                if b.fract() == 0.0 && *b > 0.0 {
                    let chunks = *b as usize;
                    let chunk_size = (a.len() as f64 / *b).ceil() as usize;
                    let mut result = Vec::new();
                    
                    for i in 0..chunks {
                        let start = i * chunk_size;
                        let end = (i + 1) * chunk_size;
                        if start >= a.len() {
                            break;
                        }
                        let end = end.min(a.len());
                        result.push(Value::List(a[start..end].to_vec()));
                    }
                    
                    Value::List(result)
                } else {
                    Value::None
                }
            },
            
            // Dict chunking to lists of dictionaries
            (Value::Dict(a), Value::Numerical(b)) => {
                if b.fract() == 0.0 && *b > 0.0 {
                    let chunks = *b as usize;
                    
                    // Convert to entries
                    let entries: Vec<(String, Value)> = a.iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();
                    
                    let chunk_size = (entries.len() as f64 / *b).ceil() as usize;
                    let mut result = Vec::new();
                    
                    for i in 0..chunks {
                        let start = i * chunk_size;
                        if start >= entries.len() {
                            break;
                        }
                        let end = ((i + 1) * chunk_size).min(entries.len());
                        
                        let mut chunk_dict = HashMap::new();
                        for (k, v) in &entries[start..end] {
                            chunk_dict.insert(k.clone(), v.clone());
                        }
                        
                        result.push(Value::Dict(chunk_dict));
                    }
                    
                    Value::List(result)
                } else {
                    Value::None
                }
            },
            
            // Error fallback
            _ => Value::None,
        }
    }

    /// Attempts to divide two values according to Python-like type rules.
    ///
    /// This method returns a Result that is Ok(Value) on success or an Error on failure.
    /// It follows the same type rules as the `div` method but will return errors for
    /// incompatible types instead of defaulting to None.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let num1 = Value::Numerical(10.0);
    /// let num2 = Value::Numerical(2.0);
    /// assert_eq!(num1.try_div(&num2).unwrap(), Value::Numerical(5.0));
    /// 
    /// // Division by zero
    /// let num1 = Value::Numerical(10.0);
    /// let num2 = Value::Numerical(0.0);
    /// assert!(num1.try_div(&num2).is_err());
    /// ```
    pub fn try_div(&self, rhs: &Value) -> Result<Value, ValueError> {
        match (self, rhs) {
            // Numerical / Numerical
            (Value::Numerical(a), Value::Numerical(b)) => {
                if *b == 0.0 {
                    Err(ValueError::DivisionByZeroError)
                } else {
                    Ok(Value::Numerical(a / b))
                }
            },
            
            // Boolean conversions
            (Value::Boolean(a), Value::Boolean(b)) => {
                if !*b {
                    Err(ValueError::DivisionByZeroError)
                } else {
                    Ok(Value::Numerical((*a as i64) as f64))
                }
            },
            (Value::Numerical(a), Value::Boolean(b)) => {
                if !*b {
                    Err(ValueError::DivisionByZeroError)
                } else {
                    Ok(Value::Numerical(a / 1.0))
                }
            },
            (Value::Boolean(a), Value::Numerical(b)) => {
                if *b == 0.0 {
                    Err(ValueError::DivisionByZeroError)
                } else {
                    Ok(Value::Numerical((*a as i64) as f64 / b))
                }
            },
            
            // List chunking
            (Value::List(a), Value::Numerical(b)) => {
                if b.fract() == 0.0 && *b > 0.0 {
                    let chunks = *b as usize;
                    let chunk_size = (a.len() as f64 / *b).ceil() as usize;
                    let mut result = Vec::new();
                    
                    for i in 0..chunks {
                        let start = i * chunk_size;
                        let end = (i + 1) * chunk_size;
                        if start >= a.len() {
                            break;
                        }
                        let end = end.min(a.len());
                        result.push(Value::List(a[start..end].to_vec()));
                    }
                    
                    Ok(Value::List(result))
                } else if *b <= 0.0 {
                    Err(ValueError::InvalidOperationError)
                } else {
                    Err(ValueError::InvalidOperationError)
                }
            },
            
            // Dict chunking
            (Value::Dict(a), Value::Numerical(b)) => {
                if b.fract() == 0.0 && *b > 0.0 {
                    let chunks = *b as usize;
                    
                    // Convert to entries
                    let entries: Vec<(String, Value)> = a.iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();
                    
                    let chunk_size = (entries.len() as f64 / *b).ceil() as usize;
                    let mut result = Vec::new();
                    
                    for i in 0..chunks {
                        let start = i * chunk_size;
                        if start >= entries.len() {
                            break;
                        }
                        let end = ((i + 1) * chunk_size).min(entries.len());
                        
                        let mut chunk_dict = HashMap::new();
                        for (k, v) in &entries[start..end] {
                            chunk_dict.insert(k.clone(), v.clone());
                        }
                        
                        result.push(Value::Dict(chunk_dict));
                    }
                    
                    Ok(Value::List(result))
                } else if *b <= 0.0 {
                    Err(ValueError::InvalidOperationError)
                } else {
                    Err(ValueError::InvalidOperationError)
                }
            },
            
            // Error for incompatible types
            _ => Err(ValueError::TypeError),
        }
    }

    /// Divides two values with a custom default value if the operation would fail.
    ///
    /// This method follows the same rules as `div` but allows specifying a default value
    /// in case the operation cannot be performed with the given types.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let num1 = Value::Numerical(10.0);
    /// let num2 = Value::Numerical(0.0);  // Division by zero
    /// let default = Value::Numerical(0.0);
    /// 
    /// assert_eq!(num1.div_or(&num2, &default), Value::Numerical(0.0));
    /// ```
    pub fn div_or<'a>(&self, rhs: &Value, default: &'a Value) -> Value {
        self.try_div(rhs).unwrap_or(default.clone())
    }

    /// Divides two values, panicking if the operation would fail.
    ///
    /// # Panics
    /// This method will panic if the operation cannot be performed with the given types,
    /// including division by zero.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let num1 = Value::Numerical(10.0);
    /// let num2 = Value::Numerical(2.0);
    /// 
    /// // This works fine
    /// assert_eq!(num1.div_unchecked(&num2), Value::Numerical(5.0));
    /// ```
    pub fn div_unchecked(&self, rhs: &Value) -> Value {
        self.try_div(rhs).unwrap()
    }

    /// In-place division. Equivalent to the /= operator.
    pub fn div_assign(&mut self, rhs: &Value) {
        *self = self.div(rhs);
    }

    /// Tries to perform in-place division. Returns an error if the operation fails.
    pub fn try_div_assign(&mut self, rhs: &Value) -> Result<(), ValueError> {
        match self.try_div(rhs) {
            Ok(result) => {
                *self = result;
                Ok(())
            },
            Err(e) => Err(e),
        }
    }

    /// In-place division with a default value if the operation would fail.
    pub fn div_assign_or(&mut self, rhs: &Value, default: &Value) {
        *self = self.div_or(rhs, default);
    }

    /// In-place division that panics if the operation would fail.
    pub fn div_assign_unchecked(&mut self, rhs: &Value) {
        *self = self.div_unchecked(rhs);
    } 

    // --- MODULO OPERATIONS ---
    
    /// Performs modulo operation between two values according to Python-like type rules.
    /// 
    /// This method will convert the values based on the following rules:
    /// - Numerical % Numerical = Numerical
    /// - Boolean % Boolean = Numerical (True=1, False=0)
    /// - Numerical % Boolean = Numerical (Converting Boolean to int)
    /// - Boolean % Numerical = Numerical (Converting Boolean to int)
    /// - Str % Str = Str (Python-style string formatting: "a %s" % "b" = "a b")
    /// - Str % Dict = Str (Python-style format with dict: "%(key)s" % {"key": "value"})
    /// - Otherwise, returns None
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let num1 = Value::Numerical(10.0);
    /// let num2 = Value::Numerical(3.0);
    /// assert_eq!(num1.modulo(&num2), Value::Numerical(1.0));
    /// 
    /// let str_fmt = Value::Str("Hello, %s!".to_string());
    /// let str_arg = Value::Str("world".to_string());
    /// assert_eq!(str_fmt.modulo(&str_arg), Value::Str("Hello, world!".to_string()));
    /// ```
    pub fn modulo(&self, rhs: &Value) -> Value {
        match (self, rhs) {
            // Numerical % Numerical
            (Value::Numerical(a), Value::Numerical(b)) => {
                if *b == 0.0 {
                    Value::None
                } else {
                    // Follow Python's modulo behavior: a % b = a - b * floor(a / b)
                    let result = a - b * (a / b).floor();
                    Value::Numerical(result)
                }
            },
            
            // Boolean conversions
            (Value::Boolean(a), Value::Boolean(b)) => {
                if !*b {
                    Value::None
                } else {
                    Value::Numerical((*a as i64 % *b as i64) as f64)
                }
            },
            (Value::Numerical(a), Value::Boolean(b)) => {
                if !*b {
                    Value::None
                } else {
                    let b_int = *b as i64 as f64;
                    let result = a - b_int * (a / b_int).floor();
                    Value::Numerical(result)
                }
            },
            (Value::Boolean(a), Value::Numerical(b)) => {
                if *b == 0.0 {
                    Value::None
                } else {
                    let a_int = *a as i64 as f64;
                    let result = a_int - b * (a_int / b).floor();
                    Value::Numerical(result)
                }
            },
            
            // String formatting (very simplified version of Python's)
            (Value::Str(format_str), Value::Str(value)) => {
                // Super simple implementation - just replace %s with the value
                let result = format_str.replace("%s", value);
                Value::Str(result)
            },
            
            // Dict-based formatting (simplified)
            (Value::Str(format_str), Value::Dict(dict)) => {
                // Very basic implementation - replace %(key)s with dict[key]
                let mut result = format_str.clone();
                
                for (key, value) in dict {
                    let placeholder = format!("%({})s", key);
                    if result.contains(&placeholder) {
                        result = result.replace(&placeholder, &value.string());
                    }
                }
                
                Value::Str(result)
            },
            
            // Error fallback
            _ => Value::None,
        }
    }

    /// Attempts to perform modulo operation between two values.
    ///
    /// This method returns a Result that is Ok(Value) on success or an Error on failure.
    /// It follows the same type rules as the `modulo` method.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let num1 = Value::Numerical(10.0);
    /// let num2 = Value::Numerical(3.0);
    /// assert_eq!(num1.try_modulo(&num2).unwrap(), Value::Numerical(1.0));
    /// 
    /// // Modulo by zero
    /// let num1 = Value::Numerical(10.0);
    /// let num2 = Value::Numerical(0.0);
    /// assert!(num1.try_modulo(&num2).is_err());
    /// ```
    pub fn try_modulo(&self, rhs: &Value) -> Result<Value, ValueError> {
        match (self, rhs) {
            // Numerical % Numerical
            (Value::Numerical(a), Value::Numerical(b)) => {
                if *b == 0.0 {
                    Err(ValueError::DivisionByZeroError)
                } else {
                    // Follow Python's modulo behavior: a % b = a - b * floor(a / b)
                    let result = a - b * (a / b).floor();
                    Ok(Value::Numerical(result))
                }
            },
            
            // Boolean conversions
            (Value::Boolean(a), Value::Boolean(b)) => {
                if !*b {
                    Err(ValueError::DivisionByZeroError)
                } else {
                    Ok(Value::Numerical((*a as i64 % *b as i64) as f64))
                }
            },
            (Value::Numerical(a), Value::Boolean(b)) => {
                if !*b {
                    Err(ValueError::DivisionByZeroError)
                } else {
                    let b_int = *b as i64 as f64;
                    let result = a - b_int * (a / b_int).floor();
                    Ok(Value::Numerical(result))
                }
            },
            (Value::Boolean(a), Value::Numerical(b)) => {
                if *b == 0.0 {
                    Err(ValueError::DivisionByZeroError)
                } else {
                    let a_int = *a as i64 as f64;
                    let result = a_int - b * (a_int / b).floor();
                    Ok(Value::Numerical(result))
                }
            },
            
            // String formatting
            (Value::Str(format_str), Value::Str(value)) => {
                let result = format_str.replace("%s", value);
                Ok(Value::Str(result))
            },
            
            // Dict-based formatting
            (Value::Str(format_str), Value::Dict(dict)) => {
                let mut result = format_str.clone();
                
                for (key, value) in dict {
                    let placeholder = format!("%({})s", key);
                    if result.contains(&placeholder) {
                        result = result.replace(&placeholder, &value.string());
                    }
                }
                
                Ok(Value::Str(result))
            },
            
            // Error for incompatible types
            _ => Err(ValueError::TypeError),
        }
    }

    /// Performs modulo operation with a custom default value if the operation would fail.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let num1 = Value::Numerical(10.0);
    /// let num2 = Value::Numerical(0.0);  // Modulo by zero
    /// let default = Value::Numerical(0.0);
    /// 
    /// assert_eq!(num1.modulo_or(&num2, &default), Value::Numerical(0.0));
    /// ```
    pub fn modulo_or<'a>(&self, rhs: &Value, default: &'a Value) -> Value {
        self.try_modulo(rhs).unwrap_or(default.clone())
    }

    /// Performs modulo operation, panicking if it would fail.
    ///
    /// # Panics
    /// This method will panic if the operation cannot be performed with the given types,
    /// including modulo by zero.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let num1 = Value::Numerical(10.0);
    /// let num2 = Value::Numerical(3.0);
    /// 
    /// assert_eq!(num1.modulo_unchecked(&num2), Value::Numerical(1.0));
    /// ```
    pub fn modulo_unchecked(&self, rhs: &Value) -> Value {
        self.try_modulo(rhs).unwrap()
    } 

    /// In-place modulo. Equivalent to the %= operator.
    pub fn modulo_assign(&mut self, rhs: &Value) {
        *self = self.modulo(rhs);
    }

    /// Tries to perform in-place modulo. Returns an error if the operation fails.
    pub fn try_modulo_assign(&mut self, rhs: &Value) -> Result<(), ValueError> {
        match self.try_modulo(rhs) {
            Ok(result) => {
                *self = result;
                Ok(())
            },
            Err(e) => Err(e),
        }
    }

    /// In-place modulo with a default value if the operation would fail.
    pub fn modulo_assign_or(&mut self, rhs: &Value, default: &Value) {
        *self = self.modulo_or(rhs, default);
    }

    /// In-place modulo that panics if the operation would fail.
    pub fn modulo_assign_unchecked(&mut self, rhs: &Value) {
        *self = self.modulo_unchecked(rhs);
    } 
    
    // --- EXPONENTIATION OPERATIONS ---
    
    /// Raises one value to the power of another according to Python-like type rules.
    /// 
    /// This method will convert the values based on the following rules:
    /// - Numerical ** Numerical = Numerical
    /// - Boolean ** Boolean = Numerical (True=1, False=0)
    /// - Str ** Numerical = Str (Same as Str * Numerical)
    /// - List ** Numerical = List (Same as List * Numerical)
    /// - Otherwise, returns None
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let num1 = Value::Numerical(2.0);
    /// let num2 = Value::Numerical(3.0);
    /// assert_eq!(num1.pow(&num2), Value::Numerical(8.0));
    /// ```
    pub fn pow(&self, rhs: &Value) -> Value {
        match (self, rhs) {
            // Numerical ** Numerical
            (Value::Numerical(a), Value::Numerical(b)) => {
                Value::Numerical(a.powf(*b))
            },
            
            // Boolean conversions
            (Value::Boolean(a), Value::Boolean(b)) => {
                Value::Numerical((*a as i64 as f64).powf(*b as i64 as f64))
            },
            (Value::Numerical(a), Value::Boolean(b)) => {
                Value::Numerical(a.powf(*b as i64 as f64))
            },
            (Value::Boolean(a), Value::Numerical(b)) => {
                Value::Numerical((*a as i64 as f64).powf(*b))
            },
            
            // String repetition (same as multiplication)
            (Value::Str(a), Value::Numerical(b)) => {
                if b.fract() == 0.0 && *b >= 0.0 {
                    let count = *b as i64;
                    Value::Str(a.repeat(count as usize))
                } else {
                    Value::None
                }
            },
            
            // List repetition (same as multiplication)
            (Value::List(a), Value::Numerical(b)) => {
                if b.fract() == 0.0 && *b >= 0.0 {
                    let count = *b as i64;
                    let mut result = Vec::new();
                    for _ in 0..count {
                        result.extend(a.clone());
                    }
                    Value::List(result)
                } else {
                    Value::None
                }
            },
            
            // Error fallback
            _ => Value::None,
        }
    }

    /// Attempts to raise one value to the power of another.
    ///
    /// This method returns a Result that is Ok(Value) on success or an Error on failure.
    /// It follows the same type rules as the `pow` method.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let num1 = Value::Numerical(2.0);
    /// let num2 = Value::Numerical(3.0);
    /// assert_eq!(num1.try_pow(&num2).unwrap(), Value::Numerical(8.0));
    /// 
    /// // Invalid exponentiation
    /// let str_val = Value::Str("abc".to_string());
    /// let frac = Value::Numerical(2.5);  // Fractional powers not allowed for strings
    /// assert!(str_val.try_pow(&frac).is_err());
    /// ```
    pub fn try_pow(&self, rhs: &Value) -> Result<Value, ValueError> {
        match (self, rhs) {
            // Numerical ** Numerical
            (Value::Numerical(a), Value::Numerical(b)) => {
                Ok(Value::Numerical(a.powf(*b)))
            },
            
            // Boolean conversions
            (Value::Boolean(a), Value::Boolean(b)) => {
                Ok(Value::Numerical((*a as i64 as f64).powf(*b as i64 as f64)))
            },
            (Value::Numerical(a), Value::Boolean(b)) => {
                Ok(Value::Numerical(a.powf(*b as i64 as f64)))
            },
            (Value::Boolean(a), Value::Numerical(b)) => {
                Ok(Value::Numerical((*a as i64 as f64).powf(*b)))
            },
            
            // String repetition
            (Value::Str(a), Value::Numerical(b)) => {
                if b.fract() == 0.0 && *b >= 0.0 {
                    let count = *b as i64;
                    Ok(Value::Str(a.repeat(count as usize)))
                } else {
                    Err(ValueError::InvalidOperationError)
                }
            },
            
            // List repetition
            (Value::List(a), Value::Numerical(b)) => {
                if b.fract() == 0.0 && *b >= 0.0 {
                    let count = *b as i64;
                    let mut result = Vec::new();
                    for _ in 0..count {
                        result.extend(a.clone());
                    }
                    Ok(Value::List(result))
                } else {
                    Err(ValueError::InvalidOperationError)
                }
            },
            
            // Error for incompatible types
            _ => Err(ValueError::TypeError),
        }
    }

    /// Performs exponentiation with a custom default value if the operation would fail.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let str_val = Value::Str("abc".to_string());
    /// let frac = Value::Numerical(2.5);  // Fractional powers not allowed for strings
    /// let default = Value::Str("default".to_string());
    /// 
    /// assert_eq!(str_val.pow_or(&frac, &default), Value::Str("default".to_string()));
    /// ```
    pub fn pow_or<'a>(&self, rhs: &Value, default: &'a Value) -> Value {
        self.try_pow(rhs).unwrap_or(default.clone())
    }

    /// Performs exponentiation, panicking if it would fail.
    ///
    /// # Panics
    /// This method will panic if the operation cannot be performed with the given types.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let num1 = Value::Numerical(2.0);
    /// let num2 = Value::Numerical(3.0);
    /// 
    /// assert_eq!(num1.pow_unchecked(&num2), Value::Numerical(8.0));
    /// ```
    pub fn pow_unchecked(&self, rhs: &Value) -> Value {
        self.try_pow(rhs).unwrap()
    } 

    /// In-place exponentiation. Equivalent to the **= operator.
    pub fn pow_assign(&mut self, rhs: &Value) {
        *self = self.pow(rhs);
    }

    /// Tries to perform in-place exponentiation. Returns an error if the operation fails.
    pub fn try_pow_assign(&mut self, rhs: &Value) -> Result<(), ValueError> {
        match self.try_pow(rhs) {
            Ok(result) => {
                *self = result;
                Ok(())
            },
            Err(e) => Err(e),
        }
    }

    /// In-place exponentiation with a default value if the operation would fail.
    pub fn pow_assign_or(&mut self, rhs: &Value, default: &Value) {
        *self = self.pow_or(rhs, default);
    }

    /// In-place exponentiation that panics if the operation would fail.
    pub fn pow_assign_unchecked(&mut self, rhs: &Value) {
        *self = self.pow_unchecked(rhs);
    } 
    
    // --- INDEXING AND SLICING OPERATIONS ---
    
    /// Gets a value at a specific index or key.
    /// 
    /// This method will access elements based on the following rules:
    /// - List[Numerical] = Value at index (supports negative indices)
    /// - List[Boolean] = Value at index (True=1, False=0)
    /// - Dict[String] = Value with the given key
    /// - String[Numerical] = Single character String at index
    /// - String[Boolean] = Single character String at index (True=1, False=0)
    /// - Otherwise, returns None
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let list = Value::new(vec![10, 20, 30]);
    /// let idx = Value::Numerical(1.0);
    /// assert_eq!(list.get_index(&idx), Value::Numerical(20.0));
    /// 
    /// let dict = Value::new(std::collections::HashMap::from([("key", "value")]));
    /// let key = Value::Str("key".to_string());
    /// assert_eq!(dict.get_index(&key), Value::Str("value".to_string()));
    /// ```
    pub fn get_index(&self, index: &Value) -> Value {
        match self {
            Value::List(list) => {
                if let Value::Numerical(n) = index {
                    let idx = if *n < 0.0 {
                        // Handle negative indices like Python
                        (list.len() as f64 + *n) as usize
                    } else {
                        *n as usize
                    };
                    
                    if idx < list.len() {
                        list[idx].clone()
                    } else {
                        Value::None
                    }
                } else if let Value::Boolean(b) = index {
                    let idx = if *b { 1 } else { 0 };
                    if idx < list.len() {
                        list[idx].clone()
                    } else {
                        Value::None
                    }
                } else {
                    Value::None
                }
            },
            Value::Dict(dict) => {
                if let Value::Str(key) = index {
                    dict.get(key).cloned().unwrap_or(Value::None)
                } else {
                    Value::None
                }
            },
            Value::Str(s) => {
                if let Value::Numerical(n) = index {
                    let idx = if *n < 0.0 {
                        // Handle negative indices like Python
                        (s.len() as f64 + *n) as usize
                    } else {
                        *n as usize
                    };
                    
                    if idx < s.len() {
                        let char = s.chars().nth(idx).unwrap().to_string();
                        Value::Str(char)
                    } else {
                        Value::None
                    }
                } else if let Value::Boolean(b) = index {
                    let idx = if *b { 1 } else { 0 };
                    if idx < s.len() {
                        let char = s.chars().nth(idx).unwrap().to_string();
                        Value::Str(char)
                    } else {
                        Value::None
                    }
                } else {
                    Value::None
                }
            },
            _ => Value::None,
        }
    }

    /// Attempts to get a value at a specific index or key.
    ///
    /// This method returns a Result that is Ok(Value) on success or an Error on failure.
    /// It follows the same type rules as the `get_index` method.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let list = Value::new(vec![10, 20, 30]);
    /// let idx = Value::Numerical(1.0);
    /// assert_eq!(list.try_get_index(&idx).unwrap(), Value::Numerical(20.0));
    /// 
    /// // Index out of bounds
    /// let idx = Value::Numerical(5.0);
    /// assert!(list.try_get_index(&idx).is_err());
    /// ```
    pub fn try_get_index(&self, index: &Value) -> Result<Value, ValueError> {
        match self {
            Value::List(list) => {
                if let Value::Numerical(n) = index {
                    let idx = if *n < 0.0 {
                        (list.len() as f64 + *n) as usize
                    } else {
                        *n as usize
                    };
                    
                    if idx < list.len() {
                        Ok(list[idx].clone())
                    } else {
                        Err(ValueError::IndexOutOfBoundsError)
                    }
                } else if let Value::Boolean(b) = index {
                    let idx = if *b { 1 } else { 0 };
                    if idx < list.len() {
                        Ok(list[idx].clone())
                    } else {
                        Err(ValueError::IndexOutOfBoundsError)
                    }
                } else {
                    Err(ValueError::TypeError)
                }
            },
            Value::Dict(dict) => {
                if let Value::Str(key) = index {
                    dict.get(key)
                        .cloned()
                        .ok_or(ValueError::KeyNotFoundError)
                } else {
                    Err(ValueError::TypeError)
                }
            },
            Value::Str(s) => {
                if let Value::Numerical(n) = index {
                    let idx = if *n < 0.0 {
                        (s.len() as f64 + *n) as usize
                    } else {
                        *n as usize
                    };
                    
                    if idx < s.len() {
                        let char = s.chars().nth(idx).unwrap().to_string();
                        Ok(Value::Str(char))
                    } else {
                        Err(ValueError::IndexOutOfBoundsError)
                    }
                } else if let Value::Boolean(b) = index {
                    let idx = if *b { 1 } else { 0 };
                    if idx < s.len() {
                        let char = s.chars().nth(idx).unwrap().to_string();
                        Ok(Value::Str(char))
                    } else {
                        Err(ValueError::IndexOutOfBoundsError)
                    }
                } else {
                    Err(ValueError::TypeError)
                }
            },
            _ => Err(ValueError::TypeError),
        }
    }
    
    /// Gets a value at an index with a custom default if the operation would fail.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let list = Value::new(vec![10, 20, 30]);
    /// let idx = Value::Numerical(5.0);  // Out of bounds
    /// let default = Value::Numerical(-1.0);
    /// 
    /// assert_eq!(list.get_index_or(&idx, &default), Value::Numerical(-1.0));
    /// ```
    pub fn get_index_or<'a>(&self, index: &Value, default: &'a Value) -> Value {
        self.try_get_index(index).unwrap_or(default.clone())
    }

    /// Gets a value at an index, panicking if the operation would fail.
    ///
    /// # Panics
    /// This method will panic if the index is invalid or out of bounds.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let list = Value::new(vec![10, 20, 30]);
    /// let idx = Value::Numerical(1.0);
    /// 
    /// assert_eq!(list.get_index_unchecked(&idx), Value::Numerical(20.0));
    /// ```
    pub fn get_index_unchecked(&self, index: &Value) -> Value {
        self.try_get_index(index).unwrap()
    }

    // --- SLICE OPERATIONS ---

    /// Gets a slice of a container using a Range.
    ///
    /// This method performs slicing based on the following rules:
    /// - List[Range] = List containing elements in the range
    /// - String[Range] = String containing characters in the range
    /// - Dict[Range] = Dict containing entries in the range (after converting to key-sorted entries)
    /// - Otherwise, returns None
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let list = Value::new(vec![10, 20, 30, 40, 50]);
    /// let slice = list.slice(1..4);
    /// assert_eq!(slice, Value::new(vec![20, 30, 40]));
    /// 
    /// let str_val = Value::Str("abcdef".to_string());
    /// let slice = str_val.slice(1..4);
    /// assert_eq!(slice, Value::Str("bcd".to_string()));
    /// ```
    pub fn slice(&self, range: Range<usize>) -> Value {
        self.try_slice(range).unwrap_or(Value::None)
    }

    /// Attempts to get a slice of a container using a Range.
    ///
    /// This method returns a Result that is Ok(Value) on success or an Error on failure.
    /// It follows the same type rules as the `slice` method.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let list = Value::new(vec![10, 20, 30, 40, 50]);
    /// let slice = list.try_slice(1..4).unwrap();
    /// assert_eq!(slice, Value::new(vec![20, 30, 40]));
    /// ```
    pub fn try_slice(&self, range: Range<usize>) -> Result<Value, ValueError> {
        match self {
            Value::List(list) => {
                let start = range.start.min(list.len());
                let end = range.end.min(list.len());
                
                if start <= end {
                    Ok(Value::List(list[start..end].to_vec()))
                } else {
                    Ok(Value::List(Vec::new()))
                }
            },
            Value::Str(s) => {
                let chars: Vec<char> = s.chars().collect();
                let start = range.start.min(chars.len());
                let end = range.end.min(chars.len());
                
                if start <= end {
                    let slice: String = chars[start..end].iter().collect();
                    Ok(Value::Str(slice))
                } else {
                    Ok(Value::Str(String::new()))
                }
            },
            Value::Dict(dict) => {
                // Convert to entries, sort by key
                let mut entries: Vec<(String, Value)> = dict.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                
                entries.sort_by(|(a, _), (b, _)| a.cmp(b));
                
                let start = range.start.min(entries.len());
                let end = range.end.min(entries.len());
                
                if start <= end {
                    let mut new_dict = HashMap::new();
                    for (key, value) in &entries[start..end] {
                        new_dict.insert(key.clone(), value.clone());
                    }
                    Ok(Value::Dict(new_dict))
                } else {
                    Ok(Value::Dict(HashMap::new()))
                }
            },
            _ => Err(ValueError::TypeError),
        }
    }

    /// Gets a slice with a custom default if the operation would fail.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let num = Value::Numerical(42.0);  // Cannot slice a number
    /// let default = Value::List(Vec::new());
    /// 
    /// assert_eq!(num.slice_or(1..4, &default), Value::List(Vec::new()));
    /// ```
    pub fn slice_or<'a>(&self, range: Range<usize>, default: &'a Value) -> Value {
        self.try_slice(range).unwrap_or(default.clone())
    }

    /// Gets a slice, panicking if the operation would fail.
    ///
    /// # Panics
    /// This method will panic if the value is not sliceable.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let list = Value::new(vec![10, 20, 30, 40, 50]);
    /// 
    /// assert_eq!(list.slice_unchecked(1..4), Value::new(vec![20, 30, 40]));
    /// ```
    pub fn slice_unchecked(&self, range: Range<usize>) -> Value {
        self.try_slice(range).unwrap()
    }

    // --- APPEND OPERATIONS ---

    /// Appends a value to a list using the << operator.
    ///
    /// This method follows these rules:
    /// - List << Value: Append value to list
    /// - Dict << (String, Value): Insert key-value pair
    /// - Otherwise: Error
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let list = Value::new(vec![10, 20]);
    /// let val = Value::Numerical(30.0);
    /// assert_eq!(list.append(&val), Value::new(vec![10, 20, 30]));
    /// ```
    pub fn append(&self, value: &Value) -> Value {
        self.try_append(value).unwrap_or(Value::None)
    }

    /// Attempts to append a value using the << operator.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let list = Value::new(vec![10, 20]);
    /// let val = Value::Numerical(30.0);
    /// assert_eq!(list.try_append(&val).unwrap(), Value::new(vec![10, 20, 30]));
    /// ```
    pub fn try_append(&self, value: &Value) -> Result<Value, ValueError> {
        match self {
            Value::List(list) => {
                let mut new_list = list.clone();
                new_list.push(value.clone());
                Ok(Value::List(new_list))
            },
            
            Value::Dict(dict) => {
                // Extract key-value pair from tuple-like structure
                if let Value::List(tuple) = value {
                    if tuple.len() == 2 {
                        if let Value::Str(key) = &tuple[0] {
                            let mut new_dict = dict.clone();
                            new_dict.insert(key.clone(), tuple[1].clone());
                            return Ok(Value::Dict(new_dict));
                        }
                    }
                }
                Err(ValueError::TypeError)
            },
            
            // Error for incompatible types
            _ => Err(ValueError::TypeError),
        }
    }

    /// Appends a value with a custom default if the operation would fail.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let num = Value::Numerical(42.0);  // Cannot append to a number
    /// let val = Value::Numerical(10.0);
    /// let default = Value::List(Vec::new());
    /// 
    /// assert_eq!(num.append_or(&val, &default), Value::List(Vec::new()));
    /// ```
    pub fn append_or<'a>(&self, value: &Value, default: &'a Value) -> Value {
        self.try_append(value).unwrap_or(default.clone())
    }

    /// Appends a value, panicking if the operation would fail.
    ///
    /// # Panics
    /// This method will panic if the value is not appendable.
    ///
    /// # Example
    /// ```
    /// use akari::Value;
    /// 
    /// let list = Value::new(vec![10, 20]);
    /// let val = Value::Numerical(30.0);
    /// 
    /// assert_eq!(list.append_unchecked(&val), Value::new(vec![10, 20, 30]));
    /// ```
    pub fn append_unchecked(&self, value: &Value) -> Value {
        self.try_append(value).unwrap()
    } 

    /// In-place append. Equivalent to the <<= operator.
    pub fn append_assign(&mut self, rhs: &Value) {
        *self = self.append(rhs);
    }

    /// Tries to perform in-place append. Returns an error if the operation fails.
    pub fn try_append_assign(&mut self, rhs: &Value) -> Result<(), ValueError> {
        match self.try_append(rhs) {
            Ok(result) => {
                *self = result;
                Ok(())
            },
            Err(e) => Err(e),
        }
    }

    /// In-place append with a default value if the operation would fail.
    pub fn append_assign_or(&mut self, rhs: &Value, default: &Value) {
        *self = self.append_or(rhs, default);
    }

    /// In-place append that panics if the operation would fail.
    pub fn append_assign_unchecked(&mut self, rhs: &Value) {
        *self = self.append_unchecked(rhs);
    } 
}

// Operator trait implementations that delegate to the methods

impl core::ops::Add for Value {
    type Output = Value;
    
    fn add(self, rhs: Self) -> Self::Output {
        (&self).add(&rhs)
    }
}

impl core::ops::Add for &Value {
    type Output = Value;
    
    fn add(self, rhs: Self) -> Self::Output {
        self.add(rhs)
    }
}

impl core::ops::Sub for Value {
    type Output = Value;
    
    fn sub(self, rhs: Self) -> Self::Output {
        (&self).sub(&rhs)
    }
}

impl core::ops::Sub for &Value {
    type Output = Value;
    
    fn sub(self, rhs: Self) -> Self::Output {
        self.sub(rhs)
    }
}

impl core::ops::Mul for Value {
    type Output = Value;
    
    fn mul(self, rhs: Self) -> Self::Output {
        (&self).mul(&rhs)
    }
}

impl core::ops::Mul for &Value {
    type Output = Value;
    
    fn mul(self, rhs: Self) -> Self::Output {
        self.mul(rhs)
    }
}

impl core::ops::Div for Value {
    type Output = Value;
    
    fn div(self, rhs: Self) -> Self::Output {
        (&self).div(&rhs)
    }
}

impl core::ops::Div for &Value {
    type Output = Value;
    
    fn div(self, rhs: Self) -> Self::Output {
        self.div(rhs)
    }
}

impl core::ops::Rem for Value {
    type Output = Value;
    
    fn rem(self, rhs: Self) -> Self::Output {
        (&self).modulo(&rhs)
    }
}

impl core::ops::Rem for &Value {
    type Output = Value;
    
    fn rem(self, rhs: Self) -> Self::Output {
        self.modulo(rhs)
    }
} 

impl core::ops::Shl for Value {
    type Output = Value;
    
    fn shl(self, rhs: Self) -> Self::Output {
        self.append(&rhs)
    }
}

impl core::ops::Shl for &Value {
    type Output = Value;
    
    fn shl(self, rhs: Self) -> Self::Output {
        self.append(rhs)
    }
} 

// Use ^ to mean Pow 

impl core::ops::BitXor for Value {
    type Output = Value;
    
    fn bitxor(self, rhs: Self) -> Self::Output {
        self.pow(&rhs)
    }
}

impl core::ops::BitXor for &Value {
    type Output = Value;
    
    fn bitxor(self, rhs: Self) -> Self::Output {
        self.pow(rhs)
    }
} 

// Index trait implementation for [] operator

impl core::ops::Index<&Value> for Value {
    type Output = Value;
    
    fn index(&self, index: &Value) -> &Self::Output {
        static NONE: Value = Value::None;
        match self.try_get_index(index) {
            Ok(value) => Box::leak(Box::new(value)),
            Err(_) => &NONE,
        }
    }
} 

impl Index<Range<usize>> for Value {
    type Output = Value;
    
    fn index(&self, index: Range<usize>) -> &Self::Output {
        Box::leak(Box::new(self.slice(index)))
    }
}

impl Index<RangeFrom<usize>> for Value {
    type Output = Value;
    
    fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
        let range = index.start..usize::MAX;
        Box::leak(Box::new(self.slice(range)))
    }
}

impl Index<RangeTo<usize>> for Value {
    type Output = Value;
    
    fn index(&self, index: RangeTo<usize>) -> &Self::Output {
        let range = 0..index.end;
        Box::leak(Box::new(self.slice(range)))
    }
}

impl Index<RangeFull> for Value {
    type Output = Value;
    
    fn index(&self, _: RangeFull) -> &Self::Output {
        let range = 0..usize::MAX;
        Box::leak(Box::new(self.slice(range)))
    }
} 

#[derive(Debug, Clone)]
pub enum ValueIndex {
    Numeric(f64),
    Boolean(bool),
    String(String),
    Range(Range<usize>),
    // Add other range types as needed
}

impl From<f64> for ValueIndex {
    fn from(v: f64) -> Self {
        ValueIndex::Numeric(v)
    }
}

impl From<bool> for ValueIndex {
    fn from(v: bool) -> Self {
        ValueIndex::Boolean(v)
    }
}

impl From<&str> for ValueIndex {
    fn from(v: &str) -> Self {
        ValueIndex::String(v.to_string())
    }
}

impl From<Range<usize>> for ValueIndex {
    fn from(v: Range<usize>) -> Self {
        ValueIndex::Range(v)
    }
}

impl Index<ValueIndex> for Value {
    type Output = Value;
    
    fn index(&self, index: ValueIndex) -> &Self::Output {
        match index {
            ValueIndex::Numeric(n) => &self[&Value::Numerical(n)],
            ValueIndex::Boolean(b) => &self[&Value::Boolean(b)],
            ValueIndex::String(s) => &self[&Value::Str(s)],
            ValueIndex::Range(r) => &self[r],
        }
    }
}

impl Index<usize> for Value {
    type Output = Value;
    
    fn index(&self, index: usize) -> &Self::Output {
        &self[&Value::Numerical(index as f64)]
    }
}

impl Index<&str> for Value {
    type Output = Value;
    
    fn index(&self, index: &str) -> &Self::Output {
        &self[&Value::Str(index.to_string())]
    }
}

// Comparison operator traits

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.equals(other) {
            Some(std::cmp::Ordering::Equal)
        } else if self.less_than(other) {
            Some(std::cmp::Ordering::Less)
        } else if self.greater_than(other) {
            Some(std::cmp::Ordering::Greater)
        } else {
            None
        }
    }
}

// Contains trait for `in` operator

// pub trait Contains {
//     fn contains_value(&self, other: &Self) -> bool;
// }

// impl Contains for Value {
//     fn contains_value(&self, other: &Self) -> bool {
//         self.contains(other)
//     }
// }

// Logical operations for && and ||

impl core::ops::BitAnd for Value {
    type Output = Value;
    
    fn bitand(self, rhs: Self) -> Self::Output {
        Value::Boolean(self.boolean() && rhs.boolean())
    }
}

impl core::ops::BitAnd for &Value {
    type Output = Value;
    
    fn bitand(self, rhs: Self) -> Self::Output {
        Value::Boolean(self.boolean() && rhs.boolean())
    }
}

impl core::ops::BitOr for Value {
    type Output = Value;
    
    fn bitor(self, rhs: Self) -> Self::Output {
        Value::Boolean(self.boolean() || rhs.boolean())
    }
}

impl core::ops::BitOr for &Value {
    type Output = Value;
    
    fn bitor(self, rhs: Self) -> Self::Output {
        Value::Boolean(self.boolean() || rhs.boolean())
    }
}

impl core::ops::Not for Value {
    type Output = Value;
    
    fn not(self) -> Self::Output {
        Value::Boolean(!self.boolean())
    }
}

impl core::ops::Not for &Value {
    type Output = Value;
    
    fn not(self) -> Self::Output {
        Value::Boolean(!self.boolean())
    }
}

// Implement additional comparison operators

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

// Additional arithmetic implementations for mixed types

// Value + &Value
impl core::ops::Add<&Value> for Value {
    type Output = Value;
    
    fn add(self, rhs: &Value) -> Self::Output {
        (&self).add(rhs)
    }
}

// &Value + Value
impl core::ops::Add<Value> for &Value {
    type Output = Value;
    
    fn add(self, rhs: Value) -> Self::Output {
        self.add(&rhs)
    }
}

// --- COMPOUND ASSIGNMENT OPERATORS ---

// AddAssign implementation for +=
impl core::ops::AddAssign for Value {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.add(&rhs);
    }
}

impl core::ops::AddAssign<&Value> for Value {
    fn add_assign(&mut self, rhs: &Value) {
        *self = self.add(rhs);
    }
}

// SubAssign implementation for -=
impl core::ops::SubAssign for Value {
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.sub(&rhs);
    }
}

impl core::ops::SubAssign<&Value> for Value {
    fn sub_assign(&mut self, rhs: &Value) {
        *self = self.sub(rhs);
    }
}

// MulAssign implementation for *=
impl core::ops::MulAssign for Value {
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.mul(&rhs);
    }
}

impl core::ops::MulAssign<&Value> for Value {
    fn mul_assign(&mut self, rhs: &Value) {
        *self = self.mul(rhs);
    }
}

// DivAssign implementation for /=
impl core::ops::DivAssign for Value {
    fn div_assign(&mut self, rhs: Self) {
        *self = self.div(&rhs);
    }
}

impl core::ops::DivAssign<&Value> for Value {
    fn div_assign(&mut self, rhs: &Value) {
        *self = self.div(rhs);
    }
}

// RemAssign implementation for %=
impl core::ops::RemAssign for Value {
    fn rem_assign(&mut self, rhs: Self) {
        *self = self.modulo(&rhs);
    }
}

impl core::ops::RemAssign<&Value> for Value {
    fn rem_assign(&mut self, rhs: &Value) {
        *self = self.modulo(rhs);
    }
} 

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_numerical_add() {
        let a = Value::Numerical(5.0);
        let b = Value::Numerical(3.0);
        assert_eq!(a.add(&b), Value::Numerical(8.0));
    }
    
    #[test]
    fn test_string_add() {
        let a = Value::Str("Hello, ".to_string());
        let b = Value::Str("world!".to_string());
        assert_eq!(a.add(&b), Value::Str("Hello, world!".to_string()));
    }
    
    #[test]
    fn test_list_add() {
        let a = Value::List(vec![Value::Numerical(1.0), Value::Numerical(2.0)]);
        let b = Value::List(vec![Value::Numerical(3.0), Value::Numerical(4.0)]);
        assert_eq!(
            a.add(&b), 
            Value::List(vec![
                Value::Numerical(1.0), 
                Value::Numerical(2.0),
                Value::Numerical(3.0), 
                Value::Numerical(4.0)
            ])
        );
    }
    
    #[test]
    fn test_dictionary_add() {
        let mut map1 = HashMap::new();
        map1.insert("a".to_string(), Value::Numerical(1.0));
        let mut map2 = HashMap::new();
        map2.insert("b".to_string(), Value::Numerical(2.0));
        map2.insert("a".to_string(), Value::Numerical(3.0));  // This should overwrite a
        
        let a = Value::Dict(map1);
        let b = Value::Dict(map2);
        
        if let Value::Dict(result) = a.add(&b) {
            assert_eq!(result.get("a").unwrap(), &Value::Numerical(3.0));
            assert_eq!(result.get("b").unwrap(), &Value::Numerical(2.0));
        } else {
            panic!("Expected Dict result");
        }
    }
    
    #[test]
    fn test_list_multiply() {
        let list = Value::List(vec![Value::Numerical(1.0), Value::Numerical(2.0)]);
        let count = Value::Numerical(3.0);
        
        assert_eq!(
            list.mul(&count), 
            Value::List(vec![
                Value::Numerical(1.0), Value::Numerical(2.0),
                Value::Numerical(1.0), Value::Numerical(2.0),
                Value::Numerical(1.0), Value::Numerical(2.0)
            ])
        );
    }
    
    #[test]
    fn test_string_multiply() {
        let string = Value::Str("abc".to_string());
        let count = Value::Numerical(3.0);
        
        assert_eq!(string.mul(&count), Value::Str("abcabcabc".to_string()));
    }
    
    #[test]
    fn test_list_division() {
        let list = Value::List((0..6).map(|i| Value::Numerical(i as f64)).collect());
        let chunks = Value::Numerical(3.0);
        
        if let Value::List(result) = list.div(&chunks) {
            assert_eq!(result.len(), 3);
            if let Value::List(chunk1) = &result[0] {
                assert_eq!(chunk1.len(), 2);
                assert_eq!(chunk1[0], Value::Numerical(0.0));
                assert_eq!(chunk1[1], Value::Numerical(1.0));
            }
        } else {
            panic!("Expected List result");
        }
    }
    
    #[test]
    fn test_indexing() {
        let list = Value::List(vec![
            Value::Numerical(10.0),
            Value::Numerical(20.0),
            Value::Numerical(30.0)
        ]);
        
        assert_eq!(list.get_index(&Value::Numerical(1.0)), Value::Numerical(20.0));
        assert_eq!(list.get_index(&Value::Numerical(-1.0)), Value::Numerical(30.0));
        
        let string = Value::Str("hello".to_string());
        assert_eq!(string.get_index(&Value::Numerical(1.0)), Value::Str("e".to_string()));
        
        let mut dict = HashMap::new();
        dict.insert("key".to_string(), Value::Numerical(42.0));
        let dict_val = Value::Dict(dict);
        
        assert_eq!(dict_val.get_index(&Value::Str("key".to_string())), Value::Numerical(42.0));
    }
    
    #[test]
    fn test_slicing() {
        let list = Value::List((0..5).map(|i| Value::Numerical(i as f64)).collect());
        
        if let Value::List(result) = list.slice(1..4) {
            assert_eq!(result.len(), 3);
            assert_eq!(result[0], Value::Numerical(1.0));
            assert_eq!(result[1], Value::Numerical(2.0));
            assert_eq!(result[2], Value::Numerical(3.0));
        } else {
            panic!("Expected List result");
        }
        
        let string = Value::Str("hello world".to_string());
        assert_eq!(string.slice(0..5), Value::Str("hello".to_string()));
    }
    
    #[test]
    fn test_equality() {
        assert!(Value::Numerical(42.0).equals(&Value::Numerical(42.0)));
        assert!(!Value::Numerical(42.0).equals(&Value::Numerical(43.0)));
        assert!(!Value::Numerical(42.0).equals(&Value::Str("42".to_string())));
        
        let list1 = Value::List(vec![Value::Numerical(1.0), Value::Numerical(2.0)]);
        let list2 = Value::List(vec![Value::Numerical(1.0), Value::Numerical(2.0)]);
        let list3 = Value::List(vec![Value::Numerical(1.0), Value::Numerical(3.0)]);
        
        assert!(list1.equals(&list2));
        assert!(!list1.equals(&list3));
    }
    
    #[test]
    fn test_comparison() {
        assert!(Value::Numerical(10.0).greater_than(&Value::Numerical(5.0)));
        assert!(Value::Numerical(5.0).less_than(&Value::Numerical(10.0)));
        // assert!(Value::Str("ba".to_string()).greater_than(&Value::Str("a".to_string())));
        
        let list1 = Value::List(vec![Value::Numerical(1.0), Value::Numerical(2.0)]);
        let list2 = Value::List(vec![Value::Numerical(1.0), Value::Numerical(3.0), Value::Numerical(1.0), Value::Numerical(2.0)]);
        
        assert!(list1.less_than(&list2));
        assert!(list2.greater_than(&list1));
    }
    
    #[test]
    fn test_contains() {
        let string = Value::Str("hello world".to_string());
        assert!(string.contains(&Value::Str("world".to_string())));
        assert!(!string.contains(&Value::Str("goodbye".to_string())));
        
        let list = Value::List(vec![
            Value::Numerical(10.0),
            Value::Numerical(20.0),
            Value::Numerical(30.0)
        ]);
        
        assert!(list.contains(&Value::Numerical(20.0)));
        assert!(!list.contains(&Value::Numerical(40.0)));
        
        let mut dict = HashMap::new();
        dict.insert("key1".to_string(), Value::Numerical(10.0));
        dict.insert("key2".to_string(), Value::Numerical(20.0));
        let dict_val = Value::Dict(dict);
        
        assert!(dict_val.contains(&Value::Str("key1".to_string())));
        assert!(!dict_val.contains(&Value::Str("key3".to_string())));
    }
    
    #[test]
    fn test_append() {
        let list = Value::List(vec![Value::Numerical(10.0), Value::Numerical(20.0)]);
        let val = Value::Numerical(30.0);
        
        assert_eq!(
            list.append(&val),
            Value::List(vec![
                Value::Numerical(10.0),
                Value::Numerical(20.0),
                Value::Numerical(30.0)
            ])
        );
        
        let mut dict = HashMap::new();
        dict.insert("key1".to_string(), Value::Numerical(10.0));
        let dict_val = Value::Dict(dict);
        
        let tuple = Value::List(vec![
            Value::Str("key2".to_string()),
            Value::Numerical(20.0)
        ]);
        
        if let Value::Dict(result) = dict_val.append(&tuple) {
            assert_eq!(result.len(), 2);
            assert_eq!(result.get("key1").unwrap(), &Value::Numerical(10.0));
            assert_eq!(result.get("key2").unwrap(), &Value::Numerical(20.0));
        } else {
            panic!("Expected Dict result");
        }
    }
    
    #[test]
    fn test_error_handling() {
        let num = Value::Numerical(42.0);
        let str_val = Value::Str("hello".to_string());
        
        // This should return None with the basic add method
        assert_eq!(num.add(&str_val), Value::None);
        
        // This should return an error with try_add
        assert!(num.try_add(&str_val).is_err());
        
        // This should return a custom default with add_or
        let default = Value::Numerical(0.0);
        assert_eq!(num.add_or(&str_val, &default), Value::Numerical(0.0));
        
        // This would panic with add_unchecked
        // num.add_unchecked(&str_val);  // Uncomment to test panic
    }
}