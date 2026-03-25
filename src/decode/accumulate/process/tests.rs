use super::{
    advance_token_count, process_plain_part_default, process_query_part_custom,
    process_query_part_default, process_scanned_part_custom,
    process_scanned_part_default_accumulator, process_scanned_part_default_with_mode,
    update_structured_syntax_flag,
};
use crate::DecodeDecoder;
use crate::decode::flat::{DefaultAccumulator, DefaultStorageMode, FlatValues};
use crate::decode::scan::ScannedPart;
use crate::internal::node::Node;
use crate::options::{Charset, DecodeKind, DecodeOptions, Duplicates};
use crate::value::Value;

fn scalar(value: &str) -> Value {
    Value::String(value.to_owned())
}

fn stores_concrete_value(values: &FlatValues, key: &str) -> bool {
    matches!(values, FlatValues::Concrete(entries) if entries.contains_key(key))
}

fn stores_parsed_value(values: &FlatValues, key: &str) -> bool {
    matches!(values, FlatValues::Parsed(entries) if entries.contains_key(key))
}

#[test]
fn helper_functions_cover_token_limits_and_structured_detection() {
    let mut soft_count = 0usize;
    let soft_options = DecodeOptions::new().with_parameter_limit(1);
    assert!(advance_token_count(&mut soft_count, &soft_options).unwrap());
    assert!(!advance_token_count(&mut soft_count, &soft_options).unwrap());

    let mut hard_count = 0usize;
    let hard_options = DecodeOptions::new()
        .with_parameter_limit(1)
        .with_throw_on_limit_exceeded(true);
    assert!(advance_token_count(&mut hard_count, &hard_options).unwrap());
    let error = advance_token_count(&mut hard_count, &hard_options).unwrap_err();
    assert!(error.is_parameter_limit_exceeded());
    assert_eq!(error.parameter_limit(), Some(1));

    let mut has_structure = false;
    update_structured_syntax_flag(
        ScannedPart::new("a[b]=1"),
        "a[b]",
        &DecodeOptions::new(),
        &mut has_structure,
    );
    assert!(has_structure);

    has_structure = false;
    update_structured_syntax_flag(
        ScannedPart::new("a.b=1"),
        "a.b",
        &DecodeOptions::new().with_allow_dots(true),
        &mut has_structure,
    );
    assert!(has_structure);

    has_structure = false;
    update_structured_syntax_flag(
        ScannedPart::new("a%5Bb%5D=1"),
        "a[b]",
        &DecodeOptions::new(),
        &mut has_structure,
    );
    assert!(has_structure);
}

#[test]
fn default_accumulator_combines_direct_values_and_promotes_when_needed() {
    let options = DecodeOptions::new().with_duplicates(Duplicates::Combine);
    let mut values = DefaultAccumulator::direct();
    let mut token_count = 0usize;
    let mut has_any_structured_syntax = false;

    process_scanned_part_default_accumulator(
        ScannedPart::new("a=1"),
        Charset::Utf8,
        &options,
        &mut values,
        &mut token_count,
        &mut has_any_structured_syntax,
    )
    .unwrap();
    process_scanned_part_default_accumulator(
        ScannedPart::new("a=2"),
        Charset::Utf8,
        &options,
        &mut values,
        &mut token_count,
        &mut has_any_structured_syntax,
    )
    .unwrap();

    let DefaultAccumulator::Direct(entries) = &values else {
        panic!("expected direct accumulator")
    };
    assert_eq!(
        entries.get("a"),
        Some(&Value::Array(vec![scalar("1"), scalar("2")]))
    );
    assert!(!has_any_structured_syntax);

    let promote_options = DecodeOptions::new().with_comma(true).with_list_limit(1);
    let mut promoted = DefaultAccumulator::direct();
    let mut promote_tokens = 0usize;
    let mut promote_structure = false;
    process_scanned_part_default_accumulator(
        ScannedPart::new("a=1,2"),
        Charset::Utf8,
        &promote_options,
        &mut promoted,
        &mut promote_tokens,
        &mut promote_structure,
    )
    .unwrap();

    let DefaultAccumulator::Parsed(entries) = promoted else {
        panic!("expected parsed accumulator after overflow promotion")
    };
    assert!(entries.contains_key("a"));
}

