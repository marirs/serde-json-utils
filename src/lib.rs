use ordered_float::NotNan;
use serde_json::Value::{self, *};
use std::hash::{Hash, Hasher};

#[derive(Debug, PartialEq)]
struct DedupeHashValue<'a>(pub &'a serde_json::Value);

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
                    HashValue(x).hash(state);
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
struct HashValue<'a>(pub &'a serde_json::Value);

impl Eq for HashValue<'_> {}

impl std::cmp::PartialEq for HashValue<'_> {
    fn eq(&self, other: &Self) -> bool{
        match (&self.0, &other.0){
            (Null, Null) => true,
            (Bool(b1), Bool(b2)) => b1 == b2,
            (Number(b1), Number(b2)) => b1 == b2,
            (String(b1), String(b2)) => b1 == b2,
            (Array(b1), Array(b2)) => b1 == b2,
            (Object(b1), Object(b2)) => {
                b1.keys().collect::<Vec<&std::string::String>>() == b2.keys().collect::<Vec<&std::string::String>>()
            }
            _ => false
        }
    }
}

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
            String(ref s) => s.to_lowercase().hash(state),
            Array(ref v) => {
                for x in v {
                    HashValue(x).hash(state);
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

pub fn merge(p: &Value, v: &Value) -> Value{
    match (p, v){
        (Object(a), Object(b)) => {
            if HashValue(p) != HashValue(v){
                return Array(vec![p.clone(), v.clone()]);
            }
            let mut res = serde_json::Map::new();
            for (k, v) in a{
                let bv = b.get(k).unwrap();
                if let (Array(_arr1), Array(_arr2)) = (v, bv){
                    if v == bv{
                        res.insert(k.clone(), v.clone());
                    } else {
                        res.insert(k.clone(), Array(vec![v.clone(), bv.clone()]));
                    }
                } else if let (Array(arr1), _) = (v, bv){
                    let mut aaa = arr1.clone();
                    if !aaa.contains(bv){
                        aaa.push(bv.clone());
                    }
                    res.insert(k.clone(), Array(aaa));
                } else {
                    if v == bv{
                        res.insert(k.clone(), v.clone());
                    } else {
                        res.insert(k.clone(), Array(vec![v.clone(), bv.clone()]));
                    }
                }
            }
            Object(res)
        }
        _ => Array(vec![p.clone(), v.clone()])
    }
}

fn _skip_nulls(val: &mut Value, with_empties: bool) -> bool{
    match val{
        Null => {return true;}
        Bool(_) => {}
        Number(_) => {}
        String(_) => {}
        Array(a) => {
            if with_empties && a.is_empty(){
                return true;
            }
            let mut candidates = vec![];
            for v in &mut a.clone(){
                if !_skip_nulls(v, with_empties){
                    candidates.push(v.clone());
                }
            }
            a.clear();
            a.extend(candidates);
        }
        Object(o) => {
            if with_empties && o.is_empty(){
                return true;
            }
            let mut candidates = vec![];
            for (k, v) in o.iter_mut() {
                if _skip_nulls(v, with_empties){
                    candidates.push(k.clone());
                }
            }
            for c in candidates{
                o.remove(&c);
            }
        }
    }
    false
}

pub fn skip_null(val: &mut Value){
    _skip_nulls(val, false);
}

pub fn skip_null_and_empty(val: &mut Value){
    _skip_nulls(val, true);
}

pub fn deduplicate(val: &mut Value){
    match val{
        Null => {}
        Bool(_) => {}
        Number(_) => {}
        String(_) => {}
        Array(a) => {
            let mut aa = a.clone();
            for v in &mut aa {
                deduplicate(v);
            }
            let mut set = std::collections::HashSet::new();
            let mut candidates = vec![];
            for v in &aa{
                if !set.contains(&DedupeHashValue(&v)){
                    set.insert(DedupeHashValue(&v));
                    candidates.push(v.clone());
                }
            }
            a.clear();
            a.extend(candidates);
        }
        Object(o) => {
            for (_, v) in o.iter_mut() {
                deduplicate(v);
            }
        }
    }
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

        const RESULT_SKIP_NULL: &str = r###"
    [
        {
            "key2": "there is a value",
            "key3": {},
            "key4": [],
            "key5": [1, 2, 3, 3]

},
        {
            "key1": "value in here"

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
            "key5": [1, 2, 3, 3]

},
        {
            "key1": "value in here"

},
        {
            "key1": "value in here"

}

]
    "###;

        const RESULT_SKIP_NULL_AND_EMPTY_AND_DEDUPLICATE: &str = r###"
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

    const MERGE_SRC1: &str = r###"
        {
            "key1": "value in here",
            "key2": null}"###;

    const MERGE_SRC2: &str = r###"
        {
            "key1": "value in here",
            "key2": "asas"}"###;

    const MERGE_RES1: &str = r###"
        {
            "key1": "value in here",
            "key2": [null, "asas"]}"###;


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

    #[test]
    fn test_skip_null_and_empty_and_deduplicate() {
        let mut val: Value = serde_json::from_str(DATA).unwrap();
        let result: Value = serde_json::from_str(RESULT_SKIP_NULL_AND_EMPTY_AND_DEDUPLICATE).unwrap();
        skip_null_and_empty(&mut val);
        deduplicate(&mut val);
        assert_eq!(result, val)

    }


    #[test]
    fn test_merge() {
        let src1: Value = serde_json::from_str(MERGE_SRC1).unwrap();
        let src2: Value = serde_json::from_str(MERGE_SRC2).unwrap();
        let src11: Value = serde_json::from_str(MERGE_SRC1).unwrap();
        let ress1: Value = serde_json::from_str(MERGE_RES1).unwrap();
        let res1 = merge(&src1, &src11);
        assert_eq!(res1, src1);
        let res2 = merge(&src1, &src2);
        assert_eq!(res2, ress1);
    }
}
