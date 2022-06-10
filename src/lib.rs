use crate::utils::{DedupeHashValue, HashValue};
use serde_json::Value::{self, *};
use serde::de::DeserializeOwned;

#[cfg(test)]
mod tests;
mod utils;

pub trait JsonUtils {
    fn skip_null(&mut self);
    
    fn skip_null_and_empty(&mut self);
    
    fn dedup(&mut self);
    
    fn merge_similar(&mut self);
    
    fn to_struct<T: DeserializeOwned>(self) -> Option<T>;
    
    fn extend(&mut self, value: Value);
}


impl JsonUtils for Value {

    /// Remove `Null` value fields from serde_json::Value
    /// ## Example
    /// ```rust
    /// use serde_json::{Value, from_str};
    /// use serde_json_utils::{JsonUtils};
    ///
    /// let mut x: Value = from_str(r###"{"key1": null, "key2": "foo"}"###).unwrap();
    /// let x_result: Value = from_str(r###"{"key2": "foo"}"###).unwrap();
    ///
    /// x.skip_null();
    /// assert_eq!(x, x_result);
    /// ```
    fn skip_null(&mut self) {
        remove_nulls(self, false);
    }

    /// Remove `Null` value fields & `empty` value fields from serde_json::Value
    /// ## Example
    /// ```rust
    /// use serde_json::{Value, from_str};
    /// use serde_json_utils::{JsonUtils};
    ///
    /// let mut x: Value = from_str(r###"{"key1": null, "key2": "foo", "key3": [], "key4": {}}"###).unwrap();
    /// let x_result: Value = from_str(r###"{"key2": "foo"}"###).unwrap();
    ///
    /// x.skip_null_and_empty();
    /// assert_eq!(x, x_result);
    /// ```
    fn skip_null_and_empty(&mut self) {
        remove_nulls(self, true);
    }

    /// `Dedup` array of json's from serde_json::Value
    /// ## Example
    /// ```rust
    /// use serde_json::{Value, from_str};
    /// use serde_json_utils::{JsonUtils};
    ///
    /// let mut x: Value = from_str(r###"[{"key1": "foo", "key2": "bar", "key3": [1, 1, 2]}, {"key1": "foo", "key2": "bar", "key3": [1, 1, 2]}]"###).unwrap();
    /// let x_result: Value = from_str(r###"[{"key1": "foo", "key2": "bar", "key3": [1, 2]}]"###).unwrap();
    ///
    /// x.dedup();
    /// assert_eq!(x, x_result);
    /// ```
    fn dedup(&mut self) {
        match self {
            Null => {}
            Bool(_) => {}
            Number(_) => {}
            String(_) => {}
            Array(a) => {
                let mut aa = a.clone();
                for v in &mut aa {
                    v.dedup();
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
                    v.dedup();
                }
            }
        }
    }
    
    fn merge_similar(&mut self){
        match self {
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
                    v.merge_similar();
                }
            }
            _ => {}
        }
    }

    /// Converts a Value to a Struct of the provided type. The provided struct must implement the `serde::Deserialize` trait.
    /// ## Usage
    /// ```rust
    /// use serde_json::{json, Value};
    /// use serde_json_utils::JsonUtils;
    /// use serde::{Deserialize, Serialize};
    /// 
    /// #[derive(Serialize, Eq, PartialEq, Deserialize, Debug)]
    /// pub struct Car {
    ///     model: String,
    ///     make: String,
    ///     year: i32
    /// }
    /// 
    /// let mut car_val: Value = json!({"model": "Car model", "make": "Car make", "year": 2019});
    /// 
    /// let car2 = Car{
    ///     model: "Car model".to_string(),
    ///     make: "Car make".to_string(),
    ///     year: 2019
    /// };
    ///
    /// if let Some(car1) = car_val.to_struct::<Car>(){
    ///     assert_eq!(car1, car2);
    /// }
    /// ```
    fn to_struct< T: DeserializeOwned>(self) -> Option<T> {
        let cast_type: Option<T> = match serde_json::from_value(self) { 
            Ok(t) => Some(t),
            _=> {
                // Returns none if the value could not be properly parsed.
                None
            }
        };
        cast_type
    }

    /// Extends a value with another value. If a value contains values with the same keys, the values are combined.
    /// It can only combine a value of type Map or an array. For a map the new value is appended to the existing 
    /// value and  for an array the content of the value is appended to the existing array.
    /// ## Usage
    /// ```rust
    /// use serde_json::{Value, from_str};
    /// use serde_json_utils::JsonUtils;
    ///
    /// let mut x: Value = from_str(r###"{"key1": "bar"}"###).unwrap();
    /// let x_result: Value = from_str(r###"{"key1": "bar", "key2": "foo"}"###).unwrap();
    ///
    /// x.extend(from_str(r###"{"key2": "foo"}"###).unwrap());
    /// assert_eq!(x, x_result);
    /// ```
    fn extend(&mut self, mut value: Value) {
        // Iterate through all the keys
        if self.is_object() {
            if value.is_object() {
                for item in value.as_object().unwrap().iter() { 
                    self.as_object_mut().unwrap().insert(item.0.clone(), item.1.clone());
                }
            } else if value.is_array() {
                self.as_object_mut().unwrap().insert("items".to_string(), value.clone());
            }
        } else if self.is_array() {
            if value.is_object() {
                self.as_array_mut().unwrap().push( value.clone());
            } else if value.is_array() {
                self.as_array_mut().unwrap().append( value.as_array_mut().unwrap());
            }
        }
    }
}



/// Remove `Null` value fields & `empty` value fields from serde_json::Value
pub fn remove_nulls(val: &mut Value, with_empties: bool) -> bool {
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
pub fn merge_similar_objects(p: &Value, v: &Value) -> Result<Value, ()> {
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

