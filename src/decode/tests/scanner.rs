use super::{
    Charset, DecodeOptions, Delimiter, Regex, ScannedPart, Value, decode, dot_to_bracket_top_level,
    find_recoverable_balanced_open, parse_query_string_values, split_key_into_segments,
};

#[test]
fn split_key_into_segments_handles_dots_and_unterminated_groups() {
    assert_eq!(
        split_key_into_segments("user.email.name", true, 5, false).unwrap(),
        vec!["user".to_owned(), "[email]".to_owned(), "[name]".to_owned()]
    );
    assert_eq!(
        split_key_into_segments("a[b][c][d]", false, 2, false).unwrap(),
        vec![
            "a".to_owned(),
            "[b]".to_owned(),
            "[c]".to_owned(),
            "[[d]]".to_owned()
        ]
    );
    assert_eq!(
        split_key_into_segments("[", false, 5, false).unwrap(),
        vec!["[".to_owned()]
    );
    assert_eq!(
        split_key_into_segments("a[b[c]", false, 5, true).unwrap(),
        vec!["a[b".to_owned(), "[c]".to_owned()]
    );
}

#[test]
fn dot_to_bracket_top_level_preserves_degenerate_dots() {
    assert_eq!(dot_to_bracket_top_level("a..b"), "a.[b]");
    assert_eq!(dot_to_bracket_top_level("a.[b]"), "a.[b]");
    assert_eq!(dot_to_bracket_top_level("a[.].c"), "a[.][c]");
}

#[test]
fn dot_before_bracket_preserves_literal_dot_in_parent_key() {
    let decoded = decode("a.[b]=x", &DecodeOptions::new().with_allow_dots(true)).unwrap();

    assert_eq!(
        decoded.get("a."),
        Some(&Value::Object(
            [("b".to_owned(), Value::String("x".to_owned()))].into()
        ))
    );
}

#[test]
fn recoverable_balanced_open_finds_nested_group_after_unmatched_prefix() {
    assert_eq!(find_recoverable_balanced_open("a[b[c]", 2), Some(3));
    assert_eq!(find_recoverable_balanced_open("a[b[c", 2), None);
}

#[test]
fn parse_query_string_values_rejects_empty_string_delimiter() {
    let error = parse_query_string_values(
        "a=1",
        &DecodeOptions::new().with_delimiter(Delimiter::String(String::new())),
    )
    .unwrap_err();

    assert!(matches!(error, crate::error::DecodeError::EmptyDelimiter));
}

#[test]
fn parse_query_string_values_preserves_bracket_equals_split_precedence() {
    let decoded = decode("=x]=y", &DecodeOptions::new()).unwrap();
    assert_eq!(decoded.get("=x]"), Some(&Value::String("y".to_owned())));
}

#[test]
fn scanned_part_metadata_matches_split_suffix_and_sentinel_expectations() {
    let structured = ScannedPart::new("a%5D=x");
    assert_eq!(structured.split_pos, Some(4));
    assert!(!structured.has_bracket_suffix_assignment);
    assert!(!structured.is_charset_sentinel);
    assert_eq!(structured.sentinel_charset, None);

    let bracket_suffix = ScannedPart::new("tags[]=a,b");
    assert_eq!(bracket_suffix.split_pos, Some(6));
    assert!(bracket_suffix.has_bracket_suffix_assignment);
    assert!(!bracket_suffix.is_charset_sentinel);

    let utf8 = ScannedPart::new("utf8=%E2%9C%93");
    assert!(utf8.is_charset_sentinel);
    assert_eq!(utf8.sentinel_charset, Some(Charset::Utf8));
}

#[test]
fn scanned_part_metadata_tracks_default_fast_path_flags() {
    let flat = ScannedPart::new("k1=value");
    assert!(!flat.key_has_escape_or_plus);
    assert!(!flat.key_has_percent);
    assert!(!flat.key_has_open_bracket);
    assert!(!flat.key_has_dot);
    assert!(!flat.value_has_escape_or_plus);
    assert!(!flat.value_has_numeric_entity_candidate);
    assert_eq!(flat.value_comma_count, 0);

    let encoded = ScannedPart::new("a%2Eb=a,b%20c");
    assert!(encoded.key_has_escape_or_plus);
    assert!(encoded.key_has_percent);
    assert!(!encoded.key_has_open_bracket);
    assert!(!encoded.key_has_dot);
    assert!(encoded.value_has_escape_or_plus);
    assert!(!encoded.value_has_numeric_entity_candidate);
    assert_eq!(encoded.value_comma_count, 1);

    let numeric_entity = ScannedPart::new("a=%26%239786%3B");
    assert!(numeric_entity.value_has_escape_or_plus);
    assert!(numeric_entity.value_has_numeric_entity_candidate);
}

