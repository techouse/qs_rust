use qs_rust::EncodeOptions;

use super::{CaseMeta, EncodeParityCase, obj, s};

pub(crate) fn encode_cases() -> Vec<EncodeParityCase> {
    vec![
        EncodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "EncoderInternalsTests.swift",
                "allow dots nested chain still stringifies as a dot path",
                "linear fast path",
                true,
            ),
            obj(vec![(
                "root",
                obj(vec![("a", obj(vec![("leaf", s("x"))]))]),
            )]),
            EncodeOptions::new().with_allow_dots(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "EncoderInternalsTests.swift",
                "multi-key node falls back to generic traversal",
                "linear fast path",
                true,
            ),
            obj(vec![(
                "root",
                obj(vec![("a", obj(vec![("leaf", s("x"))])), ("b", s("y"))]),
            )]),
            EncodeOptions::new(),
        ),
    ]
}
