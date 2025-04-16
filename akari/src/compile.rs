use std::collections::HashMap;
use crate::object::Object as Obj; 
use crate::parse::Token; 

use super::parse;
use super::object;

pub fn compile(tokens: Vec<Token>, mut data: HashMap<String, Obj>) -> Result<String, String> {
    let mut compiler = TemplateCompiler::new(tokens, data);
    compiler.compile()
} 

// AccessType enum to distinguish between reading and writing operations
#[derive(Debug, Clone, Copy, PartialEq)]
enum AccessType {
    Read,  // Just reading a value
    Write, // Writing to a variable or property
}

struct TemplateCompiler {
    tokens: Vec<Token>,
    data: HashMap<String, Obj>,
    pos: usize,
    blocks: HashMap<String, Vec<Token>>,
    output: String,
    export_mode: bool,
    template_name: Option<String>,
}

impl TemplateCompiler {
    fn new(tokens: Vec<Token>, data: HashMap<String, Obj>) -> Self {
        TemplateCompiler {
            tokens,
            data,
            pos: 0,
            blocks: HashMap::new(),
            output: String::new(),
            export_mode: false,
            template_name: None,
        }
    }

    fn compile(&mut self) -> Result<String, String> {
        // First pass: identify blocks and template info
        self.collect_blocks_and_metadata()?; 
        
        // Reset position for second pass
        self.pos = 0;
        self.output.clear();
        
        // If this is a template file with export directive, we don't directly output
        if self.export_mode {
            return Ok(String::new());
        }
        
        // Second pass: generate output
        self.generate_output()
    }
    
    fn collect_blocks_and_metadata(&mut self) -> Result<(), String> {
        while self.pos < self.tokens.len() {
            match &self.tokens[self.pos] {
                Token::TemplateKeyword => {
                    self.pos += 1;
                    if let Some(Token::Object(Obj::Str(name))) = self.tokens.get(self.pos) {
                        self.template_name = Some(name.clone());
                        self.pos += 1;
                    } else {
                        return Err("Expected string after template keyword".to_string());
                    }
                },
                Token::BlockKeyword => {
                    self.pos += 1;
                    if let Some(Token::Identifier(name)) = self.tokens.get(self.pos).cloned() {
                        self.pos += 1;
                        // Skip END_OF_STATEMENT
                        if matches!(self.tokens.get(self.pos), Some(Token::EndOfStatement)) {
                            self.pos += 1;
                        }
                        
                        let start = self.pos;
                        let mut depth = 1;
                        
                        // Find the matching endblock
                        while self.pos < self.tokens.len() && depth > 0 {
                            match self.tokens[self.pos] {
                                Token::BlockKeyword => depth += 1,
                                Token::EndBlockKeyword => depth -= 1,
                                _ => {}
                            }
                            self.pos += 1;
                        }
                        
                        if depth > 0 {
                            return Err(format!("Unterminated block: {}", name));
                        }
                        
                        // Take the block content (excluding endblock)
                        let end = self.pos - 1;
                        let block_tokens = self.tokens[start..end].to_vec();
                        self.blocks.insert(name, block_tokens);
                    } else {
                        return Err("Expected identifier after block keyword".to_string());
                    }
                },
                Token::ExportKeyword => {
                    self.export_mode = true;
                    self.pos += 1;
                },
                _ => self.pos += 1,
            }
            
            // Skip END_OF_STATEMENT tokens
            if matches!(self.tokens.get(self.pos), Some(Token::EndOfStatement)) {
                self.pos += 1;
            }
        }
        Ok(())
    }
    
