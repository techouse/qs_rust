use super::{EncodeOptions, Value, encode};

#[test]
fn max_depth_rejects_deeper_paths() {
    let value = Value::Object(
        [(
            "root".to_owned(),
            Value::Object([("leaf".to_owned(), Value::String("x".to_owned()))].into()),
        )]
        .into(),
    );

    let error = encode(&value, &EncodeOptions::new().with_max_depth(Some(0))).unwrap_err();
    assert!(error.is_depth_exceeded());
}

#[test]
fn iterative_encode_matches_expected_output_for_mixed_payloads() {
    let value = Value::Object(
        [(
            "root".to_owned(),
            Value::Object(
                [
                    (
                        "k".to_owned(),
                        Value::Object(
                            [
                                (
                                    "arr".to_owned(),
                                    Value::Array(vec![
                                        Value::I64(1),
                                        Value::Null,
                                        Value::Object(
                                            [("x".to_owned(), Value::String("y".to_owned()))]
                                                .into(),
                                        ),
                                    ]),
                                ),
                                ("str".to_owned(), Value::String("v".to_owned())),
                            ]
                            .into(),
                        ),
                    ),
                    ("n".to_owned(), Value::I64(3)),
                ]
                .into(),
            ),
        )]
        .into(),
    );

    let encoded = encode(&value, &EncodeOptions::new()).unwrap();
    assert_eq!(
        encoded,
        "root%5Bk%5D%5Barr%5D%5B0%5D=1&root%5Bk%5D%5Barr%5D%5B1%5D=&root%5Bk%5D%5Barr%5D%5B2%5D%5Bx%5D=y&root%5Bk%5D%5Bstr%5D=v&root%5Bn%5D=3"
    );
}

#[test]
fn deep_chain_output_is_preserved_when_encoding_is_disabled() {
    let depth = 128usize;
    let mut value = Value::Object([("leaf".to_owned(), Value::String("x".to_owned()))].into());
    for _ in 0..depth {
        value = Value::Object([("a".to_owned(), value)].into());
    }

    let encoded = encode(&value, &EncodeOptions::new().with_encode(false)).unwrap();

    let mut expected = String::from("a");
    for _ in 0..depth.saturating_sub(1) {
        expected.push_str("[a]");
    }
    expected.push_str("[leaf]=x");

    assert_eq!(encoded, expected);
}

#[test]
fn deep_chain_output_is_preserved_when_allow_dots_bypasses_fast_path() {
    let depth = 128usize;
    let mut value = Value::Object([("leaf".to_owned(), Value::String("x".to_owned()))].into());
    for _ in 0..depth {
        value = Value::Object([("a".to_owned(), value)].into());
    }

    let encoded = encode(
        &value,
        &EncodeOptions::new()
            .with_encode(false)
            .with_allow_dots(true),
    )
    .unwrap();

    let mut expected = String::from("a");
    for _ in 0..depth.saturating_sub(1) {
        expected.push_str(".a");
    }
    expected.push_str(".leaf=x");

    assert_eq!(encoded, expected);
}
