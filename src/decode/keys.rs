//! Structured-key parsing for decode.

use indexmap::IndexMap;

use crate::error::DecodeError;
use crate::internal::node::Node;
use crate::internal::overflow::parse_canonical_index;
use crate::options::DecodeOptions;

use super::accumulate::combine_with_limit;
use super::scan::contains_ascii_case_insensitive_bytes;

pub(crate) fn split_key_into_segments(
    original_key: &str,
    allow_dots: bool,
    max_depth: usize,
    strict_depth: bool,
) -> Result<Vec<String>, DecodeError> {
    if max_depth == 0 {
        return Ok(vec![original_key.to_owned()]);
    }

    let key = if allow_dots {
        dot_to_bracket_top_level(original_key)
    } else {
        original_key.to_owned()
    };

    let mut segments = Vec::new();
    let first = key.find('[');
    let parent = first.map_or_else(|| key.as_str(), |index| &key[..index]);
    if !parent.is_empty() {
        segments.push(parent.to_owned());
    }

    let mut open = first;
    let mut depth = 0usize;
    let mut last_close = None;
    let mut broke_unterminated = false;

    while let Some(open_index) = open {
        if depth >= max_depth {
            break;
        }

        let mut level = 1usize;
        let mut scan = open_index + 1;
        let mut close = None;

        while scan < key.len() {
            match key.as_bytes()[scan] {
                b'[' => level += 1,
                b']' => {
                    level -= 1;
                    if level == 0 {
                        close = Some(scan);
                        break;
                    }
                }
                _ => {}
            }
            scan += 1;
        }

        let Some(close_index) = close else {
            if let Some(recovered_open) = find_recoverable_balanced_open(&key, open_index + 1) {
                if let Some(last) = segments.last_mut() {
                    last.push_str(&key[open_index..recovered_open]);
                } else {
                    segments.push(key[..recovered_open].to_owned());
                }
                open = Some(recovered_open);
                continue;
            }
            broke_unterminated = true;
            break;
        };

        segments.push(key[open_index..=close_index].to_owned());
        last_close = Some(close_index);
        depth += 1;
        open = key[close_index + 1..]
            .find('[')
            .map(|relative| close_index + 1 + relative);
    }

    if let Some(open_index) = open {
        if strict_depth && !broke_unterminated {
            return Err(DecodeError::DepthExceeded { depth: max_depth });
        }

        if broke_unterminated && first == Some(0) {
            return Ok(vec![original_key.to_owned()]);
        }

        let remainder = &key[open_index..];
        segments.push(format!("[{remainder}]"));
        return Ok(segments);
    }

    if let Some(close_index) = last_close
        && close_index + 1 < key.len()
    {
        let trailing = &key[close_index + 1..];
        if trailing != "." {
            if strict_depth {
                return Err(DecodeError::DepthExceeded { depth: max_depth });
            }
            segments.push(format!("[{trailing}]"));
        }
    }

    Ok(segments)
}

pub(super) fn parse_keys(
    given_key: &str,
    value: Node,
    options: &DecodeOptions,
) -> Result<Option<Node>, DecodeError> {
    if given_key.is_empty() {
        return Ok(None);
    }

    let segments = split_key_into_segments(
        given_key,
        options.allow_dots,
        options.depth,
        options.strict_depth,
    )?;
    Ok(Some(parse_object(segments, value, options)?))
}

