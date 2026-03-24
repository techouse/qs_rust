//! The encode linear-chain fast path.

use crate::key_path::KeyPathNode;
use crate::options::{EncodeOptions, ListFormat, SortMode};
use crate::value::Value;

use super::raw_key_component;
use super::scalar::encode_scalar_leaf;

pub(super) fn try_encode_linear_map_chain(
    value: &Value,
    path: &KeyPathNode,
    options: &EncodeOptions,
    depth: usize,
) -> Option<String> {
    if options.strict_null_handling
        || options.skip_nulls
        || options.allow_empty_lists
        || options.encode_values_only
        || options.allow_dots
        || options.encode_dot_in_keys
        || options.filter.is_some()
        || options.sorter.is_some()
        || options.encoder.is_some()
        || options.has_temporal_serializer()
        || !matches!(options.list_format, ListFormat::Indices)
        || options.comma_round_trip
        || options.comma_compact_nulls
        || !matches!(options.sort, SortMode::Preserve)
        || options.max_depth.is_some_and(|max| depth > max)
    {
        return None;
    }

    let mut current = value;
    let mut materialized = path.materialize().to_owned();
    let mut current_depth = depth;

    loop {
        match current {
            Value::Object(object) => {
                if object.len() != 1 {
                    return None;
                }
                current_depth = current_depth.saturating_add(1);
                if options.max_depth.is_some_and(|max| current_depth > max) {
                    return None;
                }
                let (key, next) = object.iter().next()?;
                append_bracketed_path_component(
                    &mut materialized,
                    &raw_key_component(key, options),
                );
                current = next;
            }
            Value::Array(_) => return None,
            scalar => return encode_scalar_leaf(scalar, &materialized, options),
        }
    }
}

fn append_bracketed_path_component(path: &mut String, component: &str) {
    path.push('[');
    path.push_str(component);
    path.push(']');
}
