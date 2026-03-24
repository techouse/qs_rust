//! Compaction and public-value materialization for decode nodes.

use std::collections::VecDeque;

use indexmap::IndexMap;

use crate::internal::node::Node;
use crate::value::{Object, Value};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CompactPhase {
    Start,
    ObjectIter,
    ObjectAwait,
    ArrayIter,
    ArrayAwait,
}

/// Removes decode placeholders and normalizes sparse internal nodes.
pub(crate) fn compact(node: Node, allow_sparse_lists: bool) -> Node {
    let mut stack = vec![CompactFrame::start(node, allow_sparse_lists)];
    let mut last_result: Option<Node> = None;

    while let Some(frame) = stack.last_mut() {
        match frame.phase {
            CompactPhase::Start => {
                let node = frame.node.take().expect("compact start node missing");
                match node {
                    Node::Object(entries) => {
                        frame.source_entries = entries.into_iter().collect();
                        frame.phase = CompactPhase::ObjectIter;
                    }
                    Node::OverflowObject { entries, max_index } => {
                        frame.source_entries = entries.into_iter().collect();
                        frame.overflow_max_index = Some(max_index);
                        frame.phase = CompactPhase::ObjectIter;
                    }
                    Node::Array(items) => {
                        frame.array_source = items;
                        frame.phase = CompactPhase::ArrayIter;
                    }
                    other => finish_compact_frame(&mut stack, &mut last_result, other),
                }
            }
            CompactPhase::ObjectIter => {
                if let Some((key, value)) = frame.source_entries.pop_front() {
                    let allow_sparse_lists = frame.allow_sparse_lists;
                    frame.pending_key = Some(key);
                    frame.phase = CompactPhase::ObjectAwait;
                    let next = CompactFrame::start(value, allow_sparse_lists);
                    stack.push(next);
                } else {
                    let result = if let Some(max_index) = frame.overflow_max_index.take() {
                        Node::OverflowObject {
                            entries: std::mem::take(&mut frame.object_result),
                            max_index,
                        }
                    } else {
                        Node::Object(std::mem::take(&mut frame.object_result))
                    };
                    finish_compact_frame(&mut stack, &mut last_result, result);
                }
            }
            CompactPhase::ObjectAwait => {
                let key = frame.pending_key.take().expect("pending key missing");
                let child = last_result.take().expect("missing compact child result");
                if !child.is_undefined() {
                    frame.object_result.insert(key, child);
                }
                frame.phase = CompactPhase::ObjectIter;
            }
            CompactPhase::ArrayIter => {
                if frame.array_index >= frame.array_source.len() {
                    let result = Node::Array(std::mem::take(&mut frame.array_result));
                    finish_compact_frame(&mut stack, &mut last_result, result);
                    continue;
                }

                let item =
                    std::mem::replace(&mut frame.array_source[frame.array_index], Node::Undefined);
                frame.array_index += 1;
                let allow_sparse_lists = frame.allow_sparse_lists;
                frame.phase = CompactPhase::ArrayAwait;
                let next = CompactFrame::start(item, allow_sparse_lists);
                stack.push(next);
            }
            CompactPhase::ArrayAwait => {
                let child = last_result
                    .take()
                    .expect("missing compact array child result");
                if child.is_undefined() {
                    if frame.allow_sparse_lists {
                        frame.array_result.push(Node::Value(Value::Null));
                    }
                } else {
                    frame.array_result.push(child);
                }
                frame.phase = CompactPhase::ArrayIter;
            }
        }
    }

    last_result.expect("compact root result missing")
}

/// Converts an internal node tree into the public ordered object shape used by
/// [`crate::decode()`].
pub(crate) fn node_to_object(node: Node) -> Object {
    match node_to_value(node) {
        Value::Object(entries) => entries,
        Value::Array(values) => {
            let mut object = Object::with_capacity(values.len());
            for (index, value) in values.into_iter().enumerate() {
                object.insert(index.to_string(), value);
            }
            object
        }
        other => {
            let mut object = Object::new();
            object.insert("0".to_owned(), other);
            object
        }
    }
}

