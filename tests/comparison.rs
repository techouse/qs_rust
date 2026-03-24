mod support;

use qs_rust::{DecodeOptions, Value, decode, encode};

use crate::support::{
    canonical_json, fixture_json_to_value, json_to_value, load_node_smoke_cases,
    load_smoke_fixtures, require_node_comparison,
};

#[test]
fn node_comparison_corpus_matches() {
    if !require_node_comparison("comparison corpus") {
        return;
    }

    let fixtures = load_smoke_fixtures();
    let node_cases = load_node_smoke_cases();
    assert_eq!(fixtures.len(), node_cases.len(), "fixture length mismatch");

    for (fixture, node_case) in fixtures.iter().zip(node_cases.iter()) {
        let encode_input = fixture_json_to_value(&fixture.data);
        let encoded = encode(&encode_input, &Default::default()).expect("encode should succeed");
        let encoded = encoded.replace('[', "%5B").replace(']', "%5D");
        assert_eq!(
            encoded, node_case.encoded,
            "encode mismatch for {}",
            fixture.encoded
        );

        let decoded =
            decode(&fixture.encoded, &DecodeOptions::default()).expect("decode should succeed");
        let decoded_json = canonical_json(&Value::Object(decoded));
        let node_json = canonical_json(&json_to_value(&node_case.decoded));
        assert_eq!(
            decoded_json, node_json,
            "decode mismatch for {}",
            fixture.encoded
        );
    }
}
