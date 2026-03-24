//! Scan entrypoints that choose the appropriate raw parsing strategy.

use crate::error::DecodeError;
use crate::options::{Charset, DecodeOptions, Delimiter};

use super::super::accumulate::{
    process_query_part_custom, process_query_part_default, process_scanned_part_custom,
    process_scanned_part_default_accumulator,
};
use super::super::flat::{DefaultAccumulator, FlatValues, ParsedInput};
use super::metadata::{ScannedPart, charset_sentinel};
use super::parts::{
    estimate_part_capacity, scan_default_parts_by_byte_delimiter, scan_string_parts,
};

pub(in crate::decode) fn parse_query_string_values(
    input: &str,
    options: &DecodeOptions,
) -> Result<ParsedInput, DecodeError> {
    let input = if options.ignore_query_prefix {
        input.strip_prefix('?').unwrap_or(input)
    } else {
        input
    };

    match options.delimiter() {
        Delimiter::String(text) => parse_string_query_string_values(input, text, options),
        Delimiter::Regex(regex) => parse_regex_query_string_values(input, regex, options),
    }
}

fn parse_string_query_string_values(
    input: &str,
    delimiter: &str,
    options: &DecodeOptions,
) -> Result<ParsedInput, DecodeError> {
    if options.decoder().is_none() {
        let effective_charset = detect_charset_sentinel_in_input(input, options)?;
        let mut values = if delimiter.len() == 1 {
            DefaultAccumulator::direct_with_capacity(estimate_part_capacity(
                input,
                delimiter.as_bytes()[0],
                options.parameter_limit,
            ))
        } else {
            DefaultAccumulator::direct()
        };
        let mut token_count = 0usize;
        let mut has_any_structured_syntax = false;

        if delimiter.len() == 1 {
            scan_default_parts_by_byte_delimiter(
                input,
                delimiter.as_bytes()[0],
                effective_charset,
                options,
                &mut values,
                &mut token_count,
                &mut has_any_structured_syntax,
            )?;
        } else {
            scan_string_parts(input, delimiter, |part| {
                process_scanned_part_default_accumulator(
                    part,
                    effective_charset,
                    options,
                    &mut values,
                    &mut token_count,
                    &mut has_any_structured_syntax,
                )
            })?;
        }

        return Ok(ParsedInput {
            values: values.into_flat_values(),
            has_any_structured_syntax,
        });
    }

    let effective_charset = detect_charset_sentinel_in_input(input, options)?;
    let mut values = FlatValues::parsed();
    let mut token_count = 0usize;
    let mut has_any_structured_syntax = false;
    scan_string_parts(input, delimiter, |part| {
        process_scanned_part_custom(
            part,
            effective_charset,
            options,
            &mut values,
            &mut token_count,
            &mut has_any_structured_syntax,
        )
    })?;

    Ok(ParsedInput {
        values,
        has_any_structured_syntax,
    })
}

fn parse_regex_query_string_values(
    input: &str,
    regex: &regex::Regex,
    options: &DecodeOptions,
) -> Result<ParsedInput, DecodeError> {
    let effective_charset = detect_charset_sentinel_in_input(input, options)?;
    let mut values = FlatValues::parsed();
    let mut token_count = 0usize;
    let mut has_any_structured_syntax = false;

    if options.decoder().is_none() {
        for part in regex.split(input).filter(|part| !part.is_empty()) {
            process_query_part_default(
                part,
                effective_charset,
                options,
                &mut values,
                &mut token_count,
                &mut has_any_structured_syntax,
            )?;
        }
    } else {
        for part in regex.split(input).filter(|part| !part.is_empty()) {
            process_query_part_custom(
                part,
                effective_charset,
                options,
                &mut values,
                &mut token_count,
                &mut has_any_structured_syntax,
            )?;
        }
    }

    Ok(ParsedInput {
        values,
        has_any_structured_syntax,
    })
}

fn detect_charset_sentinel_in_input(
    input: &str,
    options: &DecodeOptions,
) -> Result<Charset, DecodeError> {
    if !options.charset_sentinel {
        return Ok(options.charset);
    }

    match options.delimiter() {
        Delimiter::String(text) => {
            Ok(find_charset_sentinel_in_string_parts(input, text)?.unwrap_or(options.charset))
        }
        Delimiter::Regex(regex) => Ok(regex
            .split(input)
            .filter(|part| !part.is_empty())
            .find_map(charset_sentinel)
            .unwrap_or(options.charset)),
    }
}

fn find_charset_sentinel_in_string_parts(
    input: &str,
    delimiter: &str,
) -> Result<Option<Charset>, DecodeError> {
    if delimiter.is_empty() {
        return Err(DecodeError::EmptyDelimiter);
    }

    if delimiter.len() == 1 {
        let delimiter = delimiter.as_bytes()[0];
        let mut start = 0usize;
        let bytes = input.as_bytes();

        for (index, byte) in bytes.iter().enumerate() {
            if *byte != delimiter {
                continue;
            }

            if index > start {
                let sentinel = ScannedPart::new(&input[start..index]).sentinel_charset;
                if sentinel.is_some() {
                    return Ok(sentinel);
                }
            }
            start = index + 1;
        }

        if start < input.len() {
            return Ok(ScannedPart::new(&input[start..]).sentinel_charset);
        }

        return Ok(None);
    }

    let mut start = 0usize;
    while start <= input.len() {
        let next = input[start..].find(delimiter);
        let end = next.map_or(input.len(), |index| start + index);

        if end > start {
            let sentinel = ScannedPart::new(&input[start..end]).sentinel_charset;
            if sentinel.is_some() {
                return Ok(sentinel);
            }
        }

        let Some(_) = next else {
            break;
        };
        start = end + delimiter.len();
    }

    Ok(None)
}
