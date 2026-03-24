use indexmap::IndexMap;

use crate::temporal::TemporalValue;

/// The object representation used throughout the crate.
///
/// Objects preserve insertion order to match the observable behavior of the
/// sibling ports and the upstream `qs` ecosystem.
pub type Object = IndexMap<String, Value>;

/// A query-string-compatible value tree.
///
/// [`Value`] is the shared input/output representation for [`crate::encode()`]
/// and [`crate::decode()`]. It intentionally stays close to the data model used
/// by the sibling ports: scalars, temporal leaves, byte strings, arrays, and
/// ordered objects.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// A null value.
    Null,
    /// A boolean scalar.
    Bool(bool),
    /// A signed 64-bit integer scalar.
    I64(i64),
    /// An unsigned 64-bit integer scalar.
    U64(u64),
    /// A 64-bit floating-point scalar.
    F64(f64),
    /// A UTF-8 string scalar.
    String(String),
    /// A core temporal scalar.
    Temporal(TemporalValue),
    /// An opaque byte string scalar.
    Bytes(Vec<u8>),
    /// An ordered list of values.
    Array(Vec<Value>),
    /// An ordered object map.
    Object(Object),
}

impl Value {
    pub(crate) fn is_scalar(&self) -> bool {
        matches!(
            self,
            Self::Null
                | Self::Bool(_)
                | Self::I64(_)
                | Self::U64(_)
                | Self::F64(_)
                | Self::String(_)
                | Self::Temporal(_)
                | Self::Bytes(_)
        )
    }

    pub(crate) fn is_empty_for_decode(&self) -> bool {
        match self {
            Self::Null => true,
            Self::String(text) => text.is_empty(),
            Self::Array(values) => values.is_empty(),
            Self::Object(entries) => entries.is_empty(),
            _ => false,
        }
    }
}

impl From<Object> for Value {
    fn from(value: Object) -> Self {
        Self::Object(value)
    }
}

#[cfg(test)]
mod tests {
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
}
