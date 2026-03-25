use std::sync::{Arc, Mutex};

use super::{
    Charset, EncodeOptions, EncodeToken, EncodeTokenEncoder, Format, Ordering, Sorter, Value,
    WhitelistSelector, encode, encode_comma_array, encode_comma_array_controlled,
    encode_key_only_fragment, encoded_dot_escape, escape_dots_in_materialized_path,
    percent_encode_bytes, percent_encode_latin1,
};
use crate::options::ListFormat;

#[test]
fn comma_encode_values_only_keeps_literal_commas_between_encoded_elements() {
    let parts = encode_comma_array(
        &[
            Value::String("a b".to_owned()),
            Value::String("c".to_owned()),
        ],
        &super::KeyPathNode::from_raw("letters"),
        &EncodeOptions::new()
            .with_list_format(ListFormat::Comma)
            .with_encode_values_only(true),
    );
    assert_eq!(parts, vec!["letters=a%20b,c".to_owned()]);
}

#[test]
fn comma_bytes_use_the_configured_charset_when_stringifying() {
    let parts = encode_comma_array(
        &[Value::Bytes(vec![0xE4]), Value::Bytes(vec![0xF6])],
        &super::KeyPathNode::from_raw("letters"),
        &EncodeOptions::new()
            .with_list_format(ListFormat::Comma)
            .with_encode(false)
            .with_charset(Charset::Iso88591),
    );
    assert_eq!(parts, vec!["letters=\u{00E4},\u{00F6}".to_owned()]);
}

#[test]
fn comma_compact_nulls_drops_null_entries_and_omits_all_null_keys() {
    let some = encode(
        &Value::Object(
            [(
                "a".to_owned(),
                Value::Array(vec![
                    Value::String("x".to_owned()),
                    Value::Null,
                    Value::String("y".to_owned()),
                ]),
            )]
            .into(),
        ),
        &EncodeOptions::new()
            .with_list_format(ListFormat::Comma)
            .with_encode(false)
            .with_comma_compact_nulls(true),
    )
    .unwrap();
    assert_eq!(some, "a=x,y");

    let all_null = encode(
        &Value::Object([("a".to_owned(), Value::Array(vec![Value::Null, Value::Null]))].into()),
        &EncodeOptions::new()
            .with_list_format(ListFormat::Comma)
            .with_encode(false)
            .with_comma_compact_nulls(true),
    )
    .unwrap();
    assert_eq!(all_null, "");
}

#[test]
fn comma_arrays_emit_empty_list_suffixes_only_when_allowed() {
    let path = super::KeyPathNode::from_raw("letters");
    let base = EncodeOptions::new()
        .with_encode(false)
        .with_list_format(ListFormat::Comma);

    assert_eq!(encode_comma_array(&[], &path, &base), Vec::<String>::new());
    assert_eq!(
        encode_comma_array(&[], &path, &base.clone().with_allow_empty_lists(true)),
        vec!["letters[]".to_owned()]
    );
}

#[test]
fn comma_arrays_emit_key_only_fragments_for_skipped_nulls() {
    let path = super::KeyPathNode::from_raw("letters");
    let base = EncodeOptions::new()
        .with_encode(false)
        .with_list_format(ListFormat::Comma)
        .with_skip_nulls(true)
        .with_strict_null_handling(true);

    assert_eq!(
        encode_comma_array(&[Value::Null, Value::Null], &path, &base),
        vec!["letters".to_owned()]
    );
    assert_eq!(
        encode_comma_array(
            &[Value::Null],
            &path,
            &base.clone().with_comma_round_trip(true),
        ),
        vec!["letters[]".to_owned()]
    );
}

#[test]
fn comma_arrays_round_trip_single_non_null_values_with_suffixes() {
    let path = super::KeyPathNode::from_raw("letters");
    let parts = encode_comma_array(
        &[Value::String("solo".to_owned())],
        &path,
        &EncodeOptions::new()
            .with_encode(false)
            .with_list_format(ListFormat::Comma)
            .with_comma_round_trip(true),
    );

    assert_eq!(parts, vec!["letters[]=solo".to_owned()]);
}

