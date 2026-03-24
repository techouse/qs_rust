#![allow(dead_code)]

pub(crate) mod cases;

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use qs_rust::{
    Charset, DecodeError, DecodeOptions, Delimiter, Duplicates, EncodeError, EncodeOptions, Format,
    ListFormat, SortMode, Value, WhitelistSelector, decode, encode,
};
use serde_json::{Map, Value as JsonValue, json};

use self::cases::{CaseMeta, DecodeParityCase, EncodeParityCase};

const NODE_CASE_SCRIPT: &str = "tests/comparison/js/case.js";
const NODE_QS_SCRIPT: &str = "tests/comparison/js/qs.js";
const NODE_MODULES_QS: &str = "tests/comparison/js/node_modules/qs";
const SMOKE_FIXTURES: &str = "tests/comparison/test_cases.json";

#[derive(serde::Deserialize)]
pub struct FixtureCase {
    pub data: JsonValue,
    pub encoded: String,
}

#[derive(serde::Deserialize)]
pub struct NodeCorpusCase {
    pub encoded: String,
    pub decoded: JsonValue,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum NodeCaseResponse {
    Ok { value: JsonValue },
    Error { kind: String, message: String },
}

pub fn node_comparison_available() -> bool {
    Path::new(NODE_MODULES_QS).exists()
}

pub fn require_node_comparison(name: &str) -> bool {
    if node_comparison_available() {
        return true;
    }

    eprintln!("skipping {name}; run npm ci in tests/comparison/js first");
    false
}

pub fn load_smoke_fixtures() -> Vec<FixtureCase> {
    let fixture_text = fs::read_to_string(SMOKE_FIXTURES).expect("fixture file missing");
    serde_json::from_str(&fixture_text).expect("fixture JSON should parse")
}

pub fn load_node_smoke_cases() -> Vec<NodeCorpusCase> {
    let node_output = Command::new("node")
        .arg(NODE_QS_SCRIPT)
        .output()
        .expect("node should run");
    assert!(
        node_output.status.success(),
        "node comparison script failed: {}",
        String::from_utf8_lossy(&node_output.stderr)
    );

    serde_json::from_slice(&node_output.stdout).expect("node output JSON should parse")
}

pub fn fixture_json_to_value(value: &JsonValue) -> Value {
    match value {
        JsonValue::Null => Value::Null,
        JsonValue::Bool(boolean) => Value::Bool(*boolean),
        JsonValue::Number(number) => Value::String(number.to_string()),
        JsonValue::String(text) => Value::String(text.clone()),
        JsonValue::Array(items) => Value::Array(items.iter().map(fixture_json_to_value).collect()),
        JsonValue::Object(entries) => Value::Object(
            entries
                .iter()
                .map(|(key, value)| (key.clone(), fixture_json_to_value(value)))
                .collect(),
        ),
    }
}

pub fn value_to_json(value: &Value) -> JsonValue {
    match value {
        Value::Null => JsonValue::Null,
        Value::Bool(boolean) => JsonValue::Bool(*boolean),
        Value::I64(number) => JsonValue::Number((*number).into()),
        Value::U64(number) => JsonValue::Number((*number).into()),
        Value::F64(number) => serde_json::Number::from_f64(*number)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        Value::String(text) => JsonValue::String(text.clone()),
        Value::Temporal(value) => JsonValue::String(value.to_string()),
        Value::Bytes(bytes) => JsonValue::Array(
            bytes
                .iter()
                .map(|byte| JsonValue::Number(u64::from(*byte).into()))
                .collect(),
        ),
        Value::Array(values) => JsonValue::Array(values.iter().map(value_to_json).collect()),
        Value::Object(entries) => JsonValue::Object(
            entries
                .iter()
                .map(|(key, value)| (key.clone(), value_to_json(value)))
                .collect(),
        ),
    }
}

pub fn json_to_value(value: &JsonValue) -> Value {
    match value {
        JsonValue::Null => Value::Null,
        JsonValue::Bool(boolean) => Value::Bool(*boolean),
        JsonValue::Number(number) => {
            if let Some(value) = number.as_i64() {
                Value::I64(value)
            } else if let Some(value) = number.as_u64() {
                Value::U64(value)
            } else if let Some(value) = number.as_f64() {
                Value::F64(value)
            } else {
                Value::Null
            }
        }
        JsonValue::String(text) => Value::String(text.clone()),
        JsonValue::Array(items) => Value::Array(items.iter().map(json_to_value).collect()),
        JsonValue::Object(entries) => Value::Object(
            entries
                .iter()
                .map(|(key, value)| (key.clone(), json_to_value(value)))
                .collect(),
        ),
    }
}

pub fn canonical_json(value: &Value) -> JsonValue {
    match value {
        Value::Null => JsonValue::Null,
        Value::Bool(boolean) => JsonValue::Bool(*boolean),
        Value::I64(number) => JsonValue::String(number.to_string()),
        Value::U64(number) => JsonValue::String(number.to_string()),
        Value::F64(number) => JsonValue::String(number.to_string()),
        Value::String(text) => JsonValue::String(text.clone()),
        Value::Temporal(value) => JsonValue::String(value.to_string()),
        Value::Bytes(bytes) => JsonValue::String(String::from_utf8_lossy(bytes).into_owned()),
        Value::Array(items) => JsonValue::Array(items.iter().map(canonical_json).collect()),
        Value::Object(entries) => {
            let mut ordered = entries.iter().collect::<Vec<_>>();
            ordered.sort_by(|left, right| left.0.cmp(right.0));
            let mut map = Map::new();
            for (key, value) in ordered {
                map.insert(key.clone(), canonical_json(value));
            }
            JsonValue::Object(map)
        }
    }
}

pub fn assert_node_decode_matches(name: &str, query: &str, options: &DecodeOptions) {
    let node = run_node_case(json!({
        "mode": "decode",
        "input": query,
        "options": decode_options_to_node_json(options),
    }));
    let rust = decode(query, options);

    match (rust, node) {
        (Ok(value), NodeCaseResponse::Ok { value: node_value }) => {
            let rust_json = canonical_json(&Value::Object(value));
            let node_json = canonical_json(&json_to_value(&node_value));
            assert_eq!(rust_json, node_json, "{name}: decode mismatch");
        }
        (Err(error), NodeCaseResponse::Error { kind, message }) => {
            assert_eq!(
                decode_error_kind(&error),
                kind,
                "{name}: decode error mismatch (node message: {message})"
            );
        }
        (Ok(value), NodeCaseResponse::Error { kind, message }) => {
            panic!(
                "{name}: Rust decode succeeded with {value:?}, but Node failed with {kind}: {message}"
            );
        }
        (Err(error), NodeCaseResponse::Ok { value }) => {
            panic!("{name}: Rust decode failed with {error:?}, but Node succeeded with {value}");
        }
    }
}

pub fn assert_node_encode_matches(name: &str, value: &Value, options: &EncodeOptions) {
    let node = run_node_case(json!({
        "mode": "encode",
        "input": value_to_json(value),
        "options": encode_options_to_node_json(options),
    }));
    let rust = encode(value, options);

    match (rust, node) {
        (
            Ok(encoded),
            NodeCaseResponse::Ok {
                value: JsonValue::String(expected),
            },
        ) => {
            let encoded = normalize_bracket_encoding(&encoded);
            let expected = normalize_bracket_encoding(&expected);
            assert_eq!(encoded, expected, "{name}: encode mismatch");
        }
        (Err(error), NodeCaseResponse::Error { kind, message }) => {
            assert_eq!(
                encode_error_kind(&error),
                kind,
                "{name}: encode error mismatch (node message: {message})"
            );
        }
        (Ok(encoded), NodeCaseResponse::Ok { value }) => {
            panic!("{name}: Node encode returned non-string value {value}, Rust encoded {encoded}");
        }
        (Ok(encoded), NodeCaseResponse::Error { kind, message }) => {
            panic!(
                "{name}: Rust encode succeeded with {encoded}, but Node failed with {kind}: {message}"
            );
        }
        (Err(error), NodeCaseResponse::Ok { value }) => {
            panic!("{name}: Rust encode failed with {error:?}, but Node succeeded with {value}");
        }
    }
}

pub fn imported_case_name(meta: &CaseMeta) -> String {
    format!(
        "{}:{} [{}] {} ({})",
        meta.source_repo,
        meta.source_file,
        meta.family,
        meta.title,
        if meta.node_backed {
            "parity"
        } else {
            "rust-only"
        }
    )
}

pub fn assert_imported_node_decode_case(case: &DecodeParityCase) {
    assert_node_decode_matches(&imported_case_name(&case.meta), case.query, &case.options);
}

pub fn assert_imported_node_encode_case(case: &EncodeParityCase) {
    assert_node_encode_matches(&imported_case_name(&case.meta), &case.value, &case.options);
}

fn run_node_case(payload: JsonValue) -> NodeCaseResponse {
    let mut child = Command::new("node")
        .arg(NODE_CASE_SCRIPT)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("node case helper should start");

    {
        let stdin = child
            .stdin
            .as_mut()
            .expect("node case helper stdin missing");
        let payload = serde_json::to_vec(&payload).expect("case payload should serialize");
        stdin
            .write_all(&payload)
            .expect("node case helper stdin write should succeed");
    }

    let output = child
        .wait_with_output()
        .expect("node case helper should finish");
    assert!(
        output.status.success(),
        "node case helper failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("node case helper output should parse")
}

fn decode_options_to_node_json(options: &DecodeOptions) -> JsonValue {
    let delimiter = match options.delimiter() {
        Delimiter::String(text) => json!({ "kind": "string", "value": text }),
        Delimiter::Regex(regex) => json!({ "kind": "regex", "value": regex.as_str() }),
    };

    json!({
        "allowDots": options.allow_dots(),
        "decodeDotInKeys": options.decode_dot_in_keys(),
        "allowEmptyArrays": options.allow_empty_lists(),
        "allowSparse": options.allow_sparse_lists(),
        "arrayLimit": options.list_limit(),
        "charset": charset_name(options.charset()),
        "charsetSentinel": options.charset_sentinel(),
        "comma": options.comma(),
        "delimiter": delimiter,
        "depth": options.depth(),
        "duplicates": duplicates_name(options.duplicates()),
        "ignoreQueryPrefix": options.ignore_query_prefix(),
        "interpretNumericEntities": options.interpret_numeric_entities(),
        "parameterLimit": options.parameter_limit(),
        "parseArrays": options.parse_lists(),
        "strictDepth": options.strict_depth(),
        "strictNullHandling": options.strict_null_handling(),
        "throwOnLimitExceeded": options.throw_on_limit_exceeded(),
    })
}

fn encode_options_to_node_json(options: &EncodeOptions) -> JsonValue {
    let filter = options.whitelist().map(|selectors| {
        selectors
            .iter()
            .map(|selector| match selector {
                WhitelistSelector::Key(key) => JsonValue::String(key.clone()),
                WhitelistSelector::Index(index) => {
                    JsonValue::Number(u64::try_from(*index).unwrap().into())
                }
            })
            .collect::<Vec<_>>()
    });

    json!({
        "addQueryPrefix": options.add_query_prefix(),
        "allowDots": options.allow_dots(),
        "allowEmptyArrays": options.allow_empty_lists(),
        "arrayFormat": list_format_name(options.list_format()),
        "charset": charset_name(options.charset()),
        "charsetSentinel": options.charset_sentinel(),
        "commaRoundTrip": options.comma_round_trip(),
        "delimiter": options.delimiter(),
        "encode": options.encode(),
        "encodeDotInKeys": options.encode_dot_in_keys(),
        "encodeValuesOnly": options.encode_values_only(),
        "filter": filter,
        "format": format_name(options.format()),
        "skipNulls": options.skip_nulls(),
        "sort": sort_mode_name(options.sort()),
        "strictNullHandling": options.strict_null_handling(),
    })
}

fn charset_name(charset: Charset) -> &'static str {
    match charset {
        Charset::Utf8 => "utf-8",
        Charset::Iso88591 => "iso-8859-1",
    }
}

fn duplicates_name(duplicates: Duplicates) -> &'static str {
    match duplicates {
        Duplicates::Combine => "combine",
        Duplicates::First => "first",
        Duplicates::Last => "last",
    }
}

