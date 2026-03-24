//! Structured node merge logic used during decode.

use std::collections::VecDeque;

use indexmap::IndexMap;

use crate::error::DecodeError;
use crate::internal::node::Node;
use crate::internal::overflow::{
    array_to_numeric_object, max_numeric_index, parse_canonical_index,
};
use crate::options::DecodeOptions;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MergePhase {
    Start,
    MapIter,
    MapAwait,
    ArrayIter,
    ArrayAwait,
}

/// Merges two internal decode nodes according to the configured list and
/// duplicate semantics.
pub(crate) fn merge(
    target: Node,
    source: Node,
    options: &DecodeOptions,
) -> Result<Node, DecodeError> {
    let mut stack = vec![MergeFrame::start(target, source, options)];
    let mut last_result: Option<Node> = None;

    while let Some(frame) = stack.last_mut() {
        match frame.phase {
            MergePhase::Start => {
                let target = frame.target.take().expect("start frame target missing");
                let source = frame.source.take().expect("start frame source missing");

                if source.is_null_source() {
                    finish_frame(&mut stack, &mut last_result, target);
                    continue;
                }

                if !source.is_map_like() {
                    match target {
                        Node::Array(target_items) => {
                            if target_items.iter().any(Node::is_undefined) {
                                let mut indexed = target_items;
                                match source {
                                    Node::Array(source_items) => {
                                        if source_items.len() > indexed.len() {
                                            indexed.resize(source_items.len(), Node::Undefined);
                                        }

                                        for (index, item) in source_items.into_iter().enumerate() {
                                            if !item.is_undefined() {
                                                indexed[index] = item;
                                            }
                                        }
                                    }
                                    other => indexed.push(other),
                                }

                                if !options.parse_lists && indexed.iter().any(Node::is_undefined) {
                                    let entries = array_to_numeric_object(indexed, false);
                                    finish_frame(
                                        &mut stack,
                                        &mut last_result,
                                        Node::Object(entries),
                                    );
                                } else {
                                    finish_frame(
                                        &mut stack,
                                        &mut last_result,
                                        Node::Array(indexed),
                                    );
                                }
                                continue;
                            }

                            match source {
                                Node::Array(source_items)
                                    if array_all_map_like_or_undefined(&target_items)
                                        && array_all_map_like_or_undefined(&source_items) =>
                                {
                                    frame.array_items = target_items;
                                    frame.source_array = source_items;
                                    frame.array_index = 0;
                                    frame.phase = MergePhase::ArrayIter;
                                    continue;
                                }
                                Node::Array(source_items) => {
                                    let mut merged =
                                        Vec::with_capacity(target_items.len() + source_items.len());
                                    merged.extend(target_items);
                                    merged.extend(
                                        source_items
                                            .into_iter()
                                            .filter(|item| !item.is_undefined()),
                                    );
                                    finish_frame(&mut stack, &mut last_result, Node::Array(merged));
                                    continue;
                                }
                                other => {
                                    let mut merged = target_items;
                                    merged.push(other);
                                    finish_frame(&mut stack, &mut last_result, Node::Array(merged));
                                    continue;
                                }
                            }
                        }
                        Node::Object(mut entries) => match source {
                            Node::Array(source_items) => {
                                for (index, item) in source_items.into_iter().enumerate() {
                                    if item.is_undefined() {
                                        continue;
                                    }

                                    let key = index.to_string();
                                    if let Some((_, _, existing)) = entries.get_full_mut(&key) {
                                        let current = std::mem::replace(existing, Node::Undefined);
                                        *existing = merge(current, item, options)?;
                                    } else {
                                        entries.insert(key, item);
                                    }
                                }
                                finish_frame(&mut stack, &mut last_result, Node::Object(entries));
                                continue;
                            }
                            Node::Undefined => {
                                finish_frame(&mut stack, &mut last_result, Node::Object(entries));
                                continue;
                            }
                            other => {
                                finish_frame(
                                    &mut stack,
                                    &mut last_result,
                                    Node::Array(vec![Node::Object(entries), other]),
                                );
                                continue;
                            }
                        },
                        Node::OverflowObject {
                            mut entries,
                            mut max_index,
                        } => {
                            match source {
                                Node::Array(source_items) => {
                                    for item in source_items {
                                        if item.is_undefined() {
                                            continue;
                                        }

                                        max_index = max_index.saturating_add(1);
                                        entries.insert(max_index.to_string(), item);
                                    }
                                }
                                Node::Undefined => {}
                                other => {
                                    max_index = max_index.saturating_add(1);
                                    entries.insert(max_index.to_string(), other);
                                }
                            }

                            finish_frame(
                                &mut stack,
                                &mut last_result,
                                Node::OverflowObject { entries, max_index },
                            );
                            continue;
                        }
                        other => match source {
                            Node::Array(source_items) => {
                                let mut merged =
                                    Vec::with_capacity(1usize.saturating_add(source_items.len()));
                                merged.push(other);
                                merged.extend(
                                    source_items.into_iter().filter(|item| !item.is_undefined()),
                                );
                                finish_frame(&mut stack, &mut last_result, Node::Array(merged));
                                continue;
                            }
                            source_other => {
                                finish_frame(
                                    &mut stack,
                                    &mut last_result,
                                    Node::Array(vec![other, source_other]),
                                );
                                continue;
                            }
                        },
                    }
                }

                match (target, source) {
                    (Node::Object(entries), Node::Object(source_entries)) => {
                        let max_index = max_numeric_index(&entries);
                        frame.map_result = entries;
                        frame.source_entries = source_entries.into_iter().collect();
                        frame.track_overflow = false;
                        frame.max_index = max_index;
                        frame.phase = MergePhase::MapIter;
                    }
                    (
                        Node::Object(entries),
                        Node::OverflowObject {
                            entries: source_entries,
                            max_index,
                        },
                    ) => {
                        frame.max_index =
                            Some(max_numeric_index(&entries).unwrap_or(0).max(max_index));
                        frame.map_result = entries;
                        frame.source_entries = source_entries.into_iter().collect();
                        frame.track_overflow = true;
                        frame.phase = MergePhase::MapIter;
                    }
                    (Node::OverflowObject { entries, max_index }, Node::Object(source_entries)) => {
                        let mut tracked = Some(max_index);
                        if let Some(source_max) = max_numeric_index(&source_entries) {
                            tracked = Some(tracked.unwrap_or(0).max(source_max));
                        }
                        frame.max_index = tracked;
                        frame.map_result = entries;
                        frame.source_entries = source_entries.into_iter().collect();
                        frame.track_overflow = true;
                        frame.phase = MergePhase::MapIter;
                    }
                    (
                        Node::OverflowObject { entries, max_index },
                        Node::OverflowObject {
                            entries: source_entries,
                            max_index: source_max,
                        },
                    ) => {
                        frame.max_index = Some(max_index.max(source_max));
                        frame.map_result = entries;
                        frame.source_entries = source_entries.into_iter().collect();
                        frame.track_overflow = true;
                        frame.phase = MergePhase::MapIter;
                    }
                    (Node::Array(target_items), source_map) => {
                        let source_is_overflow = matches!(source_map, Node::OverflowObject { .. });
                        let initial_max = target_items.len().checked_sub(1);
                        frame.map_result = array_to_numeric_object(target_items, false);
                        frame.max_index = match (&source_map, initial_max) {
                            (Node::OverflowObject { max_index, .. }, Some(current)) => {
                                Some(current.max(*max_index))
                            }
                            (Node::OverflowObject { max_index, .. }, None) => Some(*max_index),
                            (_, max) => max,
                        };
                        frame.source_entries = map_entries(source_map);
                        frame.track_overflow = source_is_overflow;
                        frame.phase = MergePhase::MapIter;
                    }
                    (Node::Undefined, source_map) => {
                        finish_frame(&mut stack, &mut last_result, source_map);
                        continue;
                    }
                    (other, Node::OverflowObject { entries, max_index }) => {
                        let mut shifted = IndexMap::with_capacity(entries.len() + 1);
                        shifted.insert("0".to_owned(), other);
                        for (key, value) in entries {
                            if let Some(index) = parse_canonical_index(&key) {
                                shifted.insert((index + 1).to_string(), value);
                            } else {
                                shifted.insert(key, value);
                            }
                        }

                        finish_frame(
                            &mut stack,
                            &mut last_result,
                            Node::OverflowObject {
                                entries: shifted,
                                max_index: max_index + 1,
                            },
                        );
                        continue;
                    }
                    (other, source_map) => {
                        finish_frame(
                            &mut stack,
                            &mut last_result,
                            Node::Array(vec![other, source_map]),
                        );
                        continue;
                    }
                }
            }
            MergePhase::MapIter => {
                if let Some((key, value)) = frame.source_entries.pop_front() {
                    if frame.track_overflow
                        && let Some(index) = parse_canonical_index(&key)
                    {
                        frame.max_index = Some(frame.max_index.unwrap_or(index).max(index));
                    }

                    if let Some((slot_index, _, existing)) = frame.map_result.get_full_mut(&key) {
                        let child_target = std::mem::replace(existing, Node::Undefined);
                        frame.pending_map_index = Some(slot_index);
                        frame.phase = MergePhase::MapAwait;
                        stack.push(MergeFrame::start(child_target, value, options));
                    } else {
                        frame.map_result.insert(key, value);
                    }
                    continue;
                }

                let result = if frame.track_overflow {
                    Node::OverflowObject {
                        entries: std::mem::take(&mut frame.map_result),
                        max_index: frame.max_index.unwrap_or(0),
                    }
                } else {
                    Node::Object(std::mem::take(&mut frame.map_result))
                };
                finish_frame(&mut stack, &mut last_result, result);
            }
            MergePhase::MapAwait => {
                let child = last_result.take().expect("missing child merge result");
                let slot_index = frame.pending_map_index.take().expect("missing map slot");
                if let Some((_, value)) = frame.map_result.get_index_mut(slot_index) {
                    *value = child;
                }
                frame.phase = MergePhase::MapIter;
            }
            MergePhase::ArrayIter => {
                if frame.array_index >= frame.source_array.len() {
                    let result = if !options.parse_lists
                        && frame.array_items.iter().any(Node::is_undefined)
                    {
                        Node::Object(array_to_numeric_object(
                            std::mem::take(&mut frame.array_items),
                            false,
                        ))
                    } else {
                        Node::Array(std::mem::take(&mut frame.array_items))
                    };
                    finish_frame(&mut stack, &mut last_result, result);
                    continue;
                }

                let index = frame.array_index;
                frame.array_index += 1;
                let item = std::mem::replace(&mut frame.source_array[index], Node::Undefined);

                if index < frame.array_items.len() {
                    if item.is_undefined() {
                        continue;
                    }

                    let child_target =
                        std::mem::replace(&mut frame.array_items[index], Node::Undefined);
                    frame.pending_array_index = Some(index);
                    frame.phase = MergePhase::ArrayAwait;
                    stack.push(MergeFrame::start(child_target, item, options));
                } else {
                    frame.array_items.push(item);
                }
            }
            MergePhase::ArrayAwait => {
                let child = last_result.take().expect("missing child array result");
                let index = frame
                    .pending_array_index
                    .take()
                    .expect("missing array slot");
                frame.array_items[index] = child;
                frame.phase = MergePhase::ArrayIter;
            }
        }
    }

    Ok(last_result.expect("merge root result missing"))
}

