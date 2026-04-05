use super::{
    Charset, EncodeOptions, TemporalSerializer, TemporalValue, Value, encoded_scalar_text,
    plain_scalar_text, plain_string_for_comma, scalar_is_null_like,
};

#[test]
fn scalar_text_helpers_cover_nested_arrays_objects_bytes_and_temporals() {
    let options = EncodeOptions::new().with_encode(false);
    let nested = Value::Array(vec![
        Value::String("a".to_owned()),
        Value::Object([("field".to_owned(), Value::String("x".to_owned()))].into()),
    ]);
    assert_eq!(
        plain_string_for_comma(&nested, &options),
        "a,[object Object]"
    );
    assert_eq!(
        encoded_scalar_text(&nested, &options),
        Some("a,[object Object]".to_owned())
    );
    assert_eq!(
        encoded_scalar_text(
            &Value::Object([("field".to_owned(), Value::String("x".to_owned()))].into()),
            &options,
        ),
        Some("[object Object]".to_owned())
    );
    assert_eq!(
        plain_scalar_text(&nested, &options),
        Some("a,[object Object]".to_owned())
    );
    assert_eq!(
        plain_scalar_text(
            &Value::Object([("field".to_owned(), Value::String("x".to_owned()))].into()),
            &options,
        ),
        Some("[object Object]".to_owned())
    );

    let utf8_bytes = Value::Bytes(b"ok".to_vec());
    let latin1_bytes = Value::Bytes(vec![0xE9]);
    assert_eq!(plain_string_for_comma(&utf8_bytes, &options), "ok");
    assert_eq!(
        plain_string_for_comma(
            &latin1_bytes,
            &EncodeOptions::new()
                .with_encode(false)
                .with_charset(Charset::Iso88591),
        ),
        "é"
    );

    assert_eq!(encoded_scalar_text(&Value::Null, &options), None);

    let temporal = TemporalValue::datetime(2024, 1, 2, 3, 4, 5, 0, None).unwrap();
    let temporal_options =
        EncodeOptions::new().with_temporal_serializer(Some(TemporalSerializer::new(|_| None)));
    assert!(scalar_is_null_like(
        &Value::Temporal(temporal),
        &temporal_options,
    ));
}
