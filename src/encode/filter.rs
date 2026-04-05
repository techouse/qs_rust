//! Filter-aware encode traversal.

use crate::error::EncodeError;
use crate::key_path::KeyPathNode;
use crate::options::{EncodeFilter, EncodeOptions, FilterResult, ListFormat};
use crate::value::Value;

use super::comma::encode_comma_array_controlled;
use super::fast_path::try_encode_linear_map_chain;
use super::iterative::{ChildKey, EncodePhase};
use super::scalar::{encode_scalar_leaf, encoded_dot_escape, filter_prefix, finalize_key_path};
use super::{array_child_path, ordered_array_indices, ordered_object_keys, raw_key_component};

/// The filter-adjusted input owned by the filtered encode traversal.
pub(super) enum EncodeInput<'a> {
    /// Borrowed input value.
    Borrowed(&'a Value),
    /// Owned replacement value produced by a filter.
    Owned(Value),
    /// An omitted value.
    Omitted,
}

impl<'a> EncodeInput<'a> {
    pub(super) fn as_value(&self) -> Option<&Value> {
        match self {
            Self::Borrowed(value) => Some(value),
            Self::Owned(value) => Some(value),
            Self::Omitted => None,
        }
    }
}

pub(super) fn encode_node_filtered<'a>(
    input: EncodeInput<'a>,
    path: KeyPathNode,
    options: &'a EncodeOptions,
    depth: usize,
) -> Result<Vec<String>, EncodeError> {
    if let Some(value) = input.as_value()
        && let Some(fast) = try_encode_linear_map_chain(value, &path, options, depth)
    {
        return Ok(vec![fast]);
    }

    let mut stack = vec![EncodeFrame::start(input, path, depth)];
    let mut last_result: Option<Vec<String>> = None;

    while let Some(mut frame) = stack.pop() {
        match frame.phase {
            EncodePhase::Start => {
                if frame.input.as_value().is_none() {
                    last_result = Some(Vec::new());
                    continue;
                }

                if let Some(max_depth) = options.max_depth
                    && frame.depth > max_depth
                {
                    return Err(EncodeError::DepthExceeded { depth: max_depth });
                }

                if has_function_filter(options) {
                    let prefix = filter_prefix(frame.path.materialize(), options);
                    frame.input = apply_filter_result(frame.input, &prefix, options);
                }

                let Some(node) = frame.input.as_value() else {
                    last_result = Some(Vec::new());
                    continue;
                };

                match node {
                    Value::Null
                    | Value::Bool(_)
                    | Value::I64(_)
                    | Value::U64(_)
                    | Value::F64(_)
                    | Value::String(_)
                    | Value::Temporal(_)
                    | Value::Bytes(_) => {
                        last_result = Some(
                            encode_scalar_leaf(node, frame.path.materialize(), options)
                                .into_iter()
                                .collect(),
                        );
                    }
                    Value::Array(items) if matches!(options.list_format, ListFormat::Comma) => {
                        let parts = encode_comma_array_controlled(items, &frame.path, options);
                        last_result = Some(parts);
                    }
                    Value::Array(items) => {
                        if items.is_empty() {
                            if options.allow_empty_lists {
                                let key_path = frame.path.append_empty_list_suffix();
                                let part = finalize_key_path(key_path.materialize(), options);
                                last_result = Some(vec![part]);
                            } else {
                                last_result = Some(Vec::new());
                            }
                            continue;
                        }

                        frame.keys = ordered_array_indices(items, options)
                            .into_iter()
                            .map(ChildKey::Index)
                            .collect();
                        frame.index = 0;
                        frame.phase = EncodePhase::Iterate;
                    }
                    Value::Object(object) => {
                        if object.is_empty() {
                            last_result = Some(Vec::new());
                            continue;
                        }

                        frame.keys = ordered_object_keys(object, options)
                            .into_iter()
                            .map(ChildKey::Key)
                            .collect();
                        frame.index = 0;
                        frame.phase = EncodePhase::Iterate;
                    }
                }

                if matches!(frame.phase, EncodePhase::Iterate) {
                    stack.push(frame);
                }
            }
            EncodePhase::Iterate => {
                if frame.index >= frame.keys.len() {
                    let parts = std::mem::take(&mut frame.parts);
                    last_result = Some(parts);
                    continue;
                }

                let key = frame.keys[frame.index].clone();
                frame.index += 1;

                let next = {
                    let current = frame.input.as_value();

                    match (current, &key) {
                        (Some(Value::Object(object)), ChildKey::Key(key_text)) => {
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
                            let input = match object.get(key_text).cloned() {
                                Some(Value::Null) if options.skip_nulls => EncodeInput::Omitted,
                                Some(child) => EncodeInput::Owned(child),
                                None => EncodeInput::Omitted,
                            };
                            Some((input, path))
                        }
                        (Some(Value::Array(items)), ChildKey::Index(index)) => {
                            let path = array_child_path(&frame.path, *index, options);
                            let input = match items.get(*index).cloned() {
                                Some(Value::Null) if options.skip_nulls => EncodeInput::Omitted,
                                Some(child) => EncodeInput::Owned(child),
                                None => EncodeInput::Omitted,
                            };
                            Some((input, path))
                        }
                        _ => None,
                    }
                };

                let Some((input, child_path)) = next else {
                    stack.push(frame);
                    continue;
                };

                frame.phase = EncodePhase::AwaitChild;
                let child_depth = frame.depth + 1;
                stack.push(frame);
                stack.push(EncodeFrame::start(input, child_path, child_depth));
            }
            EncodePhase::AwaitChild => {
                let child_parts = last_result.take().expect("missing encoded child result");
                frame.parts.extend(child_parts);
                frame.phase = EncodePhase::Iterate;
                stack.push(frame);
            }
        }
    }

    Ok(last_result.unwrap_or_default())
}

