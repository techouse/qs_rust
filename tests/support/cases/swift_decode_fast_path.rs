use qs_rust::{Charset, DecodeOptions};

use super::{CaseMeta, DecodeParityCase};

pub(crate) fn decode_cases() -> Vec<DecodeParityCase> {
    vec![
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeFastPathTests.swift",
                "flat root collides with later structured object",
                "fast-path parity",
                true,
            ),
            "a=1&a[b]=2",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeFastPathTests.swift",
                "structured root followed by flat scalar",
                "fast-path parity",
                true,
            ),
            "a[b]=2&a=1",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeFastPathTests.swift",
                "leading zero collision preserves noncanonical key identity",
                "root collisions",
                true,
            ),
            "01=y&[01]=x",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeFastPathTests.swift",
                "encoded dot structured key plus flat key",
                "dot notation",
                true,
            ),
            "a%252Eb=1&a=2",
            DecodeOptions::new()
                .with_allow_dots(true)
                .with_decode_dot_in_keys(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeFastPathTests.swift",
                "parameter limit counts charset sentinel before skipping it",
                "parameter limit",
                true,
            ),
            "utf8=%E2%9C%93&a=1",
            DecodeOptions::new()
                .with_charset(Charset::Iso88591)
                .with_charset_sentinel(true)
                .with_parameter_limit(1)
                .with_throw_on_limit_exceeded(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeFastPathTests.swift",
                "empty key still participates in top-level list limit behavior",
                "list limits",
                true,
            ),
            "=&a[]=b&a[]=c",
            DecodeOptions::new().with_list_limit(1),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeFastPathTests.swift",
                "split precedence keeps ] with the key before =",
                "delimiter",
                true,
            ),
            "=x]=y",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeFastPathTests.swift",
                "raw part bracket marker wraps comma arrays",
                "comma",
                true,
            ),
            "a[b]=1,2[]=",
            DecodeOptions::new().with_comma(true),
        ),
    ]
}
