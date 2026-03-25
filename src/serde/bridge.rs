use ::serde::{Serialize, de::DeserializeOwned};

use crate::decode::decode;
use crate::encode::encode;
use crate::error::{DecodeError, EncodeError};
use crate::options::{DecodeOptions, EncodeOptions};
use crate::value::Value;

use super::deserializer::ValueDeserializer;
use super::serializer::ValueSerializer;

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
