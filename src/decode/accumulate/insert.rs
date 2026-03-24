//! Insert and duplicate-resolution helpers for flat decode storage.

use indexmap::map::{Entry, OccupiedEntry};

use crate::error::DecodeError;
use crate::internal::node::Node;
use crate::options::{DecodeOptions, Duplicates};

use super::super::flat::{FlatValues, ParsedFlatValue};
use super::combine::combine_with_limit;

pub(in crate::decode) fn insert_value(
    entry: Entry<'_, String, ParsedFlatValue>,
    value: ParsedFlatValue,
    options: &DecodeOptions,
) -> Result<(), DecodeError> {
    match entry {
        Entry::Occupied(mut entry) => insert_occupied_value(&mut entry, value, options),
        Entry::Vacant(entry) => {
            entry.insert(value);
            Ok(())
        }
    }
}

pub(super) fn insert_occupied_value(
    entry: &mut OccupiedEntry<'_, String, ParsedFlatValue>,
    value: ParsedFlatValue,
    options: &DecodeOptions,
) -> Result<(), DecodeError> {
    match options.duplicates {
        Duplicates::Combine => {
            let current = std::mem::replace(
                entry.get_mut(),
                ParsedFlatValue::parsed(Node::Undefined, true),
            );
            *entry.get_mut() = ParsedFlatValue::parsed(
                combine_with_limit(current.into_node(), value.into_node(), options)?,
                true,
            );
        }
        Duplicates::Last => *entry.get_mut() = value,
        Duplicates::First => {}
    }

    Ok(())
}

pub(super) fn insert_default_value(
    values: &mut FlatValues,
    key: String,
    value: ParsedFlatValue,
    options: &DecodeOptions,
) -> Result<(), DecodeError> {
    match values {
        FlatValues::Concrete(entries) => {
            if let ParsedFlatValue::Concrete(value) = value {
                match options.duplicates {
                    Duplicates::First => {
                        entries.entry(key).or_insert(value);
                        return Ok(());
                    }
                    Duplicates::Last => {
                        entries.insert(key, value);
                        return Ok(());
                    }
                    Duplicates::Combine => {
                        if !entries.contains_key(&key) {
                            entries.insert(key, value);
                            return Ok(());
                        }
                        let values = values.ensure_parsed();
                        return insert_value(
                            values.entry(key),
                            ParsedFlatValue::concrete(value),
                            options,
                        );
                    }
                }
            }

            if matches!(options.duplicates, Duplicates::First) && entries.contains_key(&key) {
                return Ok(());
            }

            let values = values.ensure_parsed();
            insert_value(values.entry(key), value, options)
        }
        FlatValues::Parsed(entries) => insert_value(entries.entry(key), value, options),
    }
}

#[cfg(test)]
mod tests {
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
        assert!(first_values.stores_concrete_value("a"));
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
        assert!(combine_values.stores_parsed_value("a"));

        let mut parsed_values = FlatValues::Concrete(Default::default());
        insert_default_value(
            &mut parsed_values,
            "a".to_owned(),
            ParsedFlatValue::parsed(Node::Array(vec![Node::scalar(scalar("1"))]), true),
            &DecodeOptions::new(),
        )
        .unwrap();
        assert!(parsed_values.stores_parsed_value_with_compaction("a"));
    }
}
