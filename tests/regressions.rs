mod support;

use std::sync::Arc;
use std::thread;

use qs_rust::{
    Charset, DecodeError, DecodeOptions, Delimiter, Duplicates, EncodeError, EncodeFilter,
    EncodeOptions, FilterResult, FunctionFilter, ListFormat, Value, WhitelistSelector, decode,
    decode_pairs, encode,
};

#[cfg(feature = "serde")]
use qs_rust::{from_str, to_string};

use crate::support::{
    cases::csharp_internal::{decode_option_invariants, encode_option_invariants},
    imported_case_name,
};

fn s(value: &str) -> Value {
    Value::String(value.to_owned())
}

fn obj(entries: Vec<(&str, Value)>) -> Value {
    Value::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect(),
    )
}

#[test]
fn decode_pairs_uses_structured_merge_not_raw_string_rules() {
    let decoded = decode_pairs(
        vec![
            ("a[b]".to_owned(), s("1")),
            ("a[b]".to_owned(), s("2")),
            ("a".to_owned(), Value::Null),
        ],
        &DecodeOptions::new(),
    )
    .unwrap();

    assert_eq!(
        decoded.get("a"),
        Some(&obj(vec![("b", Value::Array(vec![s("1"), s("2")]),)])),
    );
}

#[test]
fn csharp_decode_pairs_skip_empty_keys_and_preserve_object_leaves() {
    let decoded = decode_pairs(
        vec![
            (String::new(), s("ignored")),
            ("root".to_owned(), obj(vec![("inner", s("value"))])),
        ],
        &DecodeOptions::new(),
    )
    .unwrap();

    assert_eq!(
        decoded,
        [("root".to_owned(), obj(vec![("inner", s("value"))]),)].into()
    );
}

#[test]
fn csharp_decode_pairs_respect_duplicate_strategies_and_normalized_collisions() {
    let first = decode_pairs(
        vec![("a".to_owned(), s("1")), ("a".to_owned(), s("2"))],
        &DecodeOptions::new().with_duplicates(Duplicates::First),
    )
    .unwrap();
    assert_eq!(first.get("a"), Some(&s("1")));

    let last = decode_pairs(
        vec![("a".to_owned(), s("1")), ("a".to_owned(), s("2"))],
        &DecodeOptions::new().with_duplicates(Duplicates::Last),
    )
    .unwrap();
    assert_eq!(last.get("a"), Some(&s("2")));

    let combined = decode_pairs(
        vec![("a.b".to_owned(), s("3")), ("a[b]".to_owned(), s("4"))],
        &DecodeOptions::new()
            .with_allow_dots(true)
            .with_duplicates(Duplicates::Combine),
    )
    .unwrap();
    assert_eq!(
        combined.get("a"),
        Some(&obj(vec![("b", Value::Array(vec![s("3"), s("4")]))]))
    );
}

#[test]
fn csharp_decode_pairs_handle_percent_encoded_dot_segments_and_limit_errors() {
    let decoded = decode_pairs(
        vec![("a[%2E]".to_owned(), s("x"))],
        &DecodeOptions::new()
            .with_allow_dots(true)
            .with_decode_dot_in_keys(true),
    )
    .unwrap();
    assert_eq!(decoded.get("a"), Some(&obj(vec![(".", s("x"))])));

    let error = decode_pairs(
        vec![
            ("a".to_owned(), s("1")),
            ("a".to_owned(), s("2")),
            ("a".to_owned(), s("3")),
            ("a".to_owned(), s("4")),
        ],
        &DecodeOptions::new()
            .with_duplicates(Duplicates::Combine)
            .with_list_limit(3)
            .with_throw_on_limit_exceeded(true),
    )
    .unwrap_err();
    assert!(matches!(error, DecodeError::ListLimitExceeded { limit: 3 }));
}

#[test]
fn csharp_decode_validation_rejects_zero_parameter_limit() {
    let error = decode("a=1", &DecodeOptions::new().with_parameter_limit(0)).unwrap_err();
    assert!(matches!(error, DecodeError::InvalidParameterLimit));
}

#[test]
fn kotlin_model_defaults_match_the_public_baseline() {
    let decode_defaults = DecodeOptions::new();
    assert!(!decode_defaults.allow_dots());
    assert!(!decode_defaults.decode_dot_in_keys());
    assert_eq!(decode_defaults.list_limit(), 20);
    assert_eq!(decode_defaults.charset(), Charset::Utf8);
    assert_eq!(decode_defaults.parameter_limit(), 1000);
    assert_eq!(decode_defaults.duplicates(), Duplicates::Combine);

    let encode_defaults = EncodeOptions::new();
    assert!(encode_defaults.encode());
    assert_eq!(encode_defaults.delimiter(), "&");
    assert_eq!(encode_defaults.list_format(), ListFormat::Indices);
    assert_eq!(encode_defaults.charset(), Charset::Utf8);
    assert!(!encode_defaults.allow_dots());
    assert!(!encode_defaults.encode_dot_in_keys());
}