    fn generate_output(&mut self) -> Result<String, String> {
        while self.pos < self.tokens.len() {
            match &self.tokens[self.pos] {
                Token::HtmlContent(content) => {
                    self.output.push_str(content);
                    self.pos += 1;
                },
                Token::PlaceholderKeyword => {
                    self.pos += 1;
                    if let Some(Token::Identifier(name)) = self.tokens.get(self.pos) {
                        let block_name = name.clone();
                        if let Some(block_tokens) = self.blocks.get(&block_name) {
                            // Create a new compiler to process this block
                            let mut block_compiler = TemplateCompiler::new(
                                block_tokens.clone(), 
                                self.data.clone()
                            );
                            match block_compiler.generate_output() {
                                Ok(block_output) => {
                                    self.output.push_str(&block_output);
                                    // Update data with any changes from the block
                                    self.data = block_compiler.data;
                                },
                                Err(e) => return Err(e),
                            }
                        } else {
                            // Use empty content for missing blocks
                            self.output.push_str(&format!("<!-- Block '{}' not defined -->", block_name));
                        }
                        self.pos += 1;
                    } else {
                        return Err("Expected identifier after placeholder keyword".to_string());
                    }
                },
                Token::BlockKeyword => {
                    // Skip over blocks in the second pass, as they're already processed
                    self.pos += 1; // Skip BlockKeyword
                    
                    // Skip block name
                    if matches!(self.tokens.get(self.pos), Some(Token::Identifier(_))) {
                        self.pos += 1;
                    }
                    
                    // Skip END_OF_STATEMENT if present
                    if matches!(self.tokens.get(self.pos), Some(Token::EndOfStatement)) {
                        self.pos += 1;
                    }
                    
                    // Find the matching endblock
                    // let mut depth = 1;
                    // while self.pos < self.tokens.len() && depth > 0 {
                    //     match self.tokens[self.pos] {
                    //         Token::BlockKeyword => depth += 1,
                    //         Token::EndBlockKeyword => depth -= 1,
                    //         _ => {}
                    //     }
                    //     self.pos += 1;
                    // } 
                },
                Token::EndBlockKeyword => {
                    // Skip EndBlockKeyword as it's handled when skipping blocks
                    self.pos += 1;
                },
                Token::LetKeyword => {
                    self.handle_assignment()?;
                },
                Token::IfKeyword => {
                    self.handle_if_statement()?;
                },
                Token::ForKeyword => {
                    self.handle_for_loop()?;
                },
                Token::WhileKeyword => {
                    self.handle_while_loop()?;
                },
                Token::OutputKeyword => {
                    self.pos += 1;
                    let value = self.evaluate_expression()?;
                    self.output.push_str(&value.interal_value_as_string());
                },
                Token::DelKeyword => {
                    self.handle_deletion()?;
                },
                Token::Identifier(name) => {
                    let var_name = name.clone();
                    self.pos += 1;
                    
                    // Check for access operations (index, property, or assignment)
                    match self.handle_variable_access(&var_name, true)? {
                        Some(value) => {
                            self.output.push_str(&value.interal_value_as_string());
                        },
                        None => {} // Assignment was handled in handle_variable_access
                    }
                },
                Token::EndOfStatement => {
                    self.pos += 1;
                },
                _ => {
                    // Skip other tokens
                    self.pos += 1;
                }
            }
        }
        
        Ok(self.output.clone()) 
    }
    
