use super::{Object, Value};
use crate::temporal::{DateTimeValue, TemporalValue};

fn sample_temporal() -> TemporalValue {
    TemporalValue::from(DateTimeValue::new(2024, 1, 2, 3, 4, 5, 0, Some(0)).unwrap())
}

#[test]
fn scalar_and_empty_helpers_match_the_public_value_model() {
    for value in [
        Value::Null,
        Value::Bool(true),
        Value::I64(-1),
        Value::U64(1),
        Value::F64(1.5),
        Value::String("text".to_owned()),
        Value::Temporal(sample_temporal()),
        Value::Bytes(vec![1, 2, 3]),
    ] {
        assert!(value.is_scalar(), "{value:?} should be scalar");
    }

    for value in [
        Value::Array(vec![Value::Null]),
        Value::Object([("field".to_owned(), Value::Null)].into()),
    ] {
        assert!(!value.is_scalar(), "{value:?} should not be scalar");
    }

    assert!(Value::Null.is_empty_for_decode());
    assert!(Value::String(String::new()).is_empty_for_decode());
    assert!(Value::Array(Vec::new()).is_empty_for_decode());
    assert!(Value::Object(Object::new()).is_empty_for_decode());
    assert!(!Value::Bool(true).is_empty_for_decode());
    assert!(!Value::String("text".to_owned()).is_empty_for_decode());

    let object: Object = [("field".to_owned(), Value::Bool(true))].into();
    assert_eq!(Value::from(object.clone()), Value::Object(object));
}
