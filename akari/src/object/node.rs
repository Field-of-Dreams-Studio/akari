use std::collections::HashMap;
use std::fmt;
use super::value::Value;

/// Node structure with private fields and lazy HashMap optimization
///
/// Design principles:
/// - Small dicts (≤10 keys): Linear search through Vec (cache-friendly)
/// - Large dicts (>10 keys): Lazy-build HashMap for O(1) lookup
/// - Preserves insertion order via parallel vectors
/// - Private fields enforce invariants
#[derive(Debug, Clone)]
pub struct Node {
    /// The node's intrinsic value (must be primitive, not a graph type)
    value: Value,

    /// Child values (can be any type)
    children: Vec<Value>,

    /// Keys for dictionary-like access (parallel to children)
    keys: Vec<Value>,

    /// Lazy-initialized hash index for fast lookups (>10 keys)
    /// Maps key → index in children/keys vectors
    key_index: Option<HashMap<Value, usize>>,
}

impl Node {
    /// Threshold for switching from linear search to HashMap
    const HASH_THRESHOLD: usize = 10;

    /// Create a new node with a primitive value
    pub fn new(value: Value) -> Self {
        Node {
            value,
            children: Vec::new(),
            keys: Vec::new(),
            key_index: None,
        }
    }

    /// Create an empty container node (value = None)
    pub fn new_container() -> Self {
        Node {
            value: Value::None,
            children: Vec::new(),
            keys: Vec::new(),
            key_index: None,
        }
    }

    /// Create a node from key-value pairs (dictionary mode)
    pub fn from_pairs(pairs: Vec<(Value, Value)>) -> Self {
        let mut node = Node::new_container();
        for (key, value) in pairs {
            node.insert(key, value);
        }
        node
    }

    /// Create a node from values only (array mode)
    pub fn from_array(values: Vec<Value>) -> Self {
        Node {
            value: Value::None,
            children: values,
            keys: Vec::new(),
            key_index: None,
        }
    }

    // === Accessors ===

    /// Get the node's intrinsic value
    pub fn value(&self) -> &Value {
        &self.value
    }

    /// Set the node's intrinsic value
    pub fn set_value(&mut self, value: Value) {
        self.value = value;
    }

    /// Get number of children
    pub fn len(&self) -> usize {
        self.children.len()
    }

    /// Check if node is empty (no children)
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Check if this is an array (no keys) or dictionary (has keys)
    pub fn is_array(&self) -> bool {
        self.keys.is_empty() && !self.children.is_empty()
    }

    /// Check if this is a dictionary
    pub fn is_dict(&self) -> bool {
        !self.keys.is_empty()
    }

    // === Dictionary Operations ===

    /// Get value by key with automatic HashMap optimization
    pub fn get(&mut self, key: &Value) -> Option<&Value> {
        if self.keys.is_empty() {
            return None;
        }

        // Build index if dict is large and index doesn't exist
        if self.keys.len() > Self::HASH_THRESHOLD && self.key_index.is_none() {
            self.build_index();
        }

        if let Some(index) = &self.key_index {
            // O(1) hash lookup
            index.get(key).map(|&i| &self.children[i])
        } else {
            // O(n) linear search for small dicts
            self.keys.iter()
                .position(|k| k == key)
                .map(|i| &self.children[i])
        }
    }

    /// Get value by key (immutable version, always uses linear search or existing index)
    pub fn get_immutable(&self, key: &Value) -> Option<&Value> {
        if self.keys.is_empty() {
            return None;
        }

        if let Some(index) = &self.key_index {
            // Use existing index if available
            index.get(key).map(|&i| &self.children[i])
        } else {
            // Linear search
            self.keys.iter()
                .position(|k| k == key)
                .map(|i| &self.children[i])
        }
    }

    /// Insert or update a key-value pair
    pub fn insert(&mut self, key: Value, value: Value) {
        // Check if key already exists
        if let Some(pos) = self.keys.iter().position(|k| k == &key) {
            // Update existing
            self.children[pos] = value;
            // Update index if it exists
            if let Some(ref mut index) = self.key_index {
                index.insert(key, pos);
            }
        } else {
            // Insert new
            let new_index = self.children.len();
            self.children.push(value);
            self.keys.push(key.clone());

            // Update index if it exists
            if let Some(ref mut index) = self.key_index {
                index.insert(key, new_index);
            }
        }
    }

    /// Remove a key-value pair, returns the value if found
    pub fn remove(&mut self, key: &Value) -> Option<Value> {
        if let Some(pos) = self.keys.iter().position(|k| k == key) {
            self.keys.remove(pos);
            let value = self.children.remove(pos);

            // Invalidate index (indices have changed)
            self.key_index = None;

            Some(value)
        } else {
            None
        }
    }

