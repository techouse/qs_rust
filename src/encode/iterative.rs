//! Hot-path iterative encode traversal.

use crate::error::EncodeError;
use crate::key_path::KeyPathNode;
use crate::options::{EncodeOptions, ListFormat};
use crate::value::Value;

use super::comma::encode_comma_array;
use super::scalar::{encode_scalar_leaf, encoded_dot_escape, finalize_key_path};
use super::{
    array_child_path, emit_part, ordered_array_indices, ordered_object_keys, raw_key_component,
};

/// The phases of the iterative encode traversal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum EncodePhase {
    /// The frame has not processed its current node yet.
    Start,
    /// The frame is iterating over child keys.
    Iterate,
    /// The frame is waiting for a child to finish.
    AwaitChild,
}

/// A typed child selector used while traversing arrays and objects.
#[derive(Clone, Debug)]
pub(super) enum ChildKey {
    /// An object key.
    Key(String),
    /// An array index.
    Index(usize),
}

pub(super) fn encode_node_hot(
    body: &mut String,
    has_parts: &mut bool,
    node: &Value,
    path: KeyPathNode,
    options: &EncodeOptions,
    depth: usize,
) -> Result<(), EncodeError> {
    let mut stack = vec![BorrowedEncodeFrame::start(node, path, depth)];

    while let Some(mut frame) = stack.pop() {
        match frame.phase {
            EncodePhase::Start => {
                if let Some(max_depth) = options.max_depth
                    && frame.depth > max_depth
                {
                    return Err(EncodeError::DepthExceeded { depth: max_depth });
                }

                match frame.node {
                    Value::Null
                    | Value::Bool(_)
                    | Value::I64(_)
                    | Value::U64(_)
                    | Value::F64(_)
                    | Value::String(_)
                    | Value::Temporal(_)
                    | Value::Bytes(_) => {
                        if let Some(part) =
                            encode_scalar_leaf(frame.node, frame.path.materialize(), options)
                        {
                            emit_part(body, has_parts, &options.delimiter, &part);
                        }
                    }
                    Value::Array(items) if matches!(options.list_format, ListFormat::Comma) => {
                        for part in encode_comma_array(items, &frame.path, options) {
                            emit_part(body, has_parts, &options.delimiter, &part);
                        }
                    }
                    Value::Array(items) => {
                        if items.is_empty() {
                            if options.allow_empty_lists {
                                let key_path = frame.path.append_empty_list_suffix();
                                let part = finalize_key_path(key_path.materialize(), options);
                                emit_part(body, has_parts, &options.delimiter, &part);
                            }
                            continue;
                        }

                        frame.keys = ordered_array_indices(items, options)
                            .into_iter()
                            .map(ChildKey::Index)
                            .collect();
                        frame.index = 0;
                        frame.phase = EncodePhase::Iterate;
                        stack.push(frame);
                    }
                    Value::Object(object) => {
                        if object.is_empty() {
                            continue;
                        }

                        frame.keys = ordered_object_keys(object, options)
                            .into_iter()
                            .map(ChildKey::Key)
                            .collect();
                        frame.index = 0;
                        frame.phase = EncodePhase::Iterate;
                        stack.push(frame);
                    }
                }
            }
            EncodePhase::Iterate => {
                let Some(key) = frame.keys.get(frame.index).cloned() else {
                    continue;
                };
                frame.index += 1;

                let next = match (frame.node, &key) {
                    (Value::Object(object), ChildKey::Key(key_text)) => {
                        let child = object.get(key_text);
                        if options.skip_nulls && matches!(child, Some(Value::Null)) {
                            None
                        } else {
                            child.map(|child| {
                                let component = raw_key_component(key_text, options);
                                let path = if options.allow_dots {
                                    let base = if options.encode_dot_in_keys {
                                        frame.path.as_dot_encoded(encoded_dot_escape(options))
                                    } else {
                                        frame.path.clone()
                                    };
                                    base.append_dot_component(&component)
                                } else {
                                    frame.path.append_bracketed_component(&component)
                                };
                                (child, path)
                            })
                        }
                    }
                    (Value::Array(items), ChildKey::Index(index)) => {
                        let child = items.get(*index);
                        if options.skip_nulls && matches!(child, Some(Value::Null)) {
                            None
                        } else {
                            child.map(|child| {
                                let path = array_child_path(&frame.path, *index, options);
                                (child, path)
                            })
                        }
                    }
                    _ => None,
                };

                let child_depth = frame.depth + 1;
                stack.push(frame);
                if let Some((child, child_path)) = next {
                    stack.push(BorrowedEncodeFrame::start(child, child_path, child_depth));
                }
            }
            EncodePhase::AwaitChild => unreachable!("hot path does not await child results"),
        }
    }

    Ok(())
}

pub(super) struct BorrowedEncodeFrame<'a> {
    phase: EncodePhase,
    node: &'a Value,
    path: KeyPathNode,
    keys: Vec<ChildKey>,
    index: usize,
    depth: usize,
}

impl<'a> BorrowedEncodeFrame<'a> {
    fn start(node: &'a Value, path: KeyPathNode, depth: usize) -> Self {
        Self {
            phase: EncodePhase::Start,
            node,
            path,
            keys: Vec::new(),
            index: 0,
            depth,
        }
    }
}
