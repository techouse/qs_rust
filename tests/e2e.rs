mod support;

use qs_rust::{DecodeOptions, EncodeOptions, Value, decode, encode};

use crate::support::{canonical_json, fixture_json_to_value, load_smoke_fixtures};

#[test]
fn smoke_fixtures_round_trip_through_rust_encode_and_decode() {
    let fixtures = load_smoke_fixtures();

    for fixture in fixtures {
        let value = fixture_json_to_value(&fixture.data);
        let encoded = encode(&value, &EncodeOptions::new().with_encode(false))
            .expect("encode should succeed");
        assert_eq!(encoded, fixture.encoded, "encode mismatch for fixture");

        let decoded =
            decode(&fixture.encoded, &DecodeOptions::new()).expect("decode should succeed");
        let decoded_json = canonical_json(&Value::Object(decoded));
        let expected_json = canonical_json(&value);
        assert_eq!(decoded_json, expected_json, "decode mismatch for fixture");
    }
}