#[test]
fn parse_query_string_values_skip_adjacent_empty_segments_for_string_and_regex_delimiters() {
    let string_parsed = parse_query_string_values("a=1&&b=2&&", &DecodeOptions::new()).unwrap();
    assert_eq!(
        super::finalize_flat(string_parsed.values, &DecodeOptions::new()).unwrap(),
        [
            ("a".to_owned(), Value::String("1".to_owned())),
            ("b".to_owned(), Value::String("2".to_owned()))
        ]
        .into()
    );

    let regex_options =
        DecodeOptions::new().with_delimiter(Delimiter::Regex(Regex::new("[;,]").unwrap()));
    let regex_parsed = parse_query_string_values("a=1;b=2,,c=3;;", &regex_options).unwrap();
    assert_eq!(
        super::finalize_flat(regex_parsed.values, &regex_options).unwrap(),
        [
            ("a".to_owned(), Value::String("1".to_owned())),
            ("b".to_owned(), Value::String("2".to_owned())),
            ("c".to_owned(), Value::String("3".to_owned()))
        ]
        .into()
    );
}

#[test]
fn raw_parse_marks_only_potentially_structured_inputs_for_follow_up_scan() {
    let flat = parse_query_string_values("a=1&b=2", &DecodeOptions::new()).unwrap();
    assert!(!flat.has_any_structured_syntax);

    let bracketed = parse_query_string_values("a[b]=1&c=2", &DecodeOptions::new()).unwrap();
    assert!(bracketed.has_any_structured_syntax);

    let dotted =
        parse_query_string_values("a.b=1", &DecodeOptions::new().with_allow_dots(true)).unwrap();
    assert!(dotted.has_any_structured_syntax);

    let encoded_dot =
        parse_query_string_values("a%2Eb=1", &DecodeOptions::new().with_allow_dots(true)).unwrap();
    assert!(encoded_dot.has_any_structured_syntax);
}

#[test]
fn one_byte_literal_delimiter_scanner_matches_public_decode_behavior() {
    let decoded = decode("a=1&b=2&&c=3", &DecodeOptions::new()).unwrap();
    assert_eq!(
        decoded,
        [
            ("a".to_owned(), Value::String("1".to_owned())),
            ("b".to_owned(), Value::String("2".to_owned())),
            ("c".to_owned(), Value::String("3".to_owned())),
        ]
        .into()
    );
}

#[test]
fn multi_byte_literal_delimiter_scanner_preserves_split_precedence_and_sentinels() {
    let options = DecodeOptions::new()
        .with_delimiter(Delimiter::String("&&".to_owned()))
        .with_charset(Charset::Iso88591)
        .with_charset_sentinel(true);

    let decoded = decode("utf8=%E2%9C%93&&a%5D=x&&b=2", &options).unwrap();
    assert_eq!(
        decoded,
        [
            ("a]".to_owned(), Value::String("x".to_owned())),
            ("b".to_owned(), Value::String("2".to_owned())),
        ]
        .into()
    );
}

#[test]
fn bracket_suffix_comma_values_become_nested_arrays() {
    let decoded = decode("tags[]=a,b", &DecodeOptions::new().with_comma(true)).unwrap();
    assert_eq!(
        decoded.get("tags"),
        Some(&Value::Array(vec![Value::Array(vec![
            Value::String("a".to_owned()),
            Value::String("b".to_owned()),
        ])]))
    );

    let raw_part_marker = decode("a[b]=1,2[]=", &DecodeOptions::new().with_comma(true)).unwrap();
    assert_eq!(
        raw_part_marker.get("a"),
        Some(&Value::Object(
            [(
                "b".to_owned(),
                Value::Array(vec![Value::Array(vec![
                    Value::String("1".to_owned()),
                    Value::String("2[]=".to_owned()),
                ])]),
            )]
            .into()
        ))
    );
}

#[test]
fn single_byte_plain_fast_path_preserves_featureful_fallback_behavior() {
    let encoded_dot = decode("a%2Eb=1", &DecodeOptions::new().with_allow_dots(true)).unwrap();
    assert_eq!(
        encoded_dot.get("a"),
        Some(&Value::Object(
            [("b".to_owned(), Value::String("1".to_owned()))].into()
        ))
    );

    let plus = decode("a+b=c+d", &DecodeOptions::new()).unwrap();
    assert_eq!(plus.get("a b"), Some(&Value::String("c d".to_owned())));

    let comma = decode("a=1,2", &DecodeOptions::new().with_comma(true)).unwrap();
    assert_eq!(
        comma.get("a"),
        Some(&Value::Array(vec![
            Value::String("1".to_owned()),
            Value::String("2".to_owned()),
        ]))
    );

    let sentinel = decode(
        "utf8=foo&a=1",
        &DecodeOptions::new()
            .with_charset(Charset::Utf8)
            .with_charset_sentinel(true),
    )
    .unwrap();
    assert_eq!(
        sentinel,
        [("a".to_owned(), Value::String("1".to_owned()))].into()
    );
}
