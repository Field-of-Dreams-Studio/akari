use std::env;
use std::fs;
use std::process::exit;
use std::collections::HashMap;

use akari::TemplateManager; 
use akari::Object; 

fn main() {
    // Collect command-line arguments.
    let args: Vec<String> = env::args().collect();

    // We expect at least three arguments: executable name, command, and content (or file path).
    if args.len() < 3 {
        eprintln!("Usage: {} <command> <content|file_path> [key=value ...]", args[0]);
        exit(1);
    }

    let output = match args[1].as_str() {
        "render_string" => {
            // The second argument is the content.
            let content = &args[2]; 

            // Build the arguments HashMap from key=value pairs.
            let mut context: HashMap<String, Object> = HashMap::new();
            for arg in args.iter().skip(3) {
                if let Some(pos) = arg.find('=') {
                    let key = &arg[..pos];
                    let value = &arg[pos + 1..]; 
                    let value = Object::from_json(&value.to_string()).unwrap_or_else(|_| {
                        eprintln!("Failed to parse value: {}", key);
                        exit(1);
                    });  
                    context.insert(key.to_string(), value);
                } else {
                    eprintln!("Warning: ignoring malformed argument '{}'. Expected key=value.", arg);
                }
            }

            let template_manager = TemplateManager::new(""); 
            template_manager.render_string(content.clone(), &context) 
                .unwrap_or_else(|err| {
                    eprintln!("Failed to render template: {}", err);
                    exit(1);
                }) 
        }
        "render" => { 
            // The second argument is the file path.
            let file_path = &args[2]; 

            // Build the arguments HashMap from key=value pairs.
            let mut context: HashMap<String, Object> = HashMap::new();
            for arg in args.iter().skip(3) {
                if let Some(pos) = arg.find('=') {
                    let key = &arg[..pos];
                    let value = &arg[pos + 1..]; 
                    let value = Object::from_json(&value.to_string()).unwrap_or_else(|_| {
                        eprintln!("Failed to parse value: {}", key);
                        exit(1);
                    });  
                    context.insert(key.to_string(), value);
                } else {
                    eprintln!("Warning: ignoring malformed argument '{}'. Expected key=value.", arg);
                }
            } 

            let template_manager = TemplateManager::new(""); 
            template_manager.render(file_path, &context) 
                .unwrap_or_else(|err| {
                    eprintln!("Failed to render template: {}", err);
                    exit(1);
                }) 
        }
        unknown => {
            eprintln!("Unknown command: {}", unknown);
            exit(1);
        } 
    }; 
    
    println!("Output:\n{}", output); 
    exit(0)  
}
 
