#![forbid(unsafe_code)]

use std::sync::OnceLock;

use qs_rust::{
    Charset, DecodeOptions, Delimiter, Duplicates, EncodeOptions, Format, ListFormat, Object,
    SortMode, Value, decode, decode_pairs, encode,
};
use regex::Regex;
use serde::Deserialize;

const MAX_QUERY_LEN: usize = 512;
const MAX_STRING_LEN: usize = 64;
const MAX_KEY_LEN: usize = 32;
const MAX_BYTES_LEN: usize = 32;
const MAX_ARRAY_LEN: usize = 8;
const MAX_OBJECT_LEN: usize = 8;
const MAX_VALUE_DEPTH: usize = 4;
const MAX_DECODE_DEPTH: usize = 8;
const MAX_LIST_LIMIT: usize = 32;
const MAX_PARAMETER_LIMIT: usize = 256;
const MAX_PAIRS: usize = 16;
const MAX_ENCODE_DEPTH: usize = 8;

#[derive(Debug, Deserialize, Default)]
pub struct DecodeCase {
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub options: FuzzDecodeOptions,
}

#[derive(Debug, Deserialize, Default)]
pub struct EncodeCase {
    #[serde(default)]
    pub value: FuzzValue,
    #[serde(default)]
    pub options: FuzzEncodeOptions,
}

#[derive(Debug, Deserialize, Default)]
pub struct DecodePairsCase {
    #[serde(default)]
    pub pairs: Vec<FuzzPair>,
    #[serde(default)]
    pub options: FuzzDecodeOptions,
}

