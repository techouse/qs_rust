//! Flat-value accumulation and finalization for decode.

use indexmap::IndexMap;

use crate::compact::{compact, node_to_value};
use crate::error::DecodeError;
use crate::internal::node::Node;
use crate::options::DecodeOptions;
use crate::value::{Object, Value};

use super::accumulate::insert_value;
use super::keys::key_might_be_structured;

/// The raw decode result before the final flat-vs-structured decision is made.
#[derive(Debug)]
pub(super) struct ParsedInput {
    /// The accumulated flat values.
    pub(super) values: FlatValues,
    /// Whether any scanned key looked structured enough to require a second
    /// pass.
    pub(super) has_any_structured_syntax: bool,
}

/// The default fast-path accumulator used while decoding string-delimited
/// inputs without a custom decoder.
#[derive(Clone, Debug)]
pub(super) enum DefaultAccumulator {
    /// Directly accumulated concrete output.
    Direct(Object),
    /// Promoted parsed storage once direct accumulation is no longer enough.
    Parsed(IndexMap<String, ParsedFlatValue>),
}

impl DefaultAccumulator {
    pub(super) fn direct() -> Self {
        Self::Direct(Object::new())
    }

    pub(super) fn direct_with_capacity(capacity: usize) -> Self {
        Self::Direct(Object::with_capacity(capacity))
    }

    pub(super) fn ensure_parsed(&mut self) -> &mut IndexMap<String, ParsedFlatValue> {
        if let Self::Direct(entries) = self {
            let parsed = std::mem::take(entries)
                .into_iter()
                .map(|(key, value)| (key, ParsedFlatValue::concrete(value)))
                .collect();
            *self = Self::Parsed(parsed);
        }

        match self {
            Self::Parsed(entries) => entries,
            Self::Direct(_) => unreachable!("direct accumulator should have been promoted"),
        }
    }

    pub(super) fn into_flat_values(self) -> FlatValues {
        match self {
            Self::Direct(entries) => FlatValues::Concrete(entries),
            Self::Parsed(entries) => FlatValues::Parsed(entries),
        }
    }
}

/// The result of trying to keep default accumulation in direct concrete mode.
pub(super) enum DirectInsertOutcome {
    /// The value was stored directly with no promotion.
    Done,
    /// The accumulator must promote to parsed storage before insertion can
    /// continue.
    PromoteInsert {
        /// The target key.
        key: String,
        /// The promoted parsed value to insert.
        value: ParsedFlatValue,
        /// Whether promotion happened while resolving duplicates.
        via_duplicates: bool,
    },
}

/// A flat stored value before finalization.
#[derive(Clone, Debug)]
pub(super) enum ParsedFlatValue {
    /// A value that is already in public concrete form.
    Concrete(Value),
    /// A value that still needs node-based finalization or compaction.
    Parsed { node: Node, needs_compaction: bool },
}

impl ParsedFlatValue {
    pub(super) fn concrete(value: Value) -> Self {
        Self::Concrete(value)
    }

    pub(super) fn parsed(node: Node, needs_compaction: bool) -> Self {
        Self::Parsed {
            node,
            needs_compaction,
        }
    }

    pub(super) fn force_parsed(self) -> Self {
        match self {
            Self::Concrete(value) => Self::parsed(node_from_value(value), false),
            parsed => parsed,
        }
    }

    pub(super) fn into_node(self) -> Node {
        match self {
            Self::Concrete(value) => node_from_value(value),
            Self::Parsed { node, .. } => node,
        }
    }

    pub(super) fn list_length_for_combine(&self) -> usize {
        match self {
            Self::Concrete(Value::Array(items)) => items.len(),
            Self::Concrete(value) if !value.is_empty_for_decode() => 1,
            Self::Concrete(_) => 0,
            Self::Parsed { node, .. } => node_list_length_for_combine(node),
        }
    }

    #[cfg(test)]
    pub(super) fn is_parsed_with_compaction(&self) -> bool {
        matches!(
            self,
            Self::Parsed {
                needs_compaction: true,
                ..
            }
        )
    }
}

/// The flat key/value map produced by the scan stage.
#[derive(Clone, Debug)]
pub(super) enum FlatValues {
    /// Concrete flat output that can be returned directly.
    Concrete(Object),
    /// Parsed flat output that still needs node-aware finalization.
    Parsed(IndexMap<String, ParsedFlatValue>),
}

impl FlatValues {
    pub(super) fn parsed() -> Self {
        Self::Parsed(IndexMap::new())
    }

    pub(super) fn is_empty(&self) -> bool {
        match self {
            Self::Concrete(entries) => entries.is_empty(),
            Self::Parsed(entries) => entries.is_empty(),
        }
    }

    pub(super) fn key_refs(&self) -> Vec<&str> {
        match self {
            Self::Concrete(entries) => entries.keys().map(String::as_str).collect(),
            Self::Parsed(entries) => entries.keys().map(String::as_str).collect(),
        }
    }

