### **Akari: Dynamic & Weakly Typed Programming Powered by Rust**
```bash
cargo install akari
``` 

https://fds.rs/akari/ 

---

### **Core Components**
| Component      | Feature Flag      | Description                                                                 |
|----------------|-------------------|-----------------------------------------------------------------------------|
| **Akari Value**| `dynamic` & `object_macro`   | JSON implementation with macros and file I/O                                |
| **Extensions** | `extension`       | Type/string-based storage for middleware/app logic                          |
| **Templating** | `template`        | HTML template engine with inheritance and caching                           |

---

### **1. Akari Value (JSON)**
**Key Features:**
```rust
// Create objects
use akari::object;
let data = object!({
    number: 3, 
    nested: { 
        list: [1, 2, 3] 
    }
});

// Parse/emit JSON
let obj = Value::from_json(r#"{"key":"value"}"#)?;
obj.into_jsonf("data.json")?;  // Write to file
```

**Important Methods:**
- `to_string()`: Debug representation
- `string()`: Extract string value
- `into_json()`: Serialize to JSON string
- `is_dict()`/`is_list()`: Type checks

> Enable `object_macro` feature for `object!` syntax

---

### **2. Extensions System**
**Type-Based Storage (`Params`):**
```rust
let mut params = Params::new();
params.set(42u8);  // Store by type
params.get_mut::<u8>().map(|n| *n += 1); 
```

**String-Based Storage (`Locals`):**
```rust
let mut locals = Locals::new();
locals.set("counter", 0i32);  // Store by key
locals.keys();  // ["counter"]
```

**Cloneable Variants:**
- `ParamsClone`: Cloneable type storage
- `LocalsClone`: Cloneable key-value storage
- Methods: `combine()` (no overwrite), `merge()` (overwrite)

**Bridge Storage Types:**
```rust
locals.export_param(&params, "exported_value"); 
```

---

### **3. Templating Engine**
**Render Templates:**
```bash
akari render_string "-[output var]-" var=42  # Output: 42
```

**Key Features:**
- Template inheritance with `insert`
- File-based template caching
- Logic control structures

> See [Starberry Examples](https://github.com/Field-of-Dreams-Studio/starberry-example) for usage patterns

---

### **Development & Contribution**
**Style Guidelines:**  
Refer to `STYLE.md` for coding standards

**Update Log Highlights:**
| Version  | Key Changes                                      |
|----------|-------------------------------------------------|
| **0.2.7**| **ValueParser trait redesign with streaming support (final 0.2.x)** |
| 0.2.6    | Documentation updates, full features enabled    |
| 0.2.5    | Safer `into_json`, operator implementations     |
| 0.2.4    | Added `is_<type>()` and `contains()` methods    |
| 0.2.3    | Renamed types, separated value/template modules |
| 0.2.2    | Template caching, `insert` keyword support      |
| 0.1.3    | Critical empty HTML rendering fix               |

> Full changelog available in source documentation

---

**Security Note:** Always validate untrusted JSON input and template variables in production environments.