    /// Handles variable access, including indexing, property access, and assignments
    /// Returns Some(value) for read operations, None for write operations
    fn handle_variable_access(&mut self, var_name: &str, output_to_stream: bool) -> Result<Option<Obj>, String> {
        // Check if this is a property access or indexing
        let mut value = match self.data.get(var_name) {
            Some(v) => v.clone(),
            None => {
                if output_to_stream {
                    self.output.push_str(
                        &format!("<!-- There should be a value '{}' but not found -->", var_name)
                    );
                }
                return Ok(Some(Obj::None));
            }
        };
        
        let mut access_chain_pos = self.pos;
        
        // Handle indexing with [] syntax
        while matches!(self.tokens.get(access_chain_pos), Some(Token::LeftSquareBracket)) {
            access_chain_pos += 1;
            self.pos = access_chain_pos;
            
            // Evaluate the index expression
            let index = self.evaluate_expression()?;
            access_chain_pos = self.pos;
            
            if !matches!(self.tokens.get(access_chain_pos), Some(Token::RightSquareBracket)) {
                return Err("Expected closing bracket after index".to_string());
            }
            access_chain_pos += 1;
            
            // Apply the indexing to get the new value
            match &value {
                Obj::List(list) => {
                    match &index {
                        Obj::Numerical(n) => {
                            let idx = *n as usize;
                            if idx < list.len() {
                                value = list[idx].clone();
                            } else if output_to_stream {
                                self.output.push_str(
                                    &format!("<!-- Index {} out of bounds for list -->", idx)
                                );
                                return Ok(Some(Obj::None));
                            } else {
                                return Err(format!("Index {} out of bounds for list", idx));
                            }
                        },
                        _ => {
                            if output_to_stream {
                                self.output.push_str("<!-- List index must be a number -->");
                                return Ok(Some(Obj::None));
                            } else {
                                return Err("List index must be a number".to_string());
                            }
                        }
                    }
                },
                Obj::Dictionary(dict) => {
                    // Allow both string literals and string expressions as keys
                    let key = index.interal_value_as_string();
                    if let Some(val) = dict.get(&key) {
                        value = val.clone();
                    } else if output_to_stream {
                        self.output.push_str(
                            &format!("<!-- Key '{}' not found in dictionary -->", key)
                        );
                        return Ok(Some(Obj::None));
                    } else {
                        return Err(format!("Key '{}' not found in dictionary", key));
                    }
                },
                _ => {
                    if output_to_stream {
                        self.output.push_str(
                            &format!("<!-- Cannot index into a {} value -->", value.type_of())
                        );
                        return Ok(Some(Obj::None));
                    } else {
                        return Err(format!("Cannot index into a {} value", value.type_of()));
                    }
                }
            }
        }
        
        // Handle dot notation for property/method access
        if matches!(self.tokens.get(access_chain_pos), Some(Token::Dot)) {
            access_chain_pos += 1;
            
            if let Some(Token::Identifier(prop_name)) = self.tokens.get(access_chain_pos).cloned() {
                access_chain_pos += 1;
                
                // Handle built-in methods and properties
                match &value {
                    Obj::List(list) => {
                        match prop_name.as_str() {
                            "len" => value = Obj::Numerical(list.len() as f64),
                            // Add more list methods here as needed
                            _ => {
                                if output_to_stream {
                                    self.output.push_str(
                                        &format!("<!-- No property/method '{}' on list -->", prop_name)
                                    );
                                    return Ok(Some(Obj::None));
                                } else {
                                    return Err(format!("No property/method '{}' on list", prop_name));
                                }
                            }
                        }
                    },
                    Obj::Dictionary(dict) => {
                        match prop_name.as_str() {
                            "len" => value = Obj::Numerical(dict.len() as f64),
                            // Add more dictionary methods here as needed
                            _ => {
                                // For dictionaries, treat dot notation as an alternative to [] indexing
                                if let Some(val) = dict.get(&prop_name) {
                                    value = val.clone();
                                } else if output_to_stream {
                                    self.output.push_str(
                                        &format!("<!-- Key '{}' not found in dictionary -->", prop_name)
                                    );
                                    return Ok(Some(Obj::None));
                                } else {
                                    return Err(format!("Key '{}' not found in dictionary", prop_name));
                                }
                            }
                        }
                    },
                    Obj::Str(s) => {
                        match prop_name.as_str() {
                            "len" => value = Obj::Numerical(s.len() as f64),
                            // Add more string methods here as needed
                            _ => {
                                if output_to_stream {
                                    self.output.push_str(
                                        &format!("<!-- No property/method '{}' on string -->", prop_name)
                                    );
                                    return Ok(Some(Obj::None));
                                } else {
                                    return Err(format!("No property/method '{}' on string", prop_name));
                                }
                            }
                        }
                    },
                    _ => {
                        if output_to_stream {
                            self.output.push_str(
                                &format!("<!-- Type '{}' does not support properties/methods -->", value.type_of())
                            );
                            return Ok(Some(Obj::None));
                        } else {
                            return Err(format!("Type '{}' does not support properties/methods", value.type_of()));
                        }
                    }
                }
            }
        }
        
        // Update position after all access operations
        self.pos = access_chain_pos;
        
        // Check if this is an assignment
        if matches!(self.tokens.get(self.pos), Some(Token::Assignment)) {
            self.pos += 1;
            let new_value = self.evaluate_expression()?;
            
            // Handle the assignment based on the access chain
            if self.pos == self.tokens.len() || !matches!(self.tokens.get(access_chain_pos-1), Some(Token::RightSquareBracket) | Some(Token::Dot)) {
                // Simple variable assignment
                self.data.insert(var_name.to_string(), new_value);
            } else {
                // Assignment to indexed/property value - this is complex and requires maintaining the path
                // For now, we're not implementing this part fully
                return Err("Assignment to indexed/property values not fully implemented".to_string());
            }
            
            return Ok(None); // No value to output for assignments
        }
        
        Ok(Some(value))
    }
    
    fn handle_assignment(&mut self) -> Result<(), String> {
        self.pos += 1; // Skip let keyword
        
        if let Some(Token::Identifier(name)) = self.tokens.get(self.pos).cloned() {
            self.pos += 1;
            
            if matches!(self.tokens.get(self.pos), Some(Token::Assignment)) {
                self.pos += 1;
                let value = self.evaluate_expression()?;
                self.data.insert(name, value);
                Ok(())
            } else {
                Err("Expected assignment operator after variable name".to_string())
            }
        } else {
            Err("Expected identifier after let keyword".to_string())
        }
    }
    
