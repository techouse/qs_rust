use qs_rust::{EncodeOptions, Format, ListFormat, Value};

use super::{CaseMeta, EncodeParityCase, arr, obj, s};

pub(crate) fn cases() -> Vec<EncodeParityCase> {
    vec![
        EncodeParityCase::new(
            CaseMeta::new(
                "dart-qsdart",
                "encode_test.dart",
                "strict null handling keeps percent-escaped spaces in key-only RFC1738 output",
                "null handling",
                true,
            ),
            obj(vec![("a b", Value::Null)]),
            EncodeOptions::new()
                .with_strict_null_handling(true)
                .with_format(Format::Rfc1738),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "dart-qsdart",
                "encode_test.dart",
                "nested null leaves emit a trailing equals when strictNullHandling is disabled",
                "null handling",
                true,
            ),
            obj(vec![("a", obj(vec![("b", Value::Null)]))]),
            EncodeOptions::new().with_encode(false),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "dart-qsdart",
                "encode_test.dart",
                "nested null leaves become bare keys under strictNullHandling with encode disabled",
                "null handling",
                true,
            ),
            obj(vec![("a", obj(vec![("b", Value::Null)]))]),
            EncodeOptions::new()
                .with_encode(false)
                .with_strict_null_handling(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "dart-qsdart",
                "encode_test.dart",
                "allowDots keeps dotted output when encoding is disabled",
                "dot notation",
                true,
            ),
            obj(vec![(
                "a",
                obj(vec![("b", obj(vec![("c", Value::I64(1))]))]),
            )]),
            EncodeOptions::new()
                .with_encode(false)
                .with_allow_dots(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "dart-qsdart",
                "encode_test.dart",
                "comma round trip adds brackets for a single item when encode is disabled",
                "arrays",
                true,
            ),
            obj(vec![("a", arr(vec![s("x")]))]),
            EncodeOptions::new()
                .with_encode(false)
                .with_list_format(ListFormat::Comma)
                .with_comma_round_trip(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "dart-qsdart",
                "encode_test.dart",
                "allowEmptyLists emits key brackets when encode is disabled",
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
                "dart-qsdart",
                "encode_test.dart",
                "generic object traversal preserves multi-key output with encode disabled",
                "objects",
                true,
            ),
            obj(vec![(
                "a",
                obj(vec![("x", Value::I64(1)), ("y", Value::I64(2))]),
            )]),
            EncodeOptions::new().with_encode(false),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "dart-qsdart",
                "encode_test.dart",
                "empty key object-array edge case matches Node output",
                "empty keys",
                true,
            ),
            obj(vec![(
                "",
                obj(vec![("", arr(vec![s("2"), s("3")])), ("a", Value::I64(2))]),
            )]),
            EncodeOptions::new().with_encode(false),
        ),
    ]
}
