#![allow(dead_code)]

use qs_rust::{DecodeOptions, EncodeOptions, Value};

pub(crate) mod csharp_decode;
pub(crate) mod csharp_encode;
pub(crate) mod csharp_internal;
pub(crate) mod dart_decode;
pub(crate) mod dart_encode;
pub(crate) mod kotlin_decoder_internal;
pub(crate) mod kotlin_encoder_internal;
pub(crate) mod kotlin_utils;
pub(crate) mod node_parse;
pub(crate) mod node_stringify;
pub(crate) mod python_decode;
pub(crate) mod python_encode;
pub(crate) mod swift_decode;
pub(crate) mod swift_decode_fast_path;
pub(crate) mod swift_encode;
pub(crate) mod swift_encoder_internals;

#[derive(Clone, Copy, Debug)]
pub(crate) struct CaseMeta {
    pub(crate) source_repo: &'static str,
    pub(crate) source_file: &'static str,
    pub(crate) title: &'static str,
    pub(crate) family: &'static str,
    pub(crate) node_backed: bool,
}

impl CaseMeta {
    pub(crate) const fn new(
        source_repo: &'static str,
        source_file: &'static str,
        title: &'static str,
        family: &'static str,
        node_backed: bool,
    ) -> Self {
        Self {
            source_repo,
            source_file,
            title,
            family,
            node_backed,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DecodeParityCase {
    pub(crate) meta: CaseMeta,
    pub(crate) query: &'static str,
    pub(crate) options: DecodeOptions,
}

impl DecodeParityCase {
    pub(crate) fn new(meta: CaseMeta, query: &'static str, options: DecodeOptions) -> Self {
        Self {
            meta,
            query,
            options,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct EncodeParityCase {
    pub(crate) meta: CaseMeta,
    pub(crate) value: Value,
    pub(crate) options: EncodeOptions,
}

impl EncodeParityCase {
    pub(crate) fn new(meta: CaseMeta, value: Value, options: EncodeOptions) -> Self {
        Self {
            meta,
            value,
            options,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DecodeOptionInvariantCase {
    pub(crate) meta: CaseMeta,
    pub(crate) options: DecodeOptions,
    pub(crate) expect_allow_dots: bool,
    pub(crate) expect_decode_dot_in_keys: bool,
}

impl DecodeOptionInvariantCase {
    pub(crate) fn new(
        meta: CaseMeta,
        options: DecodeOptions,
        expect_allow_dots: bool,
        expect_decode_dot_in_keys: bool,
    ) -> Self {
        Self {
            meta,
            options,
            expect_allow_dots,
            expect_decode_dot_in_keys,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct EncodeOptionInvariantCase {
    pub(crate) meta: CaseMeta,
    pub(crate) options: EncodeOptions,
    pub(crate) expect_allow_dots: bool,
    pub(crate) expect_encode_dot_in_keys: bool,
}

impl EncodeOptionInvariantCase {
    pub(crate) fn new(
        meta: CaseMeta,
        options: EncodeOptions,
        expect_allow_dots: bool,
        expect_encode_dot_in_keys: bool,
    ) -> Self {
        Self {
            meta,
            options,
            expect_allow_dots,
            expect_encode_dot_in_keys,
        }
    }
}

pub(crate) fn imported_decode_cases() -> Vec<DecodeParityCase> {
    let mut cases = Vec::new();
    cases.extend(node_parse::cases());
    cases.extend(python_decode::cases());
    cases.extend(csharp_decode::cases());
    cases.extend(dart_decode::cases());
    cases.extend(swift_decode::decode_cases());
    cases.extend(swift_decode_fast_path::decode_cases());
    cases.extend(kotlin_decoder_internal::decode_cases());
    cases
}

pub(crate) fn imported_encode_cases() -> Vec<EncodeParityCase> {
    let mut cases = Vec::new();
    cases.extend(node_stringify::cases());
    cases.extend(python_encode::cases());
    cases.extend(csharp_encode::cases());
    cases.extend(dart_encode::cases());
    cases.extend(swift_encode::encode_cases());
    cases.extend(swift_encoder_internals::encode_cases());
    cases.extend(kotlin_encoder_internal::encode_cases());
    cases
}

pub(crate) fn s(value: &str) -> Value {
    Value::String(value.to_owned())
}

pub(crate) fn arr(values: Vec<Value>) -> Value {
    Value::Array(values)
}

pub(crate) fn obj(entries: Vec<(&str, Value)>) -> Value {
    Value::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect(),
    )
}