    pub(super) fn get_list_length_for_combine(&self, key: &str) -> usize {
        match self {
            Self::Concrete(entries) => entries.get(key).map_or(0, value_list_length_for_combine),
            Self::Parsed(entries) => entries
                .get(key)
                .map_or(0, ParsedFlatValue::list_length_for_combine),
        }
    }

    pub(super) fn ensure_parsed(&mut self) -> &mut IndexMap<String, ParsedFlatValue> {
        if let Self::Concrete(entries) = self {
            let mut parsed = IndexMap::with_capacity(entries.len());
            for (key, value) in std::mem::take(entries) {
                parsed.insert(key, ParsedFlatValue::concrete(value));
            }
            *self = Self::Parsed(parsed);
        }

        match self {
            Self::Parsed(entries) => entries,
            Self::Concrete(_) => unreachable!("concrete values should have been promoted"),
        }
    }

    pub(super) fn into_parsed_map(self) -> IndexMap<String, ParsedFlatValue> {
        match self {
            Self::Concrete(entries) => entries
                .into_iter()
                .map(|(key, value)| (key, ParsedFlatValue::concrete(value)))
                .collect(),
            Self::Parsed(entries) => entries,
        }
    }

    #[cfg(test)]
    pub(super) fn stores_concrete_value(&self, key: &str) -> bool {
        matches!(self, Self::Concrete(entries) if entries.contains_key(key))
    }

    #[cfg(test)]
    pub(super) fn stores_parsed_value_with_compaction(&self, key: &str) -> bool {
        matches!(
            self,
            Self::Parsed(entries)
                if entries
                    .get(key)
                    .is_some_and(ParsedFlatValue::is_parsed_with_compaction)
        )
    }

    #[cfg(test)]
    pub(super) fn stores_parsed_value(&self, key: &str) -> bool {
        matches!(self, Self::Parsed(entries) if entries.contains_key(key))
    }
}

pub(super) fn collect_pair_values<I>(
    pairs: I,
    options: &DecodeOptions,
) -> Result<ParsedInput, DecodeError>
where
    I: IntoIterator<Item = (String, Value)>,
{
    let mut values = FlatValues::parsed();
    let mut token_count = 0usize;
    let mut has_any_structured_syntax = false;

    for (key, value) in pairs {
        if key.is_empty() {
            continue;
        }

        token_count += 1;
        if options.throw_on_limit_exceeded && token_count > options.parameter_limit {
            return Err(DecodeError::ParameterLimitExceeded {
                limit: options.parameter_limit,
            });
        }
        if !options.throw_on_limit_exceeded && token_count > options.parameter_limit {
            break;
        }

        has_any_structured_syntax |= key_might_be_structured(&key, options);
        let node_value = ParsedFlatValue::parsed(node_from_value(value), true);
        let values = values.ensure_parsed();
        insert_value(values.entry(key), node_value, options)?;
    }

    Ok(ParsedInput {
        values,
        has_any_structured_syntax,
    })
}

/// Finalizes flat decode storage into the public ordered object output.
pub(super) fn finalize_flat(
    values: FlatValues,
    options: &DecodeOptions,
) -> Result<Object, DecodeError> {
    match values {
        FlatValues::Concrete(values) => Ok(values),
        FlatValues::Parsed(values) => {
            let mut output = Object::with_capacity(values.len());
            for (key, parsed_value) in values {
                match parsed_value {
                    ParsedFlatValue::Concrete(value) => {
                        output.insert(key, value);
                    }
                    ParsedFlatValue::Parsed {
                        node,
                        needs_compaction,
                    } => {
                        let value = if needs_compaction {
                            compact(node, options.allow_sparse_lists)
                        } else {
                            node
                        };

                        match value {
                            Node::Undefined => continue,
                            Node::Value(value) => {
                                output.insert(key, value);
                            }
                            other => {
                                output.insert(key, node_to_value(other));
                            }
                        }
                    }
                }
            }
            Ok(output)
        }
    }
}

fn node_list_length_for_combine(node: &Node) -> usize {
    match node {
        Node::Array(items) => items.len(),
        Node::OverflowObject { max_index, .. } => max_index + 1,
        Node::Value(value) if !value.is_empty_for_decode() => 1,
        Node::Object(entries) if !entries.is_empty() => 1,
        _ => 0,
    }
}

fn node_from_value(value: Value) -> Node {
    match value {
        Value::Array(items) => Node::Array(items.into_iter().map(node_from_value).collect()),
        Value::Object(entries) => Node::Object(
            entries
                .into_iter()
                .map(|(key, value)| (key, node_from_value(value)))
                .collect(),
        ),
        scalar => Node::Value(scalar),
    }
}

pub(super) fn value_list_length_for_combine(value: &Value) -> usize {
    match value {
        Value::Array(items) => items.len(),
        Value::Object(entries) if !entries.is_empty() => 1,
        value if !value.is_empty_for_decode() => 1,
        _ => 0,
    }
}

/// Controls whether the default decode path keeps concrete values when
/// possible or always promotes into parsed storage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DefaultStorageMode {
    PreferConcrete,
    ForceParsed,
}
