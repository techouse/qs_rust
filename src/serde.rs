//! Serde integration helpers.

use std::fmt;

use ::serde::de::{
    self, DeserializeOwned, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess,
    VariantAccess, Visitor,
};
use ::serde::ser::{
    self, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant,
};
use ::serde::{Serialize, forward_to_deserialize_any};

use crate::decode::decode;
use crate::encode::encode;
use crate::error::{DecodeError, EncodeError};
use crate::options::{DecodeOptions, EncodeOptions};
use crate::temporal::TemporalValue;
use crate::value::{Object, Value};

const TEMPORAL_MARKER_NAME: &str = "__qs_rust_temporal__";

/// Decodes a query string directly into a `serde`-deserializable type.
///
/// The input is first decoded into the crate's intermediate [`Value`] tree and
/// then deserialized directly from that tree.
///
/// # Errors
///
/// Returns [`DecodeError`] when query-string decoding fails or when serde
/// cannot deserialize the intermediate value into `T`.
pub fn from_str<T>(input: &str, options: &DecodeOptions) -> Result<T, DecodeError>
where
    T: DeserializeOwned,
{
    let object = decode(input, options)?;
    from_value(&Value::Object(object))
}

/// Decodes a typed value directly from the crate's dynamic [`Value`] model.
///
/// # Errors
///
/// Returns [`DecodeError`] when serde cannot deserialize the supplied value
/// tree into `T`.
pub fn from_value<T>(value: &Value) -> Result<T, DecodeError>
where
    T: DeserializeOwned,
{
    T::deserialize(ValueDeserializer::new(value)).map_err(DecodeError::from)
}

/// Encodes any `serde`-serializable value as a query string.
///
/// The value is first converted into the crate's intermediate [`Value`] tree
/// and then encoded with [`crate::encode()`].
///
/// # Errors
///
/// Returns [`EncodeError`] when serde serialization fails or when query-string
/// encoding fails.
pub fn to_string<T>(value: &T, options: &EncodeOptions) -> Result<String, EncodeError>
where
    T: Serialize,
{
    let value = to_value(value)?;
    encode(&value, options)
}

/// Converts any `serde`-serializable value into the crate's dynamic [`Value`]
/// model.
///
/// # Errors
///
/// Returns [`EncodeError`] when serde serialization fails.
pub fn to_value<T>(value: &T) -> Result<Value, EncodeError>
where
    T: Serialize,
{
    value.serialize(ValueSerializer).map_err(EncodeError::from)
}

pub mod temporal {
    //! Opt-in serde field helpers for preserving temporal leaves.

    #[cfg(feature = "chrono")]
    pub mod chrono_datetime {
        //! Serde helpers for `chrono::DateTime<chrono::FixedOffset>` fields.

        use ::serde::{Deserialize, Deserializer, Serializer, de::Error};

        use crate::temporal::TemporalValue;

        /// Serializes an offset-aware `chrono` datetime as a core temporal leaf.
        pub fn serialize<S>(
            value: &chrono::DateTime<chrono::FixedOffset>,
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let temporal = TemporalValue::from(value.to_owned());
            serializer.serialize_newtype_struct(
                super::super::TEMPORAL_MARKER_NAME,
                &super::super::TemporalMarker(&temporal),
            )
        }

