use super::{
    DecodeOptions, Duplicates, Node, Value, combine_with_limit, decode, finalize_flat,
    parse_query_string_values, stores_concrete_value, stores_parsed_value_with_compaction,
};

#[test]
fn comma_values_over_limit_promote_to_object_instead_of_truncating() {
    let decoded = decode(
        "a=1,2,3,4",
        &DecodeOptions::new().with_comma(true).with_list_limit(3),
    )
    .unwrap();

    assert_eq!(
        decoded.get("a"),
        Some(&Value::Object(
            [
                ("0".to_owned(), Value::String("1".to_owned())),
                ("1".to_owned(), Value::String("2".to_owned())),
                ("2".to_owned(), Value::String("3".to_owned())),
                ("3".to_owned(), Value::String("4".to_owned())),
            ]
            .into()
        ))
    );
}

#[test]
fn combine_with_limit_flattens_overflow_appends_and_respects_throw_on_limit_exceeded() {
    let current = Node::OverflowObject {
        entries: [
            ("0".to_owned(), Node::scalar(Value::String("a".to_owned()))),
            ("1".to_owned(), Node::scalar(Value::String("b".to_owned()))),
        ]
        .into(),
        max_index: 1,
    };
    let next = Node::Array(vec![
        Node::scalar(Value::String("c".to_owned())),
        Node::scalar(Value::String("d".to_owned())),
    ]);

    let combined =
        combine_with_limit(current.clone(), next.clone(), &DecodeOptions::new()).unwrap();
    assert_eq!(
        combined,
        Node::OverflowObject {
            entries: [
                ("0".to_owned(), Node::scalar(Value::String("a".to_owned()))),
                ("1".to_owned(), Node::scalar(Value::String("b".to_owned()))),
                ("2".to_owned(), Node::scalar(Value::String("c".to_owned()))),
                ("3".to_owned(), Node::scalar(Value::String("d".to_owned()))),
            ]
            .into(),
            max_index: 3,
        }
    );

    let error = combine_with_limit(
        current,
        next,
        &DecodeOptions::new()
            .with_list_limit(3)
            .with_throw_on_limit_exceeded(true),
    )
    .unwrap_err();
    assert!(error.is_list_limit_exceeded());
    assert_eq!(error.list_limit(), Some(3));
}

#[test]
fn list_limit_gate_still_counts_empty_key_pairs() {
    let decoded = decode("=&a[]=b&a[]=c", &DecodeOptions::new().with_list_limit(1)).unwrap();

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
fn combine_with_limit_skips_undefined_items_inside_overflow_appends() {
    let current = Node::OverflowObject {
        entries: [("0".to_owned(), Node::scalar(Value::String("a".to_owned())))].into(),
        max_index: 0,
    };
    let next = Node::Array(vec![
        Node::Undefined,
        Node::scalar(Value::String("b".to_owned())),
    ]);

    let combined = combine_with_limit(current, next, &DecodeOptions::new()).unwrap();
    assert_eq!(
        combined,
        Node::OverflowObject {
            entries: [
                ("0".to_owned(), Node::scalar(Value::String("a".to_owned()))),
                ("1".to_owned(), Node::scalar(Value::String("b".to_owned()))),
            ]
            .into(),
            max_index: 1,
        }
    );
}

#[test]
fn duplicate_first_and_last_keep_concrete_values_when_possible() {
    let first_options = DecodeOptions::new().with_duplicates(Duplicates::First);
    let first = parse_query_string_values("a=1&a=2", &first_options).unwrap();
    assert!(stores_concrete_value(&first.values, "a"));
    assert_eq!(
        finalize_flat(first.values, &first_options)
            .unwrap()
            .get("a"),
        Some(&Value::String("1".to_owned()))
    );

    let last_options = DecodeOptions::new().with_duplicates(Duplicates::Last);
    let last = parse_query_string_values("a=1&a=2", &last_options).unwrap();
    assert!(stores_concrete_value(&last.values, "a"));
    assert_eq!(
        finalize_flat(last.values, &last_options).unwrap().get("a"),
        Some(&Value::String("2".to_owned()))
    );
}

#[test]
fn duplicate_combine_keeps_flat_concrete_values_until_promotion_is_required() {
    let options = DecodeOptions::new().with_duplicates(Duplicates::Combine);

    let unique = parse_query_string_values("a=1", &options).unwrap();
    assert!(stores_concrete_value(&unique.values, "a"));
    assert_eq!(
        finalize_flat(unique.values, &options).unwrap().get("a"),
        Some(&Value::String("1".to_owned()))
    );

    let combined = parse_query_string_values("a=1&a=2", &options).unwrap();
    assert!(stores_concrete_value(&combined.values, "a"));
    assert_eq!(
        finalize_flat(combined.values, &options).unwrap().get("a"),
        Some(&Value::Array(vec![
            Value::String("1".to_owned()),
            Value::String("2".to_owned()),
        ]))
    );
}

#[test]
fn duplicate_combine_keeps_concrete_array_scalar_mixes_under_limit_and_promotes_on_overflow() {
    let under_limit = DecodeOptions::new()
        .with_duplicates(Duplicates::Combine)
        .with_comma(true)
        .with_list_limit(4);
    let combined = parse_query_string_values("a=1,2&a=3", &under_limit).unwrap();
    assert!(stores_concrete_value(&combined.values, "a"));
    assert_eq!(
        finalize_flat(combined.values, &under_limit)
            .unwrap()
            .get("a"),
        Some(&Value::Array(vec![
            Value::String("1".to_owned()),
            Value::String("2".to_owned()),
            Value::String("3".to_owned()),
        ]))
    );

    let overflow = DecodeOptions::new()
        .with_duplicates(Duplicates::Combine)
        .with_comma(true)
        .with_list_limit(2);
    let promoted = parse_query_string_values("a=1,2&a=3", &overflow).unwrap();
    assert!(stores_parsed_value_with_compaction(&promoted.values, "a"));
    assert_eq!(
        finalize_flat(promoted.values, &overflow).unwrap().get("a"),
        Some(&Value::Object(
            [
                ("0".to_owned(), Value::String("1".to_owned())),
                ("1".to_owned(), Value::String("2".to_owned())),
                ("2".to_owned(), Value::String("3".to_owned())),
            ]
            .into()
        ))
    );
}
