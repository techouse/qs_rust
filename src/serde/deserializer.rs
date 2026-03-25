use std::fmt;

use ::serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use ::serde::forward_to_deserialize_any;

use crate::value::{Object, Value};

pub(super) struct ValueDeserializer<'de> {
    value: &'de Value,
}

impl<'de> ValueDeserializer<'de> {
    pub(super) fn new(value: &'de Value) -> Self {
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

pub(super) struct ValueSeqAccess<'de> {
    items: Vec<SeqValueRef<'de>>,
    index: usize,
}

impl<'de> ValueSeqAccess<'de> {
    pub(super) fn from_values(items: &'de [Value]) -> Self {
        Self {
            items: items.iter().map(SeqValueRef::Value).collect(),
            index: 0,
        }
    }

    pub(super) fn from_bytes(items: &'de [u8]) -> Self {
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

pub(super) struct ValueMapAccess<'de> {
    iter: indexmap::map::Iter<'de, String, Value>,
    pending: Option<&'de Value>,
}

impl<'de> ValueMapAccess<'de> {
    pub(super) fn new(entries: &'de Object) -> Self {
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

pub(super) struct ValueEnumAccess<'de> {
    variant: ValueEnumVariant<'de>,
}

impl<'de> ValueEnumAccess<'de> {
    pub(super) fn new(variant: &'de str, value: Option<&'de Value>) -> Self {
        Self {
            variant: ValueEnumVariant::Borrowed(variant, value),
        }
    }

    pub(super) fn unit(variant: &'de str) -> Self {
        Self::new(variant, None)
    }

    pub(super) fn owned_unit(variant: String) -> Self {
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

pub(super) struct ValueVariantAccess<'de> {
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

fn de_error(message: impl fmt::Display) -> serde_json::Error {
    <serde_json::Error as de::Error>::custom(message.to_string())
}
