//! Pre-scan for structured decode keys.

use std::collections::HashSet;

use crate::decode::split_key_into_segments;
use crate::error::DecodeError;
use crate::options::DecodeOptions;

/// Summary of which flat keys require the structured decode path.
#[derive(Clone, Debug, Default)]
pub(crate) struct StructuredKeyScan {
    /// Whether any scanned key used bracket or dot structure.
    pub(crate) has_any_structured_syntax: bool,
    structured_roots: HashSet<String>,
    structured_keys: HashSet<String>,
}

impl StructuredKeyScan {
    pub(crate) fn contains_structured_key(&self, key: &str) -> bool {
        self.structured_keys.contains(key)
    }

    pub(crate) fn contains_structured_root(&self, key: &str) -> bool {
        self.structured_roots.contains(key)
    }
}

/// Scans flat keys to determine whether the structured merge pipeline is
/// required and which roots it needs to consider.
pub(crate) fn scan_structured_keys<'a, I>(
    keys: I,
    options: &DecodeOptions,
) -> Result<StructuredKeyScan, DecodeError>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut structured_roots = HashSet::new();
    let mut structured_keys = HashSet::new();

    for key in keys {
        let Some(split_at) = first_structured_split_index(key, options.allow_dots) else {
            continue;
        };

        structured_keys.insert(key.to_owned());
        if split_at == 0 {
            structured_roots.insert(leading_structured_root(key, options)?);
        } else {
            structured_roots.insert(key[..split_at].to_owned());
        }
    }

    if structured_keys.is_empty() {
        return Ok(StructuredKeyScan::default());
    }

    Ok(StructuredKeyScan {
        has_any_structured_syntax: true,
        structured_roots,
        structured_keys,
    })
}

pub(crate) fn first_structured_split_index(key: &str, allow_dots: bool) -> Option<usize> {
    let mut split_at = key.find('[');

    if allow_dots {
        if let Some(dot_index) = key.find('.') {
            split_at = match split_at {
                Some(index) => Some(index.min(dot_index)),
                None => Some(dot_index),
            };
        }

        if key.contains('%') {
            let upper = key.find("%2E");
            let lower = key.find("%2e");
            let encoded_dot = match (upper, lower) {
                (Some(left), Some(right)) => Some(left.min(right)),
                (Some(index), None) | (None, Some(index)) => Some(index),
                (None, None) => None,
            };

            if let Some(encoded_dot_index) = encoded_dot {
                split_at = match split_at {
                    Some(index) => Some(index.min(encoded_dot_index)),
                    None => Some(encoded_dot_index),
                };
            }
        }
    }

    split_at
}

pub(crate) fn leading_structured_root(
    key: &str,
    options: &DecodeOptions,
) -> Result<String, DecodeError> {
    let segments =
        split_key_into_segments(key, options.allow_dots, options.depth, options.strict_depth)?;
    if segments.is_empty() {
        return Ok(key.to_owned());
    }

    let first = &segments[0];
    if !first.starts_with('[') {
        return Ok(first.clone());
    }

    let clean_root = if first.starts_with('[') && first.ends_with(']') && first.len() >= 2 {
        &first[1..first.len() - 1]
    } else {
        &first[1..]
    };
    if clean_root.is_empty() {
        return Ok("0".to_owned());
    }

    Ok(clean_root.to_owned())
}

#[cfg(test)]
mod tests;