    fn handle_deletion(&mut self) -> Result<(), String> {
        self.pos += 1; // Skip del keyword
        
        if let Some(Token::Identifier(name)) = self.tokens.get(self.pos) {
            let var_name = name.clone();
            self.pos += 1;
            
            // Handle simple variable deletion
            if self.pos >= self.tokens.len() || !matches!(self.tokens.get(self.pos), Some(Token::LeftSquareBracket)) {
                self.data.remove(&var_name);
                return Ok(());
            }
            
            // Handle deletion of dictionary/list element
            self.pos += 1; // Skip left bracket
            let index = self.evaluate_expression()?;
            
            if !matches!(self.tokens.get(self.pos), Some(Token::RightSquareBracket)) {
                return Err("Expected closing bracket after index".to_string());
            }
            self.pos += 1; // Skip right bracket
            
            // Perform the deletion
            if let Some(collection) = self.data.get_mut(&var_name) {
                match collection {
                    Obj::List(list) => {
                        if let Obj::Numerical(i) = index {
                            let idx = i as usize;
                            if idx < list.len() {
                                list.remove(idx);
                            } else {
                                self.output.push_str(
                                    &format!("<!-- Index {} out of bounds for list {} -->", idx, var_name)
                                );
                            }
                        } else {
                            self.output.push_str("<!-- List index must be a number -->");
                        }
                    },
                    Obj::Dictionary(dict) => {
                        let key = index.interal_value_as_string();
                        dict.remove(&key);
                    },
                    _ => {
                        self.output.push_str(
                            &format!("<!-- Cannot delete from a {} value -->", collection.type_of())
                        );
                    }
                }
            } else {
                self.output.push_str(
                    &format!("<!-- Variable '{}' not found -->", var_name)
                );
            }
            
            Ok(())
        } else {
            Err("Expected identifier after del keyword".to_string())
        }
    }
    
    fn handle_if_statement(&mut self) -> Result<(), String> {
        self.pos += 1; // Skip if keyword
        
        let condition = self.evaluate_condition()?;
        
        if !condition  {
            // Skip to endif
            let mut depth = 1;
            while self.pos < self.tokens.len() && depth > 0 {
                match self.tokens[self.pos] {
                    Token::IfKeyword => depth += 1,
                    Token::EndIfKeyword => depth -= 1,
                    _ => {}
                }
                self.pos += 1;
            }
        }
        
        Ok(())
    }
    
    fn handle_for_loop(&mut self) -> Result<(), String> {
        self.pos += 1; // Skip for keyword
        
        if let Some(Token::Identifier(loop_var)) = self.tokens.get(self.pos).cloned() {
            self.pos += 1;
            
            // Skip "in" keyword
            if matches!(self.tokens.get(self.pos), Some(Token::InKeyword)) {
                self.pos += 1;
            }
            
            // Get the iterable collection
            let collection = self.evaluate_expression()?;
            
            // Extract the loop body by finding the matching endfor
            let loop_start = self.pos;
            let mut depth = 1;
            
            while self.pos < self.tokens.len() && depth > 0 {
                match self.tokens[self.pos] {
                    Token::ForKeyword => depth += 1,
                    Token::EndForKeyword => depth -= 1,
                    _ => {}
                }
                self.pos += 1;
            }
            
            let loop_end = self.pos - 1; // Exclude the endfor
            let loop_body = self.tokens[loop_start..loop_end].to_vec();
            
            // Now iterate over the collection and execute the loop body for each item
            match collection {
                Obj::List(items) => {
                    // Clone the items to avoid borrowing conflicts
                    let items_vec: Vec<_> = items.into_iter().collect();
                    
                    for item in items_vec {
                        // Set the loop variable to the current item
                        self.data.insert(loop_var.clone(), item.clone());
                        
                        // Execute the loop body
                        let mut body_compiler = TemplateCompiler::new(
                            loop_body.clone(),
                            self.data.clone()
                        );
                        match body_compiler.generate_output() {
                            Ok(body_output) => {
                                self.output.push_str(&body_output);
                                self.data = body_compiler.data;
                            },
                            Err(e) => return Err(e),
                        }
                    }
                },
                Obj::Dictionary(map) => {
                    // Convert dictionary entries to a Vec to avoid borrowing conflicts
                    let entries: Vec<_> = map.into_iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();
                    
                    for (k, v) in entries {
                        // In this implementation, we'll make the loop variable an object with 'key' and 'value' properties
                        let mut entry = HashMap::new();
                        entry.insert("key".to_string(), Obj::Str(k));
                        entry.insert("value".to_string(), v);
                        self.data.insert(loop_var.clone(), Obj::Dictionary(entry));
                        
                        // Execute the loop body
                        let mut body_compiler = TemplateCompiler::new(
                            loop_body.clone(),
                            self.data.clone()
                        );
                        match body_compiler.generate_output() {
                            Ok(body_output) => {
                                self.output.push_str(&body_output);
                                self.data = body_compiler.data;
                            },
                            Err(e) => return Err(e),
                        }
                    }
                },
                Obj::Numerical(n) => {
                    // No borrowing conflicts here, but keeping the same pattern
                    let iterations = n as i64;
                    
                    for i in 0..iterations {
                        self.data.insert(loop_var.clone(), Obj::Numerical(i as f64));
                        
                        // Execute the loop body
                        let mut body_compiler = TemplateCompiler::new(
                            loop_body.clone(),
                            self.data.clone()
                        );
                        match body_compiler.generate_output() {
                            Ok(body_output) => {
                                self.output.push_str(&body_output);
                                self.data = body_compiler.data;
                            },
                            Err(e) => return Err(e),
                        }
                    }
                },
                _ => {
                    self.output.push_str(
                        &format!("<!-- For loop requires a list, dictionary, or number, got {} -->", collection.type_of())
                    );
                }
            }
            
            Ok(())
        } else {
            Err("Expected identifier after for keyword".to_string())
        }
    }
    
