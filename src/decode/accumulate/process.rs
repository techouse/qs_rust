//! Part-processing orchestration for flat decode accumulation.

use indexmap::map::Entry;

use crate::error::DecodeError;
use crate::options::{Charset, DecodeKind, DecodeOptions, Duplicates};

use super::super::flat::{
    DefaultAccumulator, DefaultStorageMode, DirectInsertOutcome, FlatValues, ParsedFlatValue,
    value_list_length_for_combine,
};
use super::super::keys::key_might_be_structured;
use super::super::scalar::{decode_component, decode_scalar_with_known_flags};
use super::super::scan::ScannedPart;
use super::build::{
    DirectBuiltValue, build_custom_value, build_default_value, build_direct_value,
    build_plain_value,
};
use super::combine::try_combine_direct_values;
use super::insert::{insert_default_value, insert_occupied_value, insert_value};

pub(in crate::decode) fn process_query_part_default(
    part: &str,
    effective_charset: Charset,
    options: &DecodeOptions,
    values: &mut FlatValues,
    token_count: &mut usize,
    has_any_structured_syntax: &mut bool,
) -> Result<(), DecodeError> {
    process_scanned_part_default_with_mode(
        ScannedPart::new(part),
        effective_charset,
        options,
        values,
        token_count,
        has_any_structured_syntax,
        DefaultStorageMode::ForceParsed,
    )
}

pub(in crate::decode) fn process_query_part_custom(
    part: &str,
    effective_charset: Charset,
    options: &DecodeOptions,
    values: &mut FlatValues,
    token_count: &mut usize,
    has_any_structured_syntax: &mut bool,
) -> Result<(), DecodeError> {
    process_scanned_part_custom(
        ScannedPart::new(part),
        effective_charset,
        options,
        values,
        token_count,
        has_any_structured_syntax,
    )
}

pub(in crate::decode) fn process_scanned_part_default_accumulator(
    part: ScannedPart<'_>,
    effective_charset: Charset,
    options: &DecodeOptions,
    values: &mut DefaultAccumulator,
    token_count: &mut usize,
    has_any_structured_syntax: &mut bool,
) -> Result<(), DecodeError> {
    if !advance_token_count(token_count, options)?
        || (options.charset_sentinel && part.is_charset_sentinel)
    {
        return Ok(());
    }

    let (raw_key, raw_value) = part.raw_parts();
    let decoded_key = if part.key_has_escape_or_plus {
        decode_scalar_with_known_flags(raw_key, effective_charset, true)
    } else {
        raw_key.to_owned()
    };
    if decoded_key.is_empty() {
        return Ok(());
    }
    update_structured_syntax_flag(part, &decoded_key, options, has_any_structured_syntax);

    match values {
        DefaultAccumulator::Direct(_) => {
            let action = {
                let DefaultAccumulator::Direct(entries) = values else {
                    unreachable!("direct accumulator should still be direct")
                };

                match entries.entry(decoded_key) {
                    Entry::Vacant(entry) => {
                        match build_direct_value(raw_value, part, effective_charset, options, 0)? {
                            DirectBuiltValue::Concrete(value) => {
                                entry.insert(value);
                                DirectInsertOutcome::Done
                            }
                            DirectBuiltValue::Promote(value) => {
                                DirectInsertOutcome::PromoteInsert {
                                    key: entry.key().clone(),
                                    value,
                                    via_duplicates: false,
                                }
                            }
                        }
                    }
                    Entry::Occupied(mut entry) => match options.duplicates {
                        Duplicates::First => DirectInsertOutcome::Done,
                        Duplicates::Last => {
                            match build_direct_value(
                                raw_value,
                                part,
                                effective_charset,
                                options,
                                0,
                            )? {
                                DirectBuiltValue::Concrete(value) => {
                                    *entry.get_mut() = value;
                                    DirectInsertOutcome::Done
                                }
                                DirectBuiltValue::Promote(value) => {
                                    DirectInsertOutcome::PromoteInsert {
                                        key: entry.key().clone(),
                                        value,
                                        via_duplicates: true,
                                    }
                                }
                            }
                        }
                        Duplicates::Combine => {
                            let current_length = value_list_length_for_combine(entry.get());
                            match build_direct_value(
                                raw_value,
                                part,
                                effective_charset,
                                options,
                                current_length,
                            )? {
                                DirectBuiltValue::Concrete(value) => {
                                    match try_combine_direct_values(entry.get(), &value, options)? {
                                        Some(combined) => {
                                            *entry.get_mut() = combined;
                                            DirectInsertOutcome::Done
                                        }
                                        None => DirectInsertOutcome::PromoteInsert {
                                            key: entry.key().clone(),
                                            value: ParsedFlatValue::concrete(value),
                                            via_duplicates: true,
                                        },
                                    }
                                }
                                DirectBuiltValue::Promote(value) => {
                                    DirectInsertOutcome::PromoteInsert {
                                        key: entry.key().clone(),
                                        value,
                                        via_duplicates: true,
                                    }
                                }
                            }
                        }
                    },
                }
            };

            match action {
                DirectInsertOutcome::Done => Ok(()),
                DirectInsertOutcome::PromoteInsert {
                    key,
                    value,
                    via_duplicates,
                } => {
                    let entries = values.ensure_parsed();
                    if via_duplicates {
                        insert_value(entries.entry(key), value, options)
                    } else {
                        entries.insert(key, value);
                        Ok(())
                    }
                }
            }
        }
        DefaultAccumulator::Parsed(entries) => {
            let current_length = if matches!(options.duplicates, Duplicates::Combine) {
                entries
                    .get(&decoded_key)
                    .map_or(0, ParsedFlatValue::list_length_for_combine)
            } else {
                0
            };
            let value = build_default_value(
                raw_value,
                part,
                effective_charset,
                options,
                current_length,
                DefaultStorageMode::PreferConcrete,
            )?;
            insert_value(entries.entry(decoded_key), value, options)
        }
    }
}

