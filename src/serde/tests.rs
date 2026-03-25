use super::deserializer::{ValueDeserializer, ValueEnumAccess, ValueMapAccess, ValueSeqAccess};
use super::serializer::{
    MapKeySerializer, TemporalCaptureSerializer, ValueMapSerializer, ValueSerializer,
};
use super::temporal::TEMPORAL_MARKER_NAME;
use super::{from_str, from_value, to_string, to_value};
use crate::{DateTimeValue, DecodeOptions, EncodeOptions, TemporalValue, Value};
use serde::de::{
    DeserializeSeed, EnumAccess, IgnoredAny, MapAccess, SeqAccess, VariantAccess, Visitor,
};
use serde::{Deserialize, Serialize, de, ser};
use std::collections::BTreeMap;
use std::fmt;

fn sample_temporal() -> TemporalValue {
    TemporalValue::from(DateTimeValue::new(2024, 1, 2, 3, 4, 5, 0, Some(0)).unwrap())
}

fn sample_temporal_text() -> String {
    sample_temporal().to_string()
}

fn assert_json_error<T>(result: Result<T, serde_json::Error>, needle: &str) {
    let error = match result {
        Ok(_) => panic!("expected serde_json::Error containing {needle:?}"),
        Err(error) => error,
    };
    assert!(
        error.to_string().contains(needle),
        "expected error containing {needle:?}, got {error}"
    );
}

struct U32Seed;

impl<'de> DeserializeSeed<'de> for U32Seed {
    type Value = u32;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        u32::deserialize(deserializer)
    }
}

struct StringSeed;

impl<'de> DeserializeSeed<'de> for StringSeed {
    type Value = String;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        String::deserialize(deserializer)
    }
}

struct AnySummaryVisitor;

impl<'de> Visitor<'de> for AnySummaryVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("any qs value")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok("unit".to_owned())
    }

    fn visit_byte_buf<E>(self, value: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(format!("bytes:{value:?}"))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut len = 0usize;
        while seq.next_element::<IgnoredAny>()?.is_some() {
            len += 1;
        }
        Ok(format!("seq:{len}"))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut keys = Vec::new();
        while let Some(key) = map.next_key::<String>()? {
            let _: IgnoredAny = map.next_value()?;
            keys.push(key);
        }
        Ok(format!("map:{}", keys.join(",")))
    }
}

#[test]
fn direct_bridge_round_trips_non_temporal_values() {
    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct Query {
        page: String,
        tags: Vec<String>,
    }

    let value = to_value(&Query {
        page: "2".to_owned(),
        tags: vec!["rust".to_owned(), "qs".to_owned()],
    })
    .unwrap();

    assert_eq!(
        value,
        Value::Object(
            [
                ("page".to_owned(), Value::String("2".to_owned())),
                (
                    "tags".to_owned(),
                    Value::Array(vec![
                        Value::String("rust".to_owned()),
                        Value::String("qs".to_owned())
                    ])
                ),
            ]
            .into()
        )
    );

    let decoded: Query = from_value(&value).unwrap();
    assert_eq!(
        decoded,
        Query {
            page: "2".to_owned(),
            tags: vec!["rust".to_owned(), "qs".to_owned()],
        }
    );
}

#[test]
fn direct_bridge_stringifies_temporal_values_for_plain_fields() {
    let decoded: String = from_value(&Value::Temporal(sample_temporal())).unwrap();
    assert_eq!(decoded, "2024-01-02T03:04:05Z");
}

#[test]
fn serde_bridge_query_string_helpers_round_trip_structs() {
    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct Query {
        page: String,
        tags: Vec<String>,
    }

    let query = Query {
        page: "2".to_owned(),
        tags: vec!["rust".to_owned(), "qs".to_owned()],
    };

    let encoded = to_string(&query, &EncodeOptions::new().with_encode(false)).unwrap();
    assert_eq!(encoded, "page=2&tags[0]=rust&tags[1]=qs");

    let decoded: Query = from_str(&encoded, &DecodeOptions::new()).unwrap();
    assert_eq!(decoded, query);
}