struct MergeFrame<'a> {
    phase: MergePhase,
    target: Option<Node>,
    source: Option<Node>,
    #[allow(dead_code)]
    options: &'a DecodeOptions,
    map_result: IndexMap<String, Node>,
    source_entries: VecDeque<(String, Node)>,
    track_overflow: bool,
    max_index: Option<usize>,
    pending_map_index: Option<usize>,
    array_items: Vec<Node>,
    source_array: Vec<Node>,
    array_index: usize,
    pending_array_index: Option<usize>,
}

impl<'a> MergeFrame<'a> {
    fn start(target: Node, source: Node, options: &'a DecodeOptions) -> Self {
        Self {
            phase: MergePhase::Start,
            target: Some(target),
            source: Some(source),
            options,
            map_result: IndexMap::new(),
            source_entries: VecDeque::new(),
            track_overflow: false,
            max_index: None,
            pending_map_index: None,
            array_items: Vec::new(),
            source_array: Vec::new(),
            array_index: 0,
            pending_array_index: None,
        }
    }
}

fn finish_frame(stack: &mut Vec<MergeFrame<'_>>, last_result: &mut Option<Node>, result: Node) {
    stack.pop();
    *last_result = Some(result);
}

fn map_entries(node: Node) -> VecDeque<(String, Node)> {
    match node {
        Node::Object(entries) => entries.into_iter().collect(),
        Node::OverflowObject { entries, .. } => entries.into_iter().collect(),
        _ => VecDeque::new(),
    }
}

fn array_all_map_like_or_undefined(items: &[Node]) -> bool {
    items
        .iter()
        .all(|item| item.is_undefined() || item.is_map_like())
}

#[cfg(test)]
mod tests;