    fn handle_while_loop(&mut self) -> Result<(), String> {
        self.pos += 1; // Skip while keyword
        
        // Save the position of the condition
        let condition_pos = self.pos;
        
        // Find the end of the while loop
        let mut depth = 1;
        let mut end_pos = self.pos;
        
        while end_pos < self.tokens.len() && depth > 0 {
            match self.tokens[end_pos] {
                Token::WhileKeyword => depth += 1,
                Token::EndWhileKeyword => depth -= 1,
                _ => {}
            }
            end_pos += 1;
        }
        
        let loop_body = self.tokens[self.pos + 1..end_pos - 1].to_vec();
        
        // Execute the while loop
        let mut iteration = 0;
        const MAX_ITERATIONS: usize = 10000; // Safety limit
        
        while iteration < MAX_ITERATIONS {
            // Reset position to evaluate condition
            self.pos = condition_pos;
            let condition = self.evaluate_condition()?;
            
            if !condition {
                break;
            }
            
            // Execute the loop body
            let mut body_compiler = TemplateCompiler::new(
                loop_body.clone(),
                self.data.clone()
            );
            match body_compiler.generate_output() {
                Ok(body_output) => {
                    self.output.push_str(&body_output);
                    self.data = body_compiler.data;
                },
                Err(e) => {
                    self.output.push_str(&format!("<!-- Error in while loop: {} -->", e));
                    break; // Don't halt rendering on error
                }
            }
            
            iteration += 1;
            if iteration == MAX_ITERATIONS {
                self.output.push_str("<!-- While loop exceeded maximum iterations - possible infinite loop -->");
                break;
            }
        }
        
        // Skip to after endwhile
        self.pos = end_pos;
        
        Ok(())
    }
    
    fn process_token(&mut self) -> Result<(), String> {
        match &self.tokens[self.pos] {
            Token::HtmlContent(content) => {
                self.output.push_str(content);
                self.pos += 1;
            },
            Token::OutputKeyword => {
                self.pos += 1;
                match self.evaluate_expression() {
                    Ok(value) => self.output.push_str(&value.interal_value_as_string()),
                    Err(e) => self.output.push_str(&format!("<!-- Error evaluating expression: {} -->", e))
                }
            },
            Token::LetKeyword => {
                if let Err(e) = self.handle_assignment() {
                    self.output.push_str(&format!("<!-- Error in assignment: {} -->", e));
                }
            },
            Token::IfKeyword => {
                if let Err(e) = self.handle_if_statement() {
                    self.output.push_str(&format!("<!-- Error in if statement: {} -->", e));
                }
            },
            Token::ForKeyword => {
                if let Err(e) = self.handle_for_loop() {
                    self.output.push_str(&format!("<!-- Error in for loop: {} -->", e));
                }
            },
            Token::WhileKeyword => {
                if let Err(e) = self.handle_while_loop() {
                    self.output.push_str(&format!("<!-- Error in while loop: {} -->", e));
                }
            },
            Token::Identifier(name) => {
                let var_name = name.clone();
                self.pos += 1;
                
                if let Err(e) = self.handle_variable_access(&var_name, true) {
                    self.output.push_str(&format!("<!-- Error accessing variable {}: {} -->", var_name, e));
                }
            },
            Token::EndOfStatement => {
                self.pos += 1;
            },
            _ => {
                // Skip other tokens
                self.pos += 1;
            }
        }
        Ok(())
    }
    
