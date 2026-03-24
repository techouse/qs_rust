//! Helpers for transitioning between dense arrays and overflow objects.

use indexmap::IndexMap;

use crate::internal::node::Node;

pub(crate) fn parse_canonical_index(text: &str) -> Option<usize> {
    if text.is_empty() {
        return None;
    }

    let index = text.parse::<usize>().ok()?;
    if index.to_string() == text {
        Some(index)
    } else {
        None
    }
}

pub(crate) fn max_numeric_index(entries: &IndexMap<String, Node>) -> Option<usize> {
    entries
        .keys()
        .filter_map(|key| parse_canonical_index(key))
        .max()
}

pub(crate) fn array_to_numeric_object(
    items: Vec<Node>,
    keep_undefined: bool,
) -> IndexMap<String, Node> {
    let mut entries = IndexMap::with_capacity(items.len());
    for (index, value) in items.into_iter().enumerate() {
        if !keep_undefined && value.is_undefined() {
            continue;
        }
        entries.insert(index.to_string(), value);
    }
    entries
}

pub(crate) fn overflow_from_items(items: Vec<Node>) -> Node {
    let mut entries = IndexMap::with_capacity(items.len());
    let mut max_index = 0usize;

    for (index, value) in items.into_iter().enumerate() {
        max_index = index;
        entries.insert(index.to_string(), value);
    }

    Node::OverflowObject { entries, max_index }
}
