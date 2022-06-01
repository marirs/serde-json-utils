use crate::utils::{DedupeHashValue, HashValue};
use serde_json::Value::{self, *};

#[cfg(test)]
mod tests;
mod utils;

/// Remove `Null` value fields from serde_json::Value
/// ## Example
/// ```rust
/// use serde_json::{Value, from_str};
/// use serde_json_utils::skip_null;
///
/// let mut x: Value = from_str(r###"{"key1": null, "key2": "foo"}"###).unwrap();
/// let x_result: Value = from_str(r###"{"key2": "foo"}"###).unwrap();
///
/// skip_null(&mut x);
/// assert_eq!(x, x_result);
/// ```
pub fn skip_null(val: &mut Value) {
    remove_nulls(val, false);
}

/// Remove `Null` value fields & `empty` value fields from serde_json::Value
/// ## Example
/// ```rust
/// use serde_json::{Value, from_str};
/// use serde_json_utils::skip_null_and_empty;
///
/// let mut x: Value = from_str(r###"{"key1": null, "key2": "foo", "key3": [], "key4": {}}"###).unwrap();
/// let x_result: Value = from_str(r###"{"key2": "foo"}"###).unwrap();
///
/// skip_null_and_empty(&mut x);
/// assert_eq!(x, x_result);
/// ```
pub fn skip_null_and_empty(val: &mut Value) {
    remove_nulls(val, true);
}

/// `Dedup` array of json's from serde_json::Value
/// ## Example
/// ```rust
/// use serde_json::{Value, from_str};
/// use serde_json_utils::dedup;
///
/// let mut x: Value = from_str(r###"[{"key1": "foo", "key2": "bar", "key3": [1, 1, 2]}, {"key1": "foo", "key2": "bar", "key3": [1, 1, 2]}]"###).unwrap();
/// let x_result: Value = from_str(r###"[{"key1": "foo", "key2": "bar", "key3": [1, 2]}]"###).unwrap();
///
/// dedup(&mut x);
/// assert_eq!(x, x_result);
/// ```
pub fn dedup(val: &mut Value) {
    match val {
        Null => {}
        Bool(_) => {}
        Number(_) => {}
        String(_) => {}
        Array(a) => {
            let mut aa = a.clone();
            for v in &mut aa {
                dedup(v);
            }
            let mut set = std::collections::HashSet::new();
            let mut candidates = vec![];
            for v in &aa {
                if !set.contains(&DedupeHashValue(v)) {
                    set.insert(DedupeHashValue(v));
                    candidates.push(v.clone());
                }
            }
            a.clear();
            a.extend(candidates);
        }
        Object(o) => {
            for (_, v) in o.iter_mut() {
                dedup(v);
            }
        }
    }
}


pub fn merge_similar(v: &mut Value){
    match v {
        Array(arr) => {
            let mut res: std::collections::HashSet<HashValue> = std::collections::HashSet::new();
            let aarr = arr.clone();
            for v in aarr{
                if let Some(s) = res.take(&HashValue(v.clone())){
                    if let Ok(m) = merge_similar_objects(&s.0, &v){
                        res.insert(HashValue(m));
                    } else {
                        res.insert(s);
                        res.insert(HashValue(v.clone()));
                    }
                } else {
                    res.insert(HashValue(v.clone()));
                }
            }
            arr.clear();
            arr.extend(res.into_iter().map(|s| s.0).collect::<Vec<Value>>());
        }
        Object(obj) => {
            for (_k, v) in obj{
                merge_similar(v);
            }
        }
        _ => {}
    }
}


/// Remove `Null` value fields & `empty` value fields from serde_json::Value
fn remove_nulls(val: &mut Value, with_empties: bool) -> bool {
    match val {
        Null => {
            return true;
        }
        Bool(_) => {}
        Number(_) => {}
        String(_) => {}
        Array(arr) => {
            if with_empties && arr.is_empty() {
                return true;
            }
            let mut candidates = vec![];
            for v in &mut arr.clone() {
                if !remove_nulls(v, with_empties) {
                    candidates.push(v.clone());
                }
            }
            arr.clear();
            arr.extend(candidates);
        }
        Object(obj) => {
            if with_empties && obj.is_empty() {
                return true;
            }
            let mut candidates = vec![];
            for (k, v) in obj.iter_mut() {
                if remove_nulls(v, with_empties) {
                    candidates.push(k.clone());
                }
            }
            for c in candidates {
                obj.remove(&c);
            }
        }
    }
    false
}

/// merge similar objects
fn merge_similar_objects(p: &Value, v: &Value) -> Result<Value, ()> {
    match (p, v) {
        (Object(a), Object(b)) => {
            if HashValue(p.clone()) != HashValue(v.clone()) {
                return Err(());
            }
            let mut res = serde_json::Map::new();
            for (k, v) in a {
                let bv = b.get(k).unwrap();
                if let (Array(_arr1), Array(_arr2)) = (v, bv) {
                    if v.eq(bv) {
                        res.insert(k.clone(), v.clone());
                    } else {
                        res.insert(k.clone(), Array(vec![v.clone(), bv.clone()]));
                    }
                } else if let (Array(arr1), _) = (v, bv) {
                    let mut aaa = arr1.clone();
                    if !aaa.contains(bv) {
                        aaa.push(bv.clone());
                    }
                    res.insert(k.clone(), Array(aaa));
                } else if v.eq(bv) {
                    res.insert(k.clone(), v.clone());
                } else {
                    res.insert(k.clone(), Array(vec![v.clone(), bv.clone()]));
                }
            }
            Ok(Object(res))
        }
        _ => return Err(()),
    }
}

