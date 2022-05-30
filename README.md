# serde_json_utils
[![Linux Arm7](https://github.com/marirs/serde-json-utils/actions/workflows/linux_arm.yml/badge.svg)](https://github.com/marirs/serde-json-utils/actions/workflows/linux_arm.yml)
[![Linux x86_64](https://github.com/marirs/serde-json-utils/actions/workflows/linux_x86_64.yml/badge.svg)](https://github.com/marirs/serde-json-utils/actions/workflows/linux_x86_64.yml)
[![macOS intel](https://github.com/marirs/serde-json-utils/actions/workflows/macos_x86_64.yml/badge.svg)](https://github.com/marirs/serde-json-utils/actions/workflows/macos_x86_64.yml)
[![Windows](https://github.com/marirs/serde-json-utils/actions/workflows/windows.yml/badge.svg)](https://github.com/marirs/serde-json-utils/actions/workflows/windows.yml)

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