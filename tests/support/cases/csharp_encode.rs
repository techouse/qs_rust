use qs_rust::EncodeOptions;

use super::{CaseMeta, EncodeParityCase, arr, obj, s};

pub(crate) fn cases() -> Vec<EncodeParityCase> {
    vec![
        EncodeParityCase::new(
            CaseMeta::new(
                "csharp-qsnet",
                "HardeningRegressionTests.cs",
                "allow empty lists does not short circuit non-empty lists",
                "empty arrays",
                true,
            ),
            obj(vec![("a", arr(vec![s("x")]))]),
            EncodeOptions::new()
                .with_encode(false)
                .with_allow_empty_lists(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "csharp-qsnet",
                "EncodeTests.cs",
                "allow empty lists on default indices emits empty brackets",
                "empty arrays",
                true,
            ),
            obj(vec![("a", arr(vec![]))]),
            EncodeOptions::new()
                .with_encode(false)
                .with_allow_empty_lists(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "csharp-qsnet",
                "EncodeTests.cs",
                "encodes child key dots when allow dots and encode dot in keys are enabled",
                "dot notation",
                true,
            ),
            obj(vec![("a", obj(vec![("b.c", s("x"))]))]),
            EncodeOptions::new()
                .with_allow_dots(true)
                .with_encode_dot_in_keys(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "csharp-qsnet",
                "EncodeTests.cs",
                "encodes ancestor separators once paths become nested objects",
                "dot notation",
                true,
            ),
            obj(vec![("a", obj(vec![("b", obj(vec![("c.d", s("x"))]))]))]),
            EncodeOptions::new()
                .with_allow_dots(true)
                .with_encode_dot_in_keys(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "csharp-qsnet",
                "EncodeTests.cs",
                "charset sentinel uses ampersand before body even with custom delimiter",
                "charset",
                true,
            ),
            obj(vec![("a", s("b")), ("c", s("d"))]),
            EncodeOptions::new()
                .with_encode(false)
                .with_charset_sentinel(true)
                .with_delimiter(";"),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "csharp-qsnet",
                "EncodeTests.cs",
                "comma arrays keep literal separators with encode values only",
                "comma",
                true,
            ),
            obj(vec![("letters", arr(vec![s("a"), s("b")]))]),
            EncodeOptions::new()
                .with_list_format(qs_rust::ListFormat::Comma)
                .with_encode_values_only(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "csharp-qsnet",
                "EncodeTests.cs",
                "strict null handling returns a bare key when encoding is disabled",
                "null handling",
                true,
            ),
            obj(vec![("a", qs_rust::Value::Null)]),
            EncodeOptions::new()
                .with_encode(false)
                .with_strict_null_handling(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "csharp-qsnet",
                "EncodeTests.cs",
                "utf8 charset sentinel marker is emitted before the body",
                "charset",
                true,
            ),
            obj(vec![("a", s("b"))]),
            EncodeOptions::new()
                .with_encode(false)
                .with_charset_sentinel(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "csharp-qsnet",
                "EncodeTests.cs",
                "latin1 charset sentinel marker is emitted before the body",
                "charset",
                true,
            ),
            obj(vec![("a", s("b"))]),
            EncodeOptions::new()
                .with_encode(false)
                .with_charset_sentinel(true)
                .with_charset(qs_rust::Charset::Iso88591),
        ),
    ]
}
