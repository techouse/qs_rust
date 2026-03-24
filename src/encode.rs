//! Public query-string encoding entrypoints.

mod comma;
mod fast_path;
mod filter;
mod iterative;
mod scalar;

use crate::error::EncodeError;
use crate::key_path::KeyPathNode;
use crate::options::{EncodeOptions, SortMode, WhitelistSelector};
use crate::value::{Object, Value};

use self::fast_path::try_encode_linear_map_chain;
use self::filter::{EncodeInput, encode_node_filtered, filter_root_value, has_filter_control};
use self::iterative::encode_node_hot;
use self::scalar::encoded_dot_escape;

#[cfg(test)]
use self::comma::encode_comma_array;
#[cfg(test)]
use self::scalar::{encode_key_only_fragment, percent_encode_bytes, percent_encode_latin1};

/// Encodes a [`Value`] tree into a query string.
///
/// The encoder preserves object insertion order by default, supports multiple
/// list formats, and can be customized with filters, sorters, key/value
/// encoders,
/// and temporal serializers through [`EncodeOptions`].
///
/// # Errors
///
/// Returns [`EncodeError`] when the supplied [`EncodeOptions`] are invalid or
/// when encoding exceeds a configured maximum depth.
///
/// # Examples
///
/// ```
/// use qs_rust::{EncodeOptions, Object, Value, encode};
///
/// let mut root = Object::new();
/// root.insert("a".to_owned(), Value::String("1".to_owned()));
/// root.insert("b".to_owned(), Value::String("2".to_owned()));
///
/// let encoded = encode(&Value::Object(root), &EncodeOptions::new()).unwrap();
/// assert_eq!(encoded, "a=1&b=2");
/// ```
pub fn encode(value: &Value, options: &EncodeOptions) -> Result<String, EncodeError> {
    options.validate()?;

    let root = filter_root_value(value, options);
    let mut body = String::new();
    let mut has_parts = false;

    match root {
        EncodeInput::Omitted => return Ok(String::new()),
        EncodeInput::Borrowed(root) => {
            append_root_output(root, options, &mut body, &mut has_parts)?
        }
        EncodeInput::Owned(root) => append_root_output(&root, options, &mut body, &mut has_parts)?,
    }

    if !has_parts || body.is_empty() {
        return Ok(String::new());
    }

    let mut output = String::new();
    if options.add_query_prefix {
        output.push('?');
    }

    if options.charset_sentinel {
        output.push_str(options.charset.sentinel());
        output.push('&');
    }

    output.push_str(&body);
    Ok(output)
}

fn append_root_output(
    value: &Value,
    options: &EncodeOptions,
    body: &mut String,
    has_parts: &mut bool,
) -> Result<(), EncodeError> {
    match value {
        Value::Object(object) => {
            for key in ordered_object_keys(object, options) {
                let Some(child) = object.get(&key) else {
                    continue;
                };
                let path =
                    KeyPathNode::from_raw(raw_root_key_component(&key, Some(child), options));
                append_encoded_node(body, has_parts, child, path, options, 0)?;
            }
        }
        Value::Array(items) => {
            for index in ordered_array_indices(items, options) {
                let Some(child) = items.get(index) else {
                    continue;
                };
                let path = KeyPathNode::from_raw(raw_key_component(&index.to_string(), options));
                append_encoded_node(body, has_parts, child, path, options, 0)?;
            }
        }
        _ => {}
    }

    Ok(())
}

fn append_encoded_node(
    body: &mut String,
    has_parts: &mut bool,
    node: &Value,
    path: KeyPathNode,
    options: &EncodeOptions,
    depth: usize,
) -> Result<(), EncodeError> {
    if let Some(fast) = try_encode_linear_map_chain(node, &path, options, depth) {
        emit_part(body, has_parts, &options.delimiter, &fast);
        return Ok(());
    }

    if has_filter_control(options) {
        let encoded = encode_node_filtered(EncodeInput::Borrowed(node), path, options, depth)?;
        append_parts(body, has_parts, &options.delimiter, encoded);
        return Ok(());
    }

    encode_node_hot(body, has_parts, node, path, options, depth)
}

fn array_child_path(path: &KeyPathNode, index: usize, options: &EncodeOptions) -> KeyPathNode {
    match options.list_format {
        crate::options::ListFormat::Indices => path.append_bracketed_component(&index.to_string()),
        crate::options::ListFormat::Brackets => path.append_empty_list_suffix(),
        crate::options::ListFormat::Repeat | crate::options::ListFormat::Comma => path.clone(),
    }
}

fn emit_part(body: &mut String, has_parts: &mut bool, delimiter: &str, part: &str) {
    if part.is_empty() {
        return;
    }
    if *has_parts {
        body.push_str(delimiter);
    }
    body.push_str(part);
    *has_parts = true;
}

fn append_parts(body: &mut String, has_parts: &mut bool, delimiter: &str, parts: Vec<String>) {
    for part in parts {
        emit_part(body, has_parts, delimiter, &part);
    }
}

fn ordered_object_keys(object: &Object, options: &EncodeOptions) -> Vec<String> {
    let mut keys = match options.filter.as_ref() {
        Some(crate::options::EncodeFilter::Whitelist(whitelist)) => whitelist
            .iter()
            .filter_map(|selector| match selector {
                WhitelistSelector::Key(key) => Some(key.clone()),
                WhitelistSelector::Index(_) => None,
            })
            .collect::<Vec<_>>(),
        _ => object.keys().cloned().collect::<Vec<_>>(),
    };

    if let Some(sorter) = options.sorter.as_ref() {
        keys.sort_by(|left, right| sorter.compare(left, right));
    } else if matches!(options.sort, SortMode::LexicographicAsc) {
        keys.sort();
    }

    keys
}

fn ordered_array_indices(items: &[Value], options: &EncodeOptions) -> Vec<usize> {
    match options.filter.as_ref() {
        Some(crate::options::EncodeFilter::Whitelist(whitelist)) => whitelist
            .iter()
            .filter_map(|selector| match selector {
                WhitelistSelector::Index(index) => Some(*index),
                WhitelistSelector::Key(_) => None,
            })
            .collect(),
        _ => (0..items.len()).collect(),
    }
}

fn raw_key_component(key: &str, options: &EncodeOptions) -> String {
    if options.allow_dots && options.encode_dot_in_keys {
        key.replace('.', encoded_dot_escape(options))
    } else {
        key.to_owned()
    }
}

fn raw_root_key_component(key: &str, value: Option<&Value>, options: &EncodeOptions) -> String {
    if options.allow_dots
        && options.encode_dot_in_keys
        && matches!(value, Some(Value::Array(_)) | Some(Value::Object(_)))
    {
        key.replace('.', encoded_dot_escape(options))
    } else {
        key.to_owned()
    }
}

#[cfg(test)]
mod tests;
