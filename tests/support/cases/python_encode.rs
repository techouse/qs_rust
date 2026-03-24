use qs_rust::{EncodeOptions, ListFormat};

use super::{CaseMeta, EncodeParityCase, arr, obj, s};

pub(crate) fn cases() -> Vec<EncodeParityCase> {
    vec![
        EncodeParityCase::new(
            CaseMeta::new(
                "python-qs.py",
                "encode_test.py",
                "empty string key in repeat format",
                "empty keys",
                true,
            ),
            obj(vec![("", arr(vec![s("a"), s("b")]))]),
            EncodeOptions::new().with_list_format(ListFormat::Repeat),
        ),
        EncodeParityCase::new(
            CaseMeta::new(
                "python-qs.py",
                "encode_test.py",
                "empty string key alongside sibling key",
                "empty keys",
                true,
            ),
            obj(vec![("", s("a")), ("foo", s("b"))]),
            EncodeOptions::new(),
        ),
    ]
}
