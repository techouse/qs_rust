use qs_rust::{Charset, EncodeOptions, Format, ListFormat, SortMode, Value, WhitelistSelector};

use super::{CaseMeta, EncodeParityCase, arr, obj, s};

pub(crate) fn cases() -> Vec<EncodeParityCase> {
    vec![
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "stringifies a querystring object",
                "basic",
                true,
            ),
            obj(vec![("a", s("b")), ("c", s("d"))]),
            EncodeOptions::new(),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "stringifies falsy values",
                "basic",
                true,
            ),
            obj(vec![
                ("a", Value::Bool(false)),
                ("b", Value::I64(0)),
                ("c", Value::String(String::new())),
            ]),
            EncodeOptions::new(),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "adds query prefix",
                "prefix",
                true,
            ),
            obj(vec![("a", s("b"))]),
            EncodeOptions::new().with_add_query_prefix(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "query prefix on empty object still returns blank string",
                "prefix",
                true,
            ),
            obj(vec![]),
            EncodeOptions::new().with_add_query_prefix(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "stringifies a nested object",
                "objects",
                true,
            ),
            obj(vec![("a", obj(vec![("b", s("c"))]))]),
            EncodeOptions::new(),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "allow dots stringifies a nested object with dot notation",
                "objects",
                true,
            ),
            obj(vec![("a", obj(vec![("b", s("c"))]))]),
            EncodeOptions::new().with_allow_dots(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "uses indices notation for arrays by default",
                "arrays",
                true,
            ),
            obj(vec![("a", arr(vec![s("b"), s("c")]))]),
            EncodeOptions::new(),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "uses repeat notation for arrays",
                "arrays",
                true,
            ),
            obj(vec![("a", arr(vec![s("b"), s("c")]))]),
            EncodeOptions::new().with_list_format(ListFormat::Repeat),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "uses brackets notation for arrays",
                "arrays",
                true,
            ),
            obj(vec![("a", arr(vec![s("b"), s("c")]))]),
            EncodeOptions::new().with_list_format(ListFormat::Brackets),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "stringifies comma arrays",
                "arrays",
                true,
            ),
            obj(vec![("a", arr(vec![s("b"), s("c")]))]),
            EncodeOptions::new().with_list_format(ListFormat::Comma),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "comma round trip for single item arrays",
                "arrays",
                true,
            ),
            obj(vec![("a", arr(vec![s("b")]))]),
            EncodeOptions::new()
                .with_list_format(ListFormat::Comma)
                .with_comma_round_trip(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "skip nulls",
                "null handling",
                true,
            ),
            obj(vec![("a", Value::Null), ("b", s("2"))]),
            EncodeOptions::new().with_skip_nulls(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "strict null handling",
                "null handling",
                true,
            ),
            obj(vec![("a", Value::Null)]),
            EncodeOptions::new().with_strict_null_handling(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "omits empty arrays by default",
                "empty arrays",
                true,
            ),
            obj(vec![("a", arr(Vec::new()))]),
            EncodeOptions::new(),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "allow empty arrays",
                "empty arrays",
                true,
            ),
            obj(vec![("a", arr(Vec::new()))]),
            EncodeOptions::new()
                .with_allow_empty_lists(true)
                .with_list_format(ListFormat::Brackets),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "allow empty arrays plus strict null handling",
                "empty arrays",
                true,
            ),
            obj(vec![("a", arr(Vec::new()))]),
            EncodeOptions::new()
                .with_allow_empty_lists(true)
                .with_list_format(ListFormat::Brackets)
                .with_strict_null_handling(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "comma arrays with commas inside",
                "arrays",
                true,
            ),
            obj(vec![("a", arr(vec![s("x,y"), s("z")]))]),
            EncodeOptions::new().with_list_format(ListFormat::Comma),
        ),
        EncodeParityCase::new(
            CaseMeta::new("node-qs", "stringify.js", "nested arrays", "arrays", true),
            obj(vec![("a", arr(vec![arr(vec![s("b")]), arr(vec![s("c")])]))]),
            EncodeOptions::new(),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "objects inside arrays",
                "arrays",
                true,
            ),
            obj(vec![("a", arr(vec![obj(vec![("b", s("c"))])]))]),
            EncodeOptions::new(),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "encode dot in key of object",
                "dot notation",
                true,
            ),
            obj(vec![("name.obj", obj(vec![("first", s("John"))]))]),
            EncodeOptions::new()
                .with_allow_dots(true)
                .with_encode_dot_in_keys(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "encode values only",
                "encoding",
                true,
            ),
            obj(vec![("a b", obj(vec![("c d", s("x y"))]))]),
            EncodeOptions::new().with_encode_values_only(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "alternative delimiter",
                "delimiter",
                true,
            ),
            obj(vec![("a", s("1")), ("b", s("2"))]),
            EncodeOptions::new().with_delimiter(";"),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "sorts keys lexicographically",
                "sorting",
                true,
            ),
            obj(vec![("b", s("2")), ("a", s("1")), ("c", s("3"))]),
            EncodeOptions::new().with_sort(SortMode::LexicographicAsc),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "whitelist filter array",
                "filtering",
                true,
            ),
            obj(vec![("b", s("2")), ("a", s("1")), ("c", s("3"))]),
            EncodeOptions::new().with_whitelist(Some(vec![
                WhitelistSelector::Key("c".to_owned()),
                WhitelistSelector::Key("a".to_owned()),
            ])),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "rfc1738 formatting",
                "format",
                true,
            ),
            obj(vec![("a", s("x y"))]),
            EncodeOptions::new().with_format(Format::Rfc1738),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "charset sentinel in latin1 mode",
                "charset",
                true,
            ),
            obj(vec![("name", s("ø"))]),
            EncodeOptions::new()
                .with_charset(Charset::Iso88591)
                .with_charset_sentinel(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "empty-keys-cases.js",
                "empty string key scalar",
                "empty keys",
                true,
            ),
            obj(vec![("", s("a"))]),
            EncodeOptions::new(),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "empty-keys-cases.js",
                "empty string key array in indices format",
                "empty keys",
                true,
            ),
            obj(vec![("", arr(vec![s("a"), s("b")]))]),
            EncodeOptions::new().with_list_format(ListFormat::Indices),
        ),
        EncodeParityCase::new(
            CaseMeta::new("node-qs", "stringify.js", "numeric values", "basic", true),
            obj(vec![("a", Value::I64(1)), ("b", Value::F64(2.5))]),
            EncodeOptions::new(),
        ),
    ]
}
