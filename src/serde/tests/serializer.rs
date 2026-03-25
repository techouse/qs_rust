use super::{
    MapKeySerializer, TEMPORAL_MARKER_NAME, TemporalCaptureSerializer, U32Seed, Value,
    ValueMapAccess, ValueMapSerializer, ValueSerializer, assert_json_error, sample_temporal,
    sample_temporal_text, to_value,
};
use crate::value::Object;
use serde::{Serialize, de, ser};

#[test]
fn internal_temporal_capture_serializer_rejects_remaining_scalar_and_container_shapes() {
    macro_rules! assert_temporal_error {
        ($($expr:expr),+ $(,)?) => {
            $(
                assert_json_error($expr, "temporal markers must serialize as ISO strings");
            )+
        };
    }

    assert_temporal_error!(
        <TemporalCaptureSerializer as ser::Serializer>::serialize_i8(TemporalCaptureSerializer, -8,),
        <TemporalCaptureSerializer as ser::Serializer>::serialize_i16(
            TemporalCaptureSerializer,
            -16,
        ),
        <TemporalCaptureSerializer as ser::Serializer>::serialize_i32(
            TemporalCaptureSerializer,
            -32,
        ),
        <TemporalCaptureSerializer as ser::Serializer>::serialize_u8(TemporalCaptureSerializer, 8,),
        <TemporalCaptureSerializer as ser::Serializer>::serialize_u16(
            TemporalCaptureSerializer,
            16,
        ),
        <TemporalCaptureSerializer as ser::Serializer>::serialize_u32(
            TemporalCaptureSerializer,
            32,
        ),
        <TemporalCaptureSerializer as ser::Serializer>::serialize_u64(
            TemporalCaptureSerializer,
            64,
        ),
        <TemporalCaptureSerializer as ser::Serializer>::serialize_f32(
            TemporalCaptureSerializer,
            1.25,
        ),
        <TemporalCaptureSerializer as ser::Serializer>::serialize_f64(
            TemporalCaptureSerializer,
            2.5,
        ),
        <TemporalCaptureSerializer as ser::Serializer>::serialize_char(
            TemporalCaptureSerializer,
            'x',
        ),
        <TemporalCaptureSerializer as ser::Serializer>::serialize_some(
            TemporalCaptureSerializer,
            &"wrapped",
        ),
        <TemporalCaptureSerializer as ser::Serializer>::serialize_unit_struct(
            TemporalCaptureSerializer,
            "UnitStruct",
        ),
        <TemporalCaptureSerializer as ser::Serializer>::serialize_unit_variant(
            TemporalCaptureSerializer,
            "Variant",
            0,
            "Unit",
        ),
        <TemporalCaptureSerializer as ser::Serializer>::serialize_newtype_struct(
            TemporalCaptureSerializer,
            "Wrapper",
            &"wrapped",
        ),
        <TemporalCaptureSerializer as ser::Serializer>::serialize_newtype_variant(
            TemporalCaptureSerializer,
            "Variant",
            0,
            "Newtype",
            &"wrapped",
        ),
        <TemporalCaptureSerializer as ser::Serializer>::serialize_tuple(
            TemporalCaptureSerializer,
            2,
        ),
        <TemporalCaptureSerializer as ser::Serializer>::serialize_tuple_struct(
            TemporalCaptureSerializer,
            "TupleStruct",
            2,
        ),
        <TemporalCaptureSerializer as ser::Serializer>::serialize_tuple_variant(
            TemporalCaptureSerializer,
            "Variant",
            0,
            "Tuple",
            2,
        ),
        <TemporalCaptureSerializer as ser::Serializer>::serialize_struct(
            TemporalCaptureSerializer,
            "Struct",
            1,
        ),
        <TemporalCaptureSerializer as ser::Serializer>::serialize_struct_variant(
            TemporalCaptureSerializer,
            "Variant",
            0,
            "Struct",
            1,
        )
    );
}

