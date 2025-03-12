use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::RwLock;

use crate::object::Object as Obj;
use crate::parse::{tokenize, Token};
use crate::compile::compile;

/// Manages template loading, caching, and rendering
pub struct TemplateManager {
    /// Base directory for template files
    template_dir: PathBuf,
    /// Cache of parsed templates
    template_cache: Arc<RwLock<HashMap<String, Vec<Token>>>>,
    /// Maximum cache size (number of templates)
    max_cache_size: usize,
    /// Cache enabled flag
    cache_enabled: bool,
}

impl TemplateManager {
    /// Creates a new TemplateManager with the given template directory
    pub fn new<P: AsRef<Path>>(template_dir: P) -> Self {
        TemplateManager {
            template_dir: template_dir.as_ref().to_path_buf(),
            template_cache: Arc::new(RwLock::new(HashMap::new())),
            max_cache_size: 100,
            cache_enabled: true,
        }
    }
    
    /// Creates a new TemplateManager with default template directory "./template"
    pub fn default() -> Self {
        Self::new("./template")
    }
    
    /// Sets the maximum cache size
    pub fn with_max_cache_size(mut self, size: usize) -> Self {
        self.max_cache_size = size;
        self
    }
    
    /// Enables or disables template caching
    pub fn with_caching(mut self, enabled: bool) -> Self {
        self.cache_enabled = enabled;
        self
    }
    
    /// Loads and renders a template by name
    pub fn render(&self, template_name: &str, data: &HashMap<String, Obj>) -> Result<String, String> {
        // Load the template content from file or cache
        let template_content = self.load_template_content(template_name)?;
        // Get all template tokens needed (with inheritance resolution)
        let tokens = self.load_template_with_inheritance(template_content)?;
        // Use your existing compile function to process these tokens
        compile(tokens, data.clone())
    }

    /// Loads and renders a template by string
    pub fn render_string(&self, template_content: String, data: &HashMap<String, Obj>) -> Result<String, String> {
        // Get all template tokens needed (with inheritance resolution)
        let tokens = self.load_template_with_inheritance(template_content)?;
        
        // Use your existing compile function to process these tokens
        compile(tokens, data.clone())
    }
    
    /// Loads a template and resolves inheritance
    fn load_template_with_inheritance(&self, template_content: String) -> Result<Vec<Token>, String> {
        let tokens = tokenize(&template_content);
        
        // Check if this template extends another one
        if let Some(parent_name) = self.extract_parent_template_name(&tokens) {
            // Load the parent template
            let parent_tokens = match self.load_template_content(&parent_name) {
                Ok(content) => tokenize(&content),
                Err(e) => return Err(format!("Failed to load parent template '{}': {}", parent_name, e)),
            };
            
            // Extract blocks from both parent and child
            let parent_blocks = self.extract_blocks(&parent_tokens)?;
            let child_blocks = self.extract_blocks(&tokens)?;
            
            // Create merged blocks where child overrides parent
            let mut merged_blocks = parent_blocks;
            for (name, tokens) in child_blocks {
                merged_blocks.insert(name, tokens);
            }
            
            // Create the final template by replacing blocks in parent with merged blocks
            self.create_template_with_blocks(&parent_tokens, merged_blocks)
        } else {
            // No inheritance, just process blocks within this template
            let blocks = self.extract_blocks(&tokens)?;
            self.create_template_with_blocks(&tokens, blocks)
        }
    }
    
    /// Creates a processed template with all blocks properly defined
    fn create_template_with_blocks(
        &self, 
        template_tokens: &[Token],
        blocks: HashMap<String, Vec<Token>>
    ) -> Result<Vec<Token>, String> {
        let mut result = Vec::new();
        let mut i = 0;
        
        while i < template_tokens.len() {
            match &template_tokens[i] {
                // Skip template directive since we've already handled inheritance
                Token::TemplateKeyword => {
                    // Skip template keyword and the template name string
                    i += 2;
                },
                // Add block definition with its identifier
                Token::BlockKeyword => {
                    if i + 1 < template_tokens.len() {
                        if let Token::Identifier(name) = &template_tokens[i + 1] {
                            // Add BlockKeyword and its name
                            result.push(template_tokens[i].clone());
                            result.push(template_tokens[i + 1].clone());
                            
                            // Skip to after block name
                            i += 2;
                            
                            // Skip EndOfStatement if present
                            if i < template_tokens.len() && matches!(template_tokens[i], Token::EndOfStatement) {
                                result.push(template_tokens[i].clone());
                                i += 1;
                            }
                            
                            // Add the block content from our blocks map
                            if let Some(block_tokens) = blocks.get(name) {
                                result.extend_from_slice(block_tokens);
                            }
                            
                            // Skip to after the endblock
                            let mut depth = 1;
                            while i < template_tokens.len() && depth > 0 {
                                match template_tokens[i] {
                                    Token::BlockKeyword => depth += 1,
                                    Token::EndBlockKeyword => {
                                        depth -= 1;
                                        if depth == 0 {
                                            // Add EndBlockKeyword
                                            result.push(template_tokens[i].clone());
                                            break;
                                        }
                                    },
                                    _ => {}
                                }
                                i += 1;
                            }
                            i += 1; // Move past EndBlockKeyword
                        } else {
                            result.push(template_tokens[i].clone());
                            i += 1;
                        }
                    } else {
                        result.push(template_tokens[i].clone());
                        i += 1;
                    }
                },
                _ => {
                    // Copy all other tokens as-is
                    result.push(template_tokens[i].clone());
                    i += 1;
                }
            }
        }
        
        Ok(result)
    }
    
