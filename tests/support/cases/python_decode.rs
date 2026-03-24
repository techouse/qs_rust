use qs_rust::DecodeOptions;

use super::{CaseMeta, DecodeParityCase};

pub(crate) fn cases() -> Vec<DecodeParityCase> {
    vec![
        DecodeParityCase::new(
            CaseMeta::new(
                "python-qs.py",
                "decode_test.py",
                "encoded dot remains literal when dots are disabled",
                "dot notation",
                true,
            ),
            "a%2Eb=c",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "python-qs.py",
                "decode_test.py",
                "unterminated group does not trigger strict depth",
                "depth",
                true,
            ),
            "a[b[c]=d",
            DecodeOptions::new().with_depth(1).with_strict_depth(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "python-qs.py",
                "decode_test.py",
                "mixed encoded brackets and encoded dot",
                "dot notation",
                true,
            ),
            "a%5Bb%5D%5Bc%5D%2Ed=x",
            DecodeOptions::new().with_decode_dot_in_keys(true),
        ),
    ]
}