    fn evaluate_condition(&mut self) -> Result<bool, String> {
        match self.evaluate_expression() {
            Ok(expr_value) => {
                match expr_value {
                    Obj::Boolean(b) => Ok(b),
                    Obj::Numerical(n) => Ok(n != 0.0),
                    Obj::Str(s) => Ok(!s.is_empty()),
                    Obj::List(l) => Ok(!l.is_empty()),
                    Obj::Dictionary(d) => Ok(!d.is_empty()),
                    Obj::None => Ok(false),
                }
            },
            Err(e) => {
                // Instead of failing, return false for invalid conditions
                self.output.push_str(&format!("<!-- Error in condition: {} -->", e));
                Ok(false)
            }
        }
    }
    
    fn evaluate_expression(&mut self) -> Result<Obj, String> {
        self.parse_expression(0)
    }
    
    // A recursive descent parser for expressions
    fn parse_expression(&mut self, precedence: u8) -> Result<Obj, String> {
        let mut left = self.parse_primary()?;
        
        while self.pos < self.tokens.len() {
            let current_precedence = self.get_operator_precedence();
            
            if current_precedence <= precedence {
                break;
            }
            
            left = self.parse_binary_op(left, current_precedence)?;
        }
        
        if matches!(self.tokens.get(self.pos), Some(Token::EndOfStatement)) {
            self.pos += 1;
        }
        
        Ok(left)
    }
    
    fn parse_primary(&mut self) -> Result<Obj, String> {
        if self.pos >= self.tokens.len() {
            return Err("Unexpected end of input while parsing expression".to_string());
        }
        
        match &self.tokens[self.pos] {
            Token::Object(obj) => {
                let value = obj.clone();
                self.pos += 1;
                Ok(value)
            },
            Token::Identifier(name) => {
                let var_name = name.clone();
                self.pos += 1;
                
                // Use the centralized variable access method
                match self.handle_variable_access(&var_name, false)? {
                    Some(value) => Ok(value),
                    None => Ok(Obj::None) // Should never happen in expression context
                }
            },
            Token::LeftParen => {
                self.pos += 1;
                let expr = self.evaluate_expression()?;
                
                if matches!(self.tokens.get(self.pos), Some(Token::RightParen)) {
                    self.pos += 1;
                    
                    // After a parenthesized expression, check if there's property access or indexing
                    if matches!(self.tokens.get(self.pos), Some(Token::Dot)) {
                        self.handle_property_access(expr)
                    } else if matches!(self.tokens.get(self.pos), Some(Token::LeftSquareBracket)) {
                        self.handle_indexing(expr)
                    } else {
                        Ok(expr)
                    }
                } else {
                    Err("Expected closing parenthesis".to_string())
                }
            },
            Token::Minus => {
                self.pos += 1;
                let value = self.parse_expression(100)?; // High precedence for unary operators
                
                match value {
                    Obj::Numerical(n) => Ok(Obj::Numerical(-n)),
                    _ => Err("Unary minus can only be applied to numbers".to_string()),
                }
            },
            Token::LogicalNot => {
                self.pos += 1;
                let value = self.parse_expression(100)?; // High precedence for unary operators
                
                match value {
                    Obj::Boolean(b) => Ok(Obj::Boolean(!b)),
                    _ => Ok(Obj::Boolean(false)), // Any non-boolean value is treated as falsy
                }
            },
            _ => Err(format!("Unexpected token in expression: {:?}", self.tokens[self.pos])),
        }
    }
    