#[test]
fn controlled_comma_arrays_cover_empty_inputs_and_out_of_range_whitelists() {
    let path = super::KeyPathNode::from_raw("letters");
    let base = EncodeOptions::new()
        .with_encode(false)
        .with_list_format(ListFormat::Comma);

    assert_eq!(
        encode_comma_array_controlled(&[], &path, &base),
        Vec::<String>::new()
    );
    assert_eq!(
        encode_comma_array_controlled(&[], &path, &base.clone().with_allow_empty_lists(true)),
        vec!["letters[]".to_owned()]
    );
    assert_eq!(
        encode_comma_array_controlled(
            &[Value::String("kept".to_owned())],
            &path,
            &base
                .clone()
                .with_whitelist(Some(vec![WhitelistSelector::Index(4)])),
        ),
        Vec::<String>::new()
    );
}

#[test]
fn controlled_comma_arrays_can_compact_or_strictify_nulls() {
    let path = super::KeyPathNode::from_raw("letters");
    let items = [Value::Null];
    let base = EncodeOptions::new()
        .with_encode(false)
        .with_list_format(ListFormat::Comma)
        .with_whitelist(Some(vec![WhitelistSelector::Index(0)]));

    assert_eq!(
        encode_comma_array_controlled(&items, &path, &base.clone().with_comma_compact_nulls(true),),
        Vec::<String>::new()
    );
    assert_eq!(
        encode_comma_array_controlled(
            &items,
            &path,
            &base
                .clone()
                .with_skip_nulls(true)
                .with_strict_null_handling(true)
                .with_comma_round_trip(true),
        ),
        vec!["letters[]".to_owned()]
    );
}

#[test]
fn controlled_comma_arrays_join_selected_values_after_filtering() {
    let path = super::KeyPathNode::from_raw("letters");
    let items = [
        Value::String("a b".to_owned()),
        Value::String("ignored".to_owned()),
    ];
    let options = EncodeOptions::new()
        .with_encode(false)
        .with_list_format(ListFormat::Comma)
        .with_comma_round_trip(true)
        .with_whitelist(Some(vec![
            WhitelistSelector::Index(4),
            WhitelistSelector::Index(0),
        ]));

    assert_eq!(
        encode_comma_array_controlled(&items, &path, &options),
        vec!["letters[]=a b".to_owned()]
    );
}

#[test]
fn controlled_comma_arrays_encode_selected_values_only_with_custom_tokens() {
    let path = super::KeyPathNode::from_raw("letters");
    let seen = Arc::new(Mutex::new(Vec::new()));
    let capture = Arc::clone(&seen);
    let parts = encode_comma_array_controlled(
        &[Value::String("a b".to_owned())],
        &path,
        &EncodeOptions::new()
            .with_encode(false)
            .with_list_format(ListFormat::Comma)
            .with_encode_values_only(true)
            .with_comma_round_trip(true)
            .with_encoder(Some(EncodeTokenEncoder::new(move |token, _, _| {
                capture.lock().unwrap().push(describe_token(token));
                match token {
                    EncodeToken::Value(Value::String(text)) => format!("<{text}>"),
                    EncodeToken::Key(_) | EncodeToken::TextValue(_) | EncodeToken::Value(_) => {
                        "unexpected".to_owned()
                    }
                }
            }))),
    );

    assert_eq!(parts, vec!["letters[]=<a b>".to_owned()]);
    assert_eq!(
        *seen.lock().unwrap(),
        vec!["value:String(\"a b\")".to_owned()]
    );
}

#[test]
fn custom_value_encoder_and_sorter_take_effect() {
    let value = Value::Object(
        [
            ("b".to_owned(), Value::I64(2)),
            ("a".to_owned(), Value::I64(1)),
        ]
        .into(),
    );

    let encoded = encode(
        &value,
        &EncodeOptions::new()
            .with_encode(false)
            .with_encoder(Some(EncodeTokenEncoder::new(|token, _, _| match token {
                EncodeToken::Key(key) => key.to_owned(),
                EncodeToken::Value(Value::I64(number)) => format!("n:{number}"),
                EncodeToken::Value(_) | EncodeToken::TextValue(_) => String::new(),
            })))
            .with_sorter(Some(Sorter::new(|left, right| {
                right.cmp(left).then(Ordering::Equal)
            }))),
    )
    .unwrap();

    assert_eq!(encoded, "b=n:2&a=n:1");
}

