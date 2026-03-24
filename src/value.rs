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