    // Handle property access with dot notation
    fn handle_property_access(&mut self, obj: Obj) -> Result<Obj, String> {
        self.pos += 1; // Skip the dot
        
        if let Some(Token::Identifier(prop_name)) = self.tokens.get(self.pos).cloned() {
            self.pos += 1;
            
            match &obj {
                Obj::List(list) => {
                    match prop_name.as_str() {
                        "len" => Ok(Obj::Numerical(list.len() as f64)),
                        // Add other list methods as needed
                        _ => Err(format!("No property/method '{}' on list", prop_name)),
                    }
                },
                Obj::Dictionary(dict) => {
                    match prop_name.as_str() {
                        "len" => Ok(Obj::Numerical(dict.len() as f64)),
                        // Try to get the property by name
                        _ => {
                            if let Some(val) = dict.get(&prop_name) {
                                Ok(val.clone())
                            } else {
                                Err(format!("Key '{}' not found in dictionary", prop_name))
                            }
                        }
                    }
                },
                Obj::Str(s) => {
                    match prop_name.as_str() {
                        "len" => Ok(Obj::Numerical(s.len() as f64)),
                        // Add other string methods as needed
                        _ => Err(format!("No property/method '{}' on string", prop_name)),
                    }
                },
                _ => Err(format!("Type '{}' does not support properties/methods", obj.type_of()))
            }
        } else {
            Err("Expected identifier after dot".to_string())
        }
    }
    
    // Handle indexing with [] syntax
    fn handle_indexing(&mut self, obj: Obj) -> Result<Obj, String> {
        self.pos += 1; // Skip the left bracket
        
        let index = self.evaluate_expression()?;
        
        if !matches!(self.tokens.get(self.pos), Some(Token::RightSquareBracket)) {
            return Err("Expected closing bracket after index".to_string());
        }
        self.pos += 1; // Skip the right bracket
        
        match &obj {
            Obj::List(list) => {
                match &index {
                    Obj::Numerical(n) => {
                        let idx = *n as usize;
                        if idx < list.len() {
                            Ok(list[idx].clone())
                        } else {
                            Err(format!("Index {} out of bounds for list", idx))
                        }
                    },
                    _ => Err("List index must be a number".to_string())
                }
            },
            Obj::Dictionary(dict) => {
                // Allow both string literals and string expressions as keys
                let key = index.interal_value_as_string();
                if let Some(val) = dict.get(&key) {
                    Ok(val.clone())
                } else {
                    Err(format!("Key '{}' not found in dictionary", key))
                }
            },
            _ => Err(format!("Cannot index into a {} value", obj.type_of()))
        }
    }
    
    fn parse_binary_op(&mut self, left: Obj, precedence: u8) -> Result<Obj, String> {
        match &self.tokens[self.pos] {
            Token::Plus => {
                self.pos += 1;
                let right = self.parse_expression(precedence)?;
                self.apply_binary_op(left, right, |a, b| a + b)
            },
            Token::Minus => {
                self.pos += 1;
                let right = self.parse_expression(precedence)?;
                self.apply_binary_op(left, right, |a, b| a - b)
            },
            Token::Multiply => {
                self.pos += 1;
                let right = self.parse_expression(precedence)?;
                self.apply_binary_op(left, right, |a, b| a * b)
            },
            Token::Divide => {
                self.pos += 1;
                let right = self.parse_expression(precedence)?;
                match right {
                    Obj::Numerical(r) if r == 0.0 => Err("Division by zero".to_string()),
                    _ => self.apply_binary_op(left, right, |a, b| a / b),
                }
            },
            Token::Modulus => {
                self.pos += 1;
                let right = self.parse_expression(precedence)?;
                match right {
                    Obj::Numerical(r) if r == 0.0 => Err("Modulo by zero".to_string()),
                    _ => self.apply_binary_op(left, right, |a, b| a % b),
                }
            },
            Token::Exponent => {
                self.pos += 1;
                let right = self.parse_expression(precedence)?;
                self.apply_binary_op(left, right, |a, b| a.powf(b))
            },
            Token::EqualsEquals => {
                self.pos += 1;
                let right = self.parse_expression(precedence)?;
                Ok(Obj::Boolean(left == right))
            },
            Token::NotEquals => {
                self.pos += 1;
                let right = self.parse_expression(precedence)?;
                Ok(Obj::Boolean(left != right))
            },
            Token::LessThan => {
                self.pos += 1;
                let right = self.parse_expression(precedence)?;
                self.apply_comparison_op(left, right, |a, b| a < b)
            },
            Token::LessThanEquals => {
                self.pos += 1;
                let right = self.parse_expression(precedence)?;
                self.apply_comparison_op(left, right, |a, b| a <= b)
            },
            Token::GreaterThan => {
                self.pos += 1;
                let right = self.parse_expression(precedence)?;
                self.apply_comparison_op(left, right, |a, b| a > b)
            },
            Token::GreaterThanEquals => {
                self.pos += 1;
                let right = self.parse_expression(precedence)?;
                self.apply_comparison_op(left, right, |a, b| a >= b)
            },
            Token::LogicalAnd => {
                self.pos += 1;
                
                // Short-circuit evaluation
                if !self.is_truthy(&left) {
                    return Ok(Obj::Boolean(false));
                }
                
                let right = self.parse_expression(precedence)?;
                Ok(Obj::Boolean(self.is_truthy(&left) && self.is_truthy(&right)))
            },
            Token::LogicalOr => {
                self.pos += 1;
                
                // Short-circuit evaluation
                if self.is_truthy(&left) {
                    return Ok(Obj::Boolean(true));
                }
                
                let right = self.parse_expression(precedence)?;
                Ok(Obj::Boolean(self.is_truthy(&left) || self.is_truthy(&right)))
            },
            _ => Err(format!("Unknown operator: {:?}", self.tokens[self.pos])),
        }
    }
    
