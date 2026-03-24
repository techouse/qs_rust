//! Core temporal value types used by the dynamic [`crate::Value`] model.

use std::fmt;
use std::str::FromStr;

use thiserror::Error;

/// A temporal leaf stored inside [`crate::Value::Temporal`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TemporalValue {
    /// A calendar date and time with an optional UTC offset.
    DateTime(DateTimeValue),
}

impl TemporalValue {
    /// Creates a validated datetime temporal value.
    #[expect(
        clippy::too_many_arguments,
        reason = "the public constructor intentionally mirrors datetime components"
    )]
    pub fn datetime(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanosecond: u32,
        offset_seconds: Option<i32>,
    ) -> Result<Self, TemporalValueError> {
        Ok(Self::DateTime(DateTimeValue::new(
            year,
            month,
            day,
            hour,
            minute,
            second,
            nanosecond,
            offset_seconds,
        )?))
    }

    /// Returns the contained datetime value when this temporal is a datetime.
    pub fn as_datetime(&self) -> Option<&DateTimeValue> {
        match self {
            Self::DateTime(value) => Some(value),
        }
    }

    /// Parses a canonical ISO-8601 datetime string into a temporal value.
    pub fn parse_iso8601(input: &str) -> Result<Self, TemporalValueError> {
        Ok(Self::DateTime(DateTimeValue::parse_iso8601(input)?))
    }
}

impl fmt::Display for TemporalValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DateTime(value) => value.fmt(f),
        }
    }
}

impl From<DateTimeValue> for TemporalValue {
    fn from(value: DateTimeValue) -> Self {
        Self::DateTime(value)
    }
}

impl FromStr for TemporalValue {
    type Err = TemporalValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_iso8601(s)
    }
}

/// A validated calendar date and time with an optional UTC offset.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateTimeValue {
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    nanosecond: u32,
    offset_seconds: Option<i32>,
}

impl DateTimeValue {
    /// Creates a validated datetime value.
    #[expect(
        clippy::too_many_arguments,
        reason = "the public constructor intentionally mirrors datetime components"
    )]
    pub fn new(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanosecond: u32,
        offset_seconds: Option<i32>,
    ) -> Result<Self, TemporalValueError> {
        let value = Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            nanosecond,
            offset_seconds,
        };
        validate_datetime(&value)?;
        Ok(value)
    }

    /// Parses a canonical ISO-8601 datetime string.
    pub fn parse_iso8601(input: &str) -> Result<Self, TemporalValueError> {
        let (date, time) = input
            .split_once('T')
            .ok_or(TemporalValueError::InvalidFormat)?;
        let (year, month, day) = parse_date(date)?;
        let (hour, minute, second, nanosecond, offset_seconds) = parse_time(time)?;
        Self::new(
            year,
            month,
            day,
            hour,
            minute,
            second,
            nanosecond,
            offset_seconds,
        )
    }

    /// Returns the year component.
    pub fn year(&self) -> i32 {
        self.year
    }

    /// Returns the month component.
    pub fn month(&self) -> u8 {
        self.month
    }

    /// Returns the day-of-month component.
    pub fn day(&self) -> u8 {
        self.day
    }

    /// Returns the hour component.
    pub fn hour(&self) -> u8 {
        self.hour
    }

    /// Returns the minute component.
    pub fn minute(&self) -> u8 {
        self.minute
    }

    /// Returns the second component.
    pub fn second(&self) -> u8 {
        self.second
    }

    /// Returns the fractional nanoseconds component.
    pub fn nanosecond(&self) -> u32 {
        self.nanosecond
    }

    /// Returns the offset from UTC in seconds, or `None` for naive datetimes.
    pub fn offset_seconds(&self) -> Option<i32> {
        self.offset_seconds
    }
}

