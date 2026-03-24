use std::sync::{Arc, Mutex};

use indexmap::IndexMap;
use qs_rust::{
    Charset, DecodeDecoder, DecodeKind, DecodeOptions, EncodeFilter, EncodeOptions, FilterResult,
    FunctionFilter, ListFormat, Value, decode, encode,
};

#[test]
fn node_compatible_parameter_counting_still_counts_sentinels_and_empty_keys() {
    let decoded = decode(
        "=&utf8=%E2%9C%93&a[]=b&a[]=c",
        &DecodeOptions::new()
            .with_charset_sentinel(true)
            .with_list_limit(1),
    )
    .unwrap();

    assert_eq!(
        decoded.get("a"),
        Some(&Value::Object(
            [
                ("0".to_owned(), Value::String("b".to_owned())),
                ("1".to_owned(), Value::String("c".to_owned())),
            ]
            .into()
        ))
    );
    assert!(!decoded.contains_key(""));
}

#[test]
fn shared_port_decode_decoder_is_key_aware_and_composes_with_dot_and_numeric_entities() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let capture = Arc::clone(&seen);
    let options = DecodeOptions::new()
        .with_allow_dots(true)
        .with_decode_dot_in_keys(true)
        .with_charset(Charset::Iso88591)
        .with_interpret_numeric_entities(true)
        .with_decoder(Some(DecodeDecoder::new(move |input, charset, kind| {
            capture.lock().unwrap().push((kind, charset));
            match kind {
                DecodeKind::Key => input.to_owned(),
                DecodeKind::Value => input
                    .replace("%26", "&")
                    .replace("%23", "#")
                    .replace("%3B", ";"),
            }
        })));

    let decoded = decode("a%2Eb=%26%239786%3B", &options).unwrap();

    assert_eq!(
        decoded,
        [("a.b".to_owned(), Value::String("☺".to_owned()))].into()
    );

    let seen = seen.lock().unwrap();
    assert!(seen.contains(&(DecodeKind::Key, Charset::Iso88591)));
    assert!(seen.contains(&(DecodeKind::Value, Charset::Iso88591)));
}

#[test]
fn shared_port_encode_extensions_are_opt_in_and_do_not_change_default_node_mode() {
    let value = Value::Object(
        [(
            "a".to_owned(),
            Value::Array(vec![
                Value::Null,
                Value::String("x".to_owned()),
                Value::Null,
            ]),
        )]
        .into(),
    );

    let default_output = encode(
        &value,
        &EncodeOptions::new()
            .with_encode(false)
            .with_list_format(ListFormat::Comma),
    )
    .unwrap();
    let compacted = encode(
        &value,
        &EncodeOptions::new()
            .with_encode(false)
            .with_list_format(ListFormat::Comma)
            .with_comma_compact_nulls(true),
    )
    .unwrap();

    assert_eq!(default_output, "a=,x,");
    assert_eq!(compacted, "a=x");
}

#[test]
fn shared_port_function_filter_can_omit_branches_without_undefined_public_values() {
    let encoded = encode(
        &Value::Object(
            [
                ("keep".to_owned(), Value::String("x".to_owned())),
                ("drop".to_owned(), Value::String("y".to_owned())),
            ]
            .into(),
        ),
        &EncodeOptions::new()
            .with_encode(false)
            .with_filter(Some(EncodeFilter::Function(FunctionFilter::new(
                |prefix, _value| {
                    if prefix.ends_with("drop") {
                        FilterResult::Omit
                    } else {
                        FilterResult::Keep
                    }
                },
            )))),
    )
    .unwrap();

    assert_eq!(encoded, "keep=x");
}

#[test]
fn node_compatible_top_level_dots_remain_raw_when_depth_is_zero() {
    let decoded = decode(
        "a.b=c",
        &DecodeOptions::new().with_allow_dots(true).with_depth(0),
    )
    .unwrap();

    assert_eq!(
        decoded,
        IndexMap::from([("a.b".to_owned(), Value::String("c".to_owned()))])
    );
}
