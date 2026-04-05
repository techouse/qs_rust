//! Raw query-string scanning façade for decode.

mod metadata;
mod parse;
mod parts;

pub(super) use self::metadata::{
    ScannedPart, ascii_case_insensitive_eq_bytes, byte_starts_numeric_entity_candidate,
    contains_ascii_case_insensitive_bytes, hex_value,
};
pub(super) use self::parse::parse_query_string_values;
#[cfg(test)]
pub(in crate::decode) use self::parts::{scan_default_parts_by_byte_delimiter, scan_string_parts};
