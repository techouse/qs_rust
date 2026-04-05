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

#[test]
fn default_and_plain_accumulators_cover_remaining_duplicate_promotions() {
    let last_options = DecodeOptions::new()
        .with_duplicates(Duplicates::Last)
        .with_comma(true)
        .with_list_limit(1);
    let mut last_values = DefaultAccumulator::direct();
    let mut token_count = 0usize;
    let mut has_any_structured_syntax = false;
    process_scanned_part_default_accumulator(
        ScannedPart::new("a=1"),
        Charset::Utf8,
        &last_options,
        &mut last_values,
        &mut token_count,
        &mut has_any_structured_syntax,
    )
    .unwrap();
    process_scanned_part_default_accumulator(
        ScannedPart::new("a=2,3"),
        Charset::Utf8,
        &last_options,
        &mut last_values,
        &mut token_count,
        &mut has_any_structured_syntax,
    )
    .unwrap();
    assert!(
        matches!(last_values, DefaultAccumulator::Parsed(entries) if entries.contains_key("a"))
    );

    let combine_options = DecodeOptions::new()
        .with_duplicates(Duplicates::Combine)
        .with_comma(true)
        .with_list_limit(1);
    let mut combine_values = DefaultAccumulator::direct();
    let mut combine_tokens = 0usize;
    let mut combine_structure = false;
    process_scanned_part_default_accumulator(
        ScannedPart::new("a="),
        Charset::Utf8,
        &combine_options,
        &mut combine_values,
        &mut combine_tokens,
        &mut combine_structure,
    )
    .unwrap();
    process_scanned_part_default_accumulator(
        ScannedPart::new("a=2,3"),
        Charset::Utf8,
        &combine_options,
        &mut combine_values,
        &mut combine_tokens,
        &mut combine_structure,
    )
    .unwrap();
    assert!(
        matches!(combine_values, DefaultAccumulator::Parsed(entries) if entries.contains_key("a"))
    );

    let mut plain_values = DefaultAccumulator::direct();
    let mut plain_tokens = 0usize;
    process_plain_part_default(
        "a=1",
        Some(1),
        &DecodeOptions::new().with_duplicates(Duplicates::Last),
        &mut plain_values,
        &mut plain_tokens,
    )
    .unwrap();
    process_plain_part_default(
        "a=2",
        Some(1),
        &DecodeOptions::new().with_duplicates(Duplicates::Last),
        &mut plain_values,
        &mut plain_tokens,
    )
    .unwrap();
    let DefaultAccumulator::Direct(entries) = &plain_values else {
        panic!("expected direct plain storage")
    };
    assert_eq!(entries.get("a"), Some(&scalar("2")));

    let mut promoted_plain_values = DefaultAccumulator::direct();
    let mut promoted_plain_tokens = 0usize;
    let promote_options = DecodeOptions::new()
        .with_duplicates(Duplicates::Combine)
        .with_list_limit(1);
    process_plain_part_default(
        "a=",
        Some(1),
        &promote_options,
        &mut promoted_plain_values,
        &mut promoted_plain_tokens,
    )
    .unwrap();
    process_plain_part_default(
        "a=tail",
        Some(1),
        &promote_options,
        &mut promoted_plain_values,
        &mut promoted_plain_tokens,
    )
    .unwrap();
    assert!(matches!(
        promoted_plain_values,
        DefaultAccumulator::Parsed(entries) if entries.contains_key("a")
    ));

    let mut parsed_plain_values = DefaultAccumulator::Parsed(Default::default());
    let mut parsed_plain_tokens = 0usize;
    process_plain_part_default(
        "b=1",
        Some(1),
        &DecodeOptions::new(),
        &mut parsed_plain_values,
        &mut parsed_plain_tokens,
    )
    .unwrap();
    let DefaultAccumulator::Parsed(entries) = parsed_plain_values else {
        panic!("expected parsed plain storage")
    };
    assert_eq!(
        entries.get("b").unwrap().clone().into_node(),
        Node::scalar(scalar("1"))
    );
}

