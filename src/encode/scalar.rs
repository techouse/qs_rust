//! Scalar formatting and percent-encoding helpers for encode.

use crate::options::{Charset, EncodeOptions, EncodeToken, Format};
use crate::temporal::TemporalValue;
use crate::value::Value;

pub(super) fn encode_scalar_leaf(
    node: &Value,
    key: &str,
    options: &EncodeOptions,
) -> Option<String> {
    if scalar_is_null_like(node, options) {
        if options.skip_nulls {
            return None;
        }
        if options.strict_null_handling {
            return Some(finalize_key_only_fragment(key, options));
        }
        let key = finalize_key_path(key, options);
        return Some(format!("{key}="));
    }

    let key = finalize_key_path(key, options);
    let value = encode_string_or_raw(
        &encoded_scalar_text(node, options).expect("non-null scalar leaf should resolve"),
        options,
    );
    Some(format!("{key}={value}"))
}

pub(super) fn finalize_key_path(key: &str, options: &EncodeOptions) -> String {
    let key = key_token_text(key, options);
    format_key_text(&key, options)
}

pub(super) fn finalize_key_only_fragment(key: &str, options: &EncodeOptions) -> String {
    encode_key_only_fragment(&key_token_text(key, options), options)
}

pub(super) fn filter_prefix(key: &str, options: &EncodeOptions) -> String {
    format_key_text(key, options)
}

pub(super) fn joined_text_value(text: &str, options: &EncodeOptions) -> String {
    if let Some(encoder) = options.encoder.as_ref() {
        return encoder.encode(
            EncodeToken::TextValue(text),
            options.charset,
            options.format,
        );
    }

    text.to_owned()
}

pub(super) fn encode_string_or_raw(text: &str, options: &EncodeOptions) -> String {
    if options.encode {
        encode_with_charset(text, options.charset, options.format)
    } else {
        text.to_owned()
    }
}

pub(super) fn encode_key_only_fragment(key: &str, options: &EncodeOptions) -> String {
    let formatted = format_key_text(key, options);
    if options.encode && matches!(options.format, Format::Rfc1738) {
        formatted.replace('+', "%20")
    } else {
        formatted
    }
}

pub(super) fn format_key_text(text: &str, options: &EncodeOptions) -> String {
    if options.encode && !options.encode_values_only {
        encode_with_charset(text, options.charset, options.format)
    } else {
        text.to_owned()
    }
}

pub(super) fn encode_with_charset(text: &str, charset: Charset, format: Format) -> String {
    match charset {
        Charset::Utf8 => percent_encode_bytes(text.as_bytes(), format),
        Charset::Iso88591 => percent_encode_latin1(text, format),
    }
}

pub(super) fn percent_encode_latin1(text: &str, format: Format) -> String {
    let mut output = String::new();
    for ch in text.chars() {
        let code = ch as u32;
        if let Ok(byte) = u8::try_from(code) {
            append_encoded_byte(byte, format, &mut output);
        } else {
            let numeric = format!("&#{code};");
            output.push_str(&percent_encode_bytes(numeric.as_bytes(), Format::Rfc3986));
        }
    }
    output
}

pub(super) fn percent_encode_bytes(bytes: &[u8], format: Format) -> String {
    let mut output = String::with_capacity(bytes.len() * 3);
    for &byte in bytes {
        append_encoded_byte(byte, format, &mut output);
    }
    output
}

fn append_encoded_byte(byte: u8, format: Format, output: &mut String) {
    let is_alphanumeric = byte.is_ascii_alphanumeric();
    let is_safe = matches!(byte, b'-' | b'.' | b'_' | b'~');
    if is_alphanumeric
        || is_safe
        || (matches!(format, Format::Rfc1738) && matches!(byte, b'(' | b')'))
    {
        output.push(byte as char);
        return;
    }

    if matches!(format, Format::Rfc1738) && byte == b' ' {
        output.push('+');
        return;
    }

    output.push('%');
    output.push(HEX[(byte >> 4) as usize] as char);
    output.push(HEX[(byte & 0x0F) as usize] as char);
}

