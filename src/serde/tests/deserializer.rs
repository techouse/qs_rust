use super::{
    AnySummaryVisitor, StringSeed, U32Seed, Value, ValueDeserializer, ValueEnumAccess,
    ValueMapAccess, ValueSeqAccess, from_value,
};
use serde::de::{EnumAccess, MapAccess, SeqAccess, VariantAccess};
use serde::{Deserialize, de};

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
