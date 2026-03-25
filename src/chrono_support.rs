//! `chrono` integration helpers for encoding temporal values.

use chrono::{DateTime, Datelike, FixedOffset, NaiveDate, NaiveDateTime, TimeZone, Timelike};

use crate::error::EncodeError;
use crate::options::EncodeOptions;
use crate::temporal::{DateTimeValue, TemporalValue, TemporalValueError};
use crate::value::{Object, Value};

/// Converts a [`chrono::DateTime`] into the crate's [`Value`] representation.
///
/// The resulting value is a core [`crate::Value::Temporal`] leaf.
pub fn to_value<Tz>(value: &DateTime<Tz>) -> Value
where
    Tz: TimeZone,
    Tz::Offset: std::fmt::Display,
{
    Value::Temporal(TemporalValue::from(value.clone()))
}

/// Encodes a [`chrono::DateTime`] at a single root key.
///
/// # Errors
///
/// Returns [`EncodeError`] when the supplied [`EncodeOptions`] are invalid.
pub fn encode_at_key<Tz>(
    key: impl Into<String>,
    value: &DateTime<Tz>,
    options: &EncodeOptions,
) -> Result<String, EncodeError>
where
    Tz: TimeZone,
    Tz::Offset: std::fmt::Display,
{
    let mut root = Object::new();
    root.insert(key.into(), to_value(value));
    crate::encode(&Value::Object(root), options)
}

impl<Tz> From<DateTime<Tz>> for TemporalValue
where
    Tz: TimeZone,
    Tz::Offset: std::fmt::Display,
{
    fn from(value: DateTime<Tz>) -> Self {
        let fixed = value.fixed_offset();
        Self::DateTime(
            DateTimeValue::new(
                fixed.year(),
                fixed.month() as u8,
                fixed.day() as u8,
                fixed.hour() as u8,
                fixed.minute() as u8,
                fixed.second() as u8,
                fixed.nanosecond(),
                Some(fixed.offset().local_minus_utc()),
            )
            .expect("chrono DateTime values should map to valid temporal leaves"),
        )
    }
}

impl From<NaiveDateTime> for TemporalValue {
    fn from(value: NaiveDateTime) -> Self {
        Self::DateTime(
            DateTimeValue::new(
                value.year(),
                value.month() as u8,
                value.day() as u8,
                value.hour() as u8,
                value.minute() as u8,
                value.second() as u8,
                value.nanosecond(),
                None,
            )
            .expect("chrono NaiveDateTime values should map to valid temporal leaves"),
        )
    }
}

impl TryFrom<&TemporalValue> for DateTime<FixedOffset> {
    type Error = TemporalValueError;

    fn try_from(value: &TemporalValue) -> Result<Self, Self::Error> {
        let datetime = value.as_datetime().ok_or(TemporalValueError::OutOfRange)?;
        let offset_seconds = datetime
            .offset_seconds()
            .ok_or(TemporalValueError::MissingOffset)?;
        let offset = FixedOffset::east_opt(offset_seconds).ok_or(TemporalValueError::OutOfRange)?;
        let naive = NaiveDate::from_ymd_opt(
            datetime.year(),
            u32::from(datetime.month()),
            u32::from(datetime.day()),
        )
        .and_then(|date| {
            date.and_hms_nano_opt(
                u32::from(datetime.hour()),
                u32::from(datetime.minute()),
                u32::from(datetime.second()),
                datetime.nanosecond(),
            )
        })
        .ok_or(TemporalValueError::OutOfRange)?;
        offset
            .from_local_datetime(&naive)
            .single()
            .ok_or(TemporalValueError::OutOfRange)
    }
}

impl TryFrom<&TemporalValue> for NaiveDateTime {
    type Error = TemporalValueError;

    fn try_from(value: &TemporalValue) -> Result<Self, Self::Error> {
        let datetime = value.as_datetime().ok_or(TemporalValueError::OutOfRange)?;
        if datetime.offset_seconds().is_some() {
            return Err(TemporalValueError::UnexpectedOffset);
        }

        NaiveDate::from_ymd_opt(
            datetime.year(),
            u32::from(datetime.month()),
            u32::from(datetime.day()),
        )
        .and_then(|date| {
            date.and_hms_nano_opt(
                u32::from(datetime.hour()),
                u32::from(datetime.minute()),
                u32::from(datetime.second()),
                datetime.nanosecond(),
            )
        })
        .ok_or(TemporalValueError::OutOfRange)
    }
}

#[cfg(test)]
mod tests;
