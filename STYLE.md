# Basic Naming Conventions 
### Variables
- Use snake_case for variable names.
- Prioritize clarity over brevity. 

```rust 
let user_id: String = ...;  // Good
let uid: String = ...;      // Avoid (ambiguous)
``` 

### Functions 

- Use snake_case for function names.
- Constructor-like functions (e.g., new, from_json) should also use snake_case. 
- Functions mimicing enum is allowed to use UpperCamelCase 

```rust 
fn parse_json(input: &str) -> Result<Value, ValueError> { ... }  // Good
fn ParseJson(...) { ... }                                       // Avoid 
fn LitUrl(input: &str) -> UrlPattern { ... }                    // Recommanded 
``` 

### Types (Structs, Enums) 
- Use PascalCase for type names. 

```rust 
struct Value { ... }   // Good
enum ValueError { ... }    // Good
``` 

# Failable Method Naming Conventions

Akari prioritizes usability through automatic type conversions. Methods should avoid panicking or returning `Result`/`Option` by default, but explicit error handling is provided when needed. Follow these rules:

---

### **1. `xxx()`: Non-Fallible (Usability-First)**  
**Signature**: `fn xxx(&self) -> T`  
**Use Case**  
Default methods for ergonomic, "just give me a value" scenarios. If conversion fails, **a type-specific default is returned** (e.g., `0` for integers, `""` for strings).  
**Examples**:  
```rust
// Returns i64, or 0 if conversion fails (e.g., invalid string, wrong type)
fn as_i64(&self) -> i64;

// Returns String, or "" if value is not string-convertible
fn as_string(&self) -> String;
```  
**Rules**:  
- Document defaults prominently (e.g., "Returns `0` for non-integer values").  
- Avoid for types without intuitive defaults (e.g., `Email`, `Uuid`). Use `try_xxx()` or `xxx_or()` instead.  

---

### **2. `xxx_or()`: Non-Fallible with Custom Default**  
**Signature**: `fn xxx_or(&self, default: T) -> T`  
**Use Case**  
When users need to explicitly define a fallback value.  
**Example**:  
```rust
// Returns the parsed i64 or the user-provided default
fn as_i64_or(&self, default: i64) -> i64;
```  
**Rules**:  
- Prefer this over `xxx()` for domain-specific defaults (e.g., `as_i64_or(42)`).  

---

### **3. `try_xxx()`: Fallible (Explicit Error Handling)**  
**Signature**: `fn try_xxx(&self) -> Result<T, Error>`  
**Use Case**  
For precise error handling. Returns `Ok(T)` on success or `Err(Error)` on failure (e.g., invalid format, type mismatch).  
**Example**:  
```rust
// Returns Ok(i64) if parsable, Err(Error) otherwise
fn try_as_i64(&self) -> Result<i64, ValueError>;
```  
**Rules**:  
- Use for all types where conversion can fail meaningfully (not just defaulting).  
- Prefer `Result` over `Option` to provide actionable error details.  

---

### **4. `xxx_unchecked()`: Panic on Failure (Use Sparingly!)**  
**Signature**: `fn xxx_unchecked(&self) -> T`  
**Use Case**  
For performance-critical code where the caller **guarantees** validity. Panics if conversion fails.  
**Example**:  
```rust
// Returns i64 if parsable, panics otherwise
fn as_i64_unchecked(&self) -> i64;
```  
**Rules**:  
- Add `# Panics` sections in documentation.  
- Reserve for rare, audited use cases. Prefer `try_xxx()` or `xxx_or()` in public APIs.  

---

### **Guidelines**  
1. **Order of Preference**:  
   ```rust
   try_xxx() > xxx_or() > xxx() // From safest to most "risky"
   ```  
2. **Avoid Ambiguity**:  
   - `as_string()` returns `String` (not `&str`) to avoid lifetime complexity.  
3. **Document Defaults**:  
   Clearly state what `xxx()` returns on failure (e.g., "`as_bool()` returns `false` for non-boolean values").  

---

### **Why This Works**  
- **Usability**: `xxx()` methods provide "just works" behavior for common cases.  
- **Safety**: `try_xxx()` and `xxx_or()` encourage explicit error handling.  
- **Rust Idioms**: Aligns with `unwrap_or()`, `try_parse()`, and `Default` conventions.  

---

This version clarifies intent, reduces ambiguity, and guides users toward safer patterns while preserving usability.