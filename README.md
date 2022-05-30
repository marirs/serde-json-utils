# serde_json_utils

Utility functions for serde_json::Value.

### Requirements
- Rust 1.56+

### Usage
```toml
[dependencies]
serde-json-utils = "0.1.0"
```

### Example
- To skip null & empty entries from serde_json::Value
```rust
use serde_json_utils::skip_null_and_empty;

const DATA: &str = r###"
    [
        {
            "key1": null,
            "key2": "there is a value",
            "key3": {},
            "key4": [],
            "key5": [1, 2, 3, 3]
        },
        {
            "key1": "value in here",
            "key2": null
        },
        {
            "key1": "value in here",
            "key2": null
        }
    ]
    "###;

fn main() {
    let mut val: Value = from_str(DATA).unwrap();
    skip_null_and_empty(&mut val);
    
    println!("{:#?}", val);
}
```

---
License: MIT