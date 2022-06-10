use serde_json::{from_str, json, Value};
use serde::{Deserialize, Serialize};
use crate::{JsonUtils, merge_similar_objects};

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

const MERGE_SRC3: &str = r###"[
        {
            "key1": "value in here",
            "key2": "asas"
        },
        {
            "key1": "value in here",
            "key2": "asas"
        }]
        "###;

const MERGE_RES3: &str = r###"[
        {
            "key1": "value in here",
            "key2": "asas"
        }]
        "###;

const MERGE_SRC4: &str = r###"[
        {
            "key1": "value in here",
            "key2": "asas1"
        },
        {
            "key1": "value in here",
            "key2": "asas2"
        }]
        "###;

const MERGE_RES4: &str = r###"[
        {
            "key1": "value in here",
            "key2": ["asas1", "asas2"]
        }]
        "###;

const TO_STRUCT: &str = r###"{
            "model": "car model",
            "make": "car make",
            "year": 2019
        }"###;

const TO_STRUCT2: &str = r###"[
        {
            "model": "car model",
            "make": "car make",
            "year": 2019
        },
        {
            "model": "car model2",
            "make": "car make2",
            "year": 2020
        }
        ]"###;

const EXTEND: &str = r###"{
            "model": "car model",
            "make": "car make"
        }"###;

const EXTEND3: &str = r###"{
            "model": "car model",
            "make": "car make",
            "year": 2019
        }"###;

const EXTEND2: &str = r###"[
        {
            "model": "car model",
            "make": "car make",
            "year": 2019
        },
        {
            "model": "car model2",
            "make": "car make2",
            "year": 2020
        },
        {
            "model": "car model3",
            "make": "car make3",
            "year": 2017
        }
        ]"###;

#[derive(Deserialize, Serialize, Eq, PartialEq, Clone, Debug)]
pub struct Car {
    model: String,
    make: String,
    year: i32
}

#[test]
fn test_skip_null() {
    let mut val: Value = from_str(DATA).unwrap();
    let result: Value = from_str(RESULT_SKIP_NULL).unwrap();
    val.skip_null();
    assert_eq!(result, val)
}

#[test]
fn test_skip_null_and_empty() {
    let mut val: Value = from_str(DATA).unwrap();
    let result: Value = from_str(RESULT_SKIP_NULL_AND_EMPTY).unwrap();
    val.skip_null_and_empty();
    assert_eq!(result, val)
}

#[test]
fn test_dedup() {
    let mut val: Value = from_str(DATA).unwrap();
    let result: Value = from_str(RESULT_DEDUP).unwrap();
    val.dedup();
    assert_eq!(result, val)
}

#[test]
fn test_skip_null_and_empty_and_dedup() {
    let mut val: Value = from_str(DATA).unwrap();
    let result: Value = from_str(RESULT_SKIP_NULL_AND_EMPTY_AND_DEDUPLICATE).unwrap();
    val.skip_null_and_empty();
    val.dedup();
    assert_eq!(result, val)
}

#[test]
fn test_merge_similar_objects() {
    let src1: Value = from_str(MERGE_SRC1).unwrap();
    let src2: Value = from_str(MERGE_SRC2).unwrap();
    let src11: Value = from_str(MERGE_SRC1).unwrap();
    let ress1: Value = from_str(MERGE_RES1).unwrap();
    let res1 = merge_similar_objects(&src1, &src11).unwrap();
    assert_eq!(res1, src1);
    let res2 = merge_similar_objects(&src1, &src2).unwrap();
    assert_eq!(res2, ress1);
}

#[test]
fn test_merge_similar(){
    let mut src1: Value = from_str(MERGE_SRC3).unwrap();
    let res1: Value = from_str(MERGE_RES3).unwrap();
    src1.merge_similar();
    assert_eq!(res1, src1);

    let mut src1: Value = from_str(MERGE_SRC4).unwrap();
    let res1: Value = from_str(MERGE_RES4).unwrap();
    src1.merge_similar();
    assert_eq!(res1, src1);

    // let mut src1: Value = from_str(MERGE_SRC5).unwrap();
    // let res1: Value = from_str(MERGE_SRC5).unwrap();
    // merge_similar(&mut src1);
    // assert_eq!(res1, src1);
}

#[test]
fn test_to_struct() {
    let src1: Value = from_str(TO_STRUCT).unwrap();
    let car1 = Car{
        model: "car model".to_string(),
        make: "car make".to_string(),
        year: 2019
    };
    if let Some(car2) = src1.to_struct::<Car>() {
        assert_eq!(car1, car2);
    }

    let src2: Value = from_str(TO_STRUCT2).unwrap();
    if let Some(cars) = src2.to_struct::<Vec<Car>>(){
        assert_eq!(cars.len(), 2);
    }
}

#[test]
fn test_extend(){
    // Test extending an array
    let src1: Value = from_str(EXTEND2).unwrap();
    let src2 = json!({
        "model": "car model3",
        "make": "car make3",
        "year": 2017i32
    });
    let mut src3 : Value = from_str(TO_STRUCT2).unwrap();
    src3.extend(src2);
    assert_eq!(src3, src1);
    
    // Test extending an object.
    let mut src4: Value = from_str(EXTEND).unwrap();
    let src5 : Value = from_str(EXTEND3).unwrap();
    src4.extend(json!({"year": 2019i32}));
    assert_eq!(src4, src5);
}
