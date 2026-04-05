use super::{insert_default_value, insert_occupied_value};
use crate::decode::flat::{FlatValues, ParsedFlatValue};
use crate::internal::node::Node;
use crate::options::{DecodeOptions, Duplicates};
use crate::value::Value;
use indexmap::IndexMap;
use indexmap::map::Entry;

fn scalar(value: &str) -> Value {
    Value::String(value.to_owned())
}

fn stores_concrete_value(values: &FlatValues, key: &str) -> bool {
    matches!(values, FlatValues::Concrete(entries) if entries.contains_key(key))
}

fn stores_parsed_value(values: &FlatValues, key: &str) -> bool {
    matches!(values, FlatValues::Parsed(entries) if entries.contains_key(key))
}

fn stores_parsed_value_with_compaction(values: &FlatValues, key: &str) -> bool {
    matches!(
        values,
        FlatValues::Parsed(entries)
            if matches!(
                entries.get(key),
                Some(ParsedFlatValue::Parsed {
                    needs_compaction: true,
                    ..
                })
            )
    )
}

#[test]
fn occupied_insert_respects_duplicate_strategies() {
    let combine_options = DecodeOptions::new().with_duplicates(Duplicates::Combine);
    let mut combine_entries =
        IndexMap::from([("a".to_owned(), ParsedFlatValue::concrete(scalar("1")))]);
    match combine_entries.entry("a".to_owned()) {
        Entry::Occupied(mut entry) => {
            insert_occupied_value(
                &mut entry,
                ParsedFlatValue::concrete(scalar("2")),
                &combine_options,
            )
            .unwrap();
        }
        Entry::Vacant(_) => unreachable!("expected occupied entry"),
    }
    assert_eq!(
        combine_entries.get("a").unwrap().list_length_for_combine(),
        2
    );

    let last_options = DecodeOptions::new().with_duplicates(Duplicates::Last);
    let mut last_entries =
        IndexMap::from([("a".to_owned(), ParsedFlatValue::concrete(scalar("1")))]);
    match last_entries.entry("a".to_owned()) {
        Entry::Occupied(mut entry) => {
            insert_occupied_value(
                &mut entry,
                ParsedFlatValue::concrete(scalar("2")),
                &last_options,
            )
            .unwrap();
        }
        Entry::Vacant(_) => unreachable!("expected occupied entry"),
    }
    assert_eq!(
        last_entries.get("a").unwrap().clone().into_node(),
        Node::scalar(scalar("2"))
    );

    let first_options = DecodeOptions::new().with_duplicates(Duplicates::First);
    let mut first_entries =
        IndexMap::from([("a".to_owned(), ParsedFlatValue::concrete(scalar("1")))]);
    match first_entries.entry("a".to_owned()) {
        Entry::Occupied(mut entry) => {
            insert_occupied_value(
                &mut entry,
                ParsedFlatValue::concrete(scalar("2")),
                &first_options,
            )
            .unwrap();
        }
        Entry::Vacant(_) => unreachable!("expected occupied entry"),
    }
    assert_eq!(
        first_entries.get("a").unwrap().clone().into_node(),
        Node::scalar(scalar("1"))
    );
}

#[test]
fn default_insert_keeps_concrete_storage_until_parsing_is_required() {
    let mut first_values = FlatValues::Concrete(Default::default());
    let first_options = DecodeOptions::new().with_duplicates(Duplicates::First);
    insert_default_value(
        &mut first_values,
        "a".to_owned(),
        ParsedFlatValue::concrete(scalar("1")),
        &first_options,
    )
    .unwrap();
    insert_default_value(
        &mut first_values,
        "a".to_owned(),
        ParsedFlatValue::concrete(scalar("2")),
        &first_options,
    )
    .unwrap();
    assert!(stores_concrete_value(&first_values, "a"));
    let FlatValues::Concrete(first_entries) = &first_values else {
        panic!("expected concrete storage");
    };
    assert_eq!(first_entries.get("a"), Some(&scalar("1")));

    let mut last_values = FlatValues::Concrete(Default::default());
    let last_options = DecodeOptions::new().with_duplicates(Duplicates::Last);
    insert_default_value(
        &mut last_values,
        "a".to_owned(),
        ParsedFlatValue::concrete(scalar("1")),
        &last_options,
    )
    .unwrap();
    insert_default_value(
        &mut last_values,
        "a".to_owned(),
        ParsedFlatValue::concrete(scalar("2")),
        &last_options,
    )
    .unwrap();
    let FlatValues::Concrete(last_entries) = &last_values else {
        panic!("expected concrete storage");
    };
    assert_eq!(last_entries.get("a"), Some(&scalar("2")));

    let mut combine_values = FlatValues::Concrete(Default::default());
    let combine_options = DecodeOptions::new().with_duplicates(Duplicates::Combine);
    insert_default_value(
        &mut combine_values,
        "a".to_owned(),
        ParsedFlatValue::concrete(scalar("1")),
        &combine_options,
    )
    .unwrap();
    insert_default_value(
        &mut combine_values,
        "a".to_owned(),
        ParsedFlatValue::concrete(scalar("2")),
        &combine_options,
    )
    .unwrap();
    assert!(stores_parsed_value(&combine_values, "a"));

    let mut parsed_values = FlatValues::Concrete(Default::default());
    insert_default_value(
        &mut parsed_values,
        "a".to_owned(),
        ParsedFlatValue::parsed(Node::Array(vec![Node::scalar(scalar("1"))]), true),
        &DecodeOptions::new(),
    )
    .unwrap();
    assert!(stores_parsed_value_with_compaction(&parsed_values, "a"));
}

#[test]
fn default_insert_first_ignores_late_parsed_values_for_existing_concrete_keys() {
    let mut values = FlatValues::Concrete([("a".to_owned(), scalar("1"))].into());
    insert_default_value(
        &mut values,
        "a".to_owned(),
        ParsedFlatValue::parsed(Node::Array(vec![Node::scalar(scalar("2"))]), true),
        &DecodeOptions::new().with_duplicates(Duplicates::First),
    )
    .unwrap();

    let FlatValues::Concrete(entries) = values else {
        panic!("expected concrete values to remain in place")
    };
    assert_eq!(entries.get("a"), Some(&scalar("1")));
}
