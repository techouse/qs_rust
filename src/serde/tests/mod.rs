pub(super) use super::deserializer::{
    ValueDeserializer, ValueEnumAccess, ValueMapAccess, ValueSeqAccess,
};
pub(super) use super::serializer::{
    MapKeySerializer, TemporalCaptureSerializer, ValueMapSerializer, ValueSerializer,
};
pub(super) use super::temporal::TEMPORAL_MARKER_NAME;
pub(super) use super::{from_str, from_value, to_string, to_value};
pub(super) use crate::{DateTimeValue, DecodeOptions, EncodeOptions, TemporalValue, Value};
use serde::de::{DeserializeSeed, IgnoredAny, Visitor};
use serde::{Deserialize, de};
use std::fmt;

pub(super) fn sample_temporal() -> TemporalValue {
    TemporalValue::from(DateTimeValue::new(2024, 1, 2, 3, 4, 5, 0, Some(0)).unwrap())
}

pub(super) fn sample_temporal_text() -> String {
    sample_temporal().to_string()
}

pub(super) fn assert_json_error<T>(result: Result<T, serde_json::Error>, needle: &str) {
    let error = match result {
        Ok(_) => panic!("expected serde_json::Error containing {needle:?}"),
        Err(error) => error,
    };
    assert!(
        error.to_string().contains(needle),
        "expected error containing {needle:?}, got {error}"
    );
}

pub(super) struct U32Seed;

impl<'de> DeserializeSeed<'de> for U32Seed {
    type Value = u32;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        u32::deserialize(deserializer)
    }
}

pub(super) struct StringSeed;

impl<'de> DeserializeSeed<'de> for StringSeed {
    type Value = String;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        String::deserialize(deserializer)
    }
}

pub(super) struct AnySummaryVisitor;

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

mod bridge;
mod deserializer;
mod serializer;
mod temporal;