#[test]
fn internal_scalar_serializers_cover_small_numeric_and_optional_paths() {
    assert_eq!(
        <ValueSerializer as ser::Serializer>::serialize_i8(ValueSerializer, -8).unwrap(),
        Value::I64(-8)
    );
    assert_eq!(
        <ValueSerializer as ser::Serializer>::serialize_i16(ValueSerializer, -16).unwrap(),
        Value::I64(-16)
    );
    assert_eq!(
        <ValueSerializer as ser::Serializer>::serialize_i64(ValueSerializer, -64).unwrap(),
        Value::I64(-64)
    );
    assert_eq!(
        <ValueSerializer as ser::Serializer>::serialize_u16(ValueSerializer, 16).unwrap(),
        Value::U64(16)
    );
    assert_eq!(
        <ValueSerializer as ser::Serializer>::serialize_u64(ValueSerializer, 64).unwrap(),
        Value::U64(64)
    );
    assert_eq!(
        <ValueSerializer as ser::Serializer>::serialize_f32(ValueSerializer, 1.25).unwrap(),
        Value::F64(1.25)
    );
    assert_eq!(
        <ValueSerializer as ser::Serializer>::serialize_some(ValueSerializer, &7u8).unwrap(),
        Value::U64(7)
    );
    assert_eq!(
        <ValueSerializer as ser::Serializer>::serialize_newtype_struct(
            ValueSerializer,
            "Wrapper",
            &7u8,
        )
        .unwrap(),
        Value::U64(7)
    );

    assert_eq!(
        <MapKeySerializer as ser::Serializer>::serialize_i8(MapKeySerializer, -8).unwrap(),
        "-8".to_owned()
    );
    assert_eq!(
        <MapKeySerializer as ser::Serializer>::serialize_i16(MapKeySerializer, -16).unwrap(),
        "-16".to_owned()
    );
    assert_eq!(
        <MapKeySerializer as ser::Serializer>::serialize_u16(MapKeySerializer, 16).unwrap(),
        "16".to_owned()
    );
    assert_eq!(
        <MapKeySerializer as ser::Serializer>::serialize_u32(MapKeySerializer, 32).unwrap(),
        "32".to_owned()
    );
    assert_eq!(
        <MapKeySerializer as ser::Serializer>::serialize_f32(MapKeySerializer, 1.25).unwrap(),
        "1.25".to_owned()
    );
    assert_eq!(
        <MapKeySerializer as ser::Serializer>::serialize_some(MapKeySerializer, &7u8).unwrap(),
        "7".to_owned()
    );
}