    fn apply_binary_op<F>(&self, left: Obj, right: Obj, op: F) -> Result<Obj, String>
    where
        F: Fn(f64, f64) -> f64,
    {
        match (left, right) {
            (Obj::Numerical(l), Obj::Numerical(r)) => Ok(Obj::Numerical(op(l, r))),
            (Obj::Str(l), Obj::Str(r)) => {
                if op(1.0, 1.0) == 2.0 {
                    // Addition for strings is concatenation
                    Ok(Obj::Str(l + &r))
                } else {
                    Err("Only addition is supported for strings".to_string())
                }
            },
            (Obj::Str(s), Obj::Numerical(n)) => {
                if op(1.0, 1.0) == 2.0 {
                    // Addition: String + Number = String + String(Number)
                    Ok(Obj::Str(s + &n.to_string()))
                } else {
                    Err("Only addition is supported for strings".to_string())
                }
            },
            (Obj::Numerical(n), Obj::Str(s)) => {
                if op(1.0, 1.0) == 2.0 {
                    // Addition: Number + String = String(Number) + String
                    Ok(Obj::Str(n.to_string() + &s))
                } else {
                    Err("Only addition is supported for strings".to_string())
                }
            },
            (Obj::List(mut l), Obj::List(r)) => {
                if op(1.0, 1.0) == 2.0 {
                    // Concatenate lists
                    let mut result = l;
                    result.extend(r);
                    Ok(Obj::List(result))
                } else {
                    Err("Only addition is supported for lists".to_string())
                }
            },
            _ => Err("Type mismatch for binary operation".to_string()),
        }
    }
    
    fn apply_comparison_op<F>(&self, left: Obj, right: Obj, op: F) -> Result<Obj, String>
    where
        F: Fn(f64, f64) -> bool,
    {
        match (left, right) {
            (Obj::Numerical(l), Obj::Numerical(r)) => Ok(Obj::Boolean(op(l, r))),
            (Obj::Str(l), Obj::Str(r)) => {
                // Compare strings lexicographically
                if op(1.0, 2.0) {
                    // If op is < or <=
                    Ok(Obj::Boolean(l < r))
                } else {
                    // If op is > or >=
                    Ok(Obj::Boolean(l > r))
                }
            },
            _ => Err("Cannot compare different types".to_string()),
        }
    }
    
    fn get_operator_precedence(&self) -> u8 {
        if self.pos >= self.tokens.len() {
            return 0;
        }
        
        match self.tokens[self.pos] {
            Token::LogicalOr => 10,
            Token::LogicalAnd => 20,
            Token::EqualsEquals | Token::NotEquals => 30,
            Token::LessThan | Token::LessThanEquals | Token::GreaterThan | Token::GreaterThanEquals => 40,
            Token::Plus | Token::Minus => 50,
            Token::Multiply | Token::Divide | Token::Modulus => 60,
            Token::Exponent => 70,
            Token::Dot => 90, // Highest precedence for property access
            Token::LeftSquareBracket => 90, // Highest precedence for array indexing
            _ => 0,
        }
    }
    
    fn is_truthy(&self, value: &Obj) -> bool {
        match value {
            Obj::Boolean(b) => *b,
            Obj::Numerical(n) => *n != 0.0,
            Obj::Str(s) => !s.is_empty(),
            Obj::List(l) => !l.is_empty(),
            Obj::Dictionary(d) => !d.is_empty(),
            Obj::None => false,
        }
    }
} 
