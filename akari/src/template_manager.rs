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
    /// Maximum recursion depth for template inheritance 
    max_recursion_depth: u32, 
    /// Cache enabled flag
    cache_enabled: bool,
} 

impl TemplateManager {
    /// Creates a new TemplateManager instance
    ///
    /// # Arguments
    /// * `template_dir` - Path to the directory containing template files 
    pub fn new<P: AsRef<Path>>(template_dir: P) -> Self {
        TemplateManager {
            template_dir: template_dir.as_ref().to_path_buf(),
            template_cache: Arc::new(RwLock::new(HashMap::new())), 
            max_recursion_depth: 10, 
            cache_enabled: true,
        }
    } 

    /// Get a full dir of a path 
    pub fn get_template_path(&self, path: &str) -> PathBuf {
        self.template_dir.join(path) 
    } 

    /// Enables or disables template caching
    pub fn with_caching(mut self, enabled: bool) -> Self {
        self.cache_enabled = enabled;
        self
    } 

    /// Set the maximum recursion depth for template inheritance 
    pub fn with_max_recursion_depth(mut self, depth: u32) -> Self {
        self.max_recursion_depth = depth;
        self
    } 

    /// Add the template's tokens to the cache. If it already exists, it will be replaced. 
    pub fn load_into_cache(&self, template_name: &str) { 
        if self.cache_enabled { 
            let mut cache = self.template_cache.write().unwrap(); 
            if let Ok(tokens) = self.read_template_token(template_name) { 
                cache.insert(template_name.to_string(), tokens); 
            } else { 
                eprintln!("Failed to load template into cache: {}", template_name); 
            }  
        } 
    } 

    /// Get the tokenized template from the cache. If it doesn't exist, it will returns a none 
    pub fn get_from_cache(&self, template_name: &str) -> Option<Vec<Token>> { 
        if self.cache_enabled { 
            let cache = self.template_cache.read().unwrap(); 
            return cache.get(template_name).cloned(); 
        } 
        None 
    } 

    /// Load tokens from a template file, using cache if enabled 
    pub fn load_tokens(&self, template_name: &str) -> Result<Vec<Token>, String> { 
        if self.cache_enabled { 
            if let Some(tokens) = self.get_from_cache(template_name) { 
                return Ok(tokens); 
            } 
        } 
        let tokens = self.read_template_token(template_name)?; 
        self.load_into_cache(template_name); 
        Ok(tokens) 
    } 

    /// Tokenizes the template file 
    pub fn read_template_token(&self, template_name: &str) -> Result<Vec<Token>, String> {
        match self.read_template_content(template_name) { 
            Ok(content) => { 
                return Ok(tokenize(&content)) 
            }, 
            Err(e) => Err(e), 
        }
    }

    /// Get the string content of a template file 
    fn read_template_content(&self, template_name: &str) -> Result<String, String> {
        fs::read_to_string(
            &self.get_template_path(template_name)
        ).map_err(
            |e| format!("Failed to read template '{}': {}", template_name, e)
        )
    } 

    /// Loads and renders a template by name
    pub fn render(&self, template_name: &str, data: &HashMap<String, Obj>) -> Result<String, String> {
        // Get all template tokens needed (with inheritance resolution)
        let tokens = self.expand_template(self.load_tokens(template_name)?, template_name, &mut 0); 
        // Use your existing compile function to process these tokens
        compile(tokens, data.clone()) 
    } 

    pub fn render_string(&self, template_str: String, data: &HashMap<String, Obj>) -> Result<String, String> { 
        // Tokenize the string content 
        let tokens = tokenize(&template_str); 
        // Insert template content into the token stream 
        let tokens = self.expand_template(tokens, "", &mut 0); 
        
        // Use your existing compile function to process these tokens
        compile(tokens, data.clone()) 
    } 