#[test]
fn internal_serializers_cover_marker_bytes_keys_and_error_paths() {
    #[derive(Debug)]
    struct MarkedTemporal;

    impl Serialize for MarkedTemporal {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_newtype_struct(TEMPORAL_MARKER_NAME, &sample_temporal_text())
        }
    }

    #[derive(Debug)]
    struct InvalidMarkedTemporal;

    impl Serialize for InvalidMarkedTemporal {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_newtype_struct(TEMPORAL_MARKER_NAME, &true)
        }
    }

    assert_eq!(
        to_value(&MarkedTemporal).unwrap(),
        Value::Temporal(sample_temporal())
    );
    let marker_err = to_value(&InvalidMarkedTemporal).unwrap_err();
    assert!(
        marker_err
            .to_string()
            .contains("temporal markers must serialize as ISO strings")
    );

    assert_eq!(
        <ValueSerializer as ser::Serializer>::serialize_bytes(ValueSerializer, &[0x41, 0xFF])
            .unwrap(),
        Value::Bytes(vec![0x41, 0xFF])
    );
    assert_json_error(
        <ValueSerializer as ser::Serializer>::serialize_f64(ValueSerializer, f64::INFINITY),
        "cannot serialize non-finite floats",
    );
    assert_eq!(
        <TemporalCaptureSerializer as ser::Serializer>::serialize_str(
            TemporalCaptureSerializer,
            &sample_temporal_text(),
        )
        .unwrap(),
        sample_temporal()
    );
    assert_json_error(
        <TemporalCaptureSerializer as ser::Serializer>::serialize_str(
            TemporalCaptureSerializer,
            "not-a-datetime",
        ),
        "invalid datetime format",
    );
    assert_json_error(
        <TemporalCaptureSerializer as ser::Serializer>::serialize_unit(TemporalCaptureSerializer),
        "temporal markers must serialize as ISO strings",
    );
    assert_json_error(
        <TemporalCaptureSerializer as ser::Serializer>::serialize_bool(
            TemporalCaptureSerializer,
            true,
        ),
        "temporal markers must serialize as ISO strings",
    );
    assert_json_error(
        <TemporalCaptureSerializer as ser::Serializer>::serialize_i64(TemporalCaptureSerializer, 1),
        "temporal markers must serialize as ISO strings",
    );
    assert_json_error(
        <TemporalCaptureSerializer as ser::Serializer>::serialize_bytes(
            TemporalCaptureSerializer,
            &[1, 2],
        ),
        "temporal markers must serialize as ISO strings",
    );
    assert_json_error(
        <TemporalCaptureSerializer as ser::Serializer>::serialize_none(TemporalCaptureSerializer),
        "temporal markers must serialize as ISO strings",
    );
    assert_json_error(
        <TemporalCaptureSerializer as ser::Serializer>::serialize_seq(
            TemporalCaptureSerializer,
            Some(1),
        ),
        "temporal markers must serialize as ISO strings",
    );
    assert_json_error(
        <TemporalCaptureSerializer as ser::Serializer>::serialize_map(
            TemporalCaptureSerializer,
            Some(1),
        ),
        "temporal markers must serialize as ISO strings",
    );

    assert_eq!(
        <MapKeySerializer as ser::Serializer>::serialize_str(MapKeySerializer, "field").unwrap(),
        "field".to_owned()
    );
    assert_eq!(
        <MapKeySerializer as ser::Serializer>::serialize_bool(MapKeySerializer, true).unwrap(),
        "true".to_owned()
    );
    assert_eq!(
        <MapKeySerializer as ser::Serializer>::serialize_i64(MapKeySerializer, -7).unwrap(),
        "-7".to_owned()
    );
    assert_eq!(
        <MapKeySerializer as ser::Serializer>::serialize_u64(MapKeySerializer, 9).unwrap(),
        "9".to_owned()
    );
    assert_eq!(
        <MapKeySerializer as ser::Serializer>::serialize_f64(MapKeySerializer, 1.5).unwrap(),
        "1.5".to_owned()
    );
    assert_eq!(
        <MapKeySerializer as ser::Serializer>::serialize_char(MapKeySerializer, 'x').unwrap(),
        "x".to_owned()
    );
    assert_eq!(
        <MapKeySerializer as ser::Serializer>::serialize_unit_variant(
            MapKeySerializer,
            "Variant",
            0,
            "Unit",
        )
        .unwrap(),
        "Unit".to_owned()
    );
    assert_eq!(
        <MapKeySerializer as ser::Serializer>::serialize_newtype_struct(
            MapKeySerializer,
            "Key",
            &7u8,
        )
        .unwrap(),
        "7".to_owned()
    );
    assert_json_error(
        <MapKeySerializer as ser::Serializer>::serialize_none(MapKeySerializer),
        "map keys must not be null",
    );
    assert_json_error(
        <MapKeySerializer as ser::Serializer>::serialize_unit(MapKeySerializer),
        "map keys must not be unit values",
    );
    assert_json_error(
        <MapKeySerializer as ser::Serializer>::serialize_unit_struct(
            MapKeySerializer,
            "UnitStruct",
        ),
        "map keys must not be unit values",
    );
    assert_json_error(
        <MapKeySerializer as ser::Serializer>::serialize_bytes(MapKeySerializer, &[1, 2]),
        "map keys must be strings or scalar values",
    );
    assert_json_error(
        <MapKeySerializer as ser::Serializer>::serialize_newtype_variant(
            MapKeySerializer,
            "Variant",
            0,
            "Key",
            &7u8,
        ),
        "map keys must be strings or scalar values",
    );
    assert_json_error(
        <MapKeySerializer as ser::Serializer>::serialize_seq(MapKeySerializer, Some(1)),
        "map keys must be strings or scalar values",
    );
    assert_json_error(
        <MapKeySerializer as ser::Serializer>::serialize_tuple(MapKeySerializer, 1),
        "map keys must be strings or scalar values",
    );
    assert_json_error(
        <MapKeySerializer as ser::Serializer>::serialize_tuple_struct(
            MapKeySerializer,
            "TupleStruct",
            1,
        ),
        "map keys must be strings or scalar values",
    );
    assert_json_error(
        <MapKeySerializer as ser::Serializer>::serialize_tuple_variant(
            MapKeySerializer,
            "Variant",
            0,
            "Tuple",
            1,
        ),
        "map keys must be strings or scalar values",
    );
    assert_json_error(
        <MapKeySerializer as ser::Serializer>::serialize_map(MapKeySerializer, Some(1)),
        "map keys must be strings or scalar values",
    );
    assert_json_error(
        <MapKeySerializer as ser::Serializer>::serialize_struct(MapKeySerializer, "Struct", 1),
        "map keys must be strings or scalar values",
    );
    assert_json_error(
        <MapKeySerializer as ser::Serializer>::serialize_struct_variant(
            MapKeySerializer,
            "Variant",
            0,
            "Struct",
            1,
        ),
        "map keys must be strings or scalar values",
    );

    let mut map = ValueMapSerializer::new(1);
    assert_json_error(
        ser::SerializeMap::serialize_value(&mut map, &1u8),
        "serialize_value called before serialize_key",
    );

    let empty_object: Object = Default::default();
    let mut access = ValueMapAccess::new(&empty_object);
    assert_json_error(
        de::MapAccess::next_value_seed(&mut access, U32Seed),
        "missing map value for previously deserialized key",
    );
}
