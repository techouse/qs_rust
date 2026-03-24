//! `time` integration helpers for encoding temporal values.

use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

use crate::error::EncodeError;
use crate::options::EncodeOptions;
use crate::temporal::{DateTimeValue, TemporalValue, TemporalValueError};
use crate::value::{Object, Value};

/// Converts an [`OffsetDateTime`] into the crate's [`Value`] representation.
///
/// The resulting value is a core [`crate::Value::Temporal`] leaf.
pub fn to_value(value: &OffsetDateTime) -> Value {
    Value::Temporal(TemporalValue::from(*value))
}

/// Encodes an [`OffsetDateTime`] at a single root key.
///
/// # Errors
///
/// Returns [`EncodeError`] when the supplied [`EncodeOptions`] are invalid.
pub fn encode_at_key(
    key: impl Into<String>,
    value: &OffsetDateTime,
    options: &EncodeOptions,
) -> Result<String, EncodeError> {
    let mut root = Object::new();
    root.insert(key.into(), to_value(value));
    crate::encode(&Value::Object(root), options)
}

impl From<OffsetDateTime> for TemporalValue {
    fn from(value: OffsetDateTime) -> Self {
        let date = value.date();
        let time = value.time();
        Self::DateTime(
            DateTimeValue::new(
                date.year(),
                u8::from(date.month()),
                date.day(),
                time.hour(),
                time.minute(),
                time.second(),
                time.nanosecond(),
                Some(value.offset().whole_seconds()),
            )
            .expect("time::OffsetDateTime values should map to valid temporal leaves"),
        )
    }
}

impl From<PrimitiveDateTime> for TemporalValue {
    fn from(value: PrimitiveDateTime) -> Self {
        let date = value.date();
        let time = value.time();
        Self::DateTime(
            DateTimeValue::new(
                date.year(),
                u8::from(date.month()),
                date.day(),
                time.hour(),
                time.minute(),
                time.second(),
                time.nanosecond(),
                None,
            )
            .expect("time::PrimitiveDateTime values should map to valid temporal leaves"),
        )
    }
}

impl TryFrom<&TemporalValue> for OffsetDateTime {
    type Error = TemporalValueError;

    fn try_from(value: &TemporalValue) -> Result<Self, Self::Error> {
        let datetime = value.as_datetime().ok_or(TemporalValueError::OutOfRange)?;
        let offset_seconds = datetime
            .offset_seconds()
            .ok_or(TemporalValueError::MissingOffset)?;
        let month =
            Month::try_from(datetime.month()).map_err(|_| TemporalValueError::OutOfRange)?;
        let date = Date::from_calendar_date(datetime.year(), month, datetime.day())
            .map_err(|_| TemporalValueError::OutOfRange)?;
        let time = Time::from_hms_nano(
            datetime.hour(),
            datetime.minute(),
            datetime.second(),
            datetime.nanosecond(),
        )
        .map_err(|_| TemporalValueError::OutOfRange)?;
        let offset = UtcOffset::from_whole_seconds(offset_seconds)
            .map_err(|_| TemporalValueError::OutOfRange)?;
        Ok(PrimitiveDateTime::new(date, time).assume_offset(offset))
    }
}

impl TryFrom<&TemporalValue> for PrimitiveDateTime {
    type Error = TemporalValueError;

    fn try_from(value: &TemporalValue) -> Result<Self, Self::Error> {
        let datetime = value.as_datetime().ok_or(TemporalValueError::OutOfRange)?;
        if datetime.offset_seconds().is_some() {
            return Err(TemporalValueError::UnexpectedOffset);
        }

        let month =
            Month::try_from(datetime.month()).map_err(|_| TemporalValueError::OutOfRange)?;
        let date = Date::from_calendar_date(datetime.year(), month, datetime.day())
            .map_err(|_| TemporalValueError::OutOfRange)?;
        let time = Time::from_hms_nano(
            datetime.hour(),
            datetime.minute(),
            datetime.second(),
            datetime.nanosecond(),
        )
        .map_err(|_| TemporalValueError::OutOfRange)?;
        Ok(PrimitiveDateTime::new(date, time))
    }
}

#[cfg(test)]
mod tests {
    use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

    use super::{encode_at_key, to_value};
    use crate::{
        EncodeOptions, ListFormat, TemporalSerializer, TemporalValue, TemporalValueError, Value,
        encode,
    };

    #[test]
    fn time_values_use_the_core_temporal_serializer_when_present() {
        let timestamp = OffsetDateTime::from_unix_timestamp(42).unwrap();
        let options =
            EncodeOptions::new().with_temporal_serializer(Some(TemporalSerializer::new(|value| {
                match value {
                    TemporalValue::DateTime(date) => Some(format!(
                        "wrapped:{}",
                        date.offset_seconds().unwrap_or_default()
                    )),
                }
            })));

        assert_eq!(
            to_value(&timestamp),
            Value::Temporal(TemporalValue::from(timestamp))
        );
        assert_eq!(
            encode_at_key("at", &timestamp, &options).unwrap(),
            "at=wrapped%3A0"
        );
    }

    #[test]
    fn time_nested_comma_lists_preserve_temporal_output() {
        let timestamp = OffsetDateTime::from_unix_timestamp(42).unwrap();
        let options = EncodeOptions::new()
            .with_encode(false)
            .with_list_format(ListFormat::Comma);
        let value =
            Value::Object([("at".to_owned(), Value::Array(vec![to_value(&timestamp)]))].into());

        assert_eq!(encode(&value, &options).unwrap(), "at=1970-01-01T00:00:42Z");
    }

    #[test]
    fn time_round_trips_through_core_temporal_value() {
        let aware = Date::from_calendar_date(2024, Month::January, 2)
            .unwrap()
            .with_hms(3, 4, 5)
            .unwrap()
            .assume_offset(UtcOffset::from_hms(1, 0, 0).unwrap());
        let temporal = TemporalValue::from(aware);
        let round_trip = OffsetDateTime::try_from(&temporal).unwrap();
        assert_eq!(round_trip, aware);

        let naive = PrimitiveDateTime::new(
            Date::from_calendar_date(2024, Month::January, 2).unwrap(),
            Time::from_hms(3, 4, 5).unwrap(),
        );
        let temporal = TemporalValue::from(naive);
        let round_trip = PrimitiveDateTime::try_from(&temporal).unwrap();
        assert_eq!(round_trip, naive);
    }

    #[test]
    fn time_aware_conversion_requires_a_temporal_offset() {
        let naive = PrimitiveDateTime::new(
            Date::from_calendar_date(2024, Month::January, 2).unwrap(),
            Time::from_hms(3, 4, 5).unwrap(),
        );
        let temporal = TemporalValue::from(naive);

        assert_eq!(
            OffsetDateTime::try_from(&temporal),
            Err(TemporalValueError::MissingOffset)
        );
    }

    #[test]
    fn time_naive_conversion_rejects_offset_aware_temporals() {
        let aware = Date::from_calendar_date(2024, Month::January, 2)
            .unwrap()
            .with_hms(3, 4, 5)
            .unwrap()
            .assume_offset(UtcOffset::from_hms(1, 0, 0).unwrap());
        let temporal = TemporalValue::from(aware);

        assert_eq!(
            PrimitiveDateTime::try_from(&temporal),
            Err(TemporalValueError::UnexpectedOffset)
        );
    }
}
