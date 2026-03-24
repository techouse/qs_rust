pub(super) use super::{
    encode, encode_comma_array, encode_key_only_fragment, encoded_dot_escape, percent_encode_bytes,
    percent_encode_latin1, try_encode_linear_map_chain,
};
pub(super) use crate::key_path::KeyPathNode;
pub(super) use crate::options::{
    Charset, EncodeFilter, EncodeOptions, EncodeToken, EncodeTokenEncoder, FilterResult, Format,
    FunctionFilter, ListFormat, Sorter, TemporalSerializer, WhitelistSelector,
};
pub(super) use crate::temporal::{DateTimeValue, TemporalValue};
pub(super) use crate::value::Value;
pub(super) use std::cmp::Ordering;

pub(super) fn escape_dots_in_materialized_path(path: &str, options: &EncodeOptions) -> String {
    path.replace('.', encoded_dot_escape(options))
}

mod fast_path;
mod filters;
mod helpers;
mod iterative;
mod temporal;
