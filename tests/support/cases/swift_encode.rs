use qs_rust::{Charset, EncodeOptions, Format, ListFormat, Value};

use super::{CaseMeta, EncodeParityCase, arr, obj, s};

pub(crate) fn encode_cases() -> Vec<EncodeParityCase> {
    vec![
        EncodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "EncodeTopLevelNormalizationTests.swift",
                "top-level arrays are promoted to string indices",
                "root normalization",
                true,
            ),
            arr(vec![s("x"), s("y")]),
            EncodeOptions::new().with_encode(false),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "EncodeTests.swift",
                "allowEmptyLists emits a bare bracket pair for empty arrays",
                "empty arrays",
                true,
            ),
            obj(vec![("a", arr(vec![])), ("b", s("zz"))]),
            EncodeOptions::new().with_allow_empty_lists(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "EncodeTests.swift",
                "empty lists are omitted when allowEmptyLists is disabled even with sibling keys",
                "empty arrays",
                true,
            ),
            obj(vec![("a", arr(vec![])), ("b", s("zz"))]),
            EncodeOptions::new(),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "EncodeTests.swift",
                "allowEmptyLists with strictNullHandling keeps the same empty array marker",
                "empty arrays",
                true,
            ),
            obj(vec![("testEmptyList", arr(vec![]))]),
            EncodeOptions::new()
                .with_allow_empty_lists(true)
                .with_strict_null_handling(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "EncodeTests.swift",
                "encode false preserves bracketed child keys",
                "encoding",
                true,
            ),
            obj(vec![("search", obj(vec![("withbracket[]", s("foobar"))]))]),
            EncodeOptions::new().with_encode(false),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "EncodeTests.swift",
                "empty key nested map list edge case keeps bracketed prefixes",
                "empty keys",
                true,
            ),
            obj(vec![(
                "",
                obj(vec![("", arr(vec![s("2"), s("3")])), ("a", s("2"))]),
            )]),
            EncodeOptions::new().with_encode(false),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "EncodeTests.swift",
                "empty key repeat list emits bare equals prefixes",
                "empty keys",
                true,
            ),
            obj(vec![("", arr(vec![s("a"), s("b")]))]),
            EncodeOptions::new()
                .with_encode(false)
                .with_list_format(ListFormat::Repeat),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "EncodeTests.swift",
                "RFC1738 leaves parentheses unescaped in keys",
                "format",
                true,
            ),
            obj(vec![("foo(ref)", s("bar"))]),
            EncodeOptions::new().with_format(Format::Rfc1738),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "EncodeTests.swift",
                "encode values only plus strict null handling keeps a bare nested key",
                "null handling",
                true,
            ),
            obj(vec![("a", obj(vec![("b", Value::Null)]))]),
            EncodeOptions::new()
                .with_encode_values_only(true)
                .with_strict_null_handling(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "EncodeTests.swift",
                "single-item lists honor indices when encodeValuesOnly is enabled",
                "arrays",
                true,
            ),
            obj(vec![("a", arr(vec![s("c")]))]),
            EncodeOptions::new()
                .with_list_format(ListFormat::Indices)
                .with_encode_values_only(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "EncodeTests.swift",
                "single-item lists honor bracket formatting when encodeValuesOnly is enabled",
                "arrays",
                true,
            ),
            obj(vec![("a", arr(vec![s("c")]))]),
            EncodeOptions::new()
                .with_list_format(ListFormat::Brackets)
                .with_encode_values_only(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "EncodeTests.swift",
                "comma arrays keep literal separators between individually encoded values",
                "arrays",
                true,
            ),
            obj(vec![("a", arr(vec![s(","), s(""), s("c,d%")]))]),
            EncodeOptions::new()
                .with_list_format(ListFormat::Comma)
                .with_encode(true)
                .with_encode_values_only(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "EncodeTests.swift",
                "comma arrays with embedded commas stay split when encodeValuesOnly is enabled",
                "arrays",
                true,
            ),
            obj(vec![("a", arr(vec![s("c,d"), s("e")]))]),
            EncodeOptions::new()
                .with_list_format(ListFormat::Comma)
                .with_encode_values_only(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "EncodeTests.swift",
                "comma arrays with a single item add brackets on round trip",
                "arrays",
                true,
            ),
            obj(vec![("a", arr(vec![s("c")]))]),
            EncodeOptions::new()
                .with_list_format(ListFormat::Comma)
                .with_encode_values_only(true)
                .with_comma_round_trip(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "EncodeTests.swift",
                "comma round trip is ignored when the list format is not comma",
                "arrays",
                true,
            ),
            obj(vec![("flags", arr(vec![s("only")]))]),
            EncodeOptions::new()
                .with_list_format(ListFormat::Indices)
                .with_comma_round_trip(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "EncodeTests.swift",
                "nested arrays under encodeValuesOnly honor bracket list formatting",
                "arrays",
                true,
            ),
            obj(vec![("a", obj(vec![("b", arr(vec![s("c"), s("d")]))]))]),
            EncodeOptions::new()
                .with_list_format(ListFormat::Brackets)
                .with_encode_values_only(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "EncodeTests.swift",
                "comma format with non-list scalars does not alter scalar emission",
                "arrays",
                true,
            ),
            obj(vec![("a", s(",")), ("b", s("")), ("c", s("c,d%"))]),
            EncodeOptions::new()
                .with_list_format(ListFormat::Comma)
                .with_encode(false),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "EncodeTests.swift",
                "repeat list format with encodeValuesOnly flattens nested arrays",
                "arrays",
                true,
            ),
            obj(vec![
                ("a", s("b")),
                ("c", arr(vec![s("d"), s("e=f")])),
                ("f", arr(vec![arr(vec![s("g")]), arr(vec![s("h")])])),
            ]),
            EncodeOptions::new()
                .with_list_format(ListFormat::Repeat)
                .with_encode_values_only(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "EncodeTests.swift",
                "charset sentinel in utf8 mode prefixes the checkmark sentinel",
                "charset",
                true,
            ),
            obj(vec![("a", s("æ"))]),
            EncodeOptions::new().with_charset_sentinel(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "EncodeTests.swift",
                "iso latin mode emits numeric entities for unrepresentable code points",
                "charset",
                true,
            ),
            obj(vec![("a", s("☺"))]),
            EncodeOptions::new().with_charset(Charset::Iso88591),
        ),
    ]
}
