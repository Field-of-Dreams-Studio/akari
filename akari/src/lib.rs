use std::collections::HashMap;

// Include all your existing modules 
mod object; 
mod template; 
mod test; 

// Export public APIs
pub use object::value::Value; 
pub use object::error::ValueError; 
pub use template::parse::{Token, tokenize};
pub use template::compile::compile;
pub use template::template_manager::TemplateManager; 
pub use akari_macro::object; 