    pub fn expand_template(&self, tokens: Vec<Token>, self_dir: &str, recursion_count: &mut u32) -> Vec<Token> { 
        if *recursion_count > self.max_recursion_depth { 
            return vec![Token::HtmlContent(format!("<!-- Template Error: Maximum recursion depth exceeded -->"))]; 
        } 

        *recursion_count += 1; // Increment recursion count 
        
        // Insert template content into the token stream
        let tokens = self.insert_template(tokens, self_dir, recursion_count); 
        
        // Extend with parent template if applicable
        match self.extend_with_parent(tokens, self_dir, recursion_count) { 
            Ok(tokens) => tokens, 
            Err(e) => vec![Token::HtmlContent(format!("<!-- Template Error: {} -->", e))], 
        } 
    } 

    pub fn insert_template(&self, mut tokens: Vec<Token>, self_dir: &str, recursion_count: &mut u32) -> Vec<Token> { 
        let mut i = 0; 
        while i < tokens.len() { 
            if matches!(tokens.get(i), Some(Token::InsertKeyword)) && 
               i + 1 < tokens.len() {
                
                if let Some(Token::Object(Obj::Str(name))) = tokens.get(i + 1) {
                    // Get the full path of the template name 
                    let full_path = get_full_dir(name, self_dir);
                    
                    // Find the position to insert template content (after EndOfStatement)
                    let mut j = i + 2;
                    let mut found_end = false;
                    
                    while j < tokens.len() {
                        if matches!(tokens[j], Token::EndOfStatement) { 
                            found_end = true;
                            break;
                        }
                        j += 1;
                    }
                    
                    // Only proceed if we found the end statement
                    if found_end {
                        // Load template tokens without unnecessary clone
                        let tokens_to_insert = match self.load_tokens(&full_path) {
                            Ok(template_tokens) => self.expand_template(template_tokens, &full_path, recursion_count),
                            Err(e) => {
                                // More informative error token
                                vec![Token::HtmlContent(format!("<!-- Template Error: {} - {} -->", full_path, e))]
                            }
                        };
                        
                        // Calculate new position before modifying vector
                        let new_position = j + 1 + tokens_to_insert.len();
                        
                        // Insert the tokens after EndOfStatement
                        tokens.splice(j + 1..j + 1, tokens_to_insert);
                        
                        // Adjust index to account for inserted tokens
                        i = new_position - 1;
                        continue; // Skip the increment at the end
                    }
                }
            }
            i += 1; // Move to the next token 
        }
        
        tokens // Return the modified tokens vector
    } 

    pub fn extend_with_parent(&self, tokens: Vec<Token>, self_dir: &str, recursion_count: &mut u32) -> Result<Vec<Token>, String> { 
        // Check if this template extends another one
        if let Some(parent_name) = self.extract_parent_template_name(&tokens) {
            // Deal with the dir 
            let parent_name = get_full_dir(&parent_name, self_dir); 

            // Load the parent template
            let parent_tokens = self.expand_template(self.load_tokens(&parent_name)?, &parent_name, recursion_count); 
            
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
} 

impl Default for TemplateManager {
    fn default() -> Self {
        TemplateManager::new("./template")
    }
} 

/// To see whether the dir is a relative dir or a absolute dir 
/// If absolute dir, return the path 
/// If relative dir, return the path with the ori path 
pub fn get_full_dir(path: &str, ori: &str) -> String { 
    if path.starts_with("/") || path.starts_with("\\") { 
        // Return path without the first character 
        let path = path.trim_start_matches("/").trim_start_matches("\\").to_string();
        return path 
    } else { 
        // Remove everything after the last separator in the original path 
        let ori = if let Some(pos) = ori.rfind('/') {
            &ori[..pos]
        } else if let Some(pos) = ori.rfind('\\') {
            &ori[..pos]
        } else {
            ori
        }; 
        // Handle relative path
        // First check if the original path ends with a separator
        if ori.ends_with("/") || ori.ends_with("\\") {
            format!("{}{}", ori, path)
        } else {
            // Add a separator between ori and path
            format!("{}/{}", ori, path)
        }
    }   
} 

