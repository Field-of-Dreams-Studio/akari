#![cfg_attr(feature = "no_std", no_std)]

extern crate alloc;

/// Re-exports of `alloc` essentials for `no_std` builds. In `std` mode these
/// are already in the prelude, so the module is omitted entirely.
#[cfg(feature = "no_std")]
pub mod prelude {
    // pub use alloc::borrow::ToOwned; // currently unused; re-enable if any module starts calling `.to_owned()`
    pub use alloc::boxed::Box;
    pub use alloc::format;
    pub use alloc::string::{String, ToString};
    pub use alloc::vec;
    pub use alloc::vec::Vec;
}

#[cfg(all(feature = "template", feature = "no_std"))]
compile_error!("the `template` feature requires std and is incompatible with `no_std`");

#[cfg(all(feature = "bin", feature = "no_std"))]
compile_error!("the `bin` feature requires std and is incompatible with `no_std`");

#[cfg(feature = "hash")]
pub mod hash;

// Export public APIs
#[cfg(feature = "dynamic")]
mod object;
#[cfg(feature = "dynamic")]
pub use object::*;

#[cfg(feature = "template")]
mod template; 
#[cfg(feature = "template")]
pub use template::parse::{Token, tokenize};
#[cfg(feature = "template")]
pub use template::compile::compile;
#[cfg(feature = "template")]
pub use template::template_manager::TemplateManager; 

#[cfg(feature = "object_macro")]
pub use akari_macro::object; 

#[cfg(any(feature = "extension"))]
pub mod extensions; 