#[derive(Debug, Deserialize, Default)]
pub struct FuzzPair {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub value: FuzzValue,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct FuzzDecodeOptions {
    #[serde(default)]
    pub allow_dots: bool,
    #[serde(default)]
    pub decode_dot_in_keys: bool,
    #[serde(default)]
    pub allow_empty_lists: bool,
    #[serde(default)]
    pub allow_sparse_lists: bool,
    #[serde(default = "default_decode_list_limit")]
    pub list_limit: usize,
    #[serde(default)]
    pub charset: FuzzCharset,
    #[serde(default)]
    pub charset_sentinel: bool,
    #[serde(default)]
    pub comma: bool,
    #[serde(default)]
    pub delimiter: FuzzDecodeDelimiter,
    #[serde(default = "default_decode_depth")]
    pub depth: usize,
    #[serde(default = "default_parameter_limit")]
    pub parameter_limit: usize,
    #[serde(default)]
    pub duplicates: FuzzDuplicates,
    #[serde(default)]
    pub ignore_query_prefix: bool,
    #[serde(default)]
    pub interpret_numeric_entities: bool,
    #[serde(default = "default_true")]
    pub parse_lists: bool,
    #[serde(default)]
    pub strict_depth: bool,
    #[serde(default)]
    pub strict_null_handling: bool,
    #[serde(default)]
    pub throw_on_limit_exceeded: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FuzzEncodeOptions {
    #[serde(default = "default_true")]
    pub encode: bool,
    #[serde(default)]
    pub delimiter: FuzzEncodeDelimiter,
    #[serde(default)]
    pub list_format: FuzzListFormat,
    #[serde(default)]
    pub format: FuzzFormat,
    #[serde(default)]
    pub charset: FuzzCharset,
    #[serde(default)]
    pub charset_sentinel: bool,
    #[serde(default)]
    pub allow_empty_lists: bool,
    #[serde(default)]
    pub strict_null_handling: bool,
    #[serde(default)]
    pub skip_nulls: bool,
    #[serde(default)]
    pub comma_round_trip: bool,
    #[serde(default)]
    pub comma_compact_nulls: bool,
    #[serde(default)]
    pub encode_values_only: bool,
    #[serde(default)]
    pub add_query_prefix: bool,
    #[serde(default)]
    pub allow_dots: bool,
    #[serde(default)]
    pub encode_dot_in_keys: bool,
    #[serde(default)]
    pub sort_mode: FuzzSortMode,
    #[serde(default)]
    pub max_depth: Option<usize>,
}

impl Default for FuzzEncodeOptions {
    fn default() -> Self {
        Self {
            encode: true,
            delimiter: FuzzEncodeDelimiter::default(),
            list_format: FuzzListFormat::default(),
            format: FuzzFormat::default(),
            charset: FuzzCharset::default(),
            charset_sentinel: false,
            allow_empty_lists: false,
            strict_null_handling: false,
            skip_nulls: false,
            comma_round_trip: false,
            comma_compact_nulls: false,
            encode_values_only: false,
            add_query_prefix: false,
            allow_dots: false,
            encode_dot_in_keys: false,
            sort_mode: FuzzSortMode::default(),
            max_depth: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FuzzCharset {
    #[default]
    Utf8,
    Iso88591,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FuzzDuplicates {
    #[default]
    Combine,
    First,
    Last,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FuzzListFormat {
    #[default]
    Indices,
    Brackets,
    Repeat,
    Comma,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FuzzFormat {
    #[default]
    Rfc3986,
    Rfc1738,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FuzzSortMode {
    #[default]
    Preserve,
    LexicographicAsc,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FuzzDecodeDelimiter {
    #[default]
    Ampersand,
    Semicolon,
    Pipe,
    DoubleAmpersand,
    Empty,
    RegexAmpOrSemicolon,
    RegexAmpSemicolonOrPipe,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FuzzEncodeDelimiter {
    #[default]
    Ampersand,
    Semicolon,
    Pipe,
    DoubleAmpersand,
    Empty,
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FuzzValue {
    #[default]
    Null,
    Bool {
        value: bool,
    },
    I64 {
        value: i64,
    },
    U64 {
        value: u64,
    },
    F64 {
        value: f64,
    },
    String {
        #[serde(default)]
        value: String,
    },
    Bytes {
        #[serde(default)]
        value: Vec<u8>,
    },
    Array {
        #[serde(default)]
        items: Vec<FuzzValue>,
    },
    Object {
        #[serde(default)]
        entries: Vec<FuzzEntry>,
    },
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct FuzzEntry {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub value: FuzzValue,
}

pub fn run_decode_bytes(data: &[u8]) {
    let Some(case) = serde_json::from_slice::<DecodeCase>(data).ok() else {
        return;
    };

    let _ = decode(
        &truncate_string(case.query, MAX_QUERY_LEN),
        &case.options.into_decode_options(),
    );
}

pub fn run_encode_bytes(data: &[u8]) {
    let Some(case) = serde_json::from_slice::<EncodeCase>(data).ok() else {
        return;
    };

    let _ = encode(
        &case.value.into_value_with_depth(MAX_VALUE_DEPTH),
        &case.options.into_encode_options(),
    );
}

pub fn run_decode_pairs_bytes(data: &[u8]) {
    let Some(case) = serde_json::from_slice::<DecodePairsCase>(data).ok() else {
        return;
    };

    let pairs = case
        .pairs
        .into_iter()
        .take(MAX_PAIRS)
        .map(|pair| {
            (
                truncate_string(pair.key, MAX_KEY_LEN),
                pair.value.into_value_with_depth(MAX_VALUE_DEPTH),
            )
        })
        .collect::<Vec<_>>();

    let _ = decode_pairs(pairs, &case.options.into_decode_options());
}

impl FuzzDecodeOptions {
    fn into_decode_options(self) -> DecodeOptions {
        DecodeOptions::new()
            .with_allow_dots(self.allow_dots)
            .with_decode_dot_in_keys(self.decode_dot_in_keys)
            .with_allow_empty_lists(self.allow_empty_lists)
            .with_allow_sparse_lists(self.allow_sparse_lists)
            .with_list_limit(self.list_limit.min(MAX_LIST_LIMIT))
            .with_charset(self.charset.into_charset())
            .with_charset_sentinel(self.charset_sentinel)
            .with_comma(self.comma)
            .with_delimiter(self.delimiter.into_delimiter())
            .with_depth(self.depth.min(MAX_DECODE_DEPTH))
            .with_parameter_limit(self.parameter_limit.min(MAX_PARAMETER_LIMIT))
            .with_duplicates(self.duplicates.into_duplicates())
            .with_ignore_query_prefix(self.ignore_query_prefix)
            .with_interpret_numeric_entities(self.interpret_numeric_entities)
            .with_parse_lists(self.parse_lists)
            .with_strict_depth(self.strict_depth)
            .with_strict_null_handling(self.strict_null_handling)
            .with_throw_on_limit_exceeded(self.throw_on_limit_exceeded)
    }
}

impl FuzzEncodeOptions {
    fn into_encode_options(self) -> EncodeOptions {
        EncodeOptions::new()
            .with_encode(self.encode)
            .with_delimiter(self.delimiter.as_str())
            .with_list_format(self.list_format.into_list_format())
            .with_format(self.format.into_format())
            .with_charset(self.charset.into_charset())
            .with_charset_sentinel(self.charset_sentinel)
            .with_allow_empty_lists(self.allow_empty_lists)
            .with_strict_null_handling(self.strict_null_handling)
            .with_skip_nulls(self.skip_nulls)
            .with_comma_round_trip(self.comma_round_trip)
            .with_comma_compact_nulls(self.comma_compact_nulls)
            .with_encode_values_only(self.encode_values_only)
            .with_add_query_prefix(self.add_query_prefix)
            .with_allow_dots(self.allow_dots)
            .with_encode_dot_in_keys(self.encode_dot_in_keys)
            .with_sort(self.sort_mode.into_sort_mode())
            .with_max_depth(self.max_depth.map(|depth| depth.min(MAX_ENCODE_DEPTH)))
    }
}

impl FuzzCharset {
    fn into_charset(self) -> Charset {
        match self {
            Self::Utf8 => Charset::Utf8,
            Self::Iso88591 => Charset::Iso88591,
        }
    }
}

impl FuzzDuplicates {
    fn into_duplicates(self) -> Duplicates {
        match self {
            Self::Combine => Duplicates::Combine,
            Self::First => Duplicates::First,
            Self::Last => Duplicates::Last,
        }
    }
}

impl FuzzListFormat {
    fn into_list_format(self) -> ListFormat {
        match self {
            Self::Indices => ListFormat::Indices,
            Self::Brackets => ListFormat::Brackets,
            Self::Repeat => ListFormat::Repeat,
            Self::Comma => ListFormat::Comma,
        }
    }
}

impl FuzzFormat {
    fn into_format(self) -> Format {
        match self {
            Self::Rfc3986 => Format::Rfc3986,
            Self::Rfc1738 => Format::Rfc1738,
        }
    }
}

impl FuzzSortMode {
    fn into_sort_mode(self) -> SortMode {
        match self {
            Self::Preserve => SortMode::Preserve,
            Self::LexicographicAsc => SortMode::LexicographicAsc,
        }
    }
}

impl FuzzDecodeDelimiter {
    fn into_delimiter(self) -> Delimiter {
        match self {
            Self::Ampersand => Delimiter::String("&".to_owned()),
            Self::Semicolon => Delimiter::String(";".to_owned()),
            Self::Pipe => Delimiter::String("|".to_owned()),
            Self::DoubleAmpersand => Delimiter::String("&&".to_owned()),
            Self::Empty => Delimiter::String(String::new()),
            Self::RegexAmpOrSemicolon => Delimiter::Regex(regex_amp_or_semicolon()),
            Self::RegexAmpSemicolonOrPipe => Delimiter::Regex(regex_amp_semicolon_or_pipe()),
        }
    }
}

impl FuzzEncodeDelimiter {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ampersand => "&",
            Self::Semicolon => ";",
            Self::Pipe => "|",
            Self::DoubleAmpersand => "&&",
            Self::Empty => "",
        }
    }
}

impl FuzzValue {
    fn into_value_with_depth(self, depth: usize) -> Value {
        match self {
            Self::Null => Value::Null,
            Self::Bool { value } => Value::Bool(value),
            Self::I64 { value } => Value::I64(value),
            Self::U64 { value } => Value::U64(value),
            Self::F64 { value } => Value::F64(value),
            Self::String { value } => Value::String(truncate_string(value, MAX_STRING_LEN)),
            Self::Bytes { value } => Value::Bytes(value.into_iter().take(MAX_BYTES_LEN).collect()),
            Self::Array { items } => {
                if depth == 0 {
                    return Value::Array(Vec::new());
                }

                Value::Array(
                    items
                        .into_iter()
                        .take(MAX_ARRAY_LEN)
                        .map(|item| item.into_value_with_depth(depth - 1))
                        .collect(),
                )
            }
            Self::Object { entries } => {
                if depth == 0 {
                    return Value::Object(Object::new());
                }

                let mut object = Object::new();
                for entry in entries.into_iter().take(MAX_OBJECT_LEN) {
                    object.insert(
                        truncate_string(entry.key, MAX_KEY_LEN),
                        entry.value.into_value_with_depth(depth - 1),
                    );
                }
                Value::Object(object)
            }
        }
    }
}

fn regex_amp_or_semicolon() -> Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX
        .get_or_init(|| Regex::new("[&;]").expect("static regex"))
        .clone()
}

fn regex_amp_semicolon_or_pipe() -> Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX
        .get_or_init(|| Regex::new("[&;|]").expect("static regex"))
        .clone()
}

fn truncate_string(input: String, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input;
    }

    input.chars().take(max_chars).collect()
}

const fn default_true() -> bool {
    true
}

const fn default_decode_list_limit() -> usize {
    20
}

const fn default_decode_depth() -> usize {
    5
}

const fn default_parameter_limit() -> usize {
    1000
}
