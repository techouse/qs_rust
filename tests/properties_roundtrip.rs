use proptest::prelude::*;
use qs_rust::{DecodeOptions, EncodeOptions, Value, decode, encode};

fn roundtrip_key_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("a".to_owned()),
        Just("b".to_owned()),
        Just("user".to_owned()),
        Just("tags".to_owned()),
        Just("name".to_owned()),
        Just("meta".to_owned()),
    ]
}

fn roundtrip_leaf_strategy() -> impl Strategy<Value = Value> {
    prop_oneof![
        Just(Value::Null),
        Just(Value::String(String::new())),
        Just(Value::String("x".to_owned())),
        Just(Value::String("x y".to_owned())),
        Just(Value::String("name".to_owned())),
    ]
}

fn roundtrip_value_strategy() -> impl Strategy<Value = Value> {
    roundtrip_leaf_strategy().prop_recursive(4, 96, 8, |inner| {
        prop_oneof![
            prop::collection::vec(roundtrip_leaf_strategy(), 0..4).prop_map(Value::Array),
            prop::collection::vec((roundtrip_key_strategy(), inner), 1..4)
                .prop_map(|entries| Value::Object(entries.into_iter().collect())),
        ]
    })
}

fn roundtrip_object_strategy() -> impl Strategy<Value = Value> {
    prop::collection::vec((roundtrip_key_strategy(), roundtrip_value_strategy()), 0..4)
        .prop_map(|entries| Value::Object(entries.into_iter().collect()))
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 72,
        .. ProptestConfig::default()
    })]

    #[test]
    fn string_null_container_values_round_trip_through_encode_and_decode(value in roundtrip_object_strategy()) {
        let encode_options = EncodeOptions::new()
            .with_allow_empty_lists(true)
            .with_strict_null_handling(true);
        let decode_options = DecodeOptions::new()
            .with_allow_empty_lists(true)
            .with_strict_null_handling(true);

        let encoded = encode(&value, &encode_options).unwrap();
        let decoded = decode(&encoded, &decode_options).unwrap();
        prop_assert_eq!(Value::Object(decoded), value);
    }
}