        /// Deserializes an offset-aware `chrono` datetime from canonical ISO text
        /// or a core temporal leaf.
        pub fn deserialize<'de, D>(
            deserializer: D,
        ) -> Result<chrono::DateTime<chrono::FixedOffset>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let text = String::deserialize(deserializer)?;
            let temporal = TemporalValue::parse_iso8601(&text)
                .map_err(|error| D::Error::custom(error.to_string()))?;
            chrono::DateTime::<chrono::FixedOffset>::try_from(&temporal)
                .map_err(|error| D::Error::custom(error.to_string()))
        }
    }

    #[cfg(feature = "chrono")]
    pub mod chrono_naive_datetime {
        //! Serde helpers for `chrono::NaiveDateTime` fields.

        use ::serde::{Deserialize, Deserializer, Serializer, de::Error};

        use crate::temporal::TemporalValue;

        /// Serializes a naive `chrono` datetime as a core temporal leaf.
        pub fn serialize<S>(value: &chrono::NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let temporal = TemporalValue::from(value.to_owned());
            serializer.serialize_newtype_struct(
                super::super::TEMPORAL_MARKER_NAME,
                &super::super::TemporalMarker(&temporal),
            )
        }

        /// Deserializes a naive `chrono` datetime from canonical ISO text or
        /// a core temporal leaf.
        pub fn deserialize<'de, D>(deserializer: D) -> Result<chrono::NaiveDateTime, D::Error>
        where
            D: Deserializer<'de>,
        {
            let text = String::deserialize(deserializer)?;
            let temporal = TemporalValue::parse_iso8601(&text)
                .map_err(|error| D::Error::custom(error.to_string()))?;
            chrono::NaiveDateTime::try_from(&temporal)
                .map_err(|error| D::Error::custom(error.to_string()))
        }
    }

    #[cfg(feature = "time")]
    pub mod time_offset_datetime {
        //! Serde helpers for `time::OffsetDateTime` fields.

        use ::serde::{Deserialize, Deserializer, Serializer, de::Error};

        use crate::temporal::TemporalValue;

        /// Serializes an offset-aware `time` datetime as a core temporal leaf.
        pub fn serialize<S>(value: &time::OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let temporal = TemporalValue::from(*value);
            serializer.serialize_newtype_struct(
                super::super::TEMPORAL_MARKER_NAME,
                &super::super::TemporalMarker(&temporal),
            )
        }

        /// Deserializes an offset-aware `time` datetime from canonical ISO
        /// text or a core temporal leaf.
        pub fn deserialize<'de, D>(deserializer: D) -> Result<time::OffsetDateTime, D::Error>
        where
            D: Deserializer<'de>,
        {
            let text = String::deserialize(deserializer)?;
            let temporal = TemporalValue::parse_iso8601(&text)
                .map_err(|error| D::Error::custom(error.to_string()))?;
            time::OffsetDateTime::try_from(&temporal)
                .map_err(|error| D::Error::custom(error.to_string()))
        }
    }

    #[cfg(feature = "time")]
    pub mod time_primitive_datetime {
        //! Serde helpers for `time::PrimitiveDateTime` fields.

        use ::serde::{Deserialize, Deserializer, Serializer, de::Error};

        use crate::temporal::TemporalValue;

        /// Serializes a naive `time` datetime as a core temporal leaf.
        pub fn serialize<S>(
            value: &time::PrimitiveDateTime,
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let temporal = TemporalValue::from(*value);
            serializer.serialize_newtype_struct(
                super::super::TEMPORAL_MARKER_NAME,
                &super::super::TemporalMarker(&temporal),
            )
        }

        /// Deserializes a naive `time` datetime from canonical ISO text or a
        /// core temporal leaf.
        pub fn deserialize<'de, D>(deserializer: D) -> Result<time::PrimitiveDateTime, D::Error>
        where
            D: Deserializer<'de>,
        {
            let text = String::deserialize(deserializer)?;
            let temporal = TemporalValue::parse_iso8601(&text)
                .map_err(|error| D::Error::custom(error.to_string()))?;
            time::PrimitiveDateTime::try_from(&temporal)
                .map_err(|error| D::Error::custom(error.to_string()))
        }
    }
}

#[cfg(any(feature = "chrono", feature = "time"))]
struct TemporalMarker<'a>(&'a TemporalValue);

#[cfg(any(feature = "chrono", feature = "time"))]
impl Serialize for TemporalMarker<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

struct TemporalCaptureSerializer;

