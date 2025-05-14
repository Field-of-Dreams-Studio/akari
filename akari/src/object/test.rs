#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use akari_macro::object; 
    use super::super::value::*; 
    
    #[test]
    fn test_from_json_object() {
        let json = r#"{"a": 1, "b": true, "c": "hello"}"#;
        let obj = Value::from_json(json).expect("Failed to parse JSON");
        let mut expected_map = HashMap::new();
        expected_map.insert("a".to_string(), Value::Numerical(1.0));
        expected_map.insert("b".to_string(), Value::Boolean(true));
        expected_map.insert("c".to_string(), Value::Str("hello".to_string()));
        assert_eq!(obj, Value::Dictionary(expected_map));
    }
    
    #[test]
    fn test_from_json_array() {
        let json = r#"[1, 2, 3]"#;
        let obj = Value::from_json(json).expect("Failed to parse JSON");
        assert_eq!(obj, Value::List(vec![
            Value::Numerical(1.0),
            Value::Numerical(2.0),
            Value::Numerical(3.0)
        ]));
    }
    
    #[test]
    fn test_from_json_nested() {
        let json = r#"{"a": [true, false], "b": {"nested": "value"}}"#;
        let obj = Value::from_json(json).expect("Failed to parse JSON");
        // Further assertions can be added here.
    } 

    #[test] 
    fn test_object_macro() {
        let obj = object!({a: 1, b: true, c: "hello"});
        let mut expected_map = HashMap::new();
        expected_map.insert("a".to_string(), Value::Numerical(1.0));
        expected_map.insert("b".to_string(), Value::Boolean(true));
        expected_map.insert("c".to_string(), Value::Str("hello".to_string()));
        assert_eq!(obj, Value::Dictionary(expected_map));
    } 

    #[test] 
    fn test_object_macro_expr() { 
        let a = 1; 
        let obj = object!({a: a, b: [1, 2, 3], c: {
            hello: "world"
        }});
        let mut expected_map = HashMap::new();
        expected_map.insert("a".to_string(), Value::Numerical(1.0));
        expected_map.insert("b".to_string(), Value::List(vec![
            Value::Numerical(1.0),
            Value::Numerical(2.0),
            Value::Numerical(3.0)
        ]));
        expected_map.insert("c".to_string(), Value::Dictionary({
            let mut inner_map = HashMap::new();
            inner_map.insert("hello".to_string(), Value::Str("world".to_string()));
            inner_map
        })); 
        // assert_eq!(obj, Value::Dictionary(expected_map)); 
        println!("{:?}", obj.format()); 
        println!("{:?}", obj.into_json()); 
    } 

    #[test]
    fn test_numerical_object() {
        let num_obj = object!(3);
        assert_eq!(num_obj, Value::Numerical(3.0));
    }

    #[test]
    fn test_list_object() {
        let list_obj = object!(["aaa", "bbb"]);
        assert_eq!(
            list_obj, 
            Value::List(vec![
                Value::Str("aaa".to_string()), 
                Value::Str("bbb".to_string())
            ])
        );
    }

    #[test]
    fn test_dictionary_object() {
        let obj_obj = object!({c: String::from("p"), b: [String::from("aaa"), "bbb"], u: 32}); 
        println!("{:?}", obj_obj.format()); 
        assert_eq!(
            obj_obj, 
            Value::Dictionary(HashMap::from([
                ("c".to_string(), Value::Str("p".to_string())),
                ("b".to_string(), Value::List(vec![
                    Value::Str("aaa".to_string()), 
                    Value::Str("bbb".to_string())
                ])),
                ("u".to_string(), Value::Numerical(32.0)),
            ]))
        );
    }

    #[test]
    fn test_complex_expressions() {
        let obj_obj = object!({
            string: String::from("hello"),
            number: 42
        });
        
        let expected = {
            let mut map = HashMap::new();
            map.insert("string".to_string(), Value::Str("hello".to_string()));
            map.insert("number".to_string(), Value::Numerical(42.0));
            Value::Dictionary(map)
        };
        
        assert_eq!(obj_obj, expected);
    }

    #[test]
    fn test_nested_objects() {
        let nested_obj = object!({
            name: "nested_test",
            properties: {
                boolean: true,
                list: [1, 2, 3],
                complex: {
                    value: String::from("nested value"),
                    flag: false
                }
            }
        });
        
        // Building expected object manually for verification
        let mut inner_complex = HashMap::new();
        inner_complex.insert("value".to_string(), Value::Str("nested value".to_string()));
        inner_complex.insert("flag".to_string(), Value::Boolean(false));
        
        let mut properties = HashMap::new();
        properties.insert("boolean".to_string(), Value::Boolean(true));
        properties.insert("list".to_string(), Value::List(vec![
            Value::Numerical(1.0),
            Value::Numerical(2.0),
            Value::Numerical(3.0)
        ]));
        properties.insert("complex".to_string(), Value::Dictionary(inner_complex));
        
        let mut root = HashMap::new();
        root.insert("name".to_string(), Value::Str("nested_test".to_string()));
        root.insert("properties".to_string(), Value::Dictionary(properties));
        
        let expected = Value::Dictionary(root);
        
        assert_eq!(nested_obj, expected);
    }  

    #[test] 
    fn test_object_6(){ 
        let obj = object!({
            a: 1, 
            b: true, 
            c: "hello", 
            d: [1, 2, 3], 
            e: {x: 10, y: 20}, 
            f: {
                a: [String::from("1"), String::from("2")], 
                b: ["1", "2"], 
                c: [String::from("1"), "2"]
            }, 
            g: [ 
                {a: String::from("1"), b: 2}, 
                {a: 3, b: 4}, 
                {a: String::from("1"), b: String::from("1")} 
            ]
        }); 
        println!("{:?}", obj.into_json()); 
    } 
    #[test] 
    fn get_key_test(){
        let a = object!({ 
            a: 1, 
            b: true, 
            c: "hello", 
            d: [1, 2, 3], 
            e: {x: 10, y: 20}, 
            f: {
                a: [String::from("1"), String::from("2")], 
                b: ["1", "2"], 
                c: [String::from("1"), "2"]
            }, 
            g: [ 
                {a: String::from("1"), b: 2}, 
                {a: 3, b: 4}, 
                {a: String::from("1"), b: String::from("1")} 
            ]
        }).get("f").get("a").idx(0).unwrap_or(&Value::None)
        .to_string(); 
        assert_eq!(a, "\"1\""); 
    }
}

