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
//! The `ValueSerializer<T>` trait mirrors `ValueParser<T>` for consistency:
//!
//! | Parser Method | Serializer Method | Purpose |
//! |---------------|-------------------|---------|
//! | `create(input)` | `create(value)` | Initialize |
//! | `append(input)` | `append(value)` | Add data |
//! | `parse(input)` | `serialize(value)` | One-shot |
//! | `fparse()` | `fserialize()` | Full/complete |
//! | `pparse()` | `pserialize()` | Partial/streaming |
//! | `pos()` | `pos()` | Position tracking |

mod trait_def;
mod error;
mod writer;
pub mod json;

// Re-export the trait and error types
pub use trait_def::ValueSerializer;
pub use error::{SerializeError, SerializeErrorKind};
pub use writer::BinWriter;

// Future serializer implementations will be added here:
// pub use json::JsonSerializer;