#[test]
fn kotlin_literal_delimiter_guardrails_reject_empty_strings() {
    let decode_error = decode(
        "a=1",
        &DecodeOptions::new().with_delimiter(Delimiter::String(String::new())),
    )
    .unwrap_err();
    assert!(matches!(decode_error, DecodeError::EmptyDelimiter));

    let encode_error = encode(
        &obj(vec![("a", s("1"))]),
        &EncodeOptions::new().with_delimiter(""),
    )
    .unwrap_err();
    assert!(encode_error.is_empty_delimiter());
}

#[test]
fn deep_decode_and_encode_do_not_recurse() {
    let depth = 3000usize;
    let mut query = String::from("leaf=value");
    for _ in 0..depth {
        query = format!("a[{query}]");
    }

    let long_query = query.replace("=value", "=x");
    let _ = decode(&long_query, &DecodeOptions::new().with_depth(depth + 1)).unwrap();

    let mut value = Value::String("x".to_owned());
    for _ in 0..depth {
        value = obj(vec![("a", value)]);
    }

    let root = obj(vec![("root", value)]);
    let encoded = encode(&root, &EncodeOptions::new()).unwrap();
    assert!(encoded.starts_with("root%5Ba%5D"));
}

#[test]
fn bytes_are_encoded_as_percent_escaped_raw_bytes() {
    let value = obj(vec![("data", Value::Bytes(vec![0x41, 0x20, 0xFF]))]);
    let encoded = encode(
        &value,
        &EncodeOptions::new().with_charset(Charset::Iso88591),
    )
    .unwrap();

    assert_eq!(encoded, "data=A%20%FF");
}

#[test]
fn csharp_byte_array_regressions_cover_default_and_encode_false_paths() {
    let default_encoded = encode(
        &obj(vec![("b", Value::Bytes(b"hi".to_vec()))]),
        &EncodeOptions::new(),
    )
    .unwrap();
    assert_eq!(default_encoded, "b=hi");

    let latin1_scalar = encode(
        &obj(vec![("b", Value::Bytes(vec![0xE4]))]),
        &EncodeOptions::new()
            .with_encode(false)
            .with_charset(Charset::Iso88591),
    )
    .unwrap();
    assert_eq!(latin1_scalar, "b=\u{00E4}");

    let latin1_comma = encode(
        &obj(vec![(
            "b",
            Value::Array(vec![Value::Bytes(vec![0xE4]), Value::Bytes(vec![0xF6])]),
        )]),
        &EncodeOptions::new()
            .with_encode(false)
            .with_charset(Charset::Iso88591)
            .with_list_format(ListFormat::Comma),
    )
    .unwrap();
    assert_eq!(latin1_comma, "b=\u{00E4},\u{00F6}");
}

#[test]
fn csharp_swift_and_dart_undefined_examples_map_to_filter_result_omit() {
    let encoded = encode(
        &obj(vec![
            ("a", Value::Null),
            ("b", s("drop-me")),
            ("c", s("keep-me")),
        ]),
        &EncodeOptions::new()
            .with_encode(false)
            .with_filter(Some(EncodeFilter::Function(FunctionFilter::new(
                |prefix, _| {
                    if prefix.ends_with("b") {
                        FilterResult::Omit
                    } else {
                        FilterResult::Keep
                    }
                },
            )))),
    )
    .unwrap();

    assert_eq!(encoded, "a=&c=keep-me");
}

#[test]
fn iterable_whitelist_out_of_range_indices_are_omitted() {
    let encoded = encode(
        &obj(vec![("a", Value::Array(vec![s("x"), s("y"), s("z")]))]),
        &EncodeOptions::new()
            .with_encode(false)
            .with_whitelist(Some(vec![
                WhitelistSelector::Key("a".to_owned()),
                WhitelistSelector::Index(1),
                WhitelistSelector::Index(5),
            ])),
    )
    .unwrap();

    assert_eq!(encoded, "a[1]=y");
}

#[test]
fn csharp_option_invariants_hold_for_decode_and_encode_builders() {
    for case in decode_option_invariants() {
        assert_eq!(
            case.options.allow_dots(),
            case.expect_allow_dots,
            "{}: unexpected allow_dots",
            imported_case_name(&case.meta)
        );
        assert_eq!(
            case.options.decode_dot_in_keys(),
            case.expect_decode_dot_in_keys,
            "{}: unexpected decode_dot_in_keys",
            imported_case_name(&case.meta)
        );
    }

    for case in encode_option_invariants() {
        assert_eq!(
            case.options.allow_dots(),
            case.expect_allow_dots,
            "{}: unexpected allow_dots",
            imported_case_name(&case.meta)
        );
        assert_eq!(
            case.options.encode_dot_in_keys(),
            case.expect_encode_dot_in_keys,
            "{}: unexpected encode_dot_in_keys",
            imported_case_name(&case.meta)
        );
    }
}