#[test]
fn strict_null_key_only_fragments_route_keys_through_the_encoder() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let capture = Arc::clone(&seen);
    let encoded = encode(
        &Value::Object([("key".to_owned(), Value::Null)].into()),
        &EncodeOptions::new()
            .with_encode(false)
            .with_strict_null_handling(true)
            .with_encoder(Some(EncodeTokenEncoder::new(move |token, _, _| {
                capture.lock().unwrap().push(describe_token(token));
                match token {
                    EncodeToken::Key(key) => format!("K({key})"),
                    EncodeToken::Value(_) | EncodeToken::TextValue(_) => "unused".to_owned(),
                }
            }))),
    )
    .unwrap();

    assert_eq!(encoded, "K(key)");
    assert_eq!(*seen.lock().unwrap(), vec!["key:key".to_owned()]);
}

#[test]
fn nested_strict_null_fragments_bypass_the_encoder_when_encode_values_only_is_true() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let capture = Arc::clone(&seen);
    let encoded = encode(
        &Value::Object(
            [(
                "root".to_owned(),
                Value::Object([("a".to_owned(), Value::Null)].into()),
            )]
            .into(),
        ),
        &EncodeOptions::new()
            .with_encode(false)
            .with_strict_null_handling(true)
            .with_encode_values_only(true)
            .with_encoder(Some(EncodeTokenEncoder::new(move |token, _, _| {
                capture.lock().unwrap().push(describe_token(token));
                "unused".to_owned()
            }))),
    )
    .unwrap();

    assert_eq!(encoded, "root[a]");
    assert!(seen.lock().unwrap().is_empty());
}

#[test]
fn nested_keys_and_values_route_through_the_encoder_when_encode_values_only_is_false() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let capture = Arc::clone(&seen);
    let encoded = encode(
        &Value::Object(
            [(
                "root".to_owned(),
                Value::Object([("a".to_owned(), Value::String("v".to_owned()))].into()),
            )]
            .into(),
        ),
        &EncodeOptions::new()
            .with_encode(false)
            .with_encode_values_only(false)
            .with_encoder(Some(EncodeTokenEncoder::new(
                move |token, _, _| match token {
                    EncodeToken::Key(key) => {
                        capture.lock().unwrap().push(format!("key:{key}"));
                        format!("K({key})")
                    }
                    EncodeToken::Value(Value::String(text)) => {
                        capture.lock().unwrap().push(format!("value:{text}"));
                        format!("V({text})")
                    }
                    EncodeToken::Value(other) => format!("V({other:?})"),
                    EncodeToken::TextValue(text) => format!("T({text})"),
                },
            ))),
    )
    .unwrap();

    assert_eq!(encoded, "K(root[a])=V(v)");
    assert_eq!(
        *seen.lock().unwrap(),
        vec!["key:root[a]".to_owned(), "value:v".to_owned()]
    );
}

#[test]
fn comma_lists_use_per_element_value_tokens_when_encode_values_only_is_true() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let capture = Arc::clone(&seen);
    let encoded = encode(
        &Value::Object(
            [(
                "tags".to_owned(),
                Value::Array(vec![
                    Value::String("a b".to_owned()),
                    Value::String("c".to_owned()),
                ]),
            )]
            .into(),
        ),
        &EncodeOptions::new()
            .with_list_format(ListFormat::Comma)
            .with_encode_values_only(true)
            .with_encoder(Some(EncodeTokenEncoder::new(move |token, _, _| {
                capture.lock().unwrap().push(describe_token(token));
                match token {
                    EncodeToken::Value(Value::String(text)) => format!("<{text}>"),
                    EncodeToken::Key(_) | EncodeToken::TextValue(_) | EncodeToken::Value(_) => {
                        "unexpected".to_owned()
                    }
                }
            }))),
    )
    .unwrap();

    assert_eq!(encoded, "tags=%3Ca%20b%3E,%3Cc%3E");
    assert_eq!(
        *seen.lock().unwrap(),
        vec![
            "value:String(\"a b\")".to_owned(),
            "value:String(\"c\")".to_owned()
        ]
    );
}

#[test]
fn comma_lists_use_a_joined_text_value_token_when_encode_values_only_is_false() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let capture = Arc::clone(&seen);
    let encoded = encode(
        &Value::Object(
            [(
                "tags".to_owned(),
                Value::Array(vec![
                    Value::String("a b".to_owned()),
                    Value::String("c".to_owned()),
                ]),
            )]
            .into(),
        ),
        &EncodeOptions::new()
            .with_encode(false)
            .with_list_format(ListFormat::Comma)
            .with_encoder(Some(EncodeTokenEncoder::new(move |token, _, _| {
                capture.lock().unwrap().push(describe_token(token));
                match token {
                    EncodeToken::Key(key) => key.to_owned(),
                    EncodeToken::TextValue(text) => format!("J({text})"),
                    EncodeToken::Value(_) => "unexpected".to_owned(),
                }
            }))),
    )
    .unwrap();

    assert_eq!(encoded, "tags=J(a b,c)");
    assert_eq!(
        *seen.lock().unwrap(),
        vec!["key:tags".to_owned(), "text:a b,c".to_owned()]
    );
}

