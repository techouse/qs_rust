//! Metadata extraction for raw query parts.

use crate::options::Charset;

fn scan_part_metadata(text: &str) -> PartMetadata {
    let bytes = text.as_bytes();
    let mut plain_split = None;
    let mut structured_split = None;
    let mut has_bracket_suffix_assignment = false;
    let mut key_has_escape_or_plus = false;
    let mut key_has_percent = false;
    let mut key_has_open_bracket = false;
    let mut key_has_dot = false;
    let mut value_has_escape_or_plus = false;
    let mut value_has_numeric_entity_candidate = false;
    let mut value_comma_count = 0usize;
    let mut scanning_value = false;
    let mut needs_repartition = false;
    let mut index = 0usize;

    while index < bytes.len() {
        let byte = bytes[index];
        if scanning_value {
            match byte {
                b',' => value_comma_count += 1,
                b'+' | b'%' => value_has_escape_or_plus = true,
                _ => {}
            }
            if !value_has_numeric_entity_candidate
                && byte_starts_numeric_entity_candidate(bytes, index)
            {
                value_has_numeric_entity_candidate = true;
            }
        } else {
            match byte {
                b'+' => key_has_escape_or_plus = true,
                b'%' => {
                    key_has_escape_or_plus = true;
                    key_has_percent = true;
                }
                b'[' => key_has_open_bracket = true,
                b'.' => key_has_dot = true,
                b'=' => {
                    plain_split = Some(index);
                    scanning_value = true;
                }
                _ => {}
            }
        }

        if structured_split.is_none() {
            if byte == b']' && index + 1 < bytes.len() && bytes[index + 1] == b'=' {
                structured_split = Some(index + 1);
                needs_repartition |= plain_split != structured_split;
            } else if index + 4 <= bytes.len()
                && ascii_case_insensitive_eq_bytes(&bytes[index..index + 3], b"%5D")
                && bytes[index + 3] == b'='
            {
                structured_split = Some(index + 3);
                needs_repartition |= plain_split != structured_split;
            }
        }

        if !has_bracket_suffix_assignment
            && ((index + 3 <= bytes.len() && &bytes[index..index + 3] == b"[]=")
                || (index + 7 <= bytes.len()
                    && ascii_case_insensitive_eq_bytes(&bytes[index..index + 7], b"%5B%5D=")))
        {
            has_bracket_suffix_assignment = true;
        }

        index += 1;
    }

    let split_pos = structured_split.or(plain_split);
    let key_end = split_pos.unwrap_or(bytes.len());
    if split_pos.is_none() {
        value_has_escape_or_plus = false;
        value_has_numeric_entity_candidate = false;
        value_comma_count = 0;
    } else if needs_repartition {
        let key_bytes = &bytes[..key_end];
        key_has_escape_or_plus = false;
        key_has_percent = false;
        key_has_open_bracket = false;
        key_has_dot = false;
        for byte in key_bytes {
            match *byte {
                b'+' => key_has_escape_or_plus = true,
                b'%' => {
                    key_has_escape_or_plus = true;
                    key_has_percent = true;
                }
                b'[' => key_has_open_bracket = true,
                b'.' => key_has_dot = true,
                _ => {}
            }
        }

        value_has_escape_or_plus = false;
        value_has_numeric_entity_candidate = false;
        value_comma_count = 0;
        for (offset, byte) in bytes[key_end + 1..].iter().enumerate() {
            match *byte {
                b',' => value_comma_count += 1,
                b'+' | b'%' => value_has_escape_or_plus = true,
                _ => {}
            }
            if !value_has_numeric_entity_candidate
                && byte_starts_numeric_entity_candidate(bytes, key_end + 1 + offset)
            {
                value_has_numeric_entity_candidate = true;
            }
        }
    }
    let is_charset_sentinel = ascii_case_insensitive_eq_bytes(&bytes[..key_end], b"utf8");
    let sentinel_charset = if is_charset_sentinel {
        charset_sentinel(text)
    } else {
        None
    };

    PartMetadata {
        split_pos,
        has_bracket_suffix_assignment,
        is_charset_sentinel,
        sentinel_charset,
        key_has_escape_or_plus,
        key_has_percent,
        key_has_open_bracket,
        key_has_dot,
        value_has_escape_or_plus,
        value_has_numeric_entity_candidate,
        value_comma_count,
    }
}

