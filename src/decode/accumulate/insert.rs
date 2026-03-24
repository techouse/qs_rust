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
