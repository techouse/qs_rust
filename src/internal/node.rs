//! Internal node tree used while decoding and merging structured data.

use indexmap::IndexMap;

use crate::value::Value;

/// The internal decode tree.
///
/// Unlike the public [`Value`] model, this representation can carry
/// placeholders and overflow-object state while the decode pipeline is still
/// assembling the final structure.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Node {
    /// A placeholder used while compacting or merging sparse structures.
    Undefined,
    /// A finalized scalar or already-materialized public value.
    Value(Value),
    /// A dense list node.
    Array(Vec<Node>),
    /// An ordered object node.
    Object(IndexMap<String, Node>),
    /// A numeric-keyed object used once list growth has overflowed list rules.
    OverflowObject {
        /// The accumulated entries.
        entries: IndexMap<String, Node>,
        /// The greatest numeric index observed so far.
        max_index: usize,
    },
}

impl Node {
    pub(crate) fn scalar(value: Value) -> Self {
        debug_assert!(value.is_scalar());
        Self::Value(value)
    }

    pub(crate) fn is_undefined(&self) -> bool {
        matches!(self, Self::Undefined)
    }

    pub(crate) fn is_null_source(&self) -> bool {
        matches!(self, Self::Value(Value::Null))
    }

    pub(crate) fn is_map_like(&self) -> bool {
        matches!(self, Self::Object(_) | Self::OverflowObject { .. })
    }
}
