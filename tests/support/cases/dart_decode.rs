use qs_rust::{DecodeOptions, Delimiter};
use regex::Regex;

use super::{CaseMeta, DecodeParityCase};

pub(crate) fn cases() -> Vec<DecodeParityCase> {
    vec![
        DecodeParityCase::new(
            CaseMeta::new(
                "dart-qsdart",
                "decode_test.dart",
                "mixed-case encoded dots split under allowDots",
                "dot notation",
                true,
            ),
            "a%2Eb%2ec=1",
            DecodeOptions::new().with_allow_dots(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "dart-qsdart",
                "decode_test.dart",
                "top-level encoded dots split when allowDots is enabled",
                "dot notation",
                true,
            ),
            "a%2Eb=c",
            DecodeOptions::new().with_allow_dots(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "dart-qsdart",
                "decode_test.dart",
                "encoded bracket chain plus encoded dot continues into nested objects",
                "dot notation",
                true,
            ),
            "a%5Bb%5D%5Bc%5D%2Ed=x",
            DecodeOptions::new()
                .with_allow_dots(true)
                .with_decode_dot_in_keys(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "dart-qsdart",
                "decode_test.dart",
                "dotted array children keep numeric indices as arrays",
                "dot notation",
                true,
            ),
            "foo[0].baz[0]=15&foo[0].bar=2",
            DecodeOptions::new().with_allow_dots(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "dart-qsdart",
                "decode_test.dart",
                "depth remainder wrapping works for dot notation",
                "depth",
                true,
            ),
            "a.b.c=d",
            DecodeOptions::new().with_allow_dots(true).with_depth(1),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "dart-qsdart",
                "decode_test.dart",
                "strict depth throws for dotted overflow",
                "depth",
                true,
            ),
            "a.b.c=d",
            DecodeOptions::new()
                .with_allow_dots(true)
                .with_depth(1)
                .with_strict_depth(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "dart-qsdart",
                "decode_test.dart",
                "depth zero keeps the original bracketed key intact",
                "depth",
                true,
            ),
            "a[b]=1",
            DecodeOptions::new().with_depth(0).with_strict_depth(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "dart-qsdart",
                "decode_test.dart",
                "regex delimiter honors throwOnLimitExceeded",
                "delimiter",
                true,
            ),
            "a=1;;b=2;;c=3",
            DecodeOptions::new()
                .with_delimiter(Delimiter::Regex(Regex::new(";+").unwrap()))
                .with_parameter_limit(2)
                .with_throw_on_limit_exceeded(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "dart-qsdart",
                "decode_test.dart",
                "regex delimiter truncates without throwing when over parameter limit",
                "delimiter",
                true,
            ),
            "a=1;;b=2;;c=3",
            DecodeOptions::new()
                .with_delimiter(Delimiter::Regex(Regex::new(";+").unwrap()))
                .with_parameter_limit(2),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "dart-qsdart",
                "decode_test.dart",
                "non-identifier punctuation after a dot still forms a child segment in Node parity",
                "dot notation",
                true,
            ),
            "a.@foo=1",
            DecodeOptions::new().with_allow_dots(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "dart-qsdart",
                "decode_test.dart",
                "comma overflow falls back to indexed objects when the array limit is exceeded",
                "comma",
                true,
            ),
            "a=1,2,3",
            DecodeOptions::new().with_comma(true).with_list_limit(2),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "dart-qsdart",
                "decode_test.dart",
                "strict comma overflow throws when the array limit is exceeded",
                "comma",
                true,
            ),
            "a=1,2",
            DecodeOptions::new()
                .with_comma(true)
                .with_list_limit(1)
                .with_throw_on_limit_exceeded(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "dart-qsdart",
                "decode_test.dart",
                "mixed implicit and explicit arrays fall back to objects at array limit zero",
                "arrays",
                true,
            ),
            "a[]=b&a=c",
            DecodeOptions::new().with_list_limit(0),
        ),
    ]
}