#[test]
fn dot_encoded_key_tokens_are_exposed_before_final_key_formatting() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let capture = Arc::clone(&seen);
    let encoded = encode(
        &Value::Object(
            [(
                "name.obj".to_owned(),
                Value::Object(
                    [("first.given".to_owned(), Value::String("John".to_owned()))].into(),
                ),
            )]
            .into(),
        ),
        &EncodeOptions::new()
            .with_encode(false)
            .with_allow_dots(true)
            .with_encode_dot_in_keys(true)
            .with_encoder(Some(EncodeTokenEncoder::new(move |token, _, _| {
                capture.lock().unwrap().push(describe_token(token));
                match token {
                    EncodeToken::Key(key) => key.to_owned(),
                    EncodeToken::Value(Value::String(text)) => text.clone(),
                    EncodeToken::Value(_) | EncodeToken::TextValue(_) => String::new(),
                }
            }))),
    )
    .unwrap();

    assert_eq!(encoded, "name%2Eobj.first%2Egiven=John");
    assert_eq!(
        *seen.lock().unwrap(),
        vec![
            "key:name%2Eobj.first%2Egiven".to_owned(),
            "value:String(\"John\")".to_owned()
        ]
    );
}

#[test]
fn empty_output_ignores_query_prefix_and_charset_sentinel() {
    assert_eq!(
        encode(
            &Value::Object(Default::default()),
            &EncodeOptions::new()
                .with_add_query_prefix(true)
                .with_charset_sentinel(true),
        )
        .unwrap(),
        ""
    );
}

#[test]
fn all_empty_fragments_are_treated_as_blank_output() {
    let encoded = encode(
        &Value::Object([("".to_owned(), Value::Null)].into()),
        &EncodeOptions::new()
            .with_strict_null_handling(true)
            .with_charset_sentinel(true),
    )
    .unwrap();

    assert_eq!(encoded, "");
}

#[test]
fn dot_escape_helper_matches_encode_flag() {
    assert_eq!(encoded_dot_escape(&EncodeOptions::new()), "%2E");
    assert_eq!(
        encoded_dot_escape(&EncodeOptions::new().with_encode(false)),
        "%2E"
    );
    assert_eq!(
        encoded_dot_escape(&EncodeOptions::new().with_encode_values_only(true)),
        "%2E"
    );
    assert_eq!(
        escape_dots_in_materialized_path("a.b", &EncodeOptions::new()),
        "a%2Eb"
    );
}

#[test]
fn strict_null_key_only_fragments_keep_percent_twenty_in_rfc1738_mode() {
    assert_eq!(
        encode_key_only_fragment("a b", &EncodeOptions::new().with_format(Format::Rfc1738)),
        "a%20b"
    );
    assert_eq!(
        encode_key_only_fragment("a b", &EncodeOptions::new().with_format(Format::Rfc3986)),
        "a%20b"
    );
    assert_eq!(
        encode_key_only_fragment("a+b", &EncodeOptions::new().with_encode(false)),
        "a+b"
    );
}

#[test]
fn percent_encoding_helpers_match_swift_utils_examples() {
    assert_eq!(percent_encode_latin1("☺", Format::Rfc3986), "%26%239786%3B");
    assert_eq!(percent_encode_latin1("()", Format::Rfc1738), "()");
    assert_eq!(
        percent_encode_bytes("💩".as_bytes(), Format::Rfc3986),
        "%F0%9F%92%A9"
    );
    assert_eq!(
        percent_encode_bytes("foo(bar)".as_bytes(), Format::Rfc1738),
        "foo(bar)"
    );
}

fn describe_token(token: EncodeToken<'_>) -> String {
    match token {
        EncodeToken::Key(key) => format!("key:{key}"),
        EncodeToken::Value(value) => format!("value:{value:?}"),
        EncodeToken::TextValue(text) => format!("text:{text}"),
    }
}
