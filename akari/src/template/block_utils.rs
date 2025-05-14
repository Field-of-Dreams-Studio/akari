use std::collections::HashMap;
use crate::object::Object as Obj;
use crate::parse::{Token, tokenize};
use crate::compile::compile;

/// Extracts and renders a specific block from a template
///
/// # Arguments
/// * `template_str` - The template string containing blocks
/// * `block_name` - Name of the block to extract
/// * `data` - Data for template rendering
///
/// # Returns
/// * Result containing either the rendered block content or an error
pub fn render_block(template_str: &str, block_name: &str, data: &HashMap<String, Obj>) 
    -> Result<String, String> 
{
    // Parse the template
    let tokens = tokenize(template_str);
    
    // Extract the block tokens
    let block_tokens = extract_block_tokens(&tokens, block_name)?;
    
    // Render the block
    compile(block_tokens, data.clone())
}

/// Extracts tokens for a specific block from parsed template tokens
fn extract_block_tokens(tokens: &[Token], block_name: &str) -> Result<Vec<Token>, String> {
    let mut i = 0;
    
    while i < tokens.len() {
        match &tokens[i] {
            Token::BlockKeyword => {
                if i + 1 < tokens.len() {
                    if let Token::Identifier(name) = &tokens[i + 1] {
                        if name == block_name {
                            // Found the block, now extract its content
                            i += 2; // Skip block keyword and name
                            
                            // Skip end-of-statement if present
                            if i < tokens.len() && matches!(tokens[i], Token::EndOfStatement) {
                                i += 1;
                            }
                            
                            let start = i;
                            let mut depth = 1;
                            
                            // Find the matching endblock
                            while i < tokens.len() && depth > 0 {
                                match tokens[i] {
                                    Token::BlockKeyword => depth += 1,
                                    Token::EndBlockKeyword => depth -= 1,
                                    _ => {}
                                }
                                i += 1;
                            }
                            
                            if depth > 0 {
                                return Err(format!("Unterminated block: {}", block_name));
                            }
                            
                            // Extract the block content (excluding endblock)
                            let end = i - 1;
                            return Ok(tokens[start..end].to_vec());
                        }
                    }
                }
            },
            _ => {}
        }
        i += 1;
    }
    
    Err(format!("Block '{}' not found in template", block_name))
}

/// Gets a list of all block names defined in a template
pub fn list_blocks(template_str: &str) -> Result<Vec<String>, String> {
    let tokens = tokenize(template_str);
    let mut blocks = Vec::new();
    let mut i = 0;
    
    while i < tokens.len() {
        match &tokens[i] {
            Token::BlockKeyword => {
                if i + 1 < tokens.len() {
                    if let Token::Identifier(name) = &tokens[i + 1] {
                        blocks.push(name.clone());
                    }
                }
            },
            _ => {}
        }
        i += 1;
    }
    
    Ok(blocks)
} 
