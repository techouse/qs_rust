use super::{DecodeCase, EncodeCase, decode_cases, encode_cases};

#[test]
fn encode_case_filter_applies_max_depth_cap() {
    let cases = encode_cases(Some(4000));
    assert_eq!(
        cases,
        vec![EncodeCase {
            depth: 2000,
            iterations: 20,
        }]
    );
}

#[test]
fn decode_case_filter_limits_available_cases() {
    let cases = decode_cases(Some(DecodeCase {
        name: "C2",
        count: 1000,
        comma: false,
        utf8_sentinel: false,
        value_len: 40,
        iterations: 16,
    }));

    assert_eq!(
        cases,
        vec![DecodeCase {
            name: "C2",
            count: 1000,
            comma: false,
            utf8_sentinel: false,
            value_len: 40,
            iterations: 16,
        }]
    );
}