fn decode_bytes(bytes: &[u8], charset: Charset) -> String {
    match charset {
        Charset::Utf8 => String::from_utf8_lossy(bytes).into_owned(),
        Charset::Iso88591 => bytes.iter().map(|byte| char::from(*byte)).collect(),
    }
}

pub(super) fn plain_string_for_comma(value: &Value, options: &EncodeOptions) -> String {
    match value {
        Value::Array(values) => values
            .iter()
            .map(|value| plain_string_for_comma(value, options))
            .collect::<Vec<_>>()
            .join(","),
        Value::Object(_) => "[object Object]".to_owned(),
        _ => plain_scalar_text(value, options).unwrap_or_default(),
    }
}

pub(super) fn scalar_is_null_like(value: &Value, options: &EncodeOptions) -> bool {
    match value {
        Value::Null => true,
        Value::Temporal(temporal) => plain_temporal_text(temporal, options).is_none(),
        _ => false,
    }
}

pub(super) fn encoded_scalar_text(value: &Value, options: &EncodeOptions) -> Option<String> {
    match value {
        Value::Null => None,
        Value::Temporal(temporal) => {
            let text = plain_temporal_text(temporal, options)?;
            Some(encoded_text_token(&text, options))
        }
        Value::Array(values) => values
            .iter()
            .map(|value| plain_string_for_comma(value, options))
            .collect::<Vec<_>>()
            .join(",")
            .into(),
        Value::Object(_) => Some("[object Object]".to_owned()),
        _ => Some(value_token_text(value, options)),
    }
}

pub(super) fn encoded_dot_escape(_options: &EncodeOptions) -> &'static str {
    "%2E"
}

const HEX: &[u8; 16] = b"0123456789ABCDEF";

fn key_token_text(key: &str, options: &EncodeOptions) -> String {
    if options.encode_values_only {
        return key.to_owned();
    }

    if let Some(encoder) = options.encoder.as_ref() {
        return encoder.encode(EncodeToken::Key(key), options.charset, options.format);
    }

    key.to_owned()
}

pub(super) fn plain_scalar_text(value: &Value, options: &EncodeOptions) -> Option<String> {
    match value {
        Value::Null => None,
        Value::Bool(boolean) => Some(boolean.to_string()),
        Value::I64(number) => Some(number.to_string()),
        Value::U64(number) => Some(number.to_string()),
        Value::F64(number) => Some(number.to_string()),
        Value::String(text) => Some(text.clone()),
        Value::Temporal(temporal) => plain_temporal_text(temporal, options),
        Value::Bytes(bytes) => Some(decode_bytes(bytes, options.charset)),
        Value::Array(values) => Some(
            values
                .iter()
                .map(|value| plain_string_for_comma(value, options))
                .collect::<Vec<_>>()
                .join(","),
        ),
        Value::Object(_) => Some("[object Object]".to_owned()),
    }
}

fn plain_temporal_text(value: &TemporalValue, options: &EncodeOptions) -> Option<String> {
    options.temporal_serializer().map_or_else(
        || Some(value.to_string()),
        |serializer| serializer.serialize(value),
    )
}

fn encoded_text_token(text: &str, options: &EncodeOptions) -> String {
    if let Some(encoder) = options.encoder.as_ref() {
        return encoder.encode(
            EncodeToken::TextValue(text),
            options.charset,
            options.format,
        );
    }

    text.to_owned()
}

fn value_token_text(value: &Value, options: &EncodeOptions) -> String {
    if let Some(encoder) = options.encoder.as_ref() {
        return encoder.encode(EncodeToken::Value(value), options.charset, options.format);
    }

    plain_scalar_text(value, options).unwrap_or_default()
}
