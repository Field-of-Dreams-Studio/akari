use std::collections::HashMap; 
use std::fs; 
use std::io::Read; 
use crate::Value;
use crate::TemplateManager;

use super::parse; 
use super::compile; 

#[macro_export]
macro_rules! insert_entry {
    ($map:expr, $key:ident = $value:literal) => {
        $map.insert(stringify!($key).to_string(), object!($value));
    };
    ($map:expr, $key:ident = $value:expr) => {
        $map.insert(stringify!($key).to_string(), $value);
    };
}

/// Macro to read a file, build the arguments HashMap, and call `parse_initernal`.
#[macro_export]
macro_rules! parse_file {
    ($path:expr, $($key:ident = $value:tt),* $(,)?) => {{
        // Read the file into bytes
        let bytes = std::fs::read($path)
            .expect(&format!("Failed to read file: {}", $path));
        let mut map = std::collections::HashMap::new();
        $(
            insert_entry!(map, $key = $value);
        )*
        render_bytes(&bytes, &mut map)
    }};
}

/// Macro to take a bytes slice, build the arguments HashMap, and call `parse_initernal`.
#[macro_export]
macro_rules! parse_bytes {
    ($bytes:expr, $($key:ident = $value:tt),* $(,)?) => {{
        let mut map = std::collections::HashMap::new();
        $(
            insert_entry!(map, $key = $value);
        )*
        render_bytes($bytes, &mut map)
    }};
}

/// Macro to take any value that can be converted into a String,
/// convert it into a byte vector, build the arguments HashMap, and call `parse_initernal`.
#[macro_export]
macro_rules! parse_string {
    ($s:expr, $($key:ident = $value:tt),* $(,)?) => {{
        let bytes = $s.to_string().into_bytes();
        let mut map = std::collections::HashMap::new();
        $(
            insert_entry!(map, $key = $value);
        )*
        render_bytes(&bytes, &mut map)
    }};
} 

#[cfg(test)] 
mod tests {
    use super::*; 
    use crate::object; 

    #[test]
    fn test_tokenize() {
        let input = r#"
        -[ template "template.html" ]-
        -[ block header ]-
            <script src="pmine.org"></script>
        -[ endblock ]-

        -[ block body ]-
            -[ let a = 1 ]-
            -[ for str in list ]-
                -[ if (a % 2 == 0) ]- 
                    -[ output str ]- 
                -[ endif ]- 
                -[ a = a + 1 ]-
            -[ endfor ]-
        -[ endblock ]- 
        "#; 
        let mut data = HashMap::new(); 
        data.insert("list".to_string(), object!(vec![        object!("a"), 
        object!("b"), 
        object!("c")])); 
        // println!("{:?}", render_string(input, data)); 
    } 
} 
 
 