#[test]
fn swift_model_coverage_decode_error_helpers_expose_stable_metadata() {
    let parameter = DecodeError::ParameterLimitExceeded { limit: 2 };
    assert!(parameter.is_parameter_limit_exceeded());
    assert_eq!(parameter.parameter_limit(), Some(2));
    assert!(!parameter.is_list_limit_exceeded());
    assert!(!parameter.is_depth_exceeded());
    assert!(parameter.to_string().contains("2"));

    let list = DecodeError::ListLimitExceeded { limit: 4 };
    assert!(list.is_list_limit_exceeded());
    assert_eq!(list.list_limit(), Some(4));
    assert_eq!(list.parameter_limit(), None);

    let depth = DecodeError::DepthExceeded { depth: 7 };
    assert!(depth.is_depth_exceeded());
    assert_eq!(depth.depth_limit(), Some(7));
    assert_eq!(depth.list_limit(), None);
}

#[test]
fn swift_model_coverage_encode_error_helpers_expose_stable_metadata() {
    let empty = EncodeError::EmptyDelimiter;
    assert!(empty.is_empty_delimiter());
    assert!(!empty.is_encode_dot_in_keys_requires_allow_dots());
    assert!(empty.to_string().contains("delimiter"));

    let dots = EncodeError::EncodeDotInKeysRequiresAllowDots;
    assert!(dots.is_encode_dot_in_keys_requires_allow_dots());
    assert!(!dots.is_empty_delimiter());
}

#[test]
fn swift_encode_false_invalid_utf8_bytes_remain_visible() {
    let invalid = vec![0xC3, 0x28];
    let expected = String::from_utf8_lossy(&invalid).into_owned();

    let plain = encode(
        &obj(vec![("a", Value::Bytes(invalid.clone()))]),
        &EncodeOptions::new().with_encode(false),
    )
    .unwrap();
    assert_eq!(plain, format!("a={expected}"));

    let comma = encode(
        &obj(vec![("a", Value::Array(vec![Value::Bytes(invalid)]))]),
        &EncodeOptions::new()
            .with_encode(false)
            .with_list_format(ListFormat::Comma),
    )
    .unwrap();
    assert_eq!(comma, format!("a={expected}"));
}

#[test]
fn python_thread_safety_regression_shares_immutable_options_across_threads() {
    let decode_options = Arc::new(
        DecodeOptions::new()
            .with_comma(true)
            .with_allow_dots(true)
            .with_list_limit(6),
    );
    let encode_options = Arc::new(
        EncodeOptions::new()
            .with_allow_dots(true)
            .with_list_format(qs_rust::ListFormat::Comma),
    );
    let value = Arc::new(obj(vec![
        ("letters", Value::Array(vec![s("a"), s("b")])),
        ("user", obj(vec![("name", s("alice"))])),
    ]));

    thread::scope(|scope| {
        for _ in 0..8 {
            let decode_options = Arc::clone(&decode_options);
            let encode_options = Arc::clone(&encode_options);
            let value = Arc::clone(&value);

            scope.spawn(move || {
                for _ in 0..128 {
                    let decoded = decode("letters=a,b&user.name=alice", &decode_options).unwrap();
                    assert_eq!(
                        decoded.get("letters"),
                        Some(&Value::Array(vec![s("a"), s("b")])),
                    );

                    let encoded = encode(&value, &encode_options).unwrap();
                    assert_eq!(encoded, "letters=a%2Cb&user.name=alice");
                }
            });
        }
    });
}

#[cfg(feature = "serde")]
#[test]
fn serde_feature_uses_the_dynamic_semantic_core() {
    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
    struct User {
        name: String,
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
    struct Payload {
        user: User,
        tags: Vec<String>,
    }

    let decoded: Payload =
        from_str("user[name]=alice&tags[]=x&tags[]=y", &DecodeOptions::new()).unwrap();
    assert_eq!(
        decoded,
        Payload {
            user: User {
                name: "alice".to_owned(),
            },
            tags: vec!["x".to_owned(), "y".to_owned()],
        }
    );

    let encoded = to_string(
        &decoded,
        &EncodeOptions::new().with_list_format(ListFormat::Brackets),
    )
    .unwrap();
    assert_eq!(encoded, "user%5Bname%5D=alice&tags%5B%5D=x&tags%5B%5D=y");
}
