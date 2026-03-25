//! Opt-in serde field helpers for preserving temporal leaves.

use ::serde::Serialize;

use crate::temporal::TemporalValue;

pub(super) const TEMPORAL_MARKER_NAME: &str = "__qs_rust_temporal__";

#[cfg(any(feature = "chrono", feature = "time"))]
pub(super) struct TemporalMarker<'a>(pub(super) &'a TemporalValue);

#[cfg(any(feature = "chrono", feature = "time"))]
impl Serialize for TemporalMarker<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

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
            super::TEMPORAL_MARKER_NAME,
            &super::TemporalMarker(&temporal),
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
            super::TEMPORAL_MARKER_NAME,
            &super::TemporalMarker(&temporal),
        )
    }

    /// Deserializes a naive `chrono` datetime from canonical ISO text or a
    /// core temporal leaf.
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
            super::TEMPORAL_MARKER_NAME,
            &super::TemporalMarker(&temporal),
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
    pub fn serialize<S>(value: &time::PrimitiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let temporal = TemporalValue::from(*value);
        serializer.serialize_newtype_struct(
            super::TEMPORAL_MARKER_NAME,
            &super::TemporalMarker(&temporal),
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
