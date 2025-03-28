use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Token, Ident, braced, bracketed};
use syn::parse::{Parse, ParseStream, Result};
use proc_macro2::TokenStream as TokenStream2;

/// A macro to create an Object from a literal or expression.
/// It can handle dictionaries, lists, booleans, strings, and numeric values. 
#[proc_macro]
pub fn object(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as ObjectExpr);
    let expanded = generate_code(&expr);
    TokenStream::from(expanded)
}

// Define our custom syntax structures
enum ObjectExpr {
    Dict(Dict),
    List(List),
    Other(syn::Expr),
}

struct Dict {
    entries: Vec<(String, ObjectExpr)>,
}

struct List {
    items: Vec<ObjectExpr>,
}

// Custom parsing for dictionary
impl Parse for Dict {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        braced!(content in input);
        let mut entries = Vec::new();
        
        while !content.is_empty() {
            let key: Ident = content.parse()?;
            content.parse::<Token![:]>()?;
            let value: ObjectExpr = content.parse()?;
            
            entries.push((key.to_string(), value));
            
            if content.is_empty() {
                break;
            }
            
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            } else {
                break;
            }
        }
        
        Ok(Dict { entries })
    }
}

// Custom parsing for list
impl Parse for List {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        bracketed!(content in input);
        let mut items = Vec::new();
        
        while !content.is_empty() {
            let item: ObjectExpr = content.parse()?;
            items.push(item);
            
            if content.is_empty() {
                break;
            }
            
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            } else {
                break;
            }
        }
        
        Ok(List { items })
    }
}

// Implement parsing for our custom syntax
impl Parse for ObjectExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(syn::token::Brace) {
            let dict = Dict::parse(input)?;
            Ok(ObjectExpr::Dict(dict))
        } else if input.peek(syn::token::Bracket) {
            let list = List::parse(input)?;
            Ok(ObjectExpr::List(list))
        } else {
            // Any other expression
            let expr: syn::Expr = input.parse()?;
            Ok(ObjectExpr::Other(expr))
        }
    }
}

// Generate code for each type of ObjectExpr
fn generate_code(expr: &ObjectExpr) -> TokenStream2 {
    match expr {
        ObjectExpr::Dict(dict) => {
            let entries = dict.entries.iter().map(|(key, value)| {
                let value_code = generate_code(value);
                quote! {
                    map.insert(#key.to_string(), #value_code);
                }
            });
            
            quote! {{
                let mut map = ::std::collections::HashMap::new();
                #(#entries)*
                Object::Dictionary(map)
            }}
        },
        ObjectExpr::List(list) => {
            let items = list.items.iter().map(|item| {
                let item_code = generate_code(item);
                quote! {
                    vec.push(#item_code);
                }
            });
            
            quote! {{
                let mut vec = Vec::new();
                #(#items)*
                Object::List(vec)
            }}
        },
        ObjectExpr::Other(expr) => {
            match expr {
                syn::Expr::Lit(lit_expr) => {
                    match &lit_expr.lit {
                        syn::Lit::Bool(b) => {
                            let value = b.value;
                            quote! { Object::new(#value) }
                        },
                        syn::Lit::Str(s) => {
                            let value = &s.value();
                            quote! { Object::new(#value) }
                        },
                        syn::Lit::Int(_) | syn::Lit::Float(_) => {
                            quote! { Object::new(#expr) }
                        },
                        _ => quote! { Object::new(#expr) }
                    }
                },
                _ => quote! { Object::new(#expr) }
            }
        },
    }
} 