impl fmt::Display for DateTimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}-{:02}-{:02}T{:02}:{:02}:{:02}",
            format_year(self.year),
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second
        )?;

        if self.nanosecond != 0 {
            let mut fraction = format!("{:09}", self.nanosecond);
            while fraction.ends_with('0') {
                fraction.pop();
            }
            write!(f, ".{fraction}")?;
        }

        if let Some(offset_seconds) = self.offset_seconds {
            if offset_seconds == 0 {
                f.write_str("Z")?;
            } else {
                let sign = if offset_seconds < 0 { '-' } else { '+' };
                let absolute = offset_seconds.unsigned_abs();
                let hours = absolute / 3_600;
                let minutes = (absolute % 3_600) / 60;
                let seconds = absolute % 60;
                write!(f, "{sign}{hours:02}:{minutes:02}")?;
                if seconds != 0 {
                    write!(f, ":{seconds:02}")?;
                }
            }
        }

        Ok(())
    }
}

impl FromStr for DateTimeValue {
    type Err = TemporalValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_iso8601(s)
    }
}

/// Validation and parsing errors for [`TemporalValue`] and [`DateTimeValue`].
#[non_exhaustive]
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum TemporalValueError {
    /// The provided month was outside `1..=12`.
    #[error("invalid month component {0}; expected 1..=12")]
    InvalidMonth(u8),

    /// The provided day was invalid for the given month and year.
    #[error("invalid day component {day} for {year:04}-{month:02}")]
    InvalidDay {
        /// The year component.
        year: i32,
        /// The month component.
        month: u8,
        /// The day component.
        day: u8,
    },

    /// The provided hour was outside `0..=23`.
    #[error("invalid hour component {0}; expected 0..=23")]
    InvalidHour(u8),

    /// The provided minute was outside `0..=59`.
    #[error("invalid minute component {0}; expected 0..=59")]
    InvalidMinute(u8),

    /// The provided second was outside `0..=59`.
    #[error("invalid second component {0}; expected 0..=59")]
    InvalidSecond(u8),

    /// The provided nanosecond value was outside `0..1_000_000_000`.
    #[error("invalid nanosecond component {0}; expected 0..1_000_000_000")]
    InvalidNanosecond(u32),

    /// The provided UTC offset was outside the supported range.
    #[error("invalid UTC offset seconds {0}; expected -86399..=86399")]
    InvalidOffsetSeconds(i32),

    /// A string could not be parsed as a canonical datetime value.
    #[error("invalid datetime format; expected ISO-8601 datetime text")]
    InvalidFormat,

    /// A conversion required an offset-aware datetime, but none was present.
    #[error("temporal value is missing a UTC offset")]
    MissingOffset,

    /// A conversion required a naive datetime, but an offset was present.
    #[error("temporal value unexpectedly contains a UTC offset")]
    UnexpectedOffset,

    /// A temporal value could not be represented by the requested target type.
    #[error("temporal value is out of range for the requested target type")]
    OutOfRange,
}

fn validate_datetime(value: &DateTimeValue) -> Result<(), TemporalValueError> {
    if !(1..=12).contains(&value.month) {
        return Err(TemporalValueError::InvalidMonth(value.month));
    }

    let max_day = days_in_month(value.year, value.month);
    if value.day == 0 || value.day > max_day {
        return Err(TemporalValueError::InvalidDay {
            year: value.year,
            month: value.month,
            day: value.day,
        });
    }

    if value.hour > 23 {
        return Err(TemporalValueError::InvalidHour(value.hour));
    }
    if value.minute > 59 {
        return Err(TemporalValueError::InvalidMinute(value.minute));
    }
    if value.second > 59 {
        return Err(TemporalValueError::InvalidSecond(value.second));
    }
    if value.nanosecond >= 1_000_000_000 {
        return Err(TemporalValueError::InvalidNanosecond(value.nanosecond));
    }
    if let Some(offset_seconds) = value.offset_seconds
        && !(-86_399..=86_399).contains(&offset_seconds)
    {
        return Err(TemporalValueError::InvalidOffsetSeconds(offset_seconds));
    }

    Ok(())
}

