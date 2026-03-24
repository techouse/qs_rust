use super::{Charset, DecodeOptions, Value, decode, decode_scalar, interpret_numeric_entities};

#[test]
fn parameter_limit_counts_charset_sentinel_before_skipping_it() {
    let error = decode(
        "utf8=%E2%9C%93&a=1",
        &DecodeOptions::new()
            .with_charset(Charset::Iso88591)
            .with_charset_sentinel(true)
            .with_parameter_limit(1)
            .with_throw_on_limit_exceeded(true),
    )
    .unwrap_err();

    assert!(error.is_parameter_limit_exceeded());
    assert_eq!(error.parameter_limit(), Some(1));
}

#[test]
fn unknown_charset_sentinel_value_is_skipped_without_switching_charsets() {
    let decoded = decode(
        "utf8=foo&%C3%B8=%C3%B8",
        &DecodeOptions::new()
            .with_charset(Charset::Utf8)
            .with_charset_sentinel(true),
    )
    .unwrap();

    assert_eq!(
        decoded,
        [("ø".to_owned(), Value::String("ø".to_owned()))].into()
    );
}

#[test]
fn charset_sentinel_applies_globally_even_when_not_first() {
    let decoded = decode(
        "a=%F8&utf8=%26%2310003%3B",
        &DecodeOptions::new()
            .with_charset(Charset::Utf8)
            .with_charset_sentinel(true),
    )
    .unwrap();

    assert_eq!(decoded.get("a"), Some(&Value::String("ø".to_owned())));
}

#[test]
fn decode_scalar_matches_swift_utils_utf8_and_latin1_examples() {
    assert_eq!(decode_scalar("foo%2Bbar", Charset::Utf8), "foo+bar");
    assert_eq!(decode_scalar("foo+bar", Charset::Utf8), "foo bar");
    assert_eq!(decode_scalar("foo%7Ebar", Charset::Iso88591), "foo~bar");
}

#[test]
fn interpret_numeric_entities_handles_decimal_hex_and_invalid_sequences() {
    assert_eq!(interpret_numeric_entities("&#9786;"), "☺");
    assert_eq!(interpret_numeric_entities("&#x1F4A9;"), "💩");
    assert_eq!(interpret_numeric_entities("x&#99999999;y"), "x&#99999999;y");
    assert_eq!(interpret_numeric_entities("&#xZZ;"), "&#xZZ;");
}

#[test]
fn latin1_decode_leaves_invalid_percent_escapes_intact() {
    let decoded = decode(
        "a=%ZZ&b=%41",
        &DecodeOptions::new().with_charset(Charset::Iso88591),
    )
    .unwrap();

    assert_eq!(decoded.get("a"), Some(&Value::String("%ZZ".to_owned())));
    assert_eq!(decoded.get("b"), Some(&Value::String("A".to_owned())));
}

#[test]
fn numeric_entity_interpretation_only_applies_in_latin1_mode() {
    let latin1 = decode(
        "a=%26%239786%3B",
        &DecodeOptions::new()
            .with_charset(Charset::Iso88591)
            .with_interpret_numeric_entities(true),
    )
    .unwrap();
    assert_eq!(latin1.get("a"), Some(&Value::String("☺".to_owned())));

    let utf8 = decode(
        "a=%26%239786%3B",
        &DecodeOptions::new()
            .with_charset(Charset::Utf8)
            .with_interpret_numeric_entities(true),
    )
    .unwrap();
    assert_eq!(utf8.get("a"), Some(&Value::String("&#9786;".to_owned())));
}

#[test]
fn utf8_decode_scalar_falls_back_to_plus_fixed_text_on_invalid_bytes() {
    assert_eq!(decode_scalar("%FF", Charset::Utf8), "%FF");
    assert_eq!(decode_scalar("a+b%FF", Charset::Utf8), "a b%FF");
    assert_eq!(decode_scalar("ø%41", Charset::Iso88591), "Ã¸A");
}
