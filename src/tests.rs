use crate::*;
use serde_json::{from_str, Value};

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

const RESULT_DEDUP: &str = r###"
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
            "key2": null
        }
        "###;

const MERGE_SRC2: &str = r###"
        {
            "key1": "value in here",
            "key2": "asas"
        }
        "###;

const MERGE_RES1: &str = r###"
        {
            "key1": "value in here",
            "key2": [null, "asas"]
        }
        "###;

#[test]
fn test_skip_null() {
    let mut val: Value = from_str(DATA).unwrap();
    let result: Value = from_str(RESULT_SKIP_NULL).unwrap();
    skip_null(&mut val);
    assert_eq!(result, val)
}

#[test]
fn test_skip_null_and_empty() {
    let mut val: Value = from_str(DATA).unwrap();
    let result: Value = from_str(RESULT_SKIP_NULL_AND_EMPTY).unwrap();
    skip_null_and_empty(&mut val);
    assert_eq!(result, val)
}

#[test]
fn test_dedup() {
    let mut val: Value = from_str(DATA).unwrap();
    let result: Value = from_str(RESULT_DEDUP).unwrap();
    dedup(&mut val);
    assert_eq!(result, val)
}

#[test]
fn test_skip_null_and_empty_and_dedup() {
    let mut val: Value = from_str(DATA).unwrap();
    let result: Value = from_str(RESULT_SKIP_NULL_AND_EMPTY_AND_DEDUPLICATE).unwrap();
    skip_null_and_empty(&mut val);
    dedup(&mut val);
    assert_eq!(result, val)
}

#[test]
fn test_merge() {
    let src1: Value = from_str(MERGE_SRC1).unwrap();
    let src2: Value = from_str(MERGE_SRC2).unwrap();
    let src11: Value = from_str(MERGE_SRC1).unwrap();
    let ress1: Value = from_str(MERGE_RES1).unwrap();
    let res1 = merge_similar(&src1, &src11);
    assert_eq!(res1, src1);
    let res2 = merge_similar(&src1, &src2);
    assert_eq!(res2, ress1);
}