fn parse_date(input: &str) -> Result<(i32, u8, u8), TemporalValueError> {
    let bytes = input.as_bytes();
    if bytes.is_empty() {
        return Err(TemporalValueError::InvalidFormat);
    }

    let mut index = 0usize;
    if matches!(bytes[index], b'+' | b'-') {
        index += 1;
    }

    let digit_start = index;
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        index += 1;
    }

    if index.saturating_sub(digit_start) < 4 {
        return Err(TemporalValueError::InvalidFormat);
    }
    if index >= bytes.len() || bytes[index] != b'-' {
        return Err(TemporalValueError::InvalidFormat);
    }

    let year = input[..index]
        .parse::<i32>()
        .map_err(|_| TemporalValueError::InvalidFormat)?;
    index += 1;

    let month = parse_u8_exact(bytes, &mut index, 2)?;
    expect_byte(bytes, &mut index, b'-')?;
    let day = parse_u8_exact(bytes, &mut index, 2)?;

    if index != bytes.len() {
        return Err(TemporalValueError::InvalidFormat);
    }

    Ok((year, month, day))
}

fn parse_time(input: &str) -> Result<(u8, u8, u8, u32, Option<i32>), TemporalValueError> {
    let bytes = input.as_bytes();
    let mut index = 0usize;

    let hour = parse_u8_exact(bytes, &mut index, 2)?;
    expect_byte(bytes, &mut index, b':')?;
    let minute = parse_u8_exact(bytes, &mut index, 2)?;
    expect_byte(bytes, &mut index, b':')?;
    let second = parse_u8_exact(bytes, &mut index, 2)?;

    let mut nanosecond = 0u32;
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        let fraction_start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        let digits = &input[fraction_start..index];
        if digits.is_empty() || digits.len() > 9 {
            return Err(TemporalValueError::InvalidFormat);
        }

        let mut padded = digits.to_owned();
        while padded.len() < 9 {
            padded.push('0');
        }
        nanosecond = padded
            .parse::<u32>()
            .map_err(|_| TemporalValueError::InvalidFormat)?;
    }

    let offset_seconds = match bytes.get(index) {
        None => None,
        Some(b'Z') => {
            index += 1;
            Some(0)
        }
        Some(b'+') | Some(b'-') => Some(parse_offset(bytes, &mut index)?),
        Some(_) => return Err(TemporalValueError::InvalidFormat),
    };

    if index != bytes.len() {
        return Err(TemporalValueError::InvalidFormat);
    }

    Ok((hour, minute, second, nanosecond, offset_seconds))
}

fn parse_offset(bytes: &[u8], index: &mut usize) -> Result<i32, TemporalValueError> {
    let sign = match bytes.get(*index) {
        Some(b'+') => 1i32,
        Some(b'-') => -1i32,
        _ => return Err(TemporalValueError::InvalidFormat),
    };
    *index += 1;

    let hours = i32::from(parse_u8_exact(bytes, index, 2)?);
    expect_byte(bytes, index, b':')?;
    let minutes = i32::from(parse_u8_exact(bytes, index, 2)?);
    let seconds = if bytes.get(*index) == Some(&b':') {
        *index += 1;
        i32::from(parse_u8_exact(bytes, index, 2)?)
    } else {
        0
    };

    Ok(sign * (hours * 3_600 + minutes * 60 + seconds))
}

fn parse_u8_exact(bytes: &[u8], index: &mut usize, width: usize) -> Result<u8, TemporalValueError> {
    let end = index.saturating_add(width);
    if end > bytes.len() {
        return Err(TemporalValueError::InvalidFormat);
    }
    let slice = &bytes[*index..end];
    if !slice.iter().all(u8::is_ascii_digit) {
        return Err(TemporalValueError::InvalidFormat);
    }
    *index = end;
    std::str::from_utf8(slice)
        .ok()
        .and_then(|text| text.parse::<u8>().ok())
        .ok_or(TemporalValueError::InvalidFormat)
}

