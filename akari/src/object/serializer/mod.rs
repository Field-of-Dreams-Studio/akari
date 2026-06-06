//! Serialization module for converting Akari Values to various formats
//!
//! This module provides the `ValueSerializer` trait and its implementations for
//! serializing Akari's `Value` type into different output formats (JSON, YAML, etc.).
//!
//! # Architecture
//!
//! The serializer system is the inverse of the parser system:
//! - **Parser**: External format → Akari Value
//! - **Serializer**: Akari Value → External format
//!
//! # Module Structure
//!
//! ```text
//! serializer/
//! ├── mod.rs           # Module exports
//! ├── trait_def.rs     # ValueSerializer<T> trait definition
//! └── json.rs          # JsonSerializer implementation (future)
//! ```
//!
//! # Trait Design
//!
//! `ValueSerializer<O>` intentionally stays minimal:
//! - `serialize_one(&Value) -> Output` for owned one-shot serialization
//! - `serialize_buf(&Value, &mut BinWriter)` for streaming into a reusable buffer
//!
//! This keeps parser/serializer APIs conceptually aligned while avoiding forced
//! stateful serializer machinery where it is not needed.

mod trait_def;
mod error;
mod writer;
pub mod json;

// Re-export the trait and error types
pub use trait_def::ValueSerializer;
pub use error::{SerializeError, SerializeErrorKind};
pub use writer::BinWriter;
pub use json::JsonSerializer;
