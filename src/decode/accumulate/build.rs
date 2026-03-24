//! Value-construction helpers for flat decode accumulation.

use crate::error::DecodeError;
use crate::internal::node::Node;
use crate::internal::overflow::overflow_from_items;
use crate::options::{Charset, DecodeKind, DecodeOptions};
use crate::value::Value;

use super::super::flat::{DefaultStorageMode, ParsedFlatValue};
use super::super::scalar::{
    decode_component, decode_scalar_with_known_flags, interpret_numeric_entities,
    interpret_numeric_entities_in_node,
};
use super::super::scan::{ScannedPart, byte_starts_numeric_entity_candidate};

/// The result of building a value for the default direct accumulator.
#[derive(Clone, Debug)]
pub(super) enum DirectBuiltValue {
    /// The value can stay in direct concrete storage.
    Concrete(Value),
    /// The accumulator must promote to parsed storage first.
    Promote(ParsedFlatValue),
}

impl DirectBuiltValue {
    fn into_parsed_flat_value(self) -> ParsedFlatValue {
        match self {
            Self::Concrete(value) => ParsedFlatValue::concrete(value),
            Self::Promote(value) => value,
        }
    }
}

fn parse_list_value(
    value: &str,
    options: &DecodeOptions,
    current_list_length: usize,
) -> Result<Node, DecodeError> {
    if options.comma && !value.is_empty() && value.contains(',') {
        let comma_count = value.bytes().filter(|byte| *byte == b',').count();
        let mut items = Vec::with_capacity(comma_count + 1);
        let mut start = 0usize;
        for (index, byte) in value.bytes().enumerate() {
            if byte != b',' {
                continue;
            }

            items.push(Node::scalar(Value::String(value[start..index].to_owned())));
            start = index + 1;
        }
        items.push(Node::scalar(Value::String(value[start..].to_owned())));

        let total_len = current_list_length.saturating_add(items.len());
        if options.throw_on_limit_exceeded && total_len > options.list_limit {
            return Err(DecodeError::ListLimitExceeded {
                limit: options.list_limit,
            });
        }

        if !options.throw_on_limit_exceeded
            && current_list_length == 0
            && items.len() > options.list_limit
        {
            return Ok(overflow_from_items(items));
        }

        return Ok(Node::Array(items));
    }

    if options.throw_on_limit_exceeded && current_list_length >= options.list_limit {
        return Err(DecodeError::ListLimitExceeded {
            limit: options.list_limit,
        });
    }

    Ok(Node::scalar(Value::String(value.to_owned())))
}

fn decode_value_text_default_scanned(
    value: &str,
    charset: Charset,
    options: &DecodeOptions,
    has_escape_or_plus: bool,
    has_numeric_entity_candidate: bool,
) -> String {
    let decoded = decode_scalar_with_known_flags(value, charset, has_escape_or_plus);
    if options.interpret_numeric_entities
        && matches!(charset, Charset::Iso88591)
        && has_numeric_entity_candidate
    {
        interpret_numeric_entities(&decoded)
    } else {
        decoded
    }
}

fn parse_list_value_default_scanned(
    value: &str,
    part: ScannedPart<'_>,
    charset: Charset,
    options: &DecodeOptions,
    current_list_length: usize,
) -> Result<ParsedFlatValue, DecodeError> {
    let needs_numeric_entities = options.interpret_numeric_entities
        && matches!(charset, Charset::Iso88591)
        && part.value_has_numeric_entity_candidate;
    let needs_component_decode = part.value_has_escape_or_plus || needs_numeric_entities;

    if options.comma && !value.is_empty() && part.value_comma_count > 0 {
        let mut items = Vec::with_capacity(part.value_comma_count + 1);
        let mut segment_has_escape_or_plus = false;
        let mut segment_has_numeric_entity_candidate = false;
        let mut start = 0usize;
        let bytes = value.as_bytes();

        for (index, byte) in bytes.iter().enumerate() {
            if *byte != b',' {
                segment_has_escape_or_plus |= matches!(*byte, b'+' | b'%');
                if !segment_has_numeric_entity_candidate
                    && byte_starts_numeric_entity_candidate(bytes, index)
                {
                    segment_has_numeric_entity_candidate = true;
                }
                continue;
            }

            items.push(Value::String(if needs_component_decode {
                decode_value_text_default_scanned(
                    value[start..index].as_ref(),
                    charset,
                    options,
                    segment_has_escape_or_plus,
                    segment_has_numeric_entity_candidate,
                )
            } else {
                value[start..index].to_owned()
            }));
            start = index + 1;
            segment_has_escape_or_plus = false;
            segment_has_numeric_entity_candidate = false;
        }

        items.push(Value::String(if needs_component_decode {
            decode_value_text_default_scanned(
                value[start..].as_ref(),
                charset,
                options,
                segment_has_escape_or_plus,
                segment_has_numeric_entity_candidate,
            )
        } else {
            value[start..].to_owned()
        }));

        let total_len = current_list_length.saturating_add(items.len());
        if options.throw_on_limit_exceeded && total_len > options.list_limit {
            return Err(DecodeError::ListLimitExceeded {
                limit: options.list_limit,
            });
        }

        if !options.throw_on_limit_exceeded
            && current_list_length == 0
            && items.len() > options.list_limit
        {
            return Ok(ParsedFlatValue::parsed(
                overflow_from_items(items.into_iter().map(Node::scalar).collect()),
                false,
            ));
        }

        return Ok(ParsedFlatValue::concrete(Value::Array(items)));
    }

    if options.throw_on_limit_exceeded && current_list_length >= options.list_limit {
        return Err(DecodeError::ListLimitExceeded {
            limit: options.list_limit,
        });
    }

    Ok(ParsedFlatValue::concrete(Value::String(
        if needs_component_decode {
            decode_value_text_default_scanned(
                value,
                charset,
                options,
                part.value_has_escape_or_plus,
                part.value_has_numeric_entity_candidate,
            )
        } else {
            value.to_owned()
        },
    )))
}