    /// Check if key exists
    pub fn contains_key(&mut self, key: &Value) -> bool {
        self.get(key).is_some()
    }

    // === Array Operations ===

    /// Get value by index
    pub fn get_index(&self, index: usize) -> Option<&Value> {
        self.children.get(index)
    }

    /// Get mutable value by index
    pub fn get_index_mut(&mut self, index: usize) -> Option<&mut Value> {
        self.children.get_mut(index)
    }

    /// Push value to end (array mode)
    pub fn push(&mut self, value: Value) {
        self.children.push(value);
        // Note: keys remain empty for array mode
    }

    /// Pop value from end
    pub fn pop(&mut self) -> Option<Value> {
        let value = self.children.pop()?;

        // Also pop key if in dict mode
        if !self.keys.is_empty() {
            self.keys.pop();
            // Invalidate index
            self.key_index = None;
        }

        Some(value)
    }

    // === Iteration ===

    /// Iterate over children (values only)
    pub fn iter_values(&self) -> impl Iterator<Item = &Value> {
        self.children.iter()
    }

    /// Iterate over key-value pairs (if dict mode)
    pub fn iter_pairs(&self) -> impl Iterator<Item = (&Value, &Value)> {
        self.keys.iter().zip(self.children.iter())
    }

    /// Get all keys
    pub fn keys(&self) -> &[Value] {
        &self.keys
    }

    /// Get all values
    pub fn values(&self) -> &[Value] {
        &self.children
    }

    // === Internal Methods ===

    /// Build the hash index for fast lookups
    fn build_index(&mut self) {
        let mut index = HashMap::with_capacity(self.keys.len());
        for (i, key) in self.keys.iter().enumerate() {
            index.insert(key.clone(), i);
        }
        self.key_index = Some(index);
    }

    /// Force rebuild the hash index (useful after bulk operations)
    pub fn rebuild_index(&mut self) {
        if !self.keys.is_empty() {
            self.build_index();
        }
    }

    /// Clear all children and keys
    pub fn clear(&mut self) {
        self.children.clear();
        self.keys.clear();
        self.key_index = None;
    }

    /// Validate internal invariants (for testing)
    #[cfg(test)]
    pub fn validate(&self) -> Result<(), String> {
        // Check parallel vectors have same length (if dict mode)
        if !self.keys.is_empty() && self.keys.len() != self.children.len() {
            return Err(format!(
                "Keys and children length mismatch: {} vs {}",
                self.keys.len(),
                self.children.len()
            ));
        }

        // Check index consistency if it exists
        if let Some(ref index) = self.key_index {
            if index.len() != self.keys.len() {
                return Err(format!(
                    "Index size mismatch: {} vs {}",
                    index.len(),
                    self.keys.len()
                ));
            }

            // Verify each index points to correct key
            for (key, &idx) in index.iter() {
                if idx >= self.keys.len() {
                    return Err(format!("Index out of bounds: {}", idx));
                }
                if &self.keys[idx] != key {
                    return Err(format!("Index key mismatch at position {}", idx));
                }
            }
        }

        Ok(())
    }
}

// === Display Implementation ===

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_array() {
            // Array mode: [val1, val2, val3]
            write!(f, "[")?;
            for (i, val) in self.children.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", val)?;
            }
            write!(f, "]")
        } else if self.is_dict() {
            // Dict mode: {key1: val1, key2: val2}
            write!(f, "{{")?;
            for (i, (key, val)) in self.iter_pairs().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}: {}", key, val)?;
            }
            write!(f, "}}")
        } else {
            // Empty container or just value
            write!(f, "Node({})", self.value)
        }
    }
}

