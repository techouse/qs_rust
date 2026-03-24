use qs_rust::{EncodeOptions, Value};

use super::{CaseMeta, EncodeParityCase, obj, s};

pub(crate) fn encode_cases() -> Vec<EncodeParityCase> {
    let nested_null = obj(vec![(
        "root",
        obj(vec![("a", obj(vec![("b", Value::Null)]))]),
    )]);

    vec![
        EncodeParityCase::new(
            CaseMeta::new(
                "kotlin-qskotlin",
                "EncoderInternalSpec.kt",
                "nested null honors strict null handling",
                "null handling",
                true,
            ),
            nested_null.clone(),
            EncodeOptions::new().with_strict_null_handling(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "kotlin-qskotlin",
                "EncoderInternalSpec.kt",
                "nested null honors skip nulls",
                "null handling",
                true,
            ),
            nested_null.clone(),
            EncodeOptions::new().with_skip_nulls(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "kotlin-qskotlin",
                "EncoderInternalSpec.kt",
                "skip nulls wins over strict null handling in nested chains",
                "null handling",
                true,
            ),
            nested_null,
            EncodeOptions::new()
                .with_skip_nulls(true)
                .with_strict_null_handling(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "kotlin-qskotlin",
                "EncoderInternalSpec.kt",
                "allow dots and encode dot in keys on nested chains",
                "dot notation",
                true,
            ),
            obj(vec![(
                "p.q",
                obj(vec![("k.v", obj(vec![("n.m", s("x"))]))]),
            )]),
            EncodeOptions::new()
                .with_allow_dots(true)
                .with_encode_dot_in_keys(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "kotlin-qskotlin",
                "EncodeSpec.kt",
                "encode dot in keys implies allow dots when omitted",
                "dot notation",
                true,
            ),
            obj(vec![(
                "name.obj.subobject",
                obj(vec![("first.godly.name", s("John")), ("last", s("Doe"))]),
            )]),
            EncodeOptions::new().with_encode_dot_in_keys(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "kotlin-qskotlin",
                "EncodeSpec.kt",
                "encode values only keeps only values percent-encoded when dots are escaped",
                "dot notation",
                true,
            ),
            obj(vec![(
                "name.obj.subobject",
                obj(vec![("first.godly.name", s("John")), ("last", s("Doe"))]),
            )]),
            EncodeOptions::new()
                .with_allow_dots(true)
                .with_encode_dot_in_keys(true)
                .with_encode_values_only(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "kotlin-qskotlin",
                "EncodeSpec.kt",
                "nested empty list is preserved when allow empty lists is enabled",
                "empty arrays",
                true,
            ),
            obj(vec![(
                "outer",
                obj(vec![("inner", Value::Array(Vec::new()))]),
            )]),
            EncodeOptions::new()
                .with_allow_empty_lists(true)
                .with_encode(false)
                .with_list_format(qs_rust::ListFormat::Indices),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "kotlin-qskotlin",
                "EncodeSpec.kt",
                "top-level dot key stays plain when encode dot in keys is enabled",
                "dot notation",
                true,
            ),
            obj(vec![("a.b", s("v"))]),
            EncodeOptions::new().with_encode_dot_in_keys(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "kotlin-qskotlin",
                "EncodeSpec.kt",
                "strict null handling with empty key suppresses the sentinel delimiter",
                "charset",
                true,
            ),
            obj(vec![("", Value::Null)]),
            EncodeOptions::new()
                .with_strict_null_handling(true)
                .with_charset_sentinel(true),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "kotlin-qskotlin",
                "EncodeSpec.kt",
                "repeat list format keeps object keys when list items are objects",
                "arrays",
                true,
            ),
            obj(vec![("a", Value::Array(vec![obj(vec![("b", s("c"))])]))]),
            EncodeOptions::new().with_list_format(qs_rust::ListFormat::Repeat),
        ),
    ]
}