impl ser::Serializer for TemporalCaptureSerializer {
    type Ok = TemporalValue;
    type Error = serde_json::Error;
    type SerializeSeq = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        TemporalValue::parse_iso8601(value).map_err(ser_error)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_bool(self, _value: bool) -> Result<Self::Ok, Self::Error> {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_i8(self, _value: i8) -> Result<Self::Ok, Self::Error> {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_i16(self, _value: i16) -> Result<Self::Ok, Self::Error> {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_i32(self, _value: i32) -> Result<Self::Ok, Self::Error> {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_i64(self, _value: i64) -> Result<Self::Ok, Self::Error> {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_u8(self, _value: u8) -> Result<Self::Ok, Self::Error> {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_u16(self, _value: u16) -> Result<Self::Ok, Self::Error> {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_u32(self, _value: u32) -> Result<Self::Ok, Self::Error> {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_u64(self, _value: u64) -> Result<Self::Ok, Self::Error> {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_f32(self, _value: f32) -> Result<Self::Ok, Self::Error> {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_f64(self, _value: f64) -> Result<Self::Ok, Self::Error> {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_char(self, _value: char) -> Result<Self::Ok, Self::Error> {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(ser_error("temporal markers must serialize as ISO strings"))
    }
}

struct ValueSerializer;

impl ser::Serializer for ValueSerializer {
    type Ok = Value;
    type Error = serde_json::Error;
    type SerializeSeq = ValueSeqSerializer;
    type SerializeTuple = ValueSeqSerializer;
    type SerializeTupleStruct = ValueSeqSerializer;
    type SerializeTupleVariant = ValueTupleVariantSerializer;
    type SerializeMap = ValueMapSerializer;
    type SerializeStruct = ValueMapSerializer;
    type SerializeStructVariant = ValueStructVariantSerializer;

    fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Bool(value))
    }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        Ok(Value::I64(i64::from(value)))
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        Ok(Value::I64(i64::from(value)))
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::I64(i64::from(value)))
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::I64(value))
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
        Ok(Value::U64(u64::from(value)))
    }

    fn serialize_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
        Ok(Value::U64(u64::from(value)))
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::U64(u64::from(value)))
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::U64(value))
    }

    fn serialize_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(f64::from(value))
    }

    fn serialize_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
        if !value.is_finite() {
            return Err(ser_error("cannot serialize non-finite floats"));
        }
        Ok(Value::F64(value))
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(value.to_owned()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Bytes(value.to_vec()))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(variant.to_owned()))
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        if name == TEMPORAL_MARKER_NAME {
            let temporal = value.serialize(TemporalCaptureSerializer)?;
            return Ok(Value::Temporal(temporal));
        }

        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let mut object = Object::new();
        object.insert(variant.to_owned(), value.serialize(ValueSerializer)?);
        Ok(Value::Object(object))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(ValueSeqSerializer::new(len.unwrap_or(0)))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(ValueSeqSerializer::new(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(ValueSeqSerializer::new(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(ValueTupleVariantSerializer::new(variant, len))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(ValueMapSerializer::new(len.unwrap_or(0)))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(ValueMapSerializer::new(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(ValueStructVariantSerializer::new(variant, len))
    }
}

struct ValueSeqSerializer {
    items: Vec<Value>,
}

impl ValueSeqSerializer {
    fn new(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
        }
    }
}

impl SerializeSeq for ValueSeqSerializer {
    type Ok = Value;
    type Error = serde_json::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.items.push(value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Array(self.items))
    }
}

impl SerializeTuple for ValueSeqSerializer {
    type Ok = Value;
    type Error = serde_json::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl SerializeTupleStruct for ValueSeqSerializer {
    type Ok = Value;
    type Error = serde_json::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

struct ValueTupleVariantSerializer {
    variant: String,
    items: Vec<Value>,
}

impl ValueTupleVariantSerializer {
    fn new(variant: &str, capacity: usize) -> Self {
        Self {
            variant: variant.to_owned(),
            items: Vec::with_capacity(capacity),
        }
    }
}

impl SerializeTupleVariant for ValueTupleVariantSerializer {
    type Ok = Value;
    type Error = serde_json::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.items.push(value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut object = Object::new();
        object.insert(self.variant, Value::Array(self.items));
        Ok(Value::Object(object))
    }
}

struct ValueMapSerializer {
    entries: Object,
    pending_key: Option<String>,
}

impl ValueMapSerializer {
    fn new(capacity: usize) -> Self {
        Self {
            entries: Object::with_capacity(capacity),
            pending_key: None,
        }
    }
}

impl SerializeMap for ValueMapSerializer {
    type Ok = Value;
    type Error = serde_json::Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.pending_key = Some(key.serialize(MapKeySerializer)?);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let key = self
            .pending_key
            .take()
            .ok_or_else(|| ser_error("serialize_value called before serialize_key"))?;
        self.entries.insert(key, value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Object(self.entries))
    }
}

impl SerializeStruct for ValueMapSerializer {
    type Ok = Value;
    type Error = serde_json::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.entries
            .insert(key.to_owned(), value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Object(self.entries))
    }
}

struct ValueStructVariantSerializer {
    variant: String,
    entries: Object,
}

impl ValueStructVariantSerializer {
    fn new(variant: &str, capacity: usize) -> Self {
        Self {
            variant: variant.to_owned(),
            entries: Object::with_capacity(capacity),
        }
    }
}

impl SerializeStructVariant for ValueStructVariantSerializer {
    type Ok = Value;
    type Error = serde_json::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.entries
            .insert(key.to_owned(), value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut object = Object::new();
        object.insert(self.variant, Value::Object(self.entries));
        Ok(Value::Object(object))
    }
}

struct MapKeySerializer;

impl ser::Serializer for MapKeySerializer {
    type Ok = String;
    type Error = serde_json::Error;
    type SerializeSeq = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_owned())
    }

    fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(variant.to_owned())
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(ser_error("map keys must not be null"))
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(ser_error("map keys must not be unit values"))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(ser_error("map keys must not be unit values"))
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(ser_error("map keys must be strings or scalar values"))
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(ser_error("map keys must be strings or scalar values"))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(ser_error("map keys must be strings or scalar values"))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(ser_error("map keys must be strings or scalar values"))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(ser_error("map keys must be strings or scalar values"))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(ser_error("map keys must be strings or scalar values"))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(ser_error("map keys must be strings or scalar values"))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(ser_error("map keys must be strings or scalar values"))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(ser_error("map keys must be strings or scalar values"))
    }
}

struct ValueDeserializer<'de> {
    value: &'de Value,
}

