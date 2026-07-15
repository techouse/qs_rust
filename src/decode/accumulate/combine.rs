//! Duplicate-combine helpers for flat decode accumulation.

use crate::error::DecodeError;
use crate::internal::node::Node;
use crate::internal::overflow::{finalize_list, list_limit_overflow};
use crate::options::DecodeOptions;
use crate::value::Value;

pub(in crate::decode) fn combine_with_limit(
    current: Node,
    next: Node,
    options: &DecodeOptions,
) -> Result<Node, DecodeError> {
    let mut next_items = Vec::new();
    flatten_for_combine(next, &mut next_items);

    if let Node::OverflowObject {
        mut entries,
        mut max_index,
    } = current
    {
        let current_len = max_index.saturating_add(1);
        list_limit_overflow(current_len.saturating_add(next_items.len()), options)?;

        for item in next_items {
            max_index = max_index.saturating_add(1);
            entries.insert(max_index.to_string(), item);
        }
        return Ok(Node::OverflowObject { entries, max_index });
    }

    let mut combined = Vec::new();
    flatten_for_combine(current, &mut combined);
    combined.extend(next_items);

    finalize_list(combined, options)
}

fn flatten_for_combine(node: Node, output: &mut Vec<Node>) {
    match node {
        Node::Array(items) => output.extend(items.into_iter().filter(|item| !item.is_undefined())),
        Node::OverflowObject { entries, .. } => {
            output.extend(entries.into_values().filter(|item| !item.is_undefined()))
        }
        other => output.push(other),
    }
}

pub(super) fn try_combine_direct_values(
    current: &Value,
    next: &Value,
    options: &DecodeOptions,
) -> Result<Option<Value>, DecodeError> {
    let mut combined = Vec::new();
    flatten_value_for_combine(current.clone(), &mut combined);
    flatten_value_for_combine(next.clone(), &mut combined);

    if list_limit_overflow(combined.len(), options)? {
        Ok(None)
    } else {
        Ok(Some(Value::Array(combined)))
    }
}

fn flatten_value_for_combine(value: Value, output: &mut Vec<Value>) {
    match value {
        Value::Array(items) => output.extend(items),
        other => output.push(other),
    }
}
