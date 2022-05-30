use ordered_float::NotNan;
use serde_json::Value::{self, *};
use std::{hash::{Hash, Hasher}, string::String};

#[derive(Debug, PartialEq)]
pub(crate) struct DedupeHashValue<'a>(pub &'a Value);

impl Eq for DedupeHashValue<'_> {}

impl Hash for DedupeHashValue<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.0 {
            Null => state.write_u32(3_221_225_473), // chosen randomly
            Bool(ref b) => b.hash(state),
            Number(ref n) => {
                "number".hash(state);
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
            String(ref s) => s.to_lowercase().hash(state),
            Array(ref v) => {
                "array".hash(state);
                for x in v {
                    HashValue(x.clone()).hash(state);
                }
            }
            Object(ref map) => {
                "map".hash(state);
                for (k, v) in map {
                    k.hash(state);
                    DedupeHashValue(v).hash(state);
                }
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct HashValue(pub Value);

impl Eq for HashValue {}

impl std::cmp::PartialEq for HashValue {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Null, Null) => true,
            (Bool(b1), Bool(b2)) => b1 == b2,
            (Number(b1), Number(b2)) => b1 == b2,
            (String(b1), String(b2)) => b1 == b2,
            (Array(b1), Array(b2)) => b1 == b2,
            (Object(b1), Object(b2)) => {
                b1.keys().collect::<Vec<&String>>()
                    == b2.keys().collect::<Vec<&String>>()
            }
            _ => false,
        }
    }
}

impl Hash for HashValue {
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
            String(ref s) => s.to_lowercase().hash(state),
            Array(ref v) => {
                for x in v {
                    HashValue(x.clone()).hash(state);
                }
            }
            Object(ref map) => {
                for (k, _v) in map {
                    k.hash(state);
                }
            }
        }
    }
}
