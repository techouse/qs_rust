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
