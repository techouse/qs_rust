//! Structured decode handoff from flat pairs into merge/compact processing.

use indexmap::IndexMap;

use crate::compact::{compact, node_to_object};
use crate::error::DecodeError;
use crate::internal::node::Node;
use crate::merge::merge;
use crate::options::DecodeOptions;
use crate::structured_scan::StructuredKeyScan;
use crate::value::Object;

use super::flat::FlatValues;
use super::keys::parse_keys;

/// Rebuilds structured objects from the flat pair map once structured key
/// syntax has been confirmed.
pub(super) fn decode_from_pairs_map(
    temp_values: FlatValues,
    options: &DecodeOptions,
    structured_scan: &StructuredKeyScan,
) -> Result<Object, DecodeError> {
    let mut root = Node::Object(IndexMap::new());

    for (key, parsed_value) in temp_values.into_parsed_map() {
        let value = parsed_value.into_node();
        if !structured_scan.contains_structured_key(&key)
            && !structured_scan.contains_structured_root(&key)
        {
            if let Node::Object(entries) = &mut root {
                if let Some((_, _, existing)) = entries.get_full_mut(&key) {
                    let current = std::mem::replace(existing, Node::Undefined);
                    *existing = merge(current, value, options)?;
                } else {
                    entries.insert(key, value);
                }
            }
            continue;
        }

        if let Some(parsed) = parse_keys(&key, value, options)? {
            root = merge(root, parsed, options)?;
        }
    }

    let compacted = compact(root, options.allow_sparse_lists);
    Ok(node_to_object(compacted))
}