#[test]
fn direct_bridge_serializes_compound_shapes_and_variants() {
    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct UnitStruct;

    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct NewtypeStruct(String);

    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct TupleStruct(i32, bool);

    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    enum Variant {
        Unit,
        Newtype(String),
        Tuple(i32, bool),
        Struct { answer: u8 },
    }

    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct Query {
        boolean: bool,
        signed: i32,
        unsigned: u32,
        float: f64,
        character: char,
        optional_some: Option<String>,
        optional_none: Option<String>,
        unit: (),
        unit_struct: UnitStruct,
        tuple: (i32, bool),
        tuple_struct: TupleStruct,
        newtype_struct: NewtypeStruct,
        numbers: Vec<u32>,
        labels: BTreeMap<i32, String>,
        unit_variant: Variant,
        newtype_variant: Variant,
        tuple_variant: Variant,
        struct_variant: Variant,
    }

    let query = Query {
        boolean: true,
        signed: -7,
        unsigned: 9,
        float: 1.5,
        character: 'x',
        optional_some: Some("present".to_owned()),
        optional_none: None,
        unit: (),
        unit_struct: UnitStruct,
        tuple: (4, false),
        tuple_struct: TupleStruct(5, true),
        newtype_struct: NewtypeStruct("wrapped".to_owned()),
        numbers: vec![1, 2],
        labels: BTreeMap::from([(1, "one".to_owned()), (2, "two".to_owned())]),
        unit_variant: Variant::Unit,
        newtype_variant: Variant::Newtype("payload".to_owned()),
        tuple_variant: Variant::Tuple(8, true),
        struct_variant: Variant::Struct { answer: 42 },
    };

    let value = to_value(&query).unwrap();
    let Value::Object(object) = value else {
        panic!("expected object")
    };

    assert_eq!(object.get("boolean"), Some(&Value::Bool(true)));
    assert_eq!(object.get("signed"), Some(&Value::I64(-7)));
    assert_eq!(object.get("unsigned"), Some(&Value::U64(9)));
    assert_eq!(object.get("float"), Some(&Value::F64(1.5)));
    assert_eq!(
        object.get("character"),
        Some(&Value::String("x".to_owned()))
    );
    assert_eq!(
        object.get("optional_some"),
        Some(&Value::String("present".to_owned()))
    );
    assert_eq!(object.get("optional_none"), Some(&Value::Null));
    assert_eq!(object.get("unit"), Some(&Value::Null));
    assert_eq!(object.get("unit_struct"), Some(&Value::Null));
    assert_eq!(
        object.get("tuple"),
        Some(&Value::Array(vec![Value::I64(4), Value::Bool(false)]))
    );
    assert_eq!(
        object.get("tuple_struct"),
        Some(&Value::Array(vec![Value::I64(5), Value::Bool(true)]))
    );
    assert_eq!(
        object.get("newtype_struct"),
        Some(&Value::String("wrapped".to_owned()))
    );
    assert_eq!(
        object.get("numbers"),
        Some(&Value::Array(vec![Value::U64(1), Value::U64(2)]))
    );
    assert_eq!(
        object.get("labels"),
        Some(&Value::Object(
            [
                ("1".to_owned(), Value::String("one".to_owned())),
                ("2".to_owned(), Value::String("two".to_owned())),
            ]
            .into()
        ))
    );
    assert_eq!(
        object.get("unit_variant"),
        Some(&Value::String("Unit".to_owned()))
    );
    assert_eq!(
        object.get("newtype_variant"),
        Some(&Value::Object(
            [("Newtype".to_owned(), Value::String("payload".to_owned()),)].into()
        ))
    );
    assert_eq!(
        object.get("tuple_variant"),
        Some(&Value::Object(
            [(
                "Tuple".to_owned(),
                Value::Array(vec![Value::I64(8), Value::Bool(true)]),
            )]
            .into()
        ))
    );
    assert_eq!(
        object.get("struct_variant"),
        Some(&Value::Object(
            [(
                "Struct".to_owned(),
                Value::Object([("answer".to_owned(), Value::U64(42))].into()),
            )]
            .into()
        ))
    );
}