/// Converts an internal node tree into the public [`Value`] representation.
pub(crate) fn node_to_value(node: Node) -> Value {
    let mut stack = vec![ValueFrame::start(node)];
    let mut last_result: Option<Value> = None;

    while let Some(frame) = stack.last_mut() {
        match frame.phase {
            CompactPhase::Start => {
                let node = frame.node.take().expect("value start node missing");
                match node {
                    Node::Value(value) => finish_value_frame(&mut stack, &mut last_result, value),
                    Node::Undefined => {
                        finish_value_frame(&mut stack, &mut last_result, Value::Null)
                    }
                    Node::Object(entries) => {
                        frame.source_entries = entries.into_iter().collect();
                        frame.phase = CompactPhase::ObjectIter;
                    }
                    Node::OverflowObject { entries, .. } => {
                        frame.source_entries = entries.into_iter().collect();
                        frame.phase = CompactPhase::ObjectIter;
                    }
                    Node::Array(items) => {
                        frame.array_source = items;
                        frame.phase = CompactPhase::ArrayIter;
                    }
                }
            }
            CompactPhase::ObjectIter => {
                if let Some((key, value)) = frame.source_entries.pop_front() {
                    frame.pending_key = Some(key);
                    frame.phase = CompactPhase::ObjectAwait;
                    let next = ValueFrame::start(value);
                    stack.push(next);
                } else {
                    let result = Value::Object(std::mem::take(&mut frame.object_result_value));
                    finish_value_frame(&mut stack, &mut last_result, result);
                }
            }
            CompactPhase::ObjectAwait => {
                let key = frame.pending_key.take().expect("value pending key missing");
                let child = last_result.take().expect("missing value child result");
                frame.object_result_value.insert(key, child);
                frame.phase = CompactPhase::ObjectIter;
            }
            CompactPhase::ArrayIter => {
                if frame.array_index >= frame.array_source.len() {
                    let result = Value::Array(std::mem::take(&mut frame.array_result_value));
                    finish_value_frame(&mut stack, &mut last_result, result);
                    continue;
                }

                let item =
                    std::mem::replace(&mut frame.array_source[frame.array_index], Node::Undefined);
                frame.array_index += 1;
                frame.phase = CompactPhase::ArrayAwait;
                let next = ValueFrame::start(item);
                stack.push(next);
            }
            CompactPhase::ArrayAwait => {
                let child = last_result
                    .take()
                    .expect("missing value array child result");
                frame.array_result_value.push(child);
                frame.phase = CompactPhase::ArrayIter;
            }
        }
    }

    last_result.expect("value root result missing")
}

struct CompactFrame {
    phase: CompactPhase,
    node: Option<Node>,
    allow_sparse_lists: bool,
    source_entries: VecDeque<(String, Node)>,
    object_result: IndexMap<String, Node>,
    pending_key: Option<String>,
    array_source: Vec<Node>,
    array_result: Vec<Node>,
    array_index: usize,
    overflow_max_index: Option<usize>,
}

impl CompactFrame {
    fn start(node: Node, allow_sparse_lists: bool) -> Self {
        Self {
            phase: CompactPhase::Start,
            node: Some(node),
            allow_sparse_lists,
            source_entries: VecDeque::new(),
            object_result: IndexMap::new(),
            pending_key: None,
            array_source: Vec::new(),
            array_result: Vec::new(),
            array_index: 0,
            overflow_max_index: None,
        }
    }
}

struct ValueFrame {
    phase: CompactPhase,
    node: Option<Node>,
    source_entries: VecDeque<(String, Node)>,
    object_result_value: Object,
    pending_key: Option<String>,
    array_source: Vec<Node>,
    array_result_value: Vec<Value>,
    array_index: usize,
}

impl ValueFrame {
    fn start(node: Node) -> Self {
        Self {
            phase: CompactPhase::Start,
            node: Some(node),
            source_entries: VecDeque::new(),
            object_result_value: Object::new(),
            pending_key: None,
            array_source: Vec::new(),
            array_result_value: Vec::new(),
            array_index: 0,
        }
    }
}

fn finish_compact_frame(
    stack: &mut Vec<CompactFrame>,
    last_result: &mut Option<Node>,
    result: Node,
) {
    stack.pop();
    *last_result = Some(result);
}

fn finish_value_frame(stack: &mut Vec<ValueFrame>, last_result: &mut Option<Value>, result: Value) {
    stack.pop();
    *last_result = Some(result);
}

#[cfg(test)]
mod tests;