pub(super) fn filter_root_value<'a>(value: &'a Value, options: &EncodeOptions) -> EncodeInput<'a> {
    let Some(EncodeFilter::Function(filter)) = options.filter.as_ref() else {
        return EncodeInput::Borrowed(value);
    };

    match filter.apply("", value) {
        FilterResult::Keep => EncodeInput::Borrowed(value),
        FilterResult::Omit => EncodeInput::Omitted,
        FilterResult::Replace(replacement) => {
            if matches!(replacement, Value::Array(_) | Value::Object(_)) {
                EncodeInput::Owned(replacement)
            } else {
                EncodeInput::Borrowed(value)
            }
        }
    }
}

fn apply_function_filter(
    prefix: &str,
    value: &Value,
    options: &EncodeOptions,
) -> Option<FilterResult> {
    match options.filter.as_ref() {
        Some(EncodeFilter::Function(filter)) => Some(filter.apply(prefix, value)),
        _ => None,
    }
}

pub(super) fn apply_filter_result<'a>(
    input: EncodeInput<'a>,
    prefix: &str,
    options: &'a EncodeOptions,
) -> EncodeInput<'a> {
    let Some(value) = input.as_value() else {
        return EncodeInput::Omitted;
    };

    match apply_function_filter(prefix, value, options) {
        Some(FilterResult::Keep) | None => input,
        Some(FilterResult::Omit) => EncodeInput::Omitted,
        Some(FilterResult::Replace(value)) => EncodeInput::Owned(value),
    }
}

fn has_function_filter(options: &EncodeOptions) -> bool {
    matches!(options.filter.as_ref(), Some(EncodeFilter::Function(_)))
}

pub(super) fn has_filter_control(options: &EncodeOptions) -> bool {
    options.filter.is_some()
}

struct EncodeFrame<'a> {
    phase: EncodePhase,
    input: EncodeInput<'a>,
    path: KeyPathNode,
    keys: Vec<ChildKey>,
    index: usize,
    parts: Vec<String>,
    depth: usize,
}

impl<'a> EncodeFrame<'a> {
    fn start(input: EncodeInput<'a>, path: KeyPathNode, depth: usize) -> Self {
        Self {
            phase: EncodePhase::Start,
            input,
            path,
            keys: Vec::new(),
            index: 0,
            parts: Vec::new(),
            depth,
        }
    }
}

#[cfg(test)]
mod tests;
