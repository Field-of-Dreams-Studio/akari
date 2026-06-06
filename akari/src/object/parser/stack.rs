//! # Stack-Based Resumable JSON Parser
//!
//! ## Why Stack-Based Parser Is Necessary (Despite Potential Overhead)
//!
//! **TL;DR**: Stack-based parser solves the **O(N²) re-parsing problem** for streaming data,
//! making it essential for network scenarios despite being overkill for local files.
//!
//! ---
//!
//! ### The Re-Parsing Problem
//!
//! **Recursive parser (bin_json.rs)** when receiving chunked data:
//! ```text
//! Chunk 1: {"name":"Jo     → Parse from start → Incomplete → discard progress
//! Chunk 2: hn","age":      → Parse entire {"name":"John","age": → Incomplete → discard progress
//! Chunk 3: 30}             → Parse entire {"name":"John","age":30} → Success
//! ```
//! **Result**: O(N²) complexity - each chunk re-parses everything before it!
//!
//! **Stack-based parser (stack.rs)**:
//! ```text
//! Chunk 1: {"name":"Jo     → Save: Object{current_key: "name"}, partial "Jo"
//! Chunk 2: hn","age":      → Resume: Complete "John", insert, save current_key: "age"
//! Chunk 3: 30}             → Resume: Parse 30, insert, close → Done
//! ```
//! **Result**: O(N) complexity - each byte parsed exactly once!
//!
//! ---
//!
//! ### Use Cases
//!
//! | Scenario | Best Parser | Reason |
//! |----------|-------------|---------|
//! | **HTTP Streaming** | Stack | Data arrives in TCP packets (1-8KB chunks) |
//! | **WebSocket** | Stack | Messages may be fragmented |
//! | **Large network downloads** | Stack | Parse while downloading (don't wait for 100MB) |
//! | **Local files** | Recursive | Can `fs::read()` entire file instantly |
//! | **Small JSON (<1MB)** | Recursive | Simpler, faster, stack overhead unnecessary |
//!
//! ---
//!
//! ### Key Insight
//!
//! For a **10MB JSON over network** arriving in **1000 chunks**:
//! - Recursive: Parses ~5MB on average per chunk = **5GB total parsing**
//! - Stack: Parses each byte once = **10MB total parsing**
//!
//! **500x difference** in work done!
//!
//! This is why we keep **both parsers** - different tools for different jobs. 

#[cfg(feature = "no_std")]
use crate::prelude::*;
use crate::hash::HashMap;
use crate::object::Value; 
use super::error::ParseErrorKind;

/// Frame state descriptor - describes what the parser expects next
///
/// This enum makes state machine logic clearer by distinguishing between
/// object's two states (waiting for key vs waiting for value) and array state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameState {
    /// Object with no current_key - expects a key string next (or '}' to close)
    ObjectWaitingForKey,
    /// Object with current_key set - expects a value next (after ':')
    ObjectWaitingForValue,
    /// Array - expects a value next (or ']' to close)
    Array,
}

/// Stack frame for resumable parsing
///
/// Each frame represents a container (object or array) that's being parsed.
/// When parsing is interrupted, the stack preserves the state so we can
/// resume exactly where we left off.
#[derive(Debug, Clone)]
enum StackFrame {
    /// Object being parsed: { "key": value, ... }
    Object {
        /// Accumulated key-value pairs
        map: HashMap<String, Value>,
        /// Current key waiting for a value (after ':')
        current_key: Option<String>,
    },
    /// Array being parsed: [ value1, value2, ... ]
    Array {
        /// Accumulated items
        items: Vec<Value>,
    },
} 

impl StackFrame {
    /// Create a new empty Object frame
    pub fn new_object() -> Self {
        StackFrame::Object {
            map: HashMap::default(),
            current_key: None,
        }
    }

    /// Create a new empty Array frame
    pub fn new_array() -> Self {
        StackFrame::Array {
            items: Vec::new(),
        }
    }
}

/// The structure of Stack of Frames
pub struct ValueStack {
    stack: Vec<StackFrame>
}