pub(in crate::decode) fn byte_starts_numeric_entity_candidate(bytes: &[u8], index: usize) -> bool {
    match bytes[index] {
        b'&' => index + 1 < bytes.len() && bytes[index + 1] == b'#',
        b'#' => true,
        b'%' if index + 2 < bytes.len() => {
            matches!(
                decode_percent_triplet(bytes[index], bytes[index + 1], bytes[index + 2]),
                Some(b'&' | b'#')
            )
        }
        _ => false,
    }
}

fn decode_percent_triplet(prefix: u8, hi: u8, lo: u8) -> Option<u8> {
    if prefix != b'%' {
        return None;
    }

    Some((hex_value(hi)? << 4) | hex_value(lo)?)
}

pub(super) fn charset_sentinel(part: &str) -> Option<Charset> {
    if ascii_case_insensitive_eq(part, Charset::UTF8_SENTINEL) {
        return Some(Charset::Utf8);
    }
    if ascii_case_insensitive_eq(part, Charset::ISO_SENTINEL) {
        return Some(Charset::Iso88591);
    }
    None
}

fn ascii_case_insensitive_eq(left: &str, right: &str) -> bool {
    left.len() == right.len()
        && left
            .bytes()
            .zip(right.bytes())
            .all(|(l, r)| l.eq_ignore_ascii_case(&r))
}

pub(in crate::decode) fn ascii_case_insensitive_eq_bytes(left: &[u8], right: &[u8]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right.iter())
            .all(|(l, r)| l.eq_ignore_ascii_case(r))
}

pub(in crate::decode) fn contains_ascii_case_insensitive_bytes(
    haystack: &[u8],
    needle: &[u8],
) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| ascii_case_insensitive_eq_bytes(window, needle))
}

pub(in crate::decode) fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug)]
struct PartMetadata {
    split_pos: Option<usize>,
    has_bracket_suffix_assignment: bool,
    is_charset_sentinel: bool,
    sentinel_charset: Option<Charset>,
    key_has_escape_or_plus: bool,
    key_has_percent: bool,
    key_has_open_bracket: bool,
    key_has_dot: bool,
    value_has_escape_or_plus: bool,
    value_has_numeric_entity_candidate: bool,
    value_comma_count: usize,
}

/// A scanned raw query part plus the metadata needed by the decode pipeline.
#[derive(Clone, Copy, Debug)]
pub(in crate::decode) struct ScannedPart<'a> {
    pub(in crate::decode) text: &'a str,
    pub(in crate::decode) split_pos: Option<usize>,
    pub(in crate::decode) has_bracket_suffix_assignment: bool,
    pub(in crate::decode) is_charset_sentinel: bool,
    pub(in crate::decode) sentinel_charset: Option<Charset>,
    pub(in crate::decode) key_has_escape_or_plus: bool,
    pub(in crate::decode) key_has_percent: bool,
    pub(in crate::decode) key_has_open_bracket: bool,
    pub(in crate::decode) key_has_dot: bool,
    pub(in crate::decode) value_has_escape_or_plus: bool,
    pub(in crate::decode) value_has_numeric_entity_candidate: bool,
    pub(in crate::decode) value_comma_count: usize,
}

impl<'a> ScannedPart<'a> {
    pub(in crate::decode) fn new(text: &'a str) -> Self {
        let metadata = scan_part_metadata(text);
        Self::from_metadata(text, metadata)
    }

    fn from_metadata(text: &'a str, metadata: PartMetadata) -> Self {
        Self {
            text,
            split_pos: metadata.split_pos,
            has_bracket_suffix_assignment: metadata.has_bracket_suffix_assignment,
            is_charset_sentinel: metadata.is_charset_sentinel,
            sentinel_charset: metadata.sentinel_charset,
            key_has_escape_or_plus: metadata.key_has_escape_or_plus,
            key_has_percent: metadata.key_has_percent,
            key_has_open_bracket: metadata.key_has_open_bracket,
            key_has_dot: metadata.key_has_dot,
            value_has_escape_or_plus: metadata.value_has_escape_or_plus,
            value_has_numeric_entity_candidate: metadata.value_has_numeric_entity_candidate,
            value_comma_count: metadata.value_comma_count,
        }
    }

    pub(in crate::decode) fn raw_parts(self) -> (&'a str, Option<&'a str>) {
        match self.split_pos {
            Some(pos) => (&self.text[..pos], Some(&self.text[pos + 1..])),
            None => (self.text, None),
        }
    }
}
