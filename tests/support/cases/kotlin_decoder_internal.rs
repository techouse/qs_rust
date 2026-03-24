use qs_rust::{Charset, DecodeOptions, Delimiter};
use regex::Regex;

use super::{CaseMeta, DecodeParityCase};

pub(crate) fn decode_cases() -> Vec<DecodeParityCase> {
    vec![
        DecodeParityCase::new(
            CaseMeta::new(
                "kotlin-qskotlin",
                "DecoderInternalSpec.kt",
                "string delimiter ignores adjacent empty segments",
                "delimiter",
                true,
            ),
            "a=1&&b=2&&",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "kotlin-qskotlin",
                "DecoderInternalSpec.kt",
                "regex delimiter ignores adjacent empty segments",
                "delimiter",
                true,
            ),
            "a=1;b=2,,c=3;;",
            DecodeOptions::new().with_delimiter(Delimiter::Regex(Regex::new("[;,]").unwrap())),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "kotlin-qskotlin",
                "DecoderInternalSpec.kt",
                "charset sentinel and numeric entities cooperate with query prefix stripping",
                "charset",
                true,
            ),
            "?utf8=%26%2310003%3B&name=%26%2365%3B",
            DecodeOptions::new()
                .with_interpret_numeric_entities(true)
                .with_charset(Charset::Iso88591)
                .with_charset_sentinel(true)
                .with_ignore_query_prefix(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "kotlin-qskotlin",
                "DecoderInternalSpec.kt",
                "bracket suffix comma values become nested arrays",
                "comma",
                true,
            ),
            "tags[]=a,b",
            DecodeOptions::new().with_comma(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "kotlin-qskotlin",
                "DecoderInternalSpec.kt",
                "comma parsing preserves empty boundary tokens",
                "comma",
                true,
            ),
            "a=,",
            DecodeOptions::new().with_comma(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "kotlin-qskotlin",
                "DecodeSpec.kt",
                "depth one wraps the dot remainder after the first structured segment",
                "depth",
                true,
            ),
            "a.b.c=d",
            DecodeOptions::new().with_allow_dots(true).with_depth(1),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "kotlin-qskotlin",
                "DecodeSpec.kt",
                "leading top-level dot collapses into the root key",
                "dot notation",
                true,
            ),
            ".a=x",
            DecodeOptions::new().with_allow_dots(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "kotlin-qskotlin",
                "DecodeSpec.kt",
                "trailing top-level dot stays literal",
                "dot notation",
                true,
            ),
            "a.=x",
            DecodeOptions::new().with_allow_dots(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "kotlin-qskotlin",
                "DecodeSpec.kt",
                "double dots preserve the empty middle segment",
                "dot notation",
                true,
            ),
            "a..b=x",
            DecodeOptions::new().with_allow_dots(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "kotlin-qskotlin",
                "DecodeSpec.kt",
                "encoded top-level dot before bracket splits with allow dots",
                "dot notation",
                true,
            ),
            "a%2E[b]=x",
            DecodeOptions::new()
                .with_allow_dots(true)
                .with_decode_dot_in_keys(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "kotlin-qskotlin",
                "DecodeSpec.kt",
                "bracket then encoded dot advances to the next segment",
                "dot notation",
                true,
            ),
            "a[b]%2Ec=x",
            DecodeOptions::new()
                .with_allow_dots(true)
                .with_decode_dot_in_keys(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "kotlin-qskotlin",
                "DecodeSpec.kt",
                "encoded brackets plus encoded dot compose a nested path",
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
                "kotlin-qskotlin",
                "DecodeSpec.kt",
                "dot before a list index preserves list semantics",
                "dot notation",
                true,
            ),
            "foo[0].baz[0]=15&foo[0].bar=2",
            DecodeOptions::new().with_allow_dots(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "kotlin-qskotlin",
                "DecodeSpec.kt",
                "unknown charset sentinel value is ignored",
                "charset",
                true,
            ),
            "utf8=foo&%C3%B8=%C3%B8",
            DecodeOptions::new()
                .with_charset_sentinel(true)
                .with_charset(Charset::Utf8),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "kotlin-qskotlin",
                "DecodeSpec.kt",
                "charset sentinel can appear after the parameter it affects",
                "charset",
                true,
            ),
            "a=%C3%B8&utf8=%26%2310003%3B",
            DecodeOptions::new()
                .with_charset_sentinel(true)
                .with_charset(Charset::Utf8),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "kotlin-qskotlin",
                "DecodeSpec.kt",
                "percent-u sequences stay literal in latin1 mode",
                "charset",
                true,
            ),
            "%u263A=%u263A",
            DecodeOptions::new().with_charset(Charset::Iso88591),
        ),
    ]
}