fn list_format_name(list_format: ListFormat) -> &'static str {
    match list_format {
        ListFormat::Indices => "indices",
        ListFormat::Brackets => "brackets",
        ListFormat::Repeat => "repeat",
        ListFormat::Comma => "comma",
    }
}

fn format_name(format: Format) -> &'static str {
    match format {
        Format::Rfc3986 => "RFC3986",
        Format::Rfc1738 => "RFC1738",
    }
}

fn sort_mode_name(sort: SortMode) -> Option<&'static str> {
    match sort {
        SortMode::Preserve => None,
        SortMode::LexicographicAsc => Some("lexicographicAsc"),
    }
}

fn decode_error_kind(error: &DecodeError) -> &'static str {
    if error.is_parameter_limit_exceeded() {
        "parameter_limit_exceeded"
    } else if error.is_list_limit_exceeded() {
        "list_limit_exceeded"
    } else if error.is_depth_exceeded() {
        "depth_exceeded"
    } else {
        "unknown"
    }
}

fn encode_error_kind(error: &EncodeError) -> &'static str {
    if error.is_empty_delimiter() {
        "empty_delimiter"
    } else if error.is_encode_dot_in_keys_requires_allow_dots() {
        "encode_dot_requires_allow_dots"
    } else {
        "unknown"
    }
}

fn normalize_bracket_encoding(value: &str) -> String {
    value
        .replace("%5B", "[")
        .replace("%5D", "]")
        .replace("%5b", "[")
        .replace("%5d", "]")
}