pub(in crate::decode) fn process_plain_part_default(
    part: &str,
    split_pos: Option<usize>,
    options: &DecodeOptions,
    values: &mut DefaultAccumulator,
    token_count: &mut usize,
) -> Result<(), DecodeError> {
    if !advance_token_count(token_count, options)? {
        return Ok(());
    }

    let (raw_key, raw_value) = match split_pos {
        Some(pos) => (&part[..pos], Some(&part[pos + 1..])),
        None => (part, None),
    };

    if options.charset_sentinel && raw_key.eq_ignore_ascii_case("utf8") {
        return Ok(());
    }

    if raw_key.is_empty() {
        return Ok(());
    }

    let decoded_key = raw_key.to_owned();

    match values {
        DefaultAccumulator::Direct(_) => {
            let action = {
                let DefaultAccumulator::Direct(entries) = values else {
                    unreachable!("direct accumulator should still be direct")
                };

                match entries.entry(decoded_key) {
                    Entry::Vacant(entry) => {
                        entry.insert(build_plain_value(raw_value, options));
                        DirectInsertOutcome::Done
                    }
                    Entry::Occupied(mut entry) => match options.duplicates {
                        Duplicates::First => DirectInsertOutcome::Done,
                        Duplicates::Last => {
                            *entry.get_mut() = build_plain_value(raw_value, options);
                            DirectInsertOutcome::Done
                        }
                        Duplicates::Combine => {
                            let value = build_plain_value(raw_value, options);
                            match try_combine_direct_values(entry.get(), &value, options)? {
                                Some(combined) => {
                                    *entry.get_mut() = combined;
                                    DirectInsertOutcome::Done
                                }
                                None => DirectInsertOutcome::PromoteInsert {
                                    key: entry.key().clone(),
                                    value: ParsedFlatValue::concrete(value),
                                    via_duplicates: true,
                                },
                            }
                        }
                    },
                }
            };

            match action {
                DirectInsertOutcome::Done => Ok(()),
                DirectInsertOutcome::PromoteInsert {
                    key,
                    value,
                    via_duplicates,
                } => {
                    let entries = values.ensure_parsed();
                    if via_duplicates {
                        insert_value(entries.entry(key), value, options)
                    } else {
                        entries.insert(key, value);
                        Ok(())
                    }
                }
            }
        }
        DefaultAccumulator::Parsed(entries) => insert_value(
            entries.entry(decoded_key),
            ParsedFlatValue::concrete(build_plain_value(raw_value, options)),
            options,
        ),
    }
}