#[test]
fn direct_bridge_deserializes_scalars_sequences_enums_and_identifiers() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct NewtypeStruct(String);

    #[derive(Debug, PartialEq, Deserialize)]
    struct TupleStruct(i32, bool);

    #[derive(Debug, PartialEq)]
    struct Identifier(String);

    impl<'de> Deserialize<'de> for Identifier {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            struct IdentifierVisitor;

            impl<'de> Visitor<'de> for IdentifierVisitor {
                type Value = Identifier;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("an identifier string")
                }

                fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(Identifier(value.to_owned()))
                }

                fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(Identifier(value))
                }
            }

            deserializer.deserialize_identifier(IdentifierVisitor)
        }
    }

    #[derive(Debug, PartialEq, Deserialize)]
    enum Variant {
        Unit,
        Newtype(String),
        Tuple(i32, bool),
        Struct {
            answer: u8,
        },
        #[serde(rename = "2024-01-02T03:04:05Z")]
        Snapshot,
    }

    assert!(from_value::<bool>(&Value::Bool(true)).unwrap());
    assert_eq!(from_value::<i64>(&Value::I64(-7)).unwrap(), -7);
    assert_eq!(from_value::<u64>(&Value::U64(9)).unwrap(), 9);
    assert_eq!(from_value::<f64>(&Value::F64(1.5)).unwrap(), 1.5);
    assert_eq!(
        from_value::<char>(&Value::String("x".to_owned())).unwrap(),
        'x'
    );
    assert_eq!(
        from_value::<String>(&Value::Temporal(sample_temporal())).unwrap(),
        sample_temporal_text()
    );
    assert_eq!(
        from_value::<Vec<String>>(&Value::Array(vec![
            Value::String("a".to_owned()),
            Value::String("b".to_owned()),
        ]))
        .unwrap(),
        vec!["a".to_owned(), "b".to_owned()]
    );
    assert_eq!(
        from_value::<(u8, u8, u8)>(&Value::Bytes(vec![1, 2, 3])).unwrap(),
        (1, 2, 3)
    );
    assert_eq!(
        from_value::<TupleStruct>(&Value::Array(vec![Value::I64(7), Value::Bool(true)])).unwrap(),
        TupleStruct(7, true)
    );
    assert_eq!(
        from_value::<NewtypeStruct>(&Value::String("wrapped".to_owned())).unwrap(),
        NewtypeStruct("wrapped".to_owned())
    );
    assert_eq!(from_value::<Option<String>>(&Value::Null).unwrap(), None);
    assert_eq!(
        from_value::<Option<String>>(&Value::String("present".to_owned())).unwrap(),
        Some("present".to_owned())
    );
    assert_eq!(
        from_value::<Variant>(&Value::String("Unit".to_owned())).unwrap(),
        Variant::Unit
    );
    assert_eq!(
        from_value::<Variant>(&Value::Object([("Unit".to_owned(), Value::Null)].into(),)).unwrap(),
        Variant::Unit
    );
    assert_eq!(
        from_value::<Variant>(&Value::Temporal(sample_temporal())).unwrap(),
        Variant::Snapshot
    );
    assert_eq!(
        from_value::<Variant>(&Value::Object(
            [("Newtype".to_owned(), Value::String("payload".to_owned()),)].into(),
        ))
        .unwrap(),
        Variant::Newtype("payload".to_owned())
    );
    assert_eq!(
        from_value::<Variant>(&Value::Object(
            [(
                "Tuple".to_owned(),
                Value::Array(vec![Value::I64(8), Value::Bool(true)]),
            )]
            .into(),
        ))
        .unwrap(),
        Variant::Tuple(8, true)
    );
    assert_eq!(
        from_value::<Variant>(&Value::Object(
            [(
                "Struct".to_owned(),
                Value::Object([("answer".to_owned(), Value::U64(42))].into()),
            )]
            .into(),
        ))
        .unwrap(),
        Variant::Struct { answer: 42 }
    );
    assert_eq!(
        from_value::<Identifier>(&Value::String("field".to_owned())).unwrap(),
        Identifier("field".to_owned())
    );
    assert_eq!(
        from_value::<Identifier>(&Value::Temporal(sample_temporal())).unwrap(),
        Identifier(sample_temporal_text())
    );
    let _: IgnoredAny = from_value(&Value::Object(
        [("ignored".to_owned(), Value::String("value".to_owned()))].into(),
    ))
    .unwrap();

    let unit_err = from_value::<()>(&Value::String("x".to_owned())).unwrap_err();
    assert!(unit_err.to_string().contains("expected unit value"));

    let seq_err = from_value::<Vec<String>>(&Value::String("x".to_owned())).unwrap_err();
    assert!(seq_err.to_string().contains("expected sequence value"));

    let map_err =
        from_value::<BTreeMap<String, String>>(&Value::String("x".to_owned())).unwrap_err();
    assert!(map_err.to_string().contains("expected map value"));

    let identifier_err = from_value::<Identifier>(&Value::Bool(true)).unwrap_err();
    assert!(
        identifier_err
            .to_string()
            .contains("expected identifier string")
    );

    let enum_repr_err = from_value::<Variant>(&Value::Object(
        [
            ("Unit".to_owned(), Value::Null),
            ("Newtype".to_owned(), Value::String("payload".to_owned())),
        ]
        .into(),
    ))
    .unwrap_err();
    assert!(
        enum_repr_err
            .to_string()
            .contains("expected enum representation")
    );

    let unit_variant_err = from_value::<Variant>(&Value::Object(
        [("Unit".to_owned(), Value::String("payload".to_owned()))].into(),
    ))
    .unwrap_err();
    assert!(
        unit_variant_err
            .to_string()
            .contains("expected unit variant")
    );

    let newtype_payload_err =
        from_value::<Variant>(&Value::String("Newtype".to_owned())).unwrap_err();
    assert!(
        newtype_payload_err
            .to_string()
            .contains("expected newtype variant payload")
    );

    let tuple_payload_err = from_value::<Variant>(&Value::String("Tuple".to_owned())).unwrap_err();
    assert!(
        tuple_payload_err
            .to_string()
            .contains("expected tuple variant payload")
    );

    let struct_payload_err =
        from_value::<Variant>(&Value::String("Struct".to_owned())).unwrap_err();
    assert!(
        struct_payload_err
            .to_string()
            .contains("expected struct variant payload")
    );
}

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
fn internal_deserializers_cover_any_seq_map_and_unit_struct_paths() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct UnitStruct;

    let array = Value::Array(vec![Value::U64(1), Value::U64(2)]);
    let bytes = Value::Bytes(vec![1, 2]);
    let object = Value::Object([("field".to_owned(), Value::String("value".to_owned()))].into());

    assert_eq!(
        de::Deserializer::deserialize_any(ValueDeserializer::new(&Value::Null), AnySummaryVisitor)
            .unwrap(),
        "unit"
    );
    assert_eq!(
        de::Deserializer::deserialize_any(ValueDeserializer::new(&array), AnySummaryVisitor)
            .unwrap(),
        "seq:2"
    );
    assert_eq!(
        de::Deserializer::deserialize_any(ValueDeserializer::new(&bytes), AnySummaryVisitor)
            .unwrap(),
        "bytes:[1, 2]"
    );
    assert_eq!(
        de::Deserializer::deserialize_any(ValueDeserializer::new(&object), AnySummaryVisitor)
            .unwrap(),
        "map:field"
    );
    assert_eq!(from_value::<UnitStruct>(&Value::Null).unwrap(), UnitStruct);

    let value_items = [Value::U64(7)];
    let mut value_access = ValueSeqAccess::from_values(&value_items);
    assert_eq!(
        SeqAccess::next_element_seed(&mut value_access, U32Seed).unwrap(),
        Some(7)
    );
    assert_eq!(
        SeqAccess::next_element_seed(&mut value_access, U32Seed).unwrap(),
        None
    );

    let byte_items = [9u8];
    let mut byte_access = ValueSeqAccess::from_bytes(&byte_items);
    assert_eq!(
        SeqAccess::next_element_seed(&mut byte_access, U32Seed).unwrap(),
        Some(9)
    );
    assert_eq!(
        SeqAccess::next_element_seed(&mut byte_access, U32Seed).unwrap(),
        None
    );

    let entries = [("answer".to_owned(), Value::U64(42))].into();
    let mut map_access = ValueMapAccess::new(&entries);
    assert_eq!(
        MapAccess::next_key_seed(&mut map_access, StringSeed).unwrap(),
        Some("answer".to_owned())
    );
    assert_eq!(
        MapAccess::next_value_seed(&mut map_access, U32Seed).unwrap(),
        42
    );
    assert_eq!(
        MapAccess::next_key_seed(&mut map_access, StringSeed).unwrap(),
        None
    );
}

