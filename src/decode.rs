//! Public query-string decoding entrypoints.

mod accumulate;
mod flat;
mod keys;
mod scalar;
mod scan;
mod structured;

use crate::error::DecodeError;
use crate::options::DecodeOptions;
use crate::structured_scan::scan_structured_keys;
use crate::value::{Object, Value};

use self::flat::{collect_pair_values, finalize_flat};
pub(crate) use self::keys::split_key_into_segments;
use self::scan::parse_query_string_values;
use self::structured::decode_from_pairs_map;

#[cfg(test)]
use self::accumulate::combine_with_limit;
#[cfg(test)]
use self::flat::{FlatValues, ParsedFlatValue};
#[cfg(test)]
use self::keys::{dot_to_bracket_top_level, find_recoverable_balanced_open};
#[cfg(test)]
use self::scalar::{decode_scalar, interpret_numeric_entities};
#[cfg(test)]
use self::scan::ScannedPart;

/// Decodes a query string into an ordered object map.
///
/// The result preserves flat outputs when the input contains no structured key
/// syntax, and falls back to the structured merge pipeline when bracket or dot
/// notation is present.
///
/// # Errors
///
/// Returns [`DecodeError`] when the supplied [`DecodeOptions`] are invalid or
/// when the input exceeds a configured limit that is enforced strictly.
///
/// # Examples
///
/// ```
/// use qs_rust::{DecodeOptions, Value, decode};
///
/// let value = decode("a=1&b=2", &DecodeOptions::new()).unwrap();
/// assert_eq!(value["a"], Value::String("1".to_owned()));
/// assert_eq!(value["b"], Value::String("2".to_owned()));
/// ```
pub fn decode(input: &str, options: &DecodeOptions) -> Result<Object, DecodeError> {
    options.validate()?;

    if input.is_empty() {
        return Ok(Object::new());
    }

    let parsed = parse_query_string_values(input, options)?;
    if parsed.values.is_empty() {
        return Ok(Object::new());
    }

    if !parsed.has_any_structured_syntax {
        return finalize_flat(parsed.values, options);
    }

    let structured_scan = scan_structured_keys(parsed.values.key_refs(), options)?;
    if !structured_scan.has_any_structured_syntax {
        return finalize_flat(parsed.values, options);
    }

    decode_from_pairs_map(parsed.values, options, &structured_scan)
}

/// Decodes an iterator of already-separated key/value pairs into an ordered
/// object map.
///
/// This entrypoint skips raw query-string scanning but otherwise applies the
/// same flat-finalization and structured-merge rules as [`decode`].
///
/// # Errors
///
/// Returns [`DecodeError`] when the supplied [`DecodeOptions`] are invalid or
/// when a configured list or depth limit is exceeded during reconstruction.
pub fn decode_pairs<I>(pairs: I, options: &DecodeOptions) -> Result<Object, DecodeError>
where
    I: IntoIterator<Item = (String, Value)>,
{
    options.validate()?;

    let parsed = collect_pair_values(pairs, options)?;
    if parsed.values.is_empty() {
        return Ok(Object::new());
    }

    if !parsed.has_any_structured_syntax {
        return finalize_flat(parsed.values, options);
    }

    let structured_scan = scan_structured_keys(parsed.values.key_refs(), options)?;
    if !structured_scan.has_any_structured_syntax {
        return finalize_flat(parsed.values, options);
    }

    decode_from_pairs_map(parsed.values, options, &structured_scan)
}

#[cfg(test)]
mod tests;