fn process_scanned_part_default_with_mode(
    part: ScannedPart<'_>,
    effective_charset: Charset,
    options: &DecodeOptions,
    values: &mut FlatValues,
    token_count: &mut usize,
    has_any_structured_syntax: &mut bool,
    mode: DefaultStorageMode,
) -> Result<(), DecodeError> {
    if !advance_token_count(token_count, options)?
        || (options.charset_sentinel && part.is_charset_sentinel)
    {
        return Ok(());
    }

    let (raw_key, raw_value) = part.raw_parts();
    let decoded_key = if part.key_has_escape_or_plus {
        decode_scalar_with_known_flags(raw_key, effective_charset, true)
    } else {
        raw_key.to_owned()
    };
    if decoded_key.is_empty() {
        return Ok(());
    }
    update_structured_syntax_flag(part, &decoded_key, options, has_any_structured_syntax);

    if matches!(mode, DefaultStorageMode::PreferConcrete)
        && matches!(values, FlatValues::Concrete(_))
    {
        let (key, value, via_duplicates) = {
            let FlatValues::Concrete(entries) = values else {
                unreachable!("prefer-concrete fast path should start on concrete storage")
            };

            match entries.entry(decoded_key) {
                Entry::Vacant(entry) => {
                    let value =
                        build_default_value(raw_value, part, effective_charset, options, 0, mode)?;
                    match value {
                        ParsedFlatValue::Concrete(value) => {
                            entry.insert(value);
                            return Ok(());
                        }
                        parsed => (entry.key().clone(), parsed, false),
                    }
                }
                Entry::Occupied(mut entry) => match options.duplicates {
                    Duplicates::First => return Ok(()),
                    Duplicates::Last => {
                        let value = build_default_value(
                            raw_value,
                            part,
                            effective_charset,
                            options,
                            0,
                            mode,
                        )?;
                        match value {
                            ParsedFlatValue::Concrete(value) => {
                                *entry.get_mut() = value;
                                return Ok(());
                            }
                            parsed => (entry.key().clone(), parsed, true),
                        }
                    }
                    Duplicates::Combine => {
                        let current_length = value_list_length_for_combine(entry.get());
                        let value = build_default_value(
                            raw_value,
                            part,
                            effective_charset,
                            options,
                            current_length,
                            mode,
                        )?;
                        (entry.key().clone(), value, true)
                    }
                },
            }
        };

        let entries = values.ensure_parsed();
        if via_duplicates {
            insert_value(entries.entry(key), value, options)?;
        } else {
            entries.insert(key, value);
        }
        return Ok(());
    }

    let current_length = if matches!(options.duplicates, Duplicates::Combine) {
        values.get_list_length_for_combine(&decoded_key)
    } else {
        0
    };
    let value = build_default_value(
        raw_value,
        part,
        effective_charset,
        options,
        current_length,
        mode,
    )?;
    insert_default_value(values, decoded_key, value, options)?;

    Ok(())
}

pub(in crate::decode) fn process_scanned_part_custom(
    part: ScannedPart<'_>,
    effective_charset: Charset,
    options: &DecodeOptions,
    values: &mut FlatValues,
    token_count: &mut usize,
    has_any_structured_syntax: &mut bool,
) -> Result<(), DecodeError> {
    if !advance_token_count(token_count, options)?
        || (options.charset_sentinel && part.is_charset_sentinel)
    {
        return Ok(());
    }

    let (raw_key, raw_value) = part.raw_parts();
    let decoded_key = decode_component(raw_key, effective_charset, DecodeKind::Key, options);
    if decoded_key.is_empty() {
        return Ok(());
    }
    update_structured_syntax_flag(part, &decoded_key, options, has_any_structured_syntax);

    match values.ensure_parsed().entry(decoded_key) {
        Entry::Occupied(mut entry) => {
            if matches!(options.duplicates, Duplicates::First) {
                return Ok(());
            }

            let current_length = if matches!(options.duplicates, Duplicates::Combine) {
                entry.get().list_length_for_combine()
            } else {
                0
            };
            let value =
                build_custom_value(raw_value, part, effective_charset, options, current_length)?;
            insert_occupied_value(&mut entry, value, options)?;
        }
        Entry::Vacant(entry) => {
            let value = build_custom_value(raw_value, part, effective_charset, options, 0)?;
            entry.insert(value);
        }
    }

    Ok(())
}

fn advance_token_count(
    token_count: &mut usize,
    options: &DecodeOptions,
) -> Result<bool, DecodeError> {
    *token_count += 1;
    if options.throw_on_limit_exceeded && *token_count > options.parameter_limit {
        return Err(DecodeError::ParameterLimitExceeded {
            limit: options.parameter_limit,
        });
    }
    Ok(*token_count <= options.parameter_limit)
}

fn update_structured_syntax_flag(
    part: ScannedPart<'_>,
    decoded_key: &str,
    options: &DecodeOptions,
    has_any_structured_syntax: &mut bool,
) {
    if *has_any_structured_syntax {
        return;
    }

    if part.key_has_open_bracket || (options.allow_dots && part.key_has_dot) {
        *has_any_structured_syntax = true;
        return;
    }

    if part.key_has_percent {
        *has_any_structured_syntax = key_might_be_structured(decoded_key, options);
    }
}

#[cfg(test)]
mod tests {
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
        let options =
            DecodeOptions::new().with_decoder(Some(DecodeDecoder::new(|input, _charset, kind| {
                match kind {
                    DecodeKind::Key => input.replace("%5B", "[").replace("%5D", "]"),
                    DecodeKind::Value => input.to_ascii_uppercase(),
                }
            })));
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
        assert!(prefer_concrete.stores_concrete_value("plain"));
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
        assert!(promote_from_prefer_concrete.stores_parsed_value("a[]"));
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
        assert!(force_parsed.stores_parsed_value("a.b"));
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
        assert!(limited_values.stores_parsed_value("a"));
        assert!(!limited_values.stores_parsed_value("b"));

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
}