#[test]
fn internal_enum_accessors_cover_owned_and_borrowed_unit_variants() {
    let (borrowed_name, borrowed_variant) =
        EnumAccess::variant_seed(ValueEnumAccess::unit("Unit"), StringSeed).unwrap();
    assert_eq!(borrowed_name, "Unit".to_owned());
    VariantAccess::unit_variant(borrowed_variant).unwrap();

    let (owned_name, owned_variant) = EnumAccess::variant_seed(
        ValueEnumAccess::owned_unit("Snapshot".to_owned()),
        StringSeed,
    )
    .unwrap();
    assert_eq!(owned_name, "Snapshot".to_owned());
    VariantAccess::unit_variant(owned_variant).unwrap();
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

    let empty_object: crate::value::Object = Default::default();
    let mut access = ValueMapAccess::new(&empty_object);
    assert_json_error(
        de::MapAccess::next_value_seed(&mut access, U32Seed),
        "missing map value for previously deserialized key",
    );
}

#[cfg(feature = "chrono")]
#[test]
fn chrono_naive_temporal_field_helpers_round_trip_directly() {
    let naive = chrono::NaiveDate::from_ymd_opt(2024, 1, 2)
        .unwrap()
        .and_hms_opt(3, 4, 5)
        .unwrap();

    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct Query {
        #[serde(with = "crate::serde::temporal::chrono_naive_datetime")]
        at: chrono::NaiveDateTime,
    }

    let query = Query { at: naive };
    let value = to_value(&query).unwrap();
    assert_eq!(
        value,
        Value::Object(
            [(
                "at".to_owned(),
                Value::Temporal(TemporalValue::from(query.at))
            )]
            .into()
        )
    );

    let decoded: Query = from_value(&value).unwrap();
    assert_eq!(decoded, query);
    assert_eq!(
        crate::serde::temporal::chrono_naive_datetime::serialize(&naive, ValueSerializer).unwrap(),
        Value::Temporal(TemporalValue::from(naive))
    );
    assert_eq!(
        crate::serde::temporal::chrono_naive_datetime::deserialize(ValueDeserializer::new(
            &Value::Temporal(TemporalValue::from(naive)),
        ))
        .unwrap(),
        naive
    );
}

