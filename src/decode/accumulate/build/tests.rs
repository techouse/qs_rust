use super::{
    DirectBuiltValue, build_custom_value, build_default_value, build_direct_value,
    parse_list_value, parse_list_value_default_scanned,
};
use crate::decode::flat::{DefaultStorageMode, ParsedFlatValue};
use crate::decode::scan::ScannedPart;
use crate::internal::node::Node;
use crate::options::{Charset, DecodeOptions};
use crate::value::Value;
use crate::{DecodeDecoder, DecodeKind};

fn scalar(value: &str) -> Node {
    Node::scalar(Value::String(value.to_owned()))
}

#[test]
fn list_builders_cover_limit_overflow_and_segment_decoding() {
    let overflow = parse_list_value(
        "a,b,c",
        &DecodeOptions::new().with_comma(true).with_list_limit(2),
        0,
    )
    .unwrap();
    assert_eq!(
        overflow,
        Node::OverflowObject {
            entries: [
                ("0".to_owned(), scalar("a")),
                ("1".to_owned(), scalar("b")),
                ("2".to_owned(), scalar("c")),
            ]
            .into(),
            max_index: 2,
        }
    );

    let error = parse_list_value(
        "tail",
        &DecodeOptions::new()
            .with_list_limit(1)
            .with_throw_on_limit_exceeded(true),
        1,
    )
    .unwrap_err();
    assert!(error.is_list_limit_exceeded());
    assert_eq!(error.list_limit(), Some(1));

    let scanned = parse_list_value_default_scanned(
        "%26%2365%3B,b%20c",
        ScannedPart::new("a=%26%2365%3B,b%20c"),
        Charset::Iso88591,
        &DecodeOptions::new()
            .with_comma(true)
            .with_interpret_numeric_entities(true),
        0,
    )
    .unwrap();
    assert!(matches!(
        scanned,
        ParsedFlatValue::Concrete(Value::Array(values))
            if values == vec![
                Value::String("A".to_owned()),
                Value::String("b c".to_owned()),
            ]
    ));

    let error = parse_list_value_default_scanned(
        "tail",
        ScannedPart::new("a=tail"),
        Charset::Utf8,
        &DecodeOptions::new()
            .with_list_limit(1)
            .with_throw_on_limit_exceeded(true),
        1,
    )
    .unwrap_err();
    assert!(error.is_list_limit_exceeded());
    assert_eq!(error.list_limit(), Some(1));
}

#[test]
fn direct_and_custom_builders_cover_suffix_and_null_paths() {
    let default_value = build_default_value(
        Some("plain"),
        ScannedPart::new("a=plain"),
        Charset::Utf8,
        &DecodeOptions::new(),
        0,
        DefaultStorageMode::ForceParsed,
    )
    .unwrap();
    assert!(matches!(default_value, ParsedFlatValue::Parsed { .. }));

    let direct_value = build_direct_value(
        None,
        ScannedPart::new("a"),
        Charset::Utf8,
        &DecodeOptions::new().with_strict_null_handling(true),
        0,
    )
    .unwrap();
    assert!(matches!(
        direct_value,
        DirectBuiltValue::Concrete(Value::Null)
    ));

    let custom_null = build_custom_value(
        None,
        ScannedPart::new("a"),
        Charset::Utf8,
        &DecodeOptions::new().with_strict_null_handling(true),
        0,
    )
    .unwrap();
    assert_eq!(custom_null.into_node(), Node::Value(Value::Null));

    let custom_empty = build_custom_value(
        None,
        ScannedPart::new("a"),
        Charset::Utf8,
        &DecodeOptions::new(),
        0,
    )
    .unwrap();
    assert_eq!(custom_empty.into_node(), scalar(""));

    let custom_wrapped = build_custom_value(
        Some("1,2"),
        ScannedPart::new("tags[]=1,2"),
        Charset::Utf8,
        &DecodeOptions::new().with_comma(true),
        0,
    )
    .unwrap();
    assert!(matches!(
        custom_wrapped,
        ParsedFlatValue::Parsed {
            node: Node::Array(items),
            needs_compaction: true,
        } if matches!(items.as_slice(), [Node::Array(_)])
    ));
}

#[test]
fn builder_limit_and_custom_array_decode_edges_are_covered() {
    let error = parse_list_value(
        "a,b",
        &DecodeOptions::new()
            .with_comma(true)
            .with_list_limit(1)
            .with_throw_on_limit_exceeded(true),
        0,
    )
    .unwrap_err();
    assert!(error.is_list_limit_exceeded());
    assert_eq!(error.list_limit(), Some(1));

    let decoded_array = build_custom_value(
        Some("a,b"),
        ScannedPart::new("tags=a,b"),
        Charset::Utf8,
        &DecodeOptions::new()
            .with_comma(true)
            .with_decoder(Some(DecodeDecoder::new(
                |input, _charset, kind| match kind {
                    DecodeKind::Key => input.to_owned(),
                    DecodeKind::Value => input.to_ascii_uppercase(),
                },
            ))),
        0,
    )
    .unwrap();
    assert_eq!(
        decoded_array.into_node(),
        Node::Array(vec![scalar("A"), scalar("B")])
    );
}