fn expect_byte(bytes: &[u8], index: &mut usize, expected: u8) -> Result<(), TemporalValueError> {
    if bytes.get(*index) != Some(&expected) {
        return Err(TemporalValueError::InvalidFormat);
    }
    *index += 1;
    Ok(())
}

fn format_year(year: i32) -> String {
    if (0..=9_999).contains(&year) {
        return format!("{year:04}");
    }

    let absolute = year.unsigned_abs();
    if year < 0 {
        let width = absolute.to_string().len().max(4);
        format!("-{absolute:0width$}")
    } else {
        let width = absolute.to_string().len().max(5);
        format!("+{absolute:0width$}")
    }
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::{DateTimeValue, TemporalValue, TemporalValueError};

    #[test]
    fn datetime_rejects_invalid_components() {
        assert_eq!(
            DateTimeValue::new(2024, 13, 1, 0, 0, 0, 0, None),
            Err(TemporalValueError::InvalidMonth(13))
        );
        assert_eq!(
            DateTimeValue::new(2023, 2, 29, 0, 0, 0, 0, None),
            Err(TemporalValueError::InvalidDay {
                year: 2023,
                month: 2,
                day: 29
            })
        );
        assert_eq!(
            DateTimeValue::new(2024, 1, 1, 24, 0, 0, 0, None),
            Err(TemporalValueError::InvalidHour(24))
        );
        assert_eq!(
            DateTimeValue::new(2024, 1, 1, 0, 0, 0, 0, Some(90_000)),
            Err(TemporalValueError::InvalidOffsetSeconds(90_000))
        );
    }

    #[test]
    fn datetime_formats_canonical_iso_strings() {
        let aware = DateTimeValue::new(2024, 1, 2, 3, 4, 5, 123_400_000, Some(3_600)).unwrap();
        assert_eq!(aware.to_string(), "2024-01-02T03:04:05.1234+01:00");

        let naive = DateTimeValue::new(2024, 1, 2, 3, 4, 5, 0, None).unwrap();
        assert_eq!(naive.to_string(), "2024-01-02T03:04:05");
    }

    #[test]
    fn datetime_parses_canonical_iso_strings() {
        let aware = DateTimeValue::parse_iso8601("2024-01-02T03:04:05.1234+01:00").unwrap();
        assert_eq!(
            aware,
            DateTimeValue::new(2024, 1, 2, 3, 4, 5, 123_400_000, Some(3_600)).unwrap()
        );

        let temporal = TemporalValue::parse_iso8601("2024-01-02T03:04:05Z").unwrap();
        assert_eq!(temporal.to_string(), "2024-01-02T03:04:05Z");
    }

    #[test]
    fn datetime_rejects_malformed_iso_strings() {
        for input in [
            "2024-01-02 03:04:05Z",
            "2024-01-02T03:04Z",
            "2024/01/02T03:04:05Z",
            "2024-01-02T03:04:05.Z",
            "2024-01-02T03:04:05+01",
            "2024-01-02T03:04:05+01:00Z",
        ] {
            assert_eq!(
                DateTimeValue::parse_iso8601(input),
                Err(TemporalValueError::InvalidFormat),
                "{input}"
            );
        }
    }

    #[test]
    fn datetime_round_trips_second_precision_offsets_and_variable_fraction_precision() {
        let aware = DateTimeValue::new(2024, 1, 2, 3, 4, 5, 100_000_000, Some(3_661)).unwrap();
        assert_eq!(aware.to_string(), "2024-01-02T03:04:05.1+01:01:01");
        assert_eq!(
            DateTimeValue::parse_iso8601(&aware.to_string()).unwrap(),
            aware
        );
    }
}
