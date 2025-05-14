# Akari: Easy template language & Json Implementation  

install by 

`cargo install akari` 

Akari is consists of 2 components, one is Json Implementation for rust, and the second component is the template rendering language 

# Akari Json 

Macro is used in creating Akari Json. 

```rust 
use akari::object; 
object!({
    number: 3, 
    string: "Hello", 
    array: [1, 2, 3], 
    object: { 
        a: 1, 
        b: 2, 
        c: 3 
    }
}) 
``` 

Then you can create a Json. 

Where you can also use 

```rust 
use akari::object; 
use akari::Value; 

let json = r#"{"key": "value", "number": 42, "list": [1, 2, 3]}"#; 
let obj = Value::from_json(json).expect("Failed to parse JSON"); 
let dir = "D://test/test.json"; 
Value::from_jsonf(dir).unwrap_or(Object::None); // Read a json from a file 
obj.into_jsonf(dir); // Write obj into the dir 
``` 

While various of methods are provided to read a value in json. 

Be carefun about the difference between `obj.to_string()`, `obj.string()` and `obj.into_json()` 

# Templating 

run to render a template 

`akari render_string "-[ output aaa ]-" aaa=1` 

output: `1` 

Read more in starberry example to find out how to write Akari template 

https://github.com/Field-of-Dreams-Studio/starberry-example/tree/main 

# Contributing 

You are free to contribute! 

Read more in STYLE.md 

# Update log 

0.2.3-rc1: Rename Akari's method & Seperate Object mods and Template mod. Change the name of Object into Value 

0.2.2: Debug insert and inheretance of templates 

0.2.2-rc1: Enabled template caching, keyword "insert" now in used to insert another template into a template 

0.2.1: Update documentations for Akari 

0.2.1-rc1: Enabling getting value from the Object through one function, no need for a match statement 

0.2.0: Enable json file read and write 

0.2.0-rc1: Update the macro, enable using complex expression and functions in the macro 

0.1.3: Important Bug Fix: Now template will not causing rendering empty HTML 

0.1.2: Changed object! macro, enable nesting objects 

0.1.1: Enable [] operation and . operation 

0.1.0: Initial Commit 