impl<'de> ValueDeserializer<'de> {
    fn new(value: &'de Value) -> Self {
        Self { value }
    }
}

impl<'de> de::Deserializer<'de> for ValueDeserializer<'de> {
    type Error = serde_json::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Null => visitor.visit_unit(),
            Value::Bool(value) => visitor.visit_bool(*value),
            Value::I64(value) => visitor.visit_i64(*value),
            Value::U64(value) => visitor.visit_u64(*value),
            Value::F64(value) => visitor.visit_f64(*value),
            Value::String(value) => visitor.visit_str(value),
            Value::Temporal(value) => visitor.visit_string(value.to_string()),
            Value::Bytes(value) => visitor.visit_byte_buf(value.clone()),
            Value::Array(items) => visitor.visit_seq(ValueSeqAccess::from_values(items)),
            Value::Object(entries) => visitor.visit_map(ValueMapAccess::new(entries)),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Null => visitor.visit_unit(),
            _ => Err(de_error("expected unit value")),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Array(items) => visitor.visit_seq(ValueSeqAccess::from_values(items)),
            Value::Bytes(bytes) => visitor.visit_seq(ValueSeqAccess::from_bytes(bytes)),
            _ => Err(de_error("expected sequence value")),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Object(entries) => visitor.visit_map(ValueMapAccess::new(entries)),
            _ => Err(de_error("expected map value")),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::String(variant) => visitor.visit_enum(ValueEnumAccess::unit(variant)),
            Value::Temporal(value) => {
                let variant = value.to_string();
                visitor.visit_enum(ValueEnumAccess::owned_unit(variant))
            }
            Value::Object(entries) if entries.len() == 1 => {
                let (variant, value) = entries.iter().next().expect("checked len == 1");
                visitor.visit_enum(ValueEnumAccess::new(variant, Some(value)))
            }
            _ => Err(de_error("expected enum representation")),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::String(value) => visitor.visit_str(value),
            Value::Temporal(value) => visitor.visit_string(value.to_string()),
            _ => Err(de_error("expected identifier string")),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf
    }
}

enum SeqValueRef<'de> {
    Value(&'de Value),
    Byte(u8),
}

struct ValueSeqAccess<'de> {
    items: Vec<SeqValueRef<'de>>,
    index: usize,
}

impl<'de> ValueSeqAccess<'de> {
    fn from_values(items: &'de [Value]) -> Self {
        Self {
            items: items.iter().map(SeqValueRef::Value).collect(),
            index: 0,
        }
    }

    fn from_bytes(items: &'de [u8]) -> Self {
        Self {
            items: items.iter().copied().map(SeqValueRef::Byte).collect(),
            index: 0,
        }
    }
}

impl<'de> SeqAccess<'de> for ValueSeqAccess<'de> {
    type Error = serde_json::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        let Some(item) = self.items.get(self.index) else {
            return Ok(None);
        };
        self.index += 1;

        let value = match item {
            SeqValueRef::Value(value) => seed.deserialize(ValueDeserializer::new(value))?,
            SeqValueRef::Byte(value) => seed.deserialize((*value).into_deserializer())?,
        };
        Ok(Some(value))
    }
}

struct ValueMapAccess<'de> {
    iter: indexmap::map::Iter<'de, String, Value>,
    pending: Option<&'de Value>,
}

impl<'de> ValueMapAccess<'de> {
    fn new(entries: &'de Object) -> Self {
        Self {
            iter: entries.iter(),
            pending: None,
        }
    }
}

impl<'de> MapAccess<'de> for ValueMapAccess<'de> {
    type Error = serde_json::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        let Some((key, value)) = self.iter.next() else {
            return Ok(None);
        };

        self.pending = Some(value);
        seed.deserialize(key.as_str().into_deserializer()).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self
            .pending
            .take()
            .ok_or_else(|| de_error("missing map value for previously deserialized key"))?;
        seed.deserialize(ValueDeserializer::new(value))
    }
}

