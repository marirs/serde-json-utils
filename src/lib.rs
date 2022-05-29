use ordered_float::NotNan;
use serde_json::Value::{self, *};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

mod error;
use error::Error;
type Result<T> = std::result::Result<T, Error>;

#[derive(PartialEq, Debug)]
struct HashValue<'a>(pub &'a Value);

impl Eq for HashValue<'_> {}

impl Hash for HashValue<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.0 {
            Null => state.write_u32(3_221_225_473), // chosen randomly
            Bool(ref b) => b.hash(state),
            Number(ref n) => {
                if let Some(x) = n.as_u64() {
                    x.hash(state);
                } else if let Some(x) = n.as_i64() {
                    x.hash(state);
                } else if let Some(x) = n.as_f64() {
                    // `f64` does not implement `Hash`. However, floats in JSON are guaranteed to be
                    // finite, so we can use the `Hash` implementation in the `ordered-float` crate.
                    NotNan::new(x).unwrap().hash(state);
                }
            }
            String(ref s) => s.hash(state),
            Array(ref v) => {
                for x in v {
                    HashValue(x).hash(state);
                }
            }
            Object(ref map) => {
                let mut hash = 0;
                for (k, v) in map {
                    // We have no way of building a new hasher of type `H`, so we
                    // hardcode using the default hasher of a hash map.
                    let mut item_hasher = DefaultHasher::new();
                    k.hash(&mut item_hasher);
                    HashValue(v).hash(&mut item_hasher);
                    hash ^= item_hasher.finish();
                }
                state.write_u64(hash);
            }
        }
    }
}

pub fn skip_null(val: &mut Value) {
    match val{
        Null => {}
        Bool(_) => {}
        Number(_) => {}
        String(_) => {}
        Array(a) => {
            for v in a {
                skip_null(v);
            }
        }
        Object(o) => {
            let mut candidates = vec![];
            for (k, v) in o.iter_mut() {
                if let serde_json::Value::Null = v {
                    candidates.push(k.to_string());
                } else {
                    skip_null(v);
                }
            }
            for c in candidates{
                o.remove(&c);
            }
        }
    }
}

pub fn skip_null_and_empty(val: &mut Value) -> Result<bool>{
    match val{
        Null => {return Ok(true);}
        Bool(_) => {}
        Number(_) => {}
        String(_) => {}
        Array(a) => {
            if a.is_empty(){
                return Ok(true);
            }
            for v in a{
                skip_null_and_empty(v);
            }
        }
        Object(o) => {
            if o.is_empty(){
                return Ok(true);
            }
            let mut candidates = vec![];
            for (k, v) in o.iter_mut() {
                if skip_null_and_empty(v)?{
                    candidates.push(k.to_string());
                }
            }
            for c in candidates{
                o.remove(&c);
            }
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATA: &str = r###"
    [
        {
            "key1": null,
            "key2": "there is a value",
            "key3": {},
            "key4": [],
            "key5": [1, 2, 3]
        },
        {
            "key1": "value in here",
            "key2": null
        }
    ]
    "###;

    const RESULT_SKIP_NULL: &str = r###"
    [
        {
            "key2": "there is a value",
            "key3": {},
            "key4": [],
            "key5": [1, 2, 3]
        },
        {
            "key1": "value in here"
        }
    ]
    "###;

    const RESULT_SKIP_NULL_AND_EMPTY: &str = r###"
    [
        {
            "key2": "there is a value",
            "key5": [1, 2, 3]
        },
        {
            "key1": "value in here"
        }
    ]
    "###;

    #[test]
    fn test_skip_null() {
        let mut val: Value = serde_json::from_str(DATA).unwrap();
        let result: Value = serde_json::from_str(RESULT_SKIP_NULL).unwrap();
        skip_null(&mut val);
        assert_eq!(result, val)
    }

    #[test]
    fn test_skip_null_and_empty() {
        let mut val: Value = serde_json::from_str(DATA).unwrap();
        let result: Value = serde_json::from_str(RESULT_SKIP_NULL_AND_EMPTY).unwrap();
        skip_null_and_empty(&mut val);
        assert_eq!(result, val)
    }
}