#[test]
fn prefer_concrete_and_custom_paths_cover_remaining_storage_modes() {
    let last_options = DecodeOptions::new().with_duplicates(Duplicates::Last);
    let mut prefer_concrete = FlatValues::Concrete([("a".to_owned(), scalar("1"))].into());
    let mut token_count = 0usize;
    let mut has_any_structured_syntax = false;
    process_scanned_part_default_with_mode(
        ScannedPart::new("a=2"),
        Charset::Utf8,
        &last_options,
        &mut prefer_concrete,
        &mut token_count,
        &mut has_any_structured_syntax,
        DefaultStorageMode::PreferConcrete,
    )
    .unwrap();
    let FlatValues::Concrete(entries) = &prefer_concrete else {
        panic!("expected concrete values")
    };
    assert_eq!(entries.get("a"), Some(&scalar("2")));

    let combine_options = DecodeOptions::new()
        .with_duplicates(Duplicates::Combine)
        .with_comma(true)
        .with_list_limit(1);
    let mut promoted =
        FlatValues::Concrete([("a".to_owned(), scalar(String::new().as_str()))].into());
    let mut promote_tokens = 0usize;
    let mut promote_structure = false;
    process_scanned_part_default_with_mode(
        ScannedPart::new("a=1,2"),
        Charset::Utf8,
        &combine_options,
        &mut promoted,
        &mut promote_tokens,
        &mut promote_structure,
        DefaultStorageMode::PreferConcrete,
    )
    .unwrap();
    assert!(stores_parsed_value(&promoted, "a"));

    let decoder = DecodeDecoder::new(|input, _charset, kind| match kind {
        DecodeKind::Key => input.to_owned(),
        DecodeKind::Value => input.to_ascii_uppercase(),
    });
    let custom_options = DecodeOptions::new()
        .with_charset_sentinel(true)
        .with_comma(true)
        .with_duplicates(Duplicates::Combine)
        .with_decoder(Some(decoder));
    let mut custom_values = FlatValues::parsed();
    let mut custom_tokens = 0usize;
    let mut custom_structure = false;
    process_scanned_part_custom(
        ScannedPart::new("utf8=%E2%9C%93"),
        Charset::Utf8,
        &custom_options,
        &mut custom_values,
        &mut custom_tokens,
        &mut custom_structure,
    )
    .unwrap();
    assert!(custom_values.is_empty());
    assert_eq!(custom_tokens, 1);

    process_scanned_part_custom(
        ScannedPart::new("letters=a"),
        Charset::Utf8,
        &custom_options,
        &mut custom_values,
        &mut custom_tokens,
        &mut custom_structure,
    )
    .unwrap();
    process_scanned_part_custom(
        ScannedPart::new("letters=b,c"),
        Charset::Utf8,
        &custom_options,
        &mut custom_values,
        &mut custom_tokens,
        &mut custom_structure,
    )
    .unwrap();
    let FlatValues::Parsed(entries) = custom_values else {
        panic!("expected parsed custom values")
    };
    assert_eq!(entries.get("letters").unwrap().list_length_for_combine(), 3);

    let mut already_structured = true;
    update_structured_syntax_flag(
        ScannedPart::new("flat=1"),
        "flat",
        &DecodeOptions::new(),
        &mut already_structured,
    );
    assert!(already_structured);
}

