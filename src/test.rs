use std::collections::HashMap;
use crate::object;

use super::{Object as Obj, TemplateManager};
use std::fs;
use std::path::Path;
#[test] 
fn test() -> Result<(), Box<dyn std::error::Error>> {
    // Set up test templates directory
    let template_dir = Path::new("./temp_templates");
    if !template_dir.exists() {
        fs::create_dir(template_dir)?;
    }
    
    // Create a base layout template
    let base_layout = r#"<!DOCTYPE html>
<html>
<head>
    <title>-[ title ]-</title>
    -[ block head ]-
    <!-- Default head content -->
    -[ endblock ]-
</head>
<body>
    <header>
        -[ block header ]-
        <h1>Default Site Header</h1>
        -[ endblock ]-
    </header>
    
    <main>
        -[ block content ]-
        <p>Default content - override this</p>
        -[ endblock ]-
    </main>
    
    <footer>
        -[ block footer ]-
        <p>&copy; 2025 Template Engine</p>
        -[ endblock ]-
    </footer>
</body>
</html>"#;
    
    fs::write(template_dir.join("base.html"), base_layout)?;
    
    // Create a page template that extends the base
    let page_template = r#"-[ template "base.html" ]-

-[ block head ]-
<link rel="stylesheet" href="style.css">
<meta name="description" content="My awesome page">
-[ endblock ]-

-[ block header ]-
<h1>-[ page_title ]-</h1>
<nav>
    <ul>
        <li><a href="/">Home</a></li>
        <li><a href="/about">About</a></li>
        <li><a href="/contact">Contact</a></li>
    </ul>
</nav>
-[ endblock ]-

-[ block content ]-
<div class="container">
    <h2>Welcome to our website</h2>
    
    -[ if show_message ]-
        <div class="message">-[ message ]-</div>
    -[ endif ]-
    
    <ul class="items">
        -[ for item items ]-
            <li class="item">-[ item ]-</li>
        -[ endfor ]-
    </ul>
</div>
-[ endblock ]-"#;
    
    fs::write(template_dir.join("home.html"), page_template)?;
    
    // Initialize the template manager
    let template_manager = TemplateManager::new(template_dir);
    
    // Set up template data
    let mut data = HashMap::new();
    data.insert("title".to_string(), Obj::Str("My Website - Home".to_string()));
    data.insert("page_title".to_string(), Obj::Str("Welcome to My Website".to_string()));
    data.insert("show_message".to_string(), Obj::Boolean(true));
    data.insert("message".to_string(), Obj::Str("Thank you for visiting!".to_string()));
    
    // Create a list for the for-loop
    let items = vec![
        Obj::Str("First item".to_string()),
        Obj::Str("Second item".to_string()),
        Obj::Str("Third item".to_string()),
    ];
    data.insert("items".to_string(), Obj::List(items));
    
    // Render the template
    let result = template_manager.render("home.html", &data)?;
    println!("Rendered Template:\n{}", result);
    
    // Clean up test directory
    // fs::remove_dir_all(template_dir)?;
    
    Ok(())
} 

#[test] 
fn test2() -> Result<(), Box<dyn std::error::Error>>{ 
    // Set up test templates directory
    let template_dir = Path::new("./temp_templates");
    if !template_dir.exists() {
        fs::create_dir(template_dir)?;
    }
    
    // Create a base layout template
    let base_layout = r#"<!DOCTYPE html>
<html>
<head>
    <title>-[ title ]-</title>
    -[ block head ]-
    <!-- Default head content -->
    -[ endblock ]-
</head>
<body>
    <header>
        -[ block header ]-
        <h1>Default Site Header</h1>
        -[ endblock ]-
    </header>
    
    <main>
        -[ block content ]-
        <p>Default content - override this</p>
        -[ endblock ]-
    </main>
    
    <footer>
        -[ block footer ]-
        <p>&copy; 2025 Template Engine</p>
        -[ endblock ]-
    </footer>
</body>
</html>"#;
    
    fs::write(template_dir.join("base.html"), base_layout)?;
    
    // Create a page template that extends the base
    let page_template = r#"-[ template "base.html" ]-

-[ block head ]-
<link rel="stylesheet" href="style.css">
<meta name="description" content="My awesome page">
-[ endblock ]-

-[ block header ]-
<h1>-[ page_title ]-</h1>
<nav>
    <ul>
        <li><a href="/">Home</a></li>
        <li><a href="/about">About</a></li>
        <li><a href="/contact">Contact</a></li>
    </ul>
</nav>
-[ endblock ]-

-[ block content ]-
<div class="container">
    <h2>Welcome to our website</h2>
    
    -[ if show_message ]-
        <div class="message">-[ message ]-</div>
    -[ endif ]-
    
    <ul class="items">
        -[ for item items ]-
            <li class="item">-[ item ]-</li>
        -[ endfor ]-
    </ul>
</div>
-[ endblock ]-"#;
    
    fs::write(template_dir.join("home.html"), page_template)?;
    
    // Initialize the template manager
    let template_manager = TemplateManager::new(template_dir);
    
    // Set up template data
    let mut data = HashMap::new();
    data.insert("title".to_string(), Obj::Str("My Website - Home".to_string()));
    data.insert("page_title".to_string(), Obj::Str("Welcome to My Website".to_string()));
    data.insert("show_message".to_string(), Obj::Boolean(true));
    data.insert("message".to_string(), Obj::Str("Thank you for visiting!".to_string()));
    
    // Create a list for the for-loop
    let items = vec![
        Obj::Str("First item".to_string()),
        Obj::Str("Second item".to_string()),
        Obj::Str("Third item".to_string()),
    ];
    data.insert("items".to_string(), Obj::List(items));
    
    // Render the template
    let result = template_manager.render("home.html", &data)?;
    println!("Rendered Template:\n{}", result);
    
    // Clean up test directory
    // fs::remove_dir_all(template_dir)?;
    
    Ok(()) 
}

#[test] 
fn test3() -> Result<(), Box<dyn std::error::Error>>{ 
    use crate::Object; 
    let page_template = r#"
<link rel="stylesheet" href="style.css">
<meta name="description" content="pageprop.desc">

<h1>-[ output pageprop["title"] ]-</h1>
"#; 
        
        // Initialize the template manager
        let template_manager = TemplateManager::new(Path::new("./temp_templates"));
        
        // Set up template data
        let mut data = HashMap::new();
        data.insert("pageprop".to_string(), object!({desc: "My Website - Home", title: "Welcome to My Website"})); 
        data.insert("title".to_string(), object!("111")); 
        
        // Render the template
        let result = template_manager.render_string(page_template.to_string(), &data)?;
        println!("Rendered Template:\n{}", result);
        
        Ok(()) 
} 