#[cfg(feature = "chrono")]
#[test]
fn chrono_temporal_field_helpers_preserve_temporal_leaves() {
    use chrono::{FixedOffset, TimeZone};

    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct Query {
        #[serde(with = "crate::serde::temporal::chrono_datetime")]
        at: chrono::DateTime<chrono::FixedOffset>,
    }

    let query = Query {
        at: FixedOffset::east_opt(3_600)
            .unwrap()
            .with_ymd_and_hms(2024, 1, 2, 3, 4, 5)
            .unwrap(),
    };

    let value = to_value(&query).unwrap();
    assert_eq!(
        value,
        Value::Object(
            [(
                "at".to_owned(),
                Value::Temporal(TemporalValue::from(query.at))
            )]
            .into()
        )
    );

    let decoded: Query = from_value(&value).unwrap();
    assert_eq!(decoded, query);

    let decoded_from_string: Query = from_value(&Value::Object(
        [(
            "at".to_owned(),
            Value::String("2024-01-02T03:04:05+01:00".to_owned()),
        )]
        .into(),
    ))
    .unwrap();
    assert_eq!(decoded_from_string, query);
}

#[cfg(feature = "chrono")]
#[test]
fn chrono_temporal_field_helpers_reject_mismatched_and_invalid_strings() {
    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct AwareQuery {
        #[serde(with = "crate::serde::temporal::chrono_datetime")]
        at: chrono::DateTime<chrono::FixedOffset>,
    }

    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct NaiveQuery {
        #[serde(with = "crate::serde::temporal::chrono_naive_datetime")]
        at: chrono::NaiveDateTime,
    }

    let aware_err = from_value::<AwareQuery>(&Value::Object(
        [(
            "at".to_owned(),
            Value::String("2024-01-02T03:04:05".to_owned()),
        )]
        .into(),
    ))
    .unwrap_err();
    assert!(aware_err.to_string().contains("missing a UTC offset"));

    let naive_err = from_value::<NaiveQuery>(&Value::Object(
        [(
            "at".to_owned(),
            Value::String("2024-01-02T03:04:05+01:00".to_owned()),
        )]
        .into(),
    ))
    .unwrap_err();
    assert!(
        naive_err
            .to_string()
            .contains("unexpectedly contains a UTC offset")
    );

    let invalid_err = from_value::<AwareQuery>(&Value::Object(
        [("at".to_owned(), Value::String("not-a-datetime".to_owned()))].into(),
    ))
    .unwrap_err();
    assert!(invalid_err.to_string().contains("invalid datetime format"));
}