#[test]
fn duplicate_modes_cover_remaining_default_and_custom_processing_paths() {
    let direct_last_options = DecodeOptions::new()
        .with_duplicates(Duplicates::Last)
        .with_comma(true)
        .with_list_limit(1);
    let mut direct_last = DefaultAccumulator::direct();
    let mut direct_last_tokens = 0usize;
    let mut direct_last_structure = false;
    process_scanned_part_default_accumulator(
        ScannedPart::new("a=1"),
        Charset::Utf8,
        &direct_last_options,
        &mut direct_last,
        &mut direct_last_tokens,
        &mut direct_last_structure,
    )
    .unwrap();
    process_scanned_part_default_accumulator(
        ScannedPart::new("a=2,3"),
        Charset::Utf8,
        &direct_last_options,
        &mut direct_last,
        &mut direct_last_tokens,
        &mut direct_last_structure,
    )
    .unwrap();
    assert!(matches!(
        direct_last,
        DefaultAccumulator::Parsed(entries) if entries.contains_key("a")
    ));

    let direct_combine_options = DecodeOptions::new()
        .with_duplicates(Duplicates::Combine)
        .with_comma(true)
        .with_list_limit(8);
    let mut direct_combine = DefaultAccumulator::direct();
    let mut direct_combine_tokens = 0usize;
    let mut direct_combine_structure = false;
    process_scanned_part_default_accumulator(
        ScannedPart::new("a=seed"),
        Charset::Utf8,
        &direct_combine_options,
        &mut direct_combine,
        &mut direct_combine_tokens,
        &mut direct_combine_structure,
    )
    .unwrap();
    process_scanned_part_default_accumulator(
        ScannedPart::new("a=2,3"),
        Charset::Utf8,
        &direct_combine_options,
        &mut direct_combine,
        &mut direct_combine_tokens,
        &mut direct_combine_structure,
    )
    .unwrap();
    assert!(matches!(direct_combine, DefaultAccumulator::Direct(_)));

    let mut parsed_default = DefaultAccumulator::Parsed(Default::default());
    let mut parsed_default_tokens = 0usize;
    let mut parsed_default_structure = false;
    process_scanned_part_default_accumulator(
        ScannedPart::new("a=1,2"),
        Charset::Utf8,
        &DecodeOptions::new()
            .with_duplicates(Duplicates::Last)
            .with_comma(true),
        &mut parsed_default,
        &mut parsed_default_tokens,
        &mut parsed_default_structure,
    )
    .unwrap();
    assert!(matches!(
        parsed_default,
        DefaultAccumulator::Parsed(entries) if entries.contains_key("a")
    ));

    let mut first_concrete = FlatValues::Concrete([("a".to_owned(), scalar("1"))].into());
    let mut first_tokens = 0usize;
    let mut first_structure = false;
    process_scanned_part_default_with_mode(
        ScannedPart::new("a=ignored"),
        Charset::Utf8,
        &DecodeOptions::new().with_duplicates(Duplicates::First),
        &mut first_concrete,
        &mut first_tokens,
        &mut first_structure,
        DefaultStorageMode::PreferConcrete,
    )
    .unwrap();
    let FlatValues::Concrete(first_entries) = &first_concrete else {
        panic!("expected concrete values to stay unchanged")
    };
    assert_eq!(first_entries.get("a"), Some(&scalar("1")));

    let mut prefer_last = FlatValues::Concrete([("a".to_owned(), scalar("1"))].into());
    let mut prefer_last_tokens = 0usize;
    let mut prefer_last_structure = false;
    process_scanned_part_default_with_mode(
        ScannedPart::new("a=2,3"),
        Charset::Utf8,
        &direct_last_options,
        &mut prefer_last,
        &mut prefer_last_tokens,
        &mut prefer_last_structure,
        DefaultStorageMode::PreferConcrete,
    )
    .unwrap();
    assert!(stores_parsed_value(&prefer_last, "a"));

    let mut prefer_combine = FlatValues::Concrete([("a".to_owned(), scalar("1"))].into());
    let mut prefer_combine_tokens = 0usize;
    let mut prefer_combine_structure = false;
    process_scanned_part_default_with_mode(
        ScannedPart::new("a=2,3"),
        Charset::Utf8,
        &DecodeOptions::new()
            .with_duplicates(Duplicates::Combine)
            .with_comma(true)
            .with_list_limit(1),
        &mut prefer_combine,
        &mut prefer_combine_tokens,
        &mut prefer_combine_structure,
        DefaultStorageMode::PreferConcrete,
    )
    .unwrap();
    assert!(stores_parsed_value(&prefer_combine, "a"));

    let mut custom_last = FlatValues::parsed();
    let mut custom_last_tokens = 0usize;
    let mut custom_last_structure = false;
    let custom_last_options = DecodeOptions::new()
        .with_duplicates(Duplicates::Last)
        .with_decoder(Some(DecodeDecoder::new(
            |input, _charset, kind| match kind {
                DecodeKind::Key => input.to_owned(),
                DecodeKind::Value => input.to_ascii_uppercase(),
            },
        )));
    process_scanned_part_custom(
        ScannedPart::new("name=one"),
        Charset::Utf8,
        &custom_last_options,
        &mut custom_last,
        &mut custom_last_tokens,
        &mut custom_last_structure,
    )
    .unwrap();
    process_scanned_part_custom(
        ScannedPart::new("name=two"),
        Charset::Utf8,
        &custom_last_options,
        &mut custom_last,
        &mut custom_last_tokens,
        &mut custom_last_structure,
    )
    .unwrap();
    let FlatValues::Parsed(custom_last_entries) = custom_last else {
        panic!("expected parsed custom values")
    };
    assert_eq!(
        custom_last_entries.get("name").unwrap().clone().into_node(),
        Node::scalar(scalar("TWO"))
    );
}

#[test]
fn hard_limit_duplicate_combine_errors_propagate_from_direct_combine_helpers() {
    let options = DecodeOptions::new()
        .with_duplicates(Duplicates::Combine)
        .with_list_limit(1)
        .with_throw_on_limit_exceeded(true);
    let mut values = DefaultAccumulator::direct();
    let mut token_count = 0usize;

    process_plain_part_default("a=1", Some(1), &options, &mut values, &mut token_count).unwrap();
    let error = process_plain_part_default("a=2", Some(1), &options, &mut values, &mut token_count)
        .unwrap_err();
    assert!(error.is_list_limit_exceeded());
    assert_eq!(error.list_limit(), Some(1));
}