#[test]
fn plain_and_custom_processing_cover_sentinel_skips_and_custom_decoding() {
    let mut plain_values = DefaultAccumulator::direct();
    let mut token_count = 0usize;
    process_plain_part_default(
        "utf8=%E2%9C%93",
        Some(4),
        &DecodeOptions::new().with_charset_sentinel(true),
        &mut plain_values,
        &mut token_count,
    )
    .unwrap();
    assert!(matches!(&plain_values, DefaultAccumulator::Direct(entries) if entries.is_empty()));
    assert_eq!(token_count, 1);

    process_plain_part_default(
        "=x",
        Some(0),
        &DecodeOptions::new(),
        &mut plain_values,
        &mut token_count,
    )
    .unwrap();
    assert!(matches!(&plain_values, DefaultAccumulator::Direct(entries) if entries.is_empty()));

    let mut custom_values = FlatValues::parsed();
    let mut custom_tokens = 0usize;
    let mut has_any_structured_syntax = false;
    let options = DecodeOptions::new().with_decoder(Some(DecodeDecoder::new(
        |input, _charset, kind| match kind {
            DecodeKind::Key => input.replace("%5B", "[").replace("%5D", "]"),
            DecodeKind::Value => input.to_ascii_uppercase(),
        },
    )));
    process_scanned_part_custom(
        ScannedPart::new("a%5Bb%5D=x"),
        Charset::Utf8,
        &options,
        &mut custom_values,
        &mut custom_tokens,
        &mut has_any_structured_syntax,
    )
    .unwrap();

    assert!(has_any_structured_syntax);
    let FlatValues::Parsed(entries) = custom_values else {
        panic!("expected parsed storage for custom decoder")
    };
    assert_eq!(
        entries.get("a[b]").unwrap().clone().into_node(),
        Node::scalar(scalar("X"))
    );
}

#[test]
fn flat_value_processing_covers_force_parsed_and_prefer_concrete_modes() {
    let mut prefer_concrete = FlatValues::Concrete(Default::default());
    let mut token_count = 0usize;
    let mut has_any_structured_syntax = false;
    process_scanned_part_default_with_mode(
        ScannedPart::new("plain=1"),
        Charset::Utf8,
        &DecodeOptions::new(),
        &mut prefer_concrete,
        &mut token_count,
        &mut has_any_structured_syntax,
        DefaultStorageMode::PreferConcrete,
    )
    .unwrap();
    assert!(stores_concrete_value(&prefer_concrete, "plain"));
    assert!(!has_any_structured_syntax);

    let mut promote_from_prefer_concrete = FlatValues::Concrete(Default::default());
    let mut promote_tokens = 0usize;
    let mut promote_structure = false;
    process_scanned_part_default_with_mode(
        ScannedPart::new("a[]=1,2"),
        Charset::Utf8,
        &DecodeOptions::new().with_comma(true).with_list_limit(1),
        &mut promote_from_prefer_concrete,
        &mut promote_tokens,
        &mut promote_structure,
        DefaultStorageMode::PreferConcrete,
    )
    .unwrap();
    assert!(stores_parsed_value(&promote_from_prefer_concrete, "a[]"));
    assert!(promote_structure);

    let mut force_parsed = FlatValues::Concrete(Default::default());
    let mut force_tokens = 0usize;
    let mut force_structure = false;
    process_query_part_default(
        "a.b=1",
        Charset::Utf8,
        &DecodeOptions::new().with_allow_dots(true),
        &mut force_parsed,
        &mut force_tokens,
        &mut force_structure,
    )
    .unwrap();
    assert!(stores_parsed_value(&force_parsed, "a.b"));
    assert!(force_structure);
}

#[test]
fn query_part_wrappers_cover_soft_limits_and_custom_first_duplicates() {
    let limit_options = DecodeOptions::new().with_parameter_limit(1);
    let mut limited_values = FlatValues::Concrete(Default::default());
    let mut limited_tokens = 0usize;
    let mut limited_structure = false;
    process_query_part_default(
        "a=1",
        Charset::Utf8,
        &limit_options,
        &mut limited_values,
        &mut limited_tokens,
        &mut limited_structure,
    )
    .unwrap();
    process_query_part_default(
        "b=2",
        Charset::Utf8,
        &limit_options,
        &mut limited_values,
        &mut limited_tokens,
        &mut limited_structure,
    )
    .unwrap();
    assert!(stores_parsed_value(&limited_values, "a"));
    assert!(!stores_parsed_value(&limited_values, "b"));

    let options = DecodeOptions::new()
        .with_duplicates(Duplicates::First)
        .with_decoder(Some(DecodeDecoder::new(
            |input, _charset, kind| match kind {
                DecodeKind::Key if input == "drop" => String::new(),
                DecodeKind::Key => input.to_owned(),
                DecodeKind::Value => input.to_ascii_uppercase(),
            },
        )));
    let mut values = FlatValues::parsed();
    let mut token_count = 0usize;
    let mut has_any_structured_syntax = false;
    process_query_part_custom(
        "drop=x",
        Charset::Utf8,
        &options,
        &mut values,
        &mut token_count,
        &mut has_any_structured_syntax,
    )
    .unwrap();
    assert!(values.is_empty());

    process_query_part_custom(
        "name=one",
        Charset::Utf8,
        &options,
        &mut values,
        &mut token_count,
        &mut has_any_structured_syntax,
    )
    .unwrap();
    process_query_part_custom(
        "name=two",
        Charset::Utf8,
        &options,
        &mut values,
        &mut token_count,
        &mut has_any_structured_syntax,
    )
    .unwrap();

    let FlatValues::Parsed(entries) = values else {
        panic!("expected parsed storage")
    };
    assert_eq!(
        entries.get("name").unwrap().clone().into_node(),
        Node::scalar(scalar("ONE"))
    );
}
