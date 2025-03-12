use std::collections::HashMap;
use crate::object::Object as Obj; 
use crate::parse::Token; 

use super::parse;
use super::object;

pub fn compile(tokens: Vec<Token>, mut data: HashMap<String, Obj>) -> Result<String, String> {
    let mut compiler = TemplateCompiler::new(tokens, data); 
    compiler.compile()
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
        // println!("tokens: {:?}\n\n\n", self.tokens); 
        // First pass: identify blocks and template info
        self.collect_blocks_and_metadata()?; 
        // println!("tokens: {:?}\n\n\n", self.tokens); 
        
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
            // println!("Processing token: {:?}, {:?}, current_string: {}\n\n\n", self.tokens[self.pos], self.tokens.get(self.pos+1).unwrap_or_else(|| {&Token::EndOfStatement}), self.output);  
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
                            // FIXED: Use empty content for missing blocks instead of treating as variable
                            self.output.push_str(&format!("<!-- Block '{}' not defined -->", block_name));
                        }
                        self.pos += 1;
                    } else {
                        return Err("Expected identifier after placeholder keyword".to_string());
                    }
                },
                Token::BlockKeyword => {
                    // Skip over blocks in the second pass, as they're already processed
                    // Find and skip to the end of this block
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
                    self.pos += 1;
                    if let Some(Token::Identifier(name)) = self.tokens.get(self.pos) {
                        let name = name.clone(); // Clone the name to own it
                        self.data.remove(&name);
                        self.pos += 1;
                        
                        // Handle dictionary deletion
                        if self.pos < self.tokens.len() && matches!(self.tokens[self.pos], Token::LeftSquareBracket) {
                            self.pos += 1;
                            
                            // Evaluate the key and store it
                            let key_string = {
                                let key = self.evaluate_expression()?;
                                key.interal_value_as_string()
                            };
                
                            // Now we can safely use both name and key_string
                            if matches!(self.tokens.get(self.pos), Some(Token::RightSquareBracket)) {
                                self.pos += 1;
                                
                                // Get the dictionary and remove the key
                                if let Some(Obj::Dictionary(dict)) = self.data.get_mut(&name) {
                                    dict.remove(&key_string);
                                }
                            } else {
                                return Err("Expected closing bracket after dictionary key".to_string());
                            }
                        }
                    } else {
                        return Err("Expected identifier after del keyword".to_string());
                    }
                }, 
                Token::Identifier(name) => {
                    let var_name = name.clone();
                    self.pos += 1;
                    
                    // Check for array/dictionary access
                    if matches!(self.tokens.get(self.pos), Some(Token::LeftSquareBracket)) {
                        self.pos += 1;
                        let index = self.evaluate_expression()?;
                        
                        if !matches!(self.tokens.get(self.pos), Some(Token::RightSquareBracket)) {
                            return Err("Expected closing bracket after array/dictionary index".to_string());
                        }
                        self.pos += 1;
                        
                        // Handle assignment to indexed element
                        if matches!(self.tokens.get(self.pos), Some(Token::Assignment)) {
                            self.pos += 1;
                            let value = self.evaluate_expression()?;
                            
                            // Update collection
                            if let Some(collection) = self.data.get_mut(&var_name) {
                                match collection {
                                    Obj::List(list) => {
                                        if let Obj::Numerical(i) = &index {
                                            let idx = *i as usize;
                                            if idx < list.len() {
                                                list[idx] = value;
                                            } else {
                                                return Err(format!("Index {} out of bounds for list {}", idx, var_name));
                                            }
                                        } else {
                                            return Err("List index must be a number".to_string());
                                        }
                                    },
                                    Obj::Dictionary(dict) => {
                                        dict.insert(index.interal_value_as_string(), value);
                                    },
                                    _ => return Err(format!("{} is not a collection that can be indexed", var_name)),
                                }
                            } else {
                                return Err(format!("Variable {} not found", var_name));
                            }
                        } else {
                            // Just accessing, add result to output
                            let value = if let Some(collection) = self.data.get(&var_name) {
                                match collection {
                                    Obj::List(list) => {
                                        if let Obj::Numerical(i) = &index {
                                            let idx = *i as usize;
                                            if idx < list.len() {
                                                list[idx].clone()
                                            } else {
                                                return Err(format!("Index {} out of bounds for list {}", idx, var_name));
                                            }
                                        } else {
                                            return Err("List index must be a number".to_string());
                                        }
                                    },
                                    Obj::Dictionary(dict) => {
                                        dict.get(&index.interal_value_as_string()).cloned().unwrap_or(Obj::None)
                                    },
                                    _ => return Err(format!("{} is not a collection that can be indexed", var_name)),
                                }
                            } else {
                                return Err(format!("Variable {} not found", var_name));
                            };
                            
                            self.output.push_str(&value.interal_value_as_string());
                        }
                    } else if matches!(self.tokens.get(self.pos), Some(Token::Assignment)) {
                        // Simple variable assignment
                        self.pos += 1;
                        let value = self.evaluate_expression()?;
                        self.data.insert(var_name, value);
                    } else {
                        // Just outputting the variable value
                        if let Some(value) = self.data.get(&var_name) {
                            self.output.push_str(&value.interal_value_as_string());
                        } else {
                            return Err(format!("Variable {} not found", var_name));
                        }
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
    
    fn handle_if_statement(&mut self) -> Result<(), String> {
        self.pos += 1; // Skip if keyword
        
        let condition = self.evaluate_condition()?;
        
        if condition {
            // Process the if block
            while self.pos < self.tokens.len() {
                match &self.tokens[self.pos] {
                    Token::EndIfKeyword => {
                        self.pos += 1;
                        break;
                    },
                    _ => {
                        self.process_token()?;
                    }
                }
            }
        } else {
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
                    
                    for (k, _) in entries {
                        self.data.insert(loop_var.clone(), Obj::Str(k));
                        
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
                _ => return Err("For loop requires a list, dictionary, or number".to_string()),
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
                Err(e) => return Err(e),
            }
            
            iteration += 1;
            if iteration == MAX_ITERATIONS {
                return Err("While loop exceeded maximum iterations - possible infinite loop".to_string());
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
                let value = self.evaluate_expression()?;
                self.output.push_str(&value.interal_value_as_string());
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
        let expr_value = self.evaluate_expression()?;
        
        match expr_value {
            Obj::Boolean(b) => Ok(b),
            Obj::Numerical(n) => Ok(n != 0.0),
            Obj::Str(s) => Ok(!s.is_empty()),
            Obj::List(l) => Ok(!l.is_empty()),
            Obj::Dictionary(d) => Ok(!d.is_empty()),
            Obj::None => Ok(false),
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
                
                // Check for array/dictionary access
                if matches!(self.tokens.get(self.pos), Some(Token::LeftSquareBracket)) {
                    self.pos += 1;
                    let index = self.evaluate_expression()?;
                    
                    if matches!(self.tokens.get(self.pos), Some(Token::RightSquareBracket)) {
                        self.pos += 1;
                        
                        // Access collection element
                        if let Some(collection) = self.data.get(&var_name) {
                            match collection {
                                Obj::List(list) => {
                                    if let Obj::Numerical(i) = &index {
                                        let idx = *i as usize;
                                        if idx < list.len() {
                                            Ok(list[idx].clone())
                                        } else {
                                            Err(format!("Index {} out of bounds for list {}", idx, var_name))
                                        }
                                    } else {
                                        Err("List index must be a number".to_string())
                                    }
                                },
                                Obj::Dictionary(dict) => {
                                    Ok(dict.get(&index.interal_value_as_string()).cloned().unwrap_or(Obj::None))
                                },
                                _ => Err(format!("{} is not a collection that can be indexed", var_name)),
                            }
                        } else {
                            Err(format!("Variable {} not found", var_name))
                        }
                    } else {
                        Err("Expected closing bracket after array/dictionary index".to_string())
                    }
                } else {
                    // Simple variable access
                    if let Some(value) = self.data.get(&var_name) {
                        Ok(value.clone())
                    } else {
                        Err(format!("Variable {} not found", var_name))
                    }
                }
            },
            Token::LeftParen => {
                self.pos += 1;
                let expr = self.evaluate_expression()?;
                
                if matches!(self.tokens.get(self.pos), Some(Token::RightParen)) {
                    self.pos += 1;
                    Ok(expr)
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

