use std::sync::{Arc, Mutex};

use super::{
    DecodeDecoder, DecodeOptions, Delimiter, FlatValues, IndexMap, Node, ParsedFlatValue, Regex,
    Value, collect_pair_values, decode, decode_from_pairs_map, finalize_flat,
    parse_query_string_values, scan_structured_keys, stores_concrete_value, stores_parsed_value,
};
use crate::options::DecodeKind;

#[test]
fn flat_fast_path_matches_full_merge_path_for_flat_inputs() {
    let options = DecodeOptions::new();
    let parsed = parse_query_string_values("a=1&b=2&b=3", &options).unwrap();
    let scan = scan_structured_keys(parsed.values.key_refs(), &options).unwrap();

    let flat = finalize_flat(parsed.values.clone(), &options).unwrap();
    let full = decode_from_pairs_map(parsed.values, &options, &scan).unwrap();

    assert_eq!(flat, full);
}

#[test]
fn allow_empty_lists_cover_bare_and_assigned_empty_leaf_paths() {
    let strict_null = decode(
        "a[]",
        &DecodeOptions::new()
            .with_allow_empty_lists(true)
            .with_strict_null_handling(true),
    )
    .unwrap();
    assert_eq!(strict_null.get("a"), Some(&Value::Array(vec![])));

    let empty_assignment =
        decode("a[]=", &DecodeOptions::new().with_allow_empty_lists(true)).unwrap();
    assert_eq!(empty_assignment.get("a"), Some(&Value::Array(vec![])));
}

#[test]
fn comma_parsing_preserves_empty_boundary_tokens() {
    let decoded = decode("a=,", &DecodeOptions::new().with_comma(true)).unwrap();
    assert_eq!(
        decoded.get("a"),
        Some(&Value::Array(vec![
            Value::String(String::new()),
            Value::String(String::new()),
        ]))
    );
}

#[test]
fn finalize_flat_preserves_concrete_nested_containers() {
    let mut values = IndexMap::new();
    values.insert(
        "list".to_owned(),
        ParsedFlatValue::parsed(
            Node::Array(vec![
                Node::Object(
                    [(
                        "inner".to_owned(),
                        Node::scalar(Value::String("value".to_owned())),
                    )]
                    .into(),
                ),
                Node::scalar(Value::String("tail".to_owned())),
            ]),
            false,
        ),
    );
    values.insert(
        "dict".to_owned(),
        ParsedFlatValue::parsed(
            Node::Object(
                [(
                    "nested".to_owned(),
                    Node::Object(
                        [(
                            "leaf".to_owned(),
                            Node::scalar(Value::String("1".to_owned())),
                        )]
                        .into(),
                    ),
                )]
                .into(),
            ),
            false,
        ),
    );

    let finalized = finalize_flat(FlatValues::Parsed(values), &DecodeOptions::new()).unwrap();
    assert_eq!(
        finalized,
        [
            (
                "list".to_owned(),
                Value::Array(vec![
                    Value::Object([("inner".to_owned(), Value::String("value".to_owned()))].into()),
                    Value::String("tail".to_owned()),
                ]),
            ),
            (
                "dict".to_owned(),
                Value::Object(
                    [(
                        "nested".to_owned(),
                        Value::Object([("leaf".to_owned(), Value::String("1".to_owned()))].into()),
                    )]
                    .into()
                ),
            ),
        ]
        .into()
    );
}

#[test]
fn flat_parse_marks_concrete_scalars_and_comma_arrays_as_clean() {
    let scalar = parse_query_string_values("a=1", &DecodeOptions::new()).unwrap();
    assert!(stores_concrete_value(&scalar.values, "a"));

    let comma =
        parse_query_string_values("a=1,2,3", &DecodeOptions::new().with_comma(true)).unwrap();
    assert!(stores_concrete_value(&comma.values, "a"));

    let duplicates = parse_query_string_values("a=1&a=2", &DecodeOptions::new()).unwrap();
    assert!(stores_concrete_value(&duplicates.values, "a"));
}

