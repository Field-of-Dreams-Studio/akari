use std::collections::HashMap; 
use crate::Value; 

pub mod parse;
pub mod compile;
pub mod template_manager; 
pub mod templates;  

/// Renders a template string with provided data
///
/// # Arguments
/// * `template_str` - The template string to render
/// * `data` - Template variables
///
/// # Returns
/// * Result containing the rendered output or an error
pub fn render(template_str: &str, data: &HashMap<String, Value>) -> Result<String, String> {
    let tokens = parse::tokenize(template_str); 
    compile::compile(tokens, data.clone())
}

/// Renders a template file with provided data
///
/// # Arguments
/// * `template_path` - Path to the template file
/// * `data` - Template variables
///
/// # Returns
/// * Result containing the rendered output or an error
pub fn render_file(template_path: &str, data: &HashMap<String, Value>) -> Result<String, String> {
    use std::fs;
    let content = fs::read_to_string(template_path)
        .map_err(|e| format!("Failed to read template file: {}", e))?;
    render(&content, data)
} 
