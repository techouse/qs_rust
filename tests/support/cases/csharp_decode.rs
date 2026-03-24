use qs_rust::DecodeOptions;

use super::{CaseMeta, DecodeParityCase};

pub(crate) fn cases() -> Vec<DecodeParityCase> {
    vec![
        DecodeParityCase::new(
            CaseMeta::new(
                "csharp-qsnet",
                "DecodeTests.cs",
                "leading dot with allow dots preserves the token after the dot",
                "dot notation",
                true,
            ),
            ".a=x",
            DecodeOptions::new().with_allow_dots(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "csharp-qsnet",
                "DecodeTests.cs",
                "double dots keep a literal dot in the parent key",
                "dot notation",
                true,
            ),
            "a..b=x",
            DecodeOptions::new().with_allow_dots(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "csharp-qsnet",
                "DecodeTests.cs",
                "depth one keeps dot remainder as a single bracketed segment",
                "depth",
                true,
            ),
            "a.b.c=x",
            DecodeOptions::new().with_allow_dots(true).with_depth(1),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "csharp-qsnet",
                "DecodeTests.cs",
                "top level encoded dot also splits when allow dots is enabled",
                "dot notation",
                true,
            ),
            "a%2Eb=c",
            DecodeOptions::new().with_allow_dots(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "csharp-qsnet",
                "DecodeTests.cs",
                "bracket then encoded dot advances to next segment",
                "dot notation",
                true,
            ),
            "a[b]%2Ec=x",
            DecodeOptions::new().with_decode_dot_in_keys(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "csharp-qsnet",
                "DecodeTests.cs",
                "comma split allowed when sum equals limit",
                "comma",
                true,
            ),
            "a=1,2&a=3,4,5",
            DecodeOptions::new()
                .with_comma(true)
                .with_list_limit(5)
                .with_throw_on_limit_exceeded(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "csharp-qsnet",
                "DecodeTests.cs",
                "comma split over the limit converts to object when throwing is off",
                "comma",
                true,
            ),
            "a=1,2&a=3,4,5,6",
            DecodeOptions::new().with_comma(true).with_list_limit(5),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "csharp-qsnet",
                "DecodeTests.cs",
                "nested object values also split on commas",
                "comma",
                true,
            ),
            "foo[bar]=coffee,tee",
            DecodeOptions::new().with_comma(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "csharp-qsnet",
                "HardeningRegressionTests.cs",
                "strict null handling and allow empty lists yield an empty list for bare brackets",
                "empty arrays",
                true,
            ),
            "a[]",
            DecodeOptions::new()
                .with_allow_empty_lists(true)
                .with_strict_null_handling(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "csharp-qsnet",
                "HardeningRegressionTests.cs",
                "allow empty lists treats an assigned empty string leaf as an empty list",
                "empty arrays",
                true,
            ),
            "a[]=",
            DecodeOptions::new().with_allow_empty_lists(true),
        ),
    ]
}