#[test]
fn finalize_flat_compacts_only_marked_values() {
    let mut values = IndexMap::new();
    values.insert(
        "clean".to_owned(),
        ParsedFlatValue::concrete(Value::String("x".to_owned())),
    );
    values.insert(
        "compacted".to_owned(),
        ParsedFlatValue::parsed(
            Node::Array(vec![
                Node::Undefined,
                Node::scalar(Value::String("y".to_owned())),
            ]),
            true,
        ),
    );

    let finalized = finalize_flat(FlatValues::Parsed(values), &DecodeOptions::new()).unwrap();
    assert_eq!(
        finalized,
        [
            ("clean".to_owned(), Value::String("x".to_owned())),
            (
                "compacted".to_owned(),
                Value::Array(vec![Value::String("y".to_owned())]),
            ),
        ]
        .into()
    );
}

#[test]
fn structured_fallback_converts_concrete_flat_entries_when_needed() {
    let options = DecodeOptions::new();
    let parsed = parse_query_string_values("a=1&b[c]=2", &options).unwrap();
    assert!(stores_concrete_value(&parsed.values, "a"));

    let scan = scan_structured_keys(parsed.values.key_refs(), &options).unwrap();
    let full = decode_from_pairs_map(parsed.values, &options, &scan).unwrap();

    assert_eq!(
        full,
        [
            ("a".to_owned(), Value::String("1".to_owned())),
            (
                "b".to_owned(),
                Value::Object([("c".to_owned(), Value::String("2".to_owned()))].into()),
            ),
        ]
        .into()
    );
}

#[test]
fn regex_custom_and_decode_pairs_keep_the_parsed_path() {
    let regex_options =
        DecodeOptions::new().with_delimiter(Delimiter::Regex(Regex::new("[&;]").unwrap()));
    let regex_parsed = parse_query_string_values("a=1;b=2", &regex_options).unwrap();
    assert!(stores_parsed_value(&regex_parsed.values, "a"));

    let custom_options =
        DecodeOptions::new().with_decoder(Some(DecodeDecoder::new(|input, _, _| input.to_owned())));
    let custom_parsed = parse_query_string_values("a=1", &custom_options).unwrap();
    assert!(stores_parsed_value(&custom_parsed.values, "a"));

    let pair_parsed = collect_pair_values(
        [("a".to_owned(), Value::String("1".to_owned()))],
        &DecodeOptions::new(),
    )
    .unwrap();
    assert!(stores_parsed_value(&pair_parsed.values, "a"));
}

#[test]
fn public_decode_applies_custom_decoder_to_plain_unescaped_values() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let capture = Arc::clone(&seen);
    let decoded = decode(
        "name=alpha-beta.gamma_123",
        &DecodeOptions::new().with_decoder(Some(DecodeDecoder::new(move |input, _, kind| {
            if matches!(kind, DecodeKind::Value) {
                capture.lock().unwrap().push(input.to_owned());
                format!("seen:{input}")
            } else {
                input.to_owned()
            }
        }))),
    )
    .unwrap();

    assert_eq!(
        decoded.get("name"),
        Some(&Value::String("seen:alpha-beta.gamma_123".to_owned()))
    );
    assert_eq!(
        *seen.lock().unwrap(),
        vec!["alpha-beta.gamma_123".to_owned()]
    );
}

#[test]
fn public_decode_applies_custom_decoder_to_each_comma_split_value() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let capture = Arc::clone(&seen);
    let decoded = decode(
        "tags=a,b",
        &DecodeOptions::new()
            .with_comma(true)
            .with_decoder(Some(DecodeDecoder::new(move |input, _, kind| {
                if matches!(kind, DecodeKind::Value) {
                    capture.lock().unwrap().push(input.to_owned());
                    input.to_ascii_uppercase()
                } else {
                    input.to_owned()
                }
            }))),
    )
    .unwrap();

    assert_eq!(
        decoded.get("tags"),
        Some(&Value::Array(vec![
            Value::String("A".to_owned()),
            Value::String("B".to_owned()),
        ]))
    );
    assert_eq!(*seen.lock().unwrap(), vec!["a".to_owned(), "b".to_owned()]);
}