enum ValueEnumVariant<'de> {
    Borrowed(&'de str, Option<&'de Value>),
    Owned(String),
}

struct ValueEnumAccess<'de> {
    variant: ValueEnumVariant<'de>,
}

impl<'de> ValueEnumAccess<'de> {
    fn new(variant: &'de str, value: Option<&'de Value>) -> Self {
        Self {
            variant: ValueEnumVariant::Borrowed(variant, value),
        }
    }

    fn unit(variant: &'de str) -> Self {
        Self::new(variant, None)
    }

    fn owned_unit(variant: String) -> Self {
        Self {
            variant: ValueEnumVariant::Owned(variant),
        }
    }
}

impl<'de> EnumAccess<'de> for ValueEnumAccess<'de> {
    type Error = serde_json::Error;
    type Variant = ValueVariantAccess<'de>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        match self.variant {
            ValueEnumVariant::Borrowed(variant, value) => {
                let key = seed.deserialize(variant.into_deserializer())?;
                Ok((key, ValueVariantAccess { value }))
            }
            ValueEnumVariant::Owned(variant) => {
                let key = seed.deserialize(variant.into_deserializer())?;
                Ok((key, ValueVariantAccess { value: None }))
            }
        }
    }
}

struct ValueVariantAccess<'de> {
    value: Option<&'de Value>,
}

impl<'de> VariantAccess<'de> for ValueVariantAccess<'de> {
    type Error = serde_json::Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        match self.value {
            None | Some(Value::Null) => Ok(()),
            Some(_) => Err(de_error("expected unit variant")),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        let value = self
            .value
            .ok_or_else(|| de_error("expected newtype variant payload"))?;
        seed.deserialize(ValueDeserializer::new(value))
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let value = self
            .value
            .ok_or_else(|| de_error("expected tuple variant payload"))?;
        de::Deserializer::deserialize_seq(ValueDeserializer::new(value), visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let value = self
            .value
            .ok_or_else(|| de_error("expected struct variant payload"))?;
        de::Deserializer::deserialize_map(ValueDeserializer::new(value), visitor)
    }
}

fn ser_error(message: impl fmt::Display) -> serde_json::Error {
    <serde_json::Error as ser::Error>::custom(message.to_string())
}

fn de_error(message: impl fmt::Display) -> serde_json::Error {
    <serde_json::Error as de::Error>::custom(message.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        MapKeySerializer, TEMPORAL_MARKER_NAME, TemporalCaptureSerializer, ValueDeserializer,
        ValueEnumAccess, ValueMapAccess, ValueMapSerializer, ValueSeqAccess, ValueSerializer,
        from_str, from_value, to_string, to_value,
    };
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
            from_value::<TupleStruct>(&Value::Array(vec![Value::I64(7), Value::Bool(true)]))
                .unwrap(),
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
            from_value::<Variant>(&Value::Object([("Unit".to_owned(), Value::Null)].into(),))
                .unwrap(),
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

        let tuple_payload_err =
            from_value::<Variant>(&Value::String("Tuple".to_owned())).unwrap_err();
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
            <TemporalCaptureSerializer as ser::Serializer>::serialize_i8(
                TemporalCaptureSerializer,
                -8,
            ),
            <TemporalCaptureSerializer as ser::Serializer>::serialize_i16(
                TemporalCaptureSerializer,
                -16,
            ),
            <TemporalCaptureSerializer as ser::Serializer>::serialize_i32(
                TemporalCaptureSerializer,
                -32,
            ),
            <TemporalCaptureSerializer as ser::Serializer>::serialize_u8(
                TemporalCaptureSerializer,
                8,
            ),
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
        let object =
            Value::Object([("field".to_owned(), Value::String("value".to_owned()))].into());

        assert_eq!(
            de::Deserializer::deserialize_any(
                ValueDeserializer::new(&Value::Null),
                AnySummaryVisitor
            )
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
            <TemporalCaptureSerializer as ser::Serializer>::serialize_unit(
                TemporalCaptureSerializer,
            ),
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
            <TemporalCaptureSerializer as ser::Serializer>::serialize_i64(
                TemporalCaptureSerializer,
                1,
            ),
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
            <TemporalCaptureSerializer as ser::Serializer>::serialize_none(
                TemporalCaptureSerializer,
            ),
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
            <MapKeySerializer as ser::Serializer>::serialize_str(MapKeySerializer, "field")
                .unwrap(),
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
            crate::serde::temporal::chrono_naive_datetime::serialize(&naive, ValueSerializer)
                .unwrap(),
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
}
