use proptest::prelude::*;
use qs_rust::{
    Charset, EncodeOptions, Format, ListFormat, SortMode, Value, WhitelistSelector, encode,
};

fn leaf_string_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        Just("x".to_owned()),
        Just("x y".to_owned()),
        Just("a,b".to_owned()),
        Just("name".to_owned()),
        Just("a.b".to_owned()),
    ]
}

fn key_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("a".to_owned()),
        Just("b".to_owned()),
        Just("c".to_owned()),
        Just("user".to_owned()),
        Just("tags".to_owned()),
        Just("a b".to_owned()),
        Just("a.b".to_owned()),
    ]
}

fn scalar_value_strategy() -> impl Strategy<Value = Value> {
    prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        (-1000i64..1000).prop_map(Value::I64),
        (0u64..1000).prop_map(Value::U64),
        (-500i32..500).prop_map(|value| Value::F64(f64::from(value) / 10.0)),
        leaf_string_strategy().prop_map(Value::String),
        prop::collection::vec(any::<u8>(), 0..5).prop_map(Value::Bytes),
    ]
}

fn value_strategy() -> impl Strategy<Value = Value> {
    scalar_value_strategy().prop_recursive(4, 96, 8, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..4).prop_map(Value::Array),
            prop::collection::vec((key_strategy(), inner), 0..4)
                .prop_map(|entries| Value::Object(entries.into_iter().collect())),
        ]
    })
}

fn list_format_strategy() -> impl Strategy<Value = ListFormat> {
    prop_oneof![
        Just(ListFormat::Indices),
        Just(ListFormat::Brackets),
        Just(ListFormat::Repeat),
        Just(ListFormat::Comma),
    ]
}

fn charset_strategy() -> impl Strategy<Value = Charset> {
    prop_oneof![Just(Charset::Utf8), Just(Charset::Iso88591)]
}

fn format_strategy() -> impl Strategy<Value = Format> {
    prop_oneof![Just(Format::Rfc3986), Just(Format::Rfc1738)]
}

fn whitelist_selector_strategy() -> impl Strategy<Value = WhitelistSelector> {
    prop_oneof![
        key_strategy().prop_map(WhitelistSelector::Key),
        (0usize..4).prop_map(WhitelistSelector::Index),
    ]
}

fn sort_mode_strategy() -> impl Strategy<Value = SortMode> {
    prop_oneof![Just(SortMode::Preserve), Just(SortMode::LexicographicAsc)]
}

fn encode_case_strategy() -> impl Strategy<Value = (Value, EncodeOptions)> {
    (
        value_strategy(),
        (
            any::<bool>(),
            prop_oneof![Just("&".to_owned()), Just(";".to_owned())],
            list_format_strategy(),
            format_strategy(),
            charset_strategy(),
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
        ),
        (
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
            prop_oneof![
                Just(None),
                prop::collection::vec(whitelist_selector_strategy(), 0..4).prop_map(Some),
            ],
            sort_mode_strategy(),
        ),
    )
        .prop_map(
            |(
                value,
                (
                    encode_flag,
                    delimiter,
                    list_format,
                    format,
                    charset,
                    charset_sentinel,
                    allow_empty_lists,
                    strict_null_handling,
                ),
                (
                    skip_nulls,
                    comma_round_trip,
                    encode_values_only,
                    add_query_prefix,
                    allow_dots,
                    encode_dot_in_keys,
                    whitelist,
                    sort,
                ),
            )| {
                let encode_dot_in_keys = allow_dots && encode_dot_in_keys && !encode_values_only;
                let options = EncodeOptions::new()
                    .with_encode(encode_flag)
                    .with_delimiter(delimiter)
                    .with_list_format(list_format)
                    .with_format(format)
                    .with_charset(charset)
                    .with_charset_sentinel(charset_sentinel)
                    .with_allow_empty_lists(allow_empty_lists)
                    .with_strict_null_handling(strict_null_handling)
                    .with_skip_nulls(skip_nulls)
                    .with_comma_round_trip(comma_round_trip)
                    .with_encode_values_only(encode_values_only)
                    .with_add_query_prefix(add_query_prefix)
                    .with_allow_dots(allow_dots)
                    .with_encode_dot_in_keys(encode_dot_in_keys)
                    .with_whitelist(whitelist)
                    .with_sort(sort);

                (value, options)
            },
        )
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 96,
        .. ProptestConfig::default()
    })]

    #[test]
    fn generated_values_encode_stably((value, options) in encode_case_strategy()) {
        let first = encode(&value, &options).unwrap();
        let second = encode(&value, &options).unwrap();
        prop_assert_eq!(first, second);
    }

    #[test]
    fn deep_generated_encode_chains_are_stack_safe(depth in 64usize..384, leaf in leaf_string_strategy()) {
        let mut value = Value::String(leaf);
        for _ in 0..depth {
            value = Value::Object([("a".to_owned(), value)].into());
        }

        let root = Value::Object([("root".to_owned(), value)].into());
        let encoded = encode(&root, &EncodeOptions::new()).unwrap();
        prop_assert!(encoded.starts_with("root"));
    }
}