fn parse_object(
    chain: Vec<String>,
    value: Node,
    options: &DecodeOptions,
) -> Result<Node, DecodeError> {
    let mut leaf = value;

    for root in chain.into_iter().rev() {
        if root == "[]" {
            if !options.parse_lists {
                let mut object = IndexMap::new();
                object.insert("0".to_owned(), leaf);
                leaf = Node::Object(object);
                continue;
            }

            if matches!(leaf, Node::OverflowObject { .. }) {
                continue;
            }

            if should_create_empty_list(options, &leaf) {
                leaf = Node::Array(Vec::new());
            } else {
                leaf = combine_with_limit(Node::Array(Vec::new()), leaf, options)?;
            }
            continue;
        }

        let mut clean_root = if root.starts_with('[') && root.ends_with(']') && root.len() >= 2 {
            root[1..root.len() - 1].to_owned()
        } else {
            root.clone()
        };

        if root.starts_with('[')
            && root.ends_with(']')
            && clean_root.matches('[').count() > clean_root.matches(']').count()
            && clean_root.ends_with(']')
        {
            clean_root.pop();
        }

        if options.decode_dot_in_keys && clean_root.contains('%') {
            clean_root = replace_ascii_case_insensitive(&clean_root, "%2E", ".");
        }

        if !options.parse_lists {
            let mut object = IndexMap::new();
            object.insert(clean_root, leaf);
            leaf = Node::Object(object);
            continue;
        }

        if let Some(index) = parse_canonical_index(&clean_root)
            && root != clean_root
        {
            if index < options.list_limit {
                let mut items = Vec::with_capacity(index + 1);
                let mut moved_leaf = Some(leaf);
                for slot in 0..=index {
                    if slot == index {
                        items.push(moved_leaf.take().expect("leaf already moved"));
                    } else {
                        items.push(Node::Undefined);
                    }
                }
                leaf = Node::Array(items);
                continue;
            }

            if options.throw_on_limit_exceeded {
                return Err(DecodeError::ListLimitExceeded {
                    limit: options.list_limit,
                });
            }
        }

        let mut object = IndexMap::new();
        object.insert(clean_root, leaf);
        leaf = Node::Object(object);
    }

    Ok(leaf)
}

fn should_create_empty_list(options: &DecodeOptions, leaf: &Node) -> bool {
    if !options.allow_empty_lists {
        return false;
    }

    matches!(
        leaf,
        Node::Value(crate::value::Value::String(text)) if text.is_empty()
    ) || matches!(leaf, Node::Value(crate::value::Value::Null))
}

pub(super) fn key_might_be_structured(key: &str, options: &DecodeOptions) -> bool {
    key.contains('[')
        || (options.allow_dots
            && (key.contains('.') || contains_ascii_case_insensitive_bytes(key.as_bytes(), b"%2E")))
}

pub(super) fn dot_to_bracket_top_level(key: &str) -> String {
    if !key.contains('.') {
        return key.to_owned();
    }

    let mut output = String::with_capacity(key.len() + 4);
    let mut depth = 0usize;
    let chars: Vec<char> = key.chars().collect();
    let mut index = 0usize;

    while index < chars.len() {
        let ch = chars[index];
        match ch {
            '[' => {
                depth += 1;
                output.push(ch);
            }
            ']' => {
                depth = depth.saturating_sub(1);
                output.push(ch);
            }
            '.' if depth == 0 => {
                let has_next = index + 1 < chars.len();
                let next = if has_next { chars[index + 1] } else { '\0' };
                if next == '[' || !has_next || next == '.' {
                    output.push('.');
                } else {
                    let mut scan = index + 1;
                    while scan < chars.len() && chars[scan] != '.' && chars[scan] != '[' {
                        scan += 1;
                    }
                    output.push('[');
                    for current in &chars[index + 1..scan] {
                        output.push(*current);
                    }
                    output.push(']');
                    index = scan - 1;
                }
            }
            _ => output.push(ch),
        }
        index += 1;
    }

    output
}

fn replace_ascii_case_insensitive(input: &str, needle: &str, replacement: &str) -> String {
    if needle.is_empty() || input.is_empty() {
        return input.to_owned();
    }

    let needle_len = needle.len();
    let needle_bytes = needle.as_bytes();
    let input_bytes = input.as_bytes();
    let mut output = String::with_capacity(input.len());
    let mut index = 0usize;

    while index < input.len() {
        if index + needle_len <= input.len()
            && super::scan::ascii_case_insensitive_eq_bytes(
                &input_bytes[index..index + needle_len],
                needle_bytes,
            )
        {
            output.push_str(replacement);
            index += needle_len;
        } else {
            let ch = input[index..]
                .chars()
                .next()
                .expect("index should always be at a char boundary");
            output.push(ch);
            index += ch.len_utf8();
        }
    }
    output
}

pub(super) fn find_recoverable_balanced_open(key: &str, start: usize) -> Option<usize> {
    let bytes = key.as_bytes();
    let mut candidate = start;

    while candidate < bytes.len() {
        if bytes[candidate] != b'[' {
            candidate += 1;
            continue;
        }

        let mut level = 1usize;
        let mut scan = candidate + 1;
        while scan < bytes.len() {
            match bytes[scan] {
                b'[' => level += 1,
                b']' => {
                    level -= 1;
                    if level == 0 {
                        return Some(candidate);
                    }
                }
                _ => {}
            }
            scan += 1;
        }

        candidate += 1;
    }

    None
}