impl ValueStack {
    /// Create a new empty ValueStack
    pub fn new() -> Self {
        Self {
            stack: Vec::new()
        }
    }

    #[allow(dead_code)]
    /// Check if stack is empty
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    #[allow(dead_code)]
    /// Get how many frames are in the stack
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    /// Get a reference to the current stack frame (if any)
    fn last(&self) -> Option<&StackFrame> {
        self.stack.last()
    }

    #[allow(dead_code)] 
    /// Get a mutable reference to the current stack frame (if any)
    fn last_mut(&mut self) -> Option<&mut StackFrame> {
        self.stack.last_mut()
    }

    /// Get the current frame state - what the parser expects next
    ///
    /// Returns None if stack is empty (top-level or before first container)
    pub fn current_frame_state(&self) -> Option<FrameState> {
        match self.last() {
            Some(StackFrame::Object { current_key: None, .. }) => Some(FrameState::ObjectWaitingForKey),
            Some(StackFrame::Object { current_key: Some(_), .. }) => Some(FrameState::ObjectWaitingForValue),
            Some(StackFrame::Array { .. }) => Some(FrameState::Array),
            None => None,
        }
    }

    #[allow(dead_code)] 
    /// Push a new container frame onto the stack
    fn push_frame(&mut self, frame: StackFrame) {
        self.stack.push(frame);
    }

    pub fn push_new_object(&mut self) {
        self.stack.push(StackFrame::new_object());
    } 

    pub fn push_new_key(&mut self, key: String) -> Result<(), ParseErrorKind> {
        match self.stack.last_mut() {
            Some(StackFrame::Object { current_key, .. }) => {
                *current_key = Some(key);
                Ok(())
            }
            _ => Err(ParseErrorKind::Message("Cannot push key - not in object")),
        }
    } 

    pub fn push_new_array(&mut self) {
        self.stack.push(StackFrame::new_array());
    } 

    #[allow(dead_code)] 
    /// Pop the top frame from the stack
    fn pop_frame(&mut self) -> Option<StackFrame> {
        self.stack.pop()
    }

    /// Push a value into the CURRENT container (stack[-1])
    ///
    /// For Object: Requires current_key to be set, inserts as map[key] = value
    /// For Array: Appends to items
    /// Error if stack is empty or Object has no current_key
    pub fn push(&mut self, value: Value) -> Result<(), ParseErrorKind> {
        match self.stack.last_mut() {
            Some(StackFrame::Object { map, current_key }) => {
                let key = current_key.take()
                    .ok_or_else(|| ParseErrorKind::Message("Object value without key"))?;
                map.insert(key, value);
                Ok(())
            }
            Some(StackFrame::Array { items }) => {
                items.push(value);
                Ok(())
            }
            None => {
                Err(ParseErrorKind::Message("Cannot push - no container on stack"))
            }
        }
    }

    /// Pop the current container and push it into its parent
    ///
    /// Converts stack[-1] to a Value, pops it, then:
    /// - If stack is now empty: returns the value (top-level done)
    /// - Otherwise: pushes into stack[-1] (now the parent)
    ///
    /// Returns Ok(Some(value)) if top-level completed
    /// Returns Ok(None) if pushed into parent successfully
    pub fn push_to_parent(&mut self) -> Result<Option<Value>, ParseErrorKind> {
        // Pop current container
        let frame = self.stack.pop()
            .ok_or_else(|| ParseErrorKind::Message("Cannot push_to_parent - stack is empty"))?;

        // Convert to Value
        let value = match frame {
            StackFrame::Object { map, .. } => Value::Dict(map),
            StackFrame::Array { items } => Value::List(items),
        };

        // If stack is empty, this was the top-level value
        if self.stack.is_empty() {
            return Ok(Some(value));
        }

        // Otherwise, push into parent container
        self.push(value)?;
        Ok(None)
    }
}

#[allow(dead_code)] 
/// Stack parser state machine
///
/// Tracks the current parsing state including checkpoint positions for resumable parsing.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseState {
    /// Waiting to parse the next value
    ExpectValue,

    /// Just parsed a value, need to decide what to do with it
    GotValue(Value),

    /// Parsing is complete
    Done,
} 
