# Akari: Fast, Easy template language with builtin Json support 

install by 

`cargo install akari` 

run to render a template 

`akari render_string "-[ output aaa ]-" aaa=1` 

output: `1` 

This crate is used by Starberry since 0.3.0 

[https://crates.io/crates/starberry](https://crates.io/crates/starberry) 

# Examples 

(Examples will be created soon) 

# Update log 

0.2.0-rc1: Update the macro, enable using complex expression and functions in the macro 

0.1.3: Important Bug Fix: Now template will not causing rendering empty HTML 

0.1.2: Changed object! macro, enable nesting objects 

0.1.1: Enable [] operation and . operation 

0.1.0: Initial Commit 
