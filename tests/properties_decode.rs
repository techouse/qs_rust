use proptest::prelude::*;
use qs_rust::{Charset, DecodeOptions, Delimiter, Duplicates, Value, decode};
use regex::Regex;

#[derive(Clone, Copy, Debug)]
enum DelimiterKind {
    Ampersand,
    Semicolon,
    Regex,
}

fn delimiter_kind_strategy() -> impl Strategy<Value = DelimiterKind> {
    prop_oneof![
        Just(DelimiterKind::Ampersand),
        Just(DelimiterKind::Semicolon),
        Just(DelimiterKind::Regex),
    ]
}

fn key_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("a".to_owned()),
        Just("b".to_owned()),
        Just("flag".to_owned()),
        Just("user[name]".to_owned()),
        Just("user.name".to_owned()),
        Just("tags[]".to_owned()),
        Just("tags[0]".to_owned()),
        Just("a%252Eb".to_owned()),
        Just("utf8".to_owned()),
        Just(String::new()),
    ]
}

fn value_strategy() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(None),
        Just(Some(String::new())),
        Just(Some("1".to_owned())),
        Just(Some("x+y".to_owned())),
        Just(Some("a,b".to_owned())),
        Just(Some("%26%239786%3B".to_owned())),
        Just(Some("%F8".to_owned())),
        Just(Some("%E2%9C%93".to_owned())),
    ]
}

fn duplicates_strategy() -> impl Strategy<Value = Duplicates> {
    prop_oneof![
        Just(Duplicates::Combine),
        Just(Duplicates::First),
        Just(Duplicates::Last),
    ]
}

fn charset_strategy() -> impl Strategy<Value = Charset> {
    prop_oneof![Just(Charset::Utf8), Just(Charset::Iso88591)]
}

fn build_query(tokens: &[(String, Option<String>)], delimiter: DelimiterKind) -> String {
    let encoded = tokens
        .iter()
        .map(|(key, value)| match value {
            Some(value) => format!("{key}={value}"),
            None => key.clone(),
        })
        .collect::<Vec<_>>();

    match delimiter {
        DelimiterKind::Ampersand => encoded.join("&"),
        DelimiterKind::Semicolon => encoded.join(";"),
        DelimiterKind::Regex => {
            encoded
                .into_iter()
                .enumerate()
                .fold(String::new(), |mut output, (index, token)| {
                    if index > 0 {
                        output.push(if index % 2 == 0 { '&' } else { ';' });
                    }
                    output.push_str(&token);
                    output
                })
        }
    }
}

fn decode_case_strategy() -> impl Strategy<Value = (String, DecodeOptions)> {
    (
        delimiter_kind_strategy(),
        prop::collection::vec((key_strategy(), value_strategy()), 0..8),
        (
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
            1usize..6,
            charset_strategy(),
            any::<bool>(),
            any::<bool>(),
            0usize..5,
        ),
        (
            1usize..8,
            duplicates_strategy(),
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
        ),
    )
        .prop_map(
            |(
                delimiter_kind,
                tokens,
                (
                    allow_dots,
                    decode_dot_in_keys,
                    allow_empty_lists,
                    allow_sparse_lists,
                    list_limit,
                    charset,
                    charset_sentinel,
                    comma,
                    depth,
                ),
                (
                    parameter_limit,
                    duplicates,
                    ignore_query_prefix,
                    interpret_numeric_entities,
                    parse_lists,
                    strict_depth,
                    strict_null_handling,
                    throw_on_limit_exceeded,
                ),
            )| {
                let delimiter = match delimiter_kind {
                    DelimiterKind::Ampersand => Delimiter::String("&".to_owned()),
                    DelimiterKind::Semicolon => Delimiter::String(";".to_owned()),
                    DelimiterKind::Regex => Delimiter::Regex(Regex::new("[&;]").unwrap()),
                };

                let query = build_query(&tokens, delimiter_kind);
                let options = DecodeOptions::new()
                    .with_allow_dots(allow_dots)
                    .with_decode_dot_in_keys(decode_dot_in_keys)
                    .with_allow_empty_lists(allow_empty_lists)
                    .with_allow_sparse_lists(allow_sparse_lists)
                    .with_list_limit(list_limit)
                    .with_charset(charset)
                    .with_charset_sentinel(charset_sentinel)
                    .with_comma(comma)
                    .with_delimiter(delimiter)
                    .with_depth(depth)
                    .with_parameter_limit(parameter_limit)
                    .with_duplicates(duplicates)
                    .with_ignore_query_prefix(ignore_query_prefix)
                    .with_interpret_numeric_entities(interpret_numeric_entities)
                    .with_parse_lists(parse_lists)
                    .with_strict_depth(strict_depth)
                    .with_strict_null_handling(strict_null_handling)
                    .with_throw_on_limit_exceeded(throw_on_limit_exceeded);

                (query, options)
            },
        )
}

fn assert_query_only_shapes(value: &Value) {
    match value {
        Value::Null | Value::String(_) => {}
        Value::Temporal(_) => {
            panic!("query decoding produced temporal data unexpectedly: {value:?}")
        }
        Value::Array(values) => {
            for value in values {
                assert_query_only_shapes(value);
            }
        }
        Value::Object(entries) => {
            for value in entries.values() {
                assert_query_only_shapes(value);
            }
        }
        Value::Bool(_) | Value::I64(_) | Value::U64(_) | Value::F64(_) | Value::Bytes(_) => {
            panic!("query decoding produced non-query scalar: {value:?}")
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 96,
        .. ProptestConfig::default()
    })]

    #[test]
    fn generated_queries_decode_without_panics_and_keep_query_shapes((query, options) in decode_case_strategy()) {
        if let Ok(decoded) = decode(&query, &options) {
            for value in decoded.values() {
                assert_query_only_shapes(value);
            }
        }
    }

    #[test]
    fn deep_generated_decode_chains_are_stack_safe(depth in 64usize..384) {
        let mut query = String::from("leaf=x");
        for _ in 0..depth {
            query = format!("a[{query}]");
        }

        let options = DecodeOptions::new().with_depth(depth + 1);
        let _ = decode(&query, &options).unwrap();
    }
}
