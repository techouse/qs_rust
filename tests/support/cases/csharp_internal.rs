use qs_rust::{DecodeOptions, EncodeOptions};

use super::{CaseMeta, DecodeOptionInvariantCase, EncodeOptionInvariantCase};

pub(crate) fn decode_option_invariants() -> Vec<DecodeOptionInvariantCase> {
    vec![
        DecodeOptionInvariantCase::new(
            CaseMeta::new(
                "csharp-qsnet",
                "DecodeOptionsTests.cs",
                "decode dot in keys implies allow dots",
                "option invariants",
                false,
            ),
            DecodeOptions::new().with_decode_dot_in_keys(true),
            true,
            true,
        ),
        DecodeOptionInvariantCase::new(
            CaseMeta::new(
                "csharp-qsnet",
                "DecodeOptionsTests.cs",
                "disabling allow dots clears decode dot in keys",
                "option invariants",
                false,
            ),
            DecodeOptions::new()
                .with_decode_dot_in_keys(true)
                .with_allow_dots(false),
            false,
            false,
        ),
    ]
}

pub(crate) fn encode_option_invariants() -> Vec<EncodeOptionInvariantCase> {
    vec![
        EncodeOptionInvariantCase::new(
            CaseMeta::new(
                "csharp-qsnet",
                "EncodeOptionsTests.cs",
                "encode dot in keys implies allow dots",
                "option invariants",
                false,
            ),
            EncodeOptions::new().with_encode_dot_in_keys(true),
            true,
            true,
        ),
        EncodeOptionInvariantCase::new(
            CaseMeta::new(
                "csharp-qsnet",
                "EncodeOptionsTests.cs",
                "disabling allow dots clears encode dot in keys",
                "option invariants",
                false,
            ),
            EncodeOptions::new()
                .with_encode_dot_in_keys(true)
                .with_allow_dots(false),
            false,
            false,
        ),
    ]
}
