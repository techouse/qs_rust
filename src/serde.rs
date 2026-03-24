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
    use super::{from_value, to_value};
    use crate::{DateTimeValue, TemporalValue, Value};
    use serde::{Deserialize, Serialize};

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
        let temporal =
            TemporalValue::from(DateTimeValue::new(2024, 1, 2, 3, 4, 5, 0, Some(0)).unwrap());
        let decoded: String = from_value(&Value::Temporal(temporal)).unwrap();
        assert_eq!(decoded, "2024-01-02T03:04:05Z");
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
}
