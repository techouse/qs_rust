//! Low-level delimiter-driven query-part iteration.

use crate::error::DecodeError;
use crate::options::{Charset, DecodeOptions};

use super::super::accumulate::{
    process_plain_part_default, process_scanned_part_default_accumulator,
};
use super::super::flat::DefaultAccumulator;
use super::metadata::ScannedPart;

pub(in crate::decode) fn scan_string_parts<F>(
    input: &str,
    delimiter: &str,
    mut visit: F,
) -> Result<(), DecodeError>
where
    F: FnMut(ScannedPart<'_>) -> Result<(), DecodeError>,
{
    if delimiter.is_empty() {
        return Err(DecodeError::EmptyDelimiter);
    }

    if delimiter.len() == 1 {
        return scan_parts_by_byte_delimiter(input, delimiter.as_bytes()[0], visit);
    }

    let mut start = 0usize;
    while start <= input.len() {
        let next = input[start..].find(delimiter);
        let end = next.map_or(input.len(), |index| start + index);
        visit_scanned_part(&input[start..end], &mut visit)?;

        let Some(_) = next else {
            break;
        };
        start = end + delimiter.len();
    }

    Ok(())
}

fn scan_parts_by_byte_delimiter<F>(
    input: &str,
    delimiter: u8,
    mut visit: F,
) -> Result<(), DecodeError>
where
    F: FnMut(ScannedPart<'_>) -> Result<(), DecodeError>,
{
    let bytes = input.as_bytes();
    let mut start = 0usize;

    for (index, byte) in bytes.iter().enumerate() {
        if *byte != delimiter {
            continue;
        }
        visit_scanned_part(&input[start..index], &mut visit)?;
        start = index + 1;
    }

    if start <= input.len() {
        visit_scanned_part(&input[start..], &mut visit)?;
    }

    Ok(())
}

pub(in crate::decode) fn scan_default_parts_by_byte_delimiter(
    input: &str,
    delimiter: u8,
    effective_charset: Charset,
    options: &DecodeOptions,
    values: &mut DefaultAccumulator,
    token_count: &mut usize,
    has_any_structured_syntax: &mut bool,
) -> Result<(), DecodeError> {
    let bytes = input.as_bytes();
    let mut start = 0usize;
    let mut split_pos = None;
    let mut scanning_value = false;
    let mut plain_candidate = true;

    for (index, byte) in bytes.iter().enumerate() {
        if *byte == delimiter {
            if index > start {
                let part = &input[start..index];
                if plain_candidate {
                    process_plain_part_default(part, split_pos, options, values, token_count)?;
                } else {
                    process_scanned_part_default_accumulator(
                        ScannedPart::new(part),
                        effective_charset,
                        options,
                        values,
                        token_count,
                        has_any_structured_syntax,
                    )?;
                }
            }
            start = index + 1;
            split_pos = None;
            scanning_value = false;
            plain_candidate = true;
            continue;
        }

        if !plain_candidate {
            continue;
        }

        if scanning_value {
            if matches!(*byte, b'%' | b'+' | b',' | b']' | b'=')
                || (options.interpret_numeric_entities
                    && matches!(effective_charset, Charset::Iso88591)
                    && matches!(*byte, b'#'))
            {
                plain_candidate = false;
            }
        } else {
            match *byte {
                b'=' => {
                    split_pos = Some(index - start);
                    scanning_value = true;
                }
                b'%' | b'+' | b'[' | b']' | b'.' => plain_candidate = false,
                _ => {}
            }
        }
    }

    if start < input.len() {
        let part = &input[start..];
        if plain_candidate {
            process_plain_part_default(part, split_pos, options, values, token_count)?;
        } else {
            process_scanned_part_default_accumulator(
                ScannedPart::new(part),
                effective_charset,
                options,
                values,
                token_count,
                has_any_structured_syntax,
            )?;
        }
    }

    Ok(())
}

pub(super) fn estimate_part_capacity(input: &str, delimiter: u8, parameter_limit: usize) -> usize {
    input
        .bytes()
        .filter(|byte| *byte == delimiter)
        .count()
        .saturating_add(1)
        .min(parameter_limit)
}

fn visit_scanned_part<F>(part: &str, visit: &mut F) -> Result<(), DecodeError>
where
    F: FnMut(ScannedPart<'_>) -> Result<(), DecodeError>,
{
    if part.is_empty() {
        return Ok(());
    }

    visit(ScannedPart::new(part))
}
