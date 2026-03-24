//! Scalar decoding and charset-sensitive text helpers.

use crate::internal::node::Node;
use crate::options::{Charset, DecodeKind, DecodeOptions};
use crate::value::Value;

use super::scan::hex_value;

pub(super) fn decode_scalar(value: &str, charset: Charset) -> String {
    decode_scalar_with_known_flags(value, charset, value.contains('+') || value.contains('%'))
}

pub(super) fn decode_scalar_with_known_flags(
    value: &str,
    charset: Charset,
    has_escape_or_plus: bool,
) -> String {
    if !has_escape_or_plus {
        return value.to_owned();
    }

    match charset {
        Charset::Utf8 => decode_scalar_utf8(value),
        Charset::Iso88591 => decode_scalar_latin1(value),
    }
}

fn decode_scalar_utf8(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut changed = false;
    let mut index = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                output.push(b' ');
                changed = true;
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                let hi = hex_value(bytes[index + 1]);
                let lo = hex_value(bytes[index + 2]);
                if let (Some(hi), Some(lo)) = (hi, lo) {
                    output.push((hi << 4) | lo);
                    changed = true;
                    index += 3;
                    continue;
                }
                output.push(bytes[index]);
                index += 1;
            }
            byte => {
                output.push(byte);
                index += 1;
            }
        }
    }

    if !changed {
        return value.to_owned();
    }

    match String::from_utf8(output) {
        Ok(decoded) => decoded,
        Err(_) => replace_plus_with_space(value),
    }
}

fn decode_scalar_latin1(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut output = String::with_capacity(bytes.len());
    let mut changed = false;
    let mut index = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                output.push(' ');
                changed = true;
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                let hi = hex_value(bytes[index + 1]);
                let lo = hex_value(bytes[index + 2]);
                if let (Some(hi), Some(lo)) = (hi, lo) {
                    output.push(char::from((hi << 4) | lo));
                    changed = true;
                    index += 3;
                    continue;
                }
                output.push('%');
                index += 1;
            }
            byte => {
                output.push(char::from(byte));
                index += 1;
            }
        }
    }

    if changed { output } else { value.to_owned() }
}

fn replace_plus_with_space(value: &str) -> String {
    if value.contains('+') {
        value.replace('+', " ")
    } else {
        value.to_owned()
    }
}

pub(super) fn decode_component(
    value: &str,
    charset: Charset,
    kind: DecodeKind,
    options: &DecodeOptions,
) -> String {
    match options.decoder() {
        Some(decoder) => decoder.decode(value, charset, kind),
        None => decode_scalar(value, charset),
    }
}

pub(super) fn interpret_numeric_entities_in_node(node: Node) -> Node {
    match node {
        Node::Value(Value::String(text)) => {
            Node::scalar(Value::String(interpret_numeric_entities(&text)))
        }
        Node::Array(items) => Node::Array(
            items
                .into_iter()
                .map(interpret_numeric_entities_in_node)
                .collect(),
        ),
        other => other,
    }
}

pub(super) fn interpret_numeric_entities(input: &str) -> String {
    if !input.contains("&#") {
        return input.to_owned();
    }

    let chars: Vec<char> = input.chars().collect();
    let mut output = String::with_capacity(input.len());
    let mut index = 0usize;

    while index < chars.len() {
        if chars[index] == '&' && index + 2 < chars.len() && chars[index + 1] == '#' {
            let mut scan = index + 2;
            let mut hex = false;
            if scan < chars.len() && (chars[scan] == 'x' || chars[scan] == 'X') {
                hex = true;
                scan += 1;
            }
            let start_digits = scan;
            while scan < chars.len()
                && if hex {
                    chars[scan].is_ascii_hexdigit()
                } else {
                    chars[scan].is_ascii_digit()
                }
            {
                scan += 1;
            }
            if scan < chars.len() && chars[scan] == ';' && scan > start_digits {
                let digits: String = chars[start_digits..scan].iter().collect();
                let value = if hex {
                    u32::from_str_radix(&digits, 16).ok()
                } else {
                    digits.parse::<u32>().ok()
                };
                if let Some(codepoint) = value.and_then(char::from_u32) {
                    output.push(codepoint);
                    index = scan + 1;
                    continue;
                }
            }
        }

        output.push(chars[index]);
        index += 1;
    }

    output
}