#[cfg(feature = "chrono")]
#[test]
fn chrono_temporal_field_helpers_round_trip_nested_structs() {
    use chrono::{FixedOffset, TimeZone};

    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct Inner {
        #[serde(with = "crate::serde::temporal::chrono_datetime")]
        at: chrono::DateTime<chrono::FixedOffset>,
    }

    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct Outer {
        inner: Inner,
    }

    let query = Outer {
        inner: Inner {
            at: FixedOffset::east_opt(3_600)
                .unwrap()
                .with_ymd_and_hms(2024, 1, 2, 3, 4, 5)
                .unwrap(),
        },
    };

    let value = to_value(&query).unwrap();
    assert_eq!(
        value,
        Value::Object(
            [(
                "inner".to_owned(),
                Value::Object(
                    [(
                        "at".to_owned(),
                        Value::Temporal(TemporalValue::from(query.inner.at))
                    )]
                    .into()
                ),
            )]
            .into()
        )
    );

    let decoded: Outer = from_value(&value).unwrap();
    assert_eq!(decoded, query);
}

#[cfg(feature = "time")]
#[test]
fn time_primitive_temporal_field_helpers_round_trip_directly() {
    use time::{Date, Month};

    let primitive = Date::from_calendar_date(2024, Month::January, 2)
        .unwrap()
        .with_hms(3, 4, 5)
        .unwrap();

    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct Query {
        #[serde(with = "crate::serde::temporal::time_primitive_datetime")]
        at: time::PrimitiveDateTime,
    }

    let query = Query { at: primitive };
    let value = to_value(&query).unwrap();
    assert_eq!(
        value,
        Value::Object(
            [(
                "at".to_owned(),
                Value::Temporal(TemporalValue::from(query.at))
            )]
            .into()
        )
    );

    let decoded: Query = from_value(&value).unwrap();
    assert_eq!(decoded, query);
    assert_eq!(
        crate::serde::temporal::time_primitive_datetime::serialize(&primitive, ValueSerializer)
            .unwrap(),
        Value::Temporal(TemporalValue::from(primitive))
    );
    assert_eq!(
        crate::serde::temporal::time_primitive_datetime::deserialize(ValueDeserializer::new(
            &Value::Temporal(TemporalValue::from(primitive)),
        ))
        .unwrap(),
        primitive
    );
}

