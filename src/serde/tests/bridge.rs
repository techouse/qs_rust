use super::{
    DecodeOptions, EncodeOptions, Value, from_str, from_value, sample_temporal,
    sample_temporal_text, to_string, to_value,
};
use serde::de::{IgnoredAny, Visitor};
use serde::{Deserialize, Serialize, de};
use std::collections::BTreeMap;
use std::fmt;

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
