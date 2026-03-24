use super::{
    DateTimeValue, EncodeFilter, EncodeOptions, EncodeToken, EncodeTokenEncoder, FilterResult,
    FunctionFilter, KeyPathNode, ListFormat, TemporalSerializer, TemporalValue, Value, encode,
    encode_comma_array,
};

fn temporal_value() -> Value {
    Value::Temporal(TemporalValue::from(
        DateTimeValue::new(2024, 1, 2, 3, 4, 5, 0, Some(0)).unwrap(),
    ))
}

#[test]
fn temporal_values_encode_in_root_nested_indexed_and_comma_positions() {
    assert_eq!(
        encode(&temporal_value(), &EncodeOptions::new().with_encode(false)).unwrap(),
        ""
    );

    assert_eq!(
        encode(
            &Value::Object([("at".to_owned(), temporal_value())].into()),
            &EncodeOptions::new().with_encode(false),
        )
        .unwrap(),
        "at=2024-01-02T03:04:05Z"
    );

    assert_eq!(
        encode(
            &Value::Object([("at".to_owned(), Value::Array(vec![temporal_value()]))].into()),
            &EncodeOptions::new().with_encode(false),
        )
        .unwrap(),
        "at[0]=2024-01-02T03:04:05Z"
    );

    assert_eq!(
        encode(
            &Value::Object(
                [(
                    "at".to_owned(),
                    Value::Array(vec![temporal_value(), temporal_value()]),
                )]
                .into(),
            ),
            &EncodeOptions::new()
                .with_encode(false)
                .with_list_format(ListFormat::Comma),
        )
        .unwrap(),
        "at=2024-01-02T03:04:05Z,2024-01-02T03:04:05Z"
    );
}

#[test]
fn temporal_serializer_none_follows_null_handling_rules() {
    let options =
        EncodeOptions::new().with_temporal_serializer(Some(TemporalSerializer::new(|_value| None)));
    assert_eq!(
        encode(
            &Value::Object([("at".to_owned(), temporal_value())].into()),
            &options.clone().with_encode(false),
        )
        .unwrap(),
        "at="
    );

    assert_eq!(
        encode(
            &Value::Object([("at".to_owned(), temporal_value())].into()),
            &options
                .clone()
                .with_encode(false)
                .with_strict_null_handling(true),
        )
        .unwrap(),
        "at"
    );

    assert_eq!(
        encode(
            &Value::Object([("at".to_owned(), temporal_value())].into()),
            &options.clone().with_skip_nulls(true),
        )
        .unwrap(),
        ""
    );

    assert_eq!(
        encode(
            &Value::Object([("at".to_owned(), Value::Array(vec![temporal_value()]))].into()),
            &options
                .with_encode(false)
                .with_list_format(ListFormat::Comma)
                .with_comma_compact_nulls(true),
        )
        .unwrap(),
        ""
    );
}

#[test]
fn temporal_serializer_none_omits_nested_and_indexed_values_when_skip_nulls() {
    let options = EncodeOptions::new()
        .with_encode(false)
        .with_skip_nulls(true)
        .with_temporal_serializer(Some(TemporalSerializer::new(|_value| None)));

    assert_eq!(
        encode(
            &Value::Object(
                [(
                    "outer".to_owned(),
                    Value::Object([("at".to_owned(), temporal_value())].into()),
                )]
                .into(),
            ),
            &options,
        )
        .unwrap(),
        ""
    );

    assert_eq!(
        encode(
            &Value::Object(
                [(
                    "at".to_owned(),
                    Value::Array(vec![temporal_value(), Value::String("kept".to_owned())]),
                )]
                .into()
            ),
            &options,
        )
        .unwrap(),
        "at[1]=kept"
    );
}

#[test]
fn temporal_serializer_none_preserves_or_compacts_mixed_comma_lists() {
    let value = Value::Object(
        [(
            "at".to_owned(),
            Value::Array(vec![
                temporal_value(),
                Value::String("kept".to_owned()),
                temporal_value(),
            ]),
        )]
        .into(),
    );
    let options = EncodeOptions::new()
        .with_encode(false)
        .with_list_format(ListFormat::Comma)
        .with_temporal_serializer(Some(TemporalSerializer::new(|_value| None)));

    assert_eq!(encode(&value, &options).unwrap(), "at=,kept,");
    assert_eq!(
        encode(&value, &options.clone().with_comma_compact_nulls(true)).unwrap(),
        "at=kept"
    );
}

#[test]
fn temporal_serializer_runs_before_the_generic_value_encoder() {
    let encoded = encode(
        &Value::Object([("at".to_owned(), temporal_value())].into()),
        &EncodeOptions::new()
            .with_encode(false)
            .with_temporal_serializer(Some(TemporalSerializer::new(|value| {
                Some(format!("t:{value}"))
            })))
            .with_encoder(Some(EncodeTokenEncoder::new(|token, _, _| match token {
                EncodeToken::Key(key) => key.to_owned(),
                EncodeToken::TextValue(text) => format!("<{text}>"),
                EncodeToken::Value(_) => "unexpected".to_owned(),
            }))),
    )
    .unwrap();

    assert_eq!(encoded, "at=<t:2024-01-02T03:04:05Z>");
}

#[test]
fn function_filters_can_replace_values_with_temporals() {
    let encoded = encode(
        &Value::Object([("at".to_owned(), Value::String("replace".to_owned()))].into()),
        &EncodeOptions::new()
            .with_encode(false)
            .with_filter(Some(EncodeFilter::Function(FunctionFilter::new(
                |prefix, _value| {
                    if prefix == "at" {
                        FilterResult::Replace(temporal_value())
                    } else {
                        FilterResult::Keep
                    }
                },
            )))),
    )
    .unwrap();

    assert_eq!(encoded, "at=2024-01-02T03:04:05Z");
}

#[test]
fn filter_replaced_temporals_that_serialize_to_none_follow_null_rules() {
    let options = EncodeOptions::new()
        .with_encode(false)
        .with_temporal_serializer(Some(TemporalSerializer::new(|_value| None)))
        .with_filter(Some(EncodeFilter::Function(FunctionFilter::new(
            |prefix, _value| {
                if prefix == "at" {
                    FilterResult::Replace(temporal_value())
                } else {
                    FilterResult::Keep
                }
            },
        ))));

    let value = Value::Object([("at".to_owned(), Value::String("replace".to_owned()))].into());

    assert_eq!(
        encode(&value, &options.clone().with_strict_null_handling(true)).unwrap(),
        "at"
    );
    assert_eq!(encode(&value, &options.with_skip_nulls(true)).unwrap(), "");
}

#[test]
fn encode_values_only_comma_lists_apply_temporal_text_per_element() {
    let parts = encode_comma_array(
        &[temporal_value(), temporal_value()],
        &KeyPathNode::from_raw("at"),
        &EncodeOptions::new()
            .with_list_format(ListFormat::Comma)
            .with_encode(false)
            .with_encode_values_only(true)
            .with_temporal_serializer(Some(TemporalSerializer::new(|value| {
                Some(format!("wrapped:{value}"))
            }))),
    );

    assert_eq!(
        parts,
        vec!["at=wrapped:2024-01-02T03:04:05Z,wrapped:2024-01-02T03:04:05Z".to_owned(),]
    );
}