#[cfg(feature = "time")]
#[test]
fn time_temporal_field_helpers_preserve_temporal_leaves() {
    use time::{Date, Month, UtcOffset};

    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct Query {
        #[serde(with = "crate::serde::temporal::time_offset_datetime")]
        at: time::OffsetDateTime,
    }

    let query = Query {
        at: Date::from_calendar_date(2024, Month::January, 2)
            .unwrap()
            .with_hms(3, 4, 5)
            .unwrap()
            .assume_offset(UtcOffset::from_hms(1, 0, 0).unwrap()),
    };

    let value = to_value(&query).unwrap();
    assert_eq!(
        value,
        Value::Object(
            [(
                "at".to_owned(),
                Value::Temporal(TemporalValue::from(query.at))
            )]
            .into()
        )
    );

    let decoded: Query = from_value(&value).unwrap();
    assert_eq!(decoded, query);

    let decoded_from_string: Query = from_value(&Value::Object(
        [(
            "at".to_owned(),
            Value::String("2024-01-02T03:04:05+01:00".to_owned()),
        )]
        .into(),
    ))
    .unwrap();
    assert_eq!(decoded_from_string, query);
}

#[cfg(feature = "time")]
#[test]
fn time_temporal_field_helpers_reject_mismatched_and_invalid_strings() {
    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct AwareQuery {
        #[serde(with = "crate::serde::temporal::time_offset_datetime")]
        at: time::OffsetDateTime,
    }

    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct NaiveQuery {
        #[serde(with = "crate::serde::temporal::time_primitive_datetime")]
        at: time::PrimitiveDateTime,
    }

    let aware_err = from_value::<AwareQuery>(&Value::Object(
        [(
            "at".to_owned(),
            Value::String("2024-01-02T03:04:05".to_owned()),
        )]
        .into(),
    ))
    .unwrap_err();
    assert!(aware_err.to_string().contains("missing a UTC offset"));

    let naive_err = from_value::<NaiveQuery>(&Value::Object(
        [(
            "at".to_owned(),
            Value::String("2024-01-02T03:04:05+01:00".to_owned()),
        )]
        .into(),
    ))
    .unwrap_err();
    assert!(
        naive_err
            .to_string()
            .contains("unexpectedly contains a UTC offset")
    );

    let invalid_err = from_value::<AwareQuery>(&Value::Object(
        [("at".to_owned(), Value::String("not-a-datetime".to_owned()))].into(),
    ))
    .unwrap_err();
    assert!(invalid_err.to_string().contains("invalid datetime format"));
}

#[cfg(feature = "time")]
#[test]
fn time_temporal_field_helpers_round_trip_nested_structs() {
    use time::{Date, Month, UtcOffset};

    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct Inner {
        #[serde(with = "crate::serde::temporal::time_offset_datetime")]
        at: time::OffsetDateTime,
    }

    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct Outer {
        inner: Inner,
    }

    let query = Outer {
        inner: Inner {
            at: Date::from_calendar_date(2024, Month::January, 2)
                .unwrap()
                .with_hms(3, 4, 5)
                .unwrap()
                .assume_offset(UtcOffset::from_hms(1, 0, 0).unwrap()),
        },
    };

    let value = to_value(&query).unwrap();
    assert_eq!(
        value,
        Value::Object(
            [(
                "inner".to_owned(),
                Value::Object(
                    [(
                        "at".to_owned(),
                        Value::Temporal(TemporalValue::from(query.inner.at))
                    )]
                    .into()
                ),
            )]
            .into()
        )
    );

    let decoded: Outer = from_value(&value).unwrap();
    assert_eq!(decoded, query);
}
