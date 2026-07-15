//! Helpers for transitioning between dense arrays and overflow objects.

use indexmap::IndexMap;

use crate::error::DecodeError;
use crate::internal::node::Node;
use crate::options::DecodeOptions;

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
    let max_index = items.len().saturating_sub(1);
    let entries = array_to_numeric_object(items, false);

    Node::OverflowObject { entries, max_index }
}

pub(crate) fn list_limit_overflow(
    item_count: usize,
    options: &DecodeOptions,
) -> Result<bool, DecodeError> {
    if item_count <= options.list_limit {
        return Ok(false);
    }

    if options.throw_on_limit_exceeded {
        return Err(DecodeError::ListLimitExceeded {
            limit: options.list_limit,
        });
    }

    Ok(true)
}

pub(crate) fn finalize_list(
    items: Vec<Node>,
    options: &DecodeOptions,
) -> Result<Node, DecodeError> {
    if list_limit_overflow(items.len(), options)? {
        Ok(overflow_from_items(items))
    } else {
        Ok(Node::Array(items))
    }
}