fn decode_value_node(node: Node, charset: Charset, options: &DecodeOptions) -> Node {
    match node {
        Node::Value(Value::String(text)) => Node::scalar(Value::String(decode_component(
            &text,
            charset,
            DecodeKind::Value,
            options,
        ))),
        Node::Array(items) => Node::Array(
            items
                .into_iter()
                .map(|item| decode_value_node(item, charset, options))
                .collect(),
        ),
        other => other,
    }
}

pub(super) fn build_default_value(
    raw_value: Option<&str>,
    part: ScannedPart<'_>,
    effective_charset: Charset,
    options: &DecodeOptions,
    current_list_length: usize,
    mode: DefaultStorageMode,
) -> Result<ParsedFlatValue, DecodeError> {
    let mut value = build_direct_value(
        raw_value,
        part,
        effective_charset,
        options,
        current_list_length,
    )?
    .into_parsed_flat_value();

    if matches!(mode, DefaultStorageMode::ForceParsed) {
        value = value.force_parsed();
    }

    Ok(value)
}

pub(super) fn build_plain_value(raw_value: Option<&str>, options: &DecodeOptions) -> Value {
    match raw_value {
        Some(raw_value) => Value::String(raw_value.to_owned()),
        None if options.strict_null_handling => Value::Null,
        None => Value::String(String::new()),
    }
}

pub(super) fn build_direct_value(
    raw_value: Option<&str>,
    part: ScannedPart<'_>,
    effective_charset: Charset,
    options: &DecodeOptions,
    current_list_length: usize,
) -> Result<DirectBuiltValue, DecodeError> {
    let mut value = match raw_value {
        None => {
            if options.strict_null_handling {
                DirectBuiltValue::Concrete(Value::Null)
            } else {
                DirectBuiltValue::Concrete(Value::String(String::new()))
            }
        }
        Some(raw_value_text) => match parse_list_value_default_scanned(
            raw_value_text,
            part,
            effective_charset,
            options,
            current_list_length,
        )? {
            ParsedFlatValue::Concrete(value) => DirectBuiltValue::Concrete(value),
            parsed => DirectBuiltValue::Promote(parsed),
        },
    };

    if part.has_bracket_suffix_assignment {
        value = match value {
            DirectBuiltValue::Concrete(Value::Array(items)) => {
                DirectBuiltValue::Concrete(Value::Array(vec![Value::Array(items)]))
            }
            DirectBuiltValue::Promote(ParsedFlatValue::Parsed {
                node: Node::Array(items),
                needs_compaction,
            }) => DirectBuiltValue::Promote(ParsedFlatValue::parsed(
                Node::Array(vec![Node::Array(items)]),
                needs_compaction,
            )),
            other => other,
        };
    }

    Ok(value)
}

pub(super) fn build_custom_value(
    raw_value: Option<&str>,
    part: ScannedPart<'_>,
    effective_charset: Charset,
    options: &DecodeOptions,
    current_list_length: usize,
) -> Result<ParsedFlatValue, DecodeError> {
    let mut value = match raw_value {
        None => {
            if options.strict_null_handling {
                Node::Value(Value::Null)
            } else {
                Node::scalar(Value::String(String::new()))
            }
        }
        Some(raw_value_text) => {
            let mut parsed_value = parse_list_value(raw_value_text, options, current_list_length)?;
            if parsed_value.is_undefined() {
                return Ok(ParsedFlatValue::parsed(parsed_value, true));
            }

            parsed_value = decode_value_node(parsed_value, effective_charset, options);
            if options.interpret_numeric_entities && matches!(effective_charset, Charset::Iso88591)
            {
                parsed_value = interpret_numeric_entities_in_node(parsed_value);
            }

            parsed_value
        }
    };

    if part.has_bracket_suffix_assignment && matches!(value, Node::Array(_)) {
        value = Node::Array(vec![value]);
    }

    Ok(ParsedFlatValue::parsed(value, true))
}
