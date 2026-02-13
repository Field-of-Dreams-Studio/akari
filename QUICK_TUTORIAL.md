# Akari Object Documentation 0.2.7 

## Basic usage 

Akari Object is a dynamic data structure implemented in rust. This can store a numerical, boolean, string, list or dictionary 

We can use the following 2 ways 

```rust 
use akari::Value; 
let a: Value = "some_string".into(); // Using Into for any primitive data type 

// Using macro 
use am::object; 
let b = object!("another_string"); 
let dict = object!({
    a: "string", 
    b: [1, 2, 3] 
}); 
``` 

We basically follows python’s design for API (which will never panic, a default value will be provided when operation fails). Safe methods are also provided, usually a `try_ ` will be added before the method name 

For example, 

```rust 
#[test] 
fn addition(){ 
    use super::super::value::Value; 
    let obj = object!({a: 1, b: true, c: "hello"});
    let mut new_obj = Value::new_dict(); 
    new_obj += object!({a: 1, b: true});
    let new_obj = new_obj + object!({c: "hello"}); 
    assert_eq!(obj, new_obj); 

    let mut list = object!([1, "pmine", true]);
    list += object!({dict: "dict"}); 
    println!("{:?}", list); // List([Numerical(1.0), Str("pmine"), Boolean(true), Dict({"dict": Str("dict")})]) 
} 

#[test] 
fn multiplication(){ 
    use super::super::value::Value; 
    let obj = object!({a: 1, b: true, c: "hello"});
    let mut new_obj = Value::new_dict(); 
    new_obj *= object!({a: 1, b: true});
    let new_obj = new_obj * object!({c: "hello"}); 
    println!("{}, {}", obj, new_obj); // {str c = "hello", bool b = true, num a = 1}, null 

    let mut list = object!([1, "pmine", true]);
    list *= object!({dict: "dict"}); 
    println!("{}", list); // null, object * list does not make sense 
} 
``` 

## Operation List 

## Iterations 

# Akari Template Documentation 0.2.7 