// === Tests ===

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_container() {
        let node = Node::new_container();
        assert_eq!(node.len(), 0);
        assert!(node.is_empty());
        assert_eq!(node.value(), &Value::None);
    }

    #[test]
    fn test_new_with_value() {
        let node = Node::new(Value::Numerical(42.0));
        assert_eq!(node.value(), &Value::Numerical(42.0));
        assert!(node.is_empty());
    }

    #[test]
    fn test_array_mode() {
        let mut node = Node::new_container();
        node.push(Value::Numerical(1.0));
        node.push(Value::Numerical(2.0));
        node.push(Value::Numerical(3.0));

        assert_eq!(node.len(), 3);
        assert!(node.is_array());
        assert!(!node.is_dict());
        assert_eq!(node.get_index(0), Some(&Value::Numerical(1.0)));
        assert_eq!(node.get_index(1), Some(&Value::Numerical(2.0)));
        assert_eq!(node.get_index(2), Some(&Value::Numerical(3.0)));
    }

    #[test]
    fn test_dict_mode_small() {
        let mut node = Node::new_container();

        // Small dict - should use linear search
        node.insert(Value::Str("name".to_string()), Value::Str("Alice".to_string()));
        node.insert(Value::Str("age".to_string()), Value::Numerical(30.0));

        assert_eq!(node.len(), 2);
        assert!(!node.is_array());
        assert!(node.is_dict());

        assert_eq!(
            node.get(&Value::Str("name".to_string())),
            Some(&Value::Str("Alice".to_string()))
        );
        assert_eq!(
            node.get(&Value::Str("age".to_string())),
            Some(&Value::Numerical(30.0))
        );

        // Index should not be built yet
        assert!(node.key_index.is_none());
        node.validate().unwrap();
    }

    #[test]
    fn test_dict_mode_large() {
        let mut node = Node::new_container();

        // Large dict - should auto-build HashMap
        for i in 0..20 {
            node.insert(
                Value::Str(format!("key{}", i)),
                Value::Numerical(i as f64)
            );
        }

        assert_eq!(node.len(), 20);

        // First get should trigger index build
        assert!(node.key_index.is_none());
        assert_eq!(
            node.get(&Value::Str("key5".to_string())),
            Some(&Value::Numerical(5.0))
        );

        // Index should now be built
        assert!(node.key_index.is_some());

        // Subsequent gets should use index
        assert_eq!(
            node.get(&Value::Str("key15".to_string())),
            Some(&Value::Numerical(15.0))
        );

        node.validate().unwrap();
    }

    #[test]
    fn test_insert_update() {
        let mut node = Node::new_container();

        node.insert(Value::Str("x".to_string()), Value::Numerical(1.0));
        assert_eq!(node.len(), 1);

        // Update existing key
        node.insert(Value::Str("x".to_string()), Value::Numerical(2.0));
        assert_eq!(node.len(), 1); // Should not grow
        assert_eq!(
            node.get(&Value::Str("x".to_string())),
            Some(&Value::Numerical(2.0))
        );

        node.validate().unwrap();
    }

    #[test]
    fn test_remove() {
        let mut node = Node::new_container();

        node.insert(Value::Str("a".to_string()), Value::Numerical(1.0));
        node.insert(Value::Str("b".to_string()), Value::Numerical(2.0));
        node.insert(Value::Str("c".to_string()), Value::Numerical(3.0));

        let removed = node.remove(&Value::Str("b".to_string()));
        assert_eq!(removed, Some(Value::Numerical(2.0)));
        assert_eq!(node.len(), 2);

        assert_eq!(
            node.get(&Value::Str("a".to_string())),
            Some(&Value::Numerical(1.0))
        );
        assert_eq!(node.get(&Value::Str("b".to_string())), None);
        assert_eq!(
            node.get(&Value::Str("c".to_string())),
            Some(&Value::Numerical(3.0))
        );

        node.validate().unwrap();
    }

    #[test]
    fn test_pop() {
        let mut node = Node::new_container();
        node.push(Value::Numerical(1.0));
        node.push(Value::Numerical(2.0));

        assert_eq!(node.pop(), Some(Value::Numerical(2.0)));
        assert_eq!(node.len(), 1);
        assert_eq!(node.pop(), Some(Value::Numerical(1.0)));
        assert_eq!(node.len(), 0);
        assert_eq!(node.pop(), None);
    }

    #[test]
    fn test_from_pairs() {
        let node = Node::from_pairs(vec![
            (Value::Str("name".to_string()), Value::Str("Bob".to_string())),
            (Value::Str("age".to_string()), Value::Numerical(25.0)),
        ]);

        assert_eq!(node.len(), 2);
        assert!(node.is_dict());
        node.validate().unwrap();
    }

    #[test]
    fn test_from_array() {
        let node = Node::from_array(vec![
            Value::Numerical(1.0),
            Value::Numerical(2.0),
            Value::Numerical(3.0),
        ]);

        assert_eq!(node.len(), 3);
        assert!(node.is_array());
        node.validate().unwrap();
    }

    #[test]
    fn test_clear() {
        let mut node = Node::new_container();
        node.push(Value::Numerical(1.0));
        node.push(Value::Numerical(2.0));

        node.clear();
        assert_eq!(node.len(), 0);
        assert!(node.is_empty());
        node.validate().unwrap();
    }
}
