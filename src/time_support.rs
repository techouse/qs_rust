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
mod tests;