    /// Loads the raw content of a template file
    fn load_template_content(&self, template_name: &str) -> Result<String, String> {
        let template_path = self.template_dir.join(template_name);
        fs::read_to_string(&template_path)
            .map_err(|e| format!("Failed to read template '{}': {}", template_name, e))
    }
    
    /// Extracts the parent template name if this template extends another
    fn extract_parent_template_name(&self, tokens: &[Token]) -> Option<String> {
        for i in 0..tokens.len() {
            if let Token::TemplateKeyword = &tokens[i] {
                if i + 1 < tokens.len() {
                    if let Token::Object(Obj::Str(name)) = &tokens[i + 1] {
                        return Some(name.clone());
                    }
                }
                break;
            }
        }
        None
    }
    
    /// Extracts all blocks from a token stream
    fn extract_blocks(&self, tokens: &[Token]) -> Result<HashMap<String, Vec<Token>>, String> {
        let mut blocks = HashMap::new();
        let mut i = 0;
        
        while i < tokens.len() {
            if let Token::BlockKeyword = tokens[i] {
                if i + 1 < tokens.len() {
                    if let Token::Identifier(name) = &tokens[i + 1] {
                        let block_name = name.clone();
                        i += 2; // Skip block keyword and name
                        
                        // Skip EndOfStatement if present
                        if i < tokens.len() && matches!(tokens[i], Token::EndOfStatement) {
                            i += 1;
                        }
                        
                        let start = i;
                        let mut depth = 1;
                        let mut end_pos = i;
                        
                        // Find matching endblock
                        while end_pos < tokens.len() && depth > 0 {
                            match &tokens[end_pos] {
                                Token::BlockKeyword => depth += 1,
                                Token::EndBlockKeyword => {
                                    depth -= 1;
                                    if depth == 0 {
                                        break;
                                    }
                                },
                                _ => {}
                            }
                            end_pos += 1;
                        }
                        
                        if depth > 0 {
                            return Err(format!("Unterminated block: {}", block_name));
                        }
                        
                        // Extract block content (excluding endblock token)
                        let content = tokens[start..end_pos].to_vec();
                        blocks.insert(block_name, content);
                        
                        i = end_pos + 1; // Skip past EndBlockKeyword
                        continue;
                    }
                }
            }
            i += 1;
        }
        
        Ok(blocks)
    }
    
    /// Reloads a template from disk, bypassing the cache
    pub fn reload_template(&self, template_name: &str) -> Result<(), String> {
        if self.cache_enabled {
            self.template_cache.write().unwrap().remove(template_name);
        }
        // Try loading to verify it exists and is valid
        self.load_template_content(template_name)?;
        Ok(())
    }
    
    /// Clears the entire template cache
    pub fn clear_cache(&self) {
        if self.cache_enabled {
            self.template_cache.write().unwrap().clear();
        }
    }
    
    /// Lists all available templates in the template directory
    pub fn list_templates(&self) -> Result<Vec<String>, String> {
        let entries = fs::read_dir(&self.template_dir)
            .map_err(|e| format!("Failed to read template directory: {}", e))?;
            
        let mut templates = Vec::new();
        
        for entry in entries {
            if let Ok(entry) = entry {
                if entry.path().is_file() {
                    if let Some(filename) = entry.path().file_name() {
                        if let Some(name) = filename.to_str() {
                            templates.push(name.to_string());
                        }
                    }
                }
            }
        }
        
        Ok(templates)
    }
} 
