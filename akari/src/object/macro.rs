// /// A macro to create an Object from a literal or expression. 
// /// It can handle dictionaries, lists, booleans, strings, and numeric values. 
// /// # Example 
// /// ```rust 
// /// use akari::Object; 
// /// use akari::object; 
// /// let num_obj = object!(3); 
// /// assert_eq!(num_obj, Object::Numerical(3.0)); 
// /// ```
// /// ```rust 
// /// use akari::Object;  
// /// use std::collections::HashMap; 
// /// use akari::object; 
// /// let list_obj = object!(["aaa", "bbb"]); 
// /// assert_eq!(list_obj, Object::List(vec![Object::Str("aaa".to_string()), Object::Str("bbb".to_string())]));  
// /// ``` 
// /// ```rust 
// /// use akari::Object; 
// /// use std::collections::HashMap; 
// /// use akari::object; 
// /// let obj_obj = object!({c: "p", b: ["aaa", "bbb"], u: 32});  
// /// assert_eq!(obj_obj, Object::Dictionary(HashMap::from([
// ///     ("c".to_string(), Object::Str("p".to_string())),
// ///     ("b".to_string(), Object::List(vec![Object::Str("aaa".to_string()), Object::Str("bbb".to_string())])), 
// ///     ("u".to_string(), Object::Numerical(32.0)),
// /// ]))); 
// /// ```
// /// ```rust 
// /// use akari::Object; 
// /// use std::collections::HashMap; 
// /// use akari::object; 
// /// let obj_obj = object!({
// ///     string: String::from("hello"), 
// ///     number: 42
// /// }); 
// /// ```
 
// #[macro_export]
// macro_rules! object {
//     // Dictionary: keys become Strings now.
//     ({ $( $key:ident : $value:tt ),* $(,)? }) => {{
//         let mut map = ::std::collections::HashMap::new();
//         $(
//             map.insert(stringify!($key).to_string(), object!($value));
//         )*
//         Object::Dictionary(map)
//     }}; 
//     // List
//     ([ $( $elem:tt ),* $(,)? ]) => {{
//         let mut vec = Vec::new();
//         $(
//             vec.push(object!($elem));
//         )*
//         Object::List(vec)
//     }};
//     // Booleans
//     (true) => {
//         Object::new(true)
//     };
//     (false) => {
//         Object::new(false)
//     };
//     // String literals
//     ($e:literal) => {
//         Object::new($e)
//     };
//     // Fallback for expressions (like numbers)
//     ($e:expr) => {
//         Object::new($e)
//     };
// }  

