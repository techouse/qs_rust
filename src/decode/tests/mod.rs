pub(super) use super::scan::{
    ScannedPart, scan_default_parts_by_byte_delimiter, scan_string_parts,
};
pub(super) use super::{
    DefaultAccumulator, FlatValues, ParsedFlatValue, collect_pair_values, combine_with_limit,
    decode, decode_component, decode_from_pairs_map, decode_pairs, decode_scalar,
    dot_to_bracket_top_level, finalize_flat, find_recoverable_balanced_open,
    interpret_numeric_entities, interpret_numeric_entities_in_node, parse_keys,
    parse_query_string_values, split_key_into_segments, value_list_length_for_combine,
};
pub(super) use crate::internal::node::Node;
pub(super) use crate::options::{
    Charset, DecodeDecoder, DecodeKind, DecodeOptions, Delimiter, Duplicates,
};
pub(super) use crate::structured_scan::scan_structured_keys;
pub(super) use crate::value::Value;
pub(super) use indexmap::IndexMap;
pub(super) use regex::Regex;

pub(super) fn stores_concrete_value(values: &FlatValues, key: &str) -> bool {
    matches!(values, FlatValues::Concrete(entries) if entries.contains_key(key))
}

pub(super) fn stores_parsed_value(values: &FlatValues, key: &str) -> bool {
    matches!(values, FlatValues::Parsed(entries) if entries.contains_key(key))
}

pub(super) fn stores_parsed_value_with_compaction(values: &FlatValues, key: &str) -> bool {
    matches!(
        values,
        FlatValues::Parsed(entries)
            if matches!(
                entries.get(key),
                Some(ParsedFlatValue::Parsed {
                    needs_compaction: true,
                    ..
                })
            )
    )
}

mod charset;
mod duplicates;
mod flat;
mod keys;
mod parts;
mod scalar_helpers;
mod scanner;
