use std::str::FromStr;

use super::{
    DateTimeValue, TemporalValue, TemporalValueError, days_in_month, expect_byte, format_year,
    parse_date, parse_offset, parse_time, parse_u8_exact,
};

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

#[test]
fn public_temporal_surface_exposes_constructors_accessors_and_from_str() {
    let temporal = TemporalValue::datetime(2024, 1, 2, 3, 4, 5, 600_000_000, Some(-3_661)).unwrap();
    let datetime = temporal.as_datetime().unwrap().clone();

    assert_eq!(datetime.year(), 2024);
    assert_eq!(datetime.month(), 1);
    assert_eq!(datetime.day(), 2);
    assert_eq!(datetime.hour(), 3);
    assert_eq!(datetime.minute(), 4);
    assert_eq!(datetime.second(), 5);
    assert_eq!(datetime.nanosecond(), 600_000_000);
    assert_eq!(datetime.offset_seconds(), Some(-3_661));
    assert_eq!(datetime.to_string(), "2024-01-02T03:04:05.6-01:01:01");

    assert_eq!(
        TemporalValue::from_str("2024-01-02T03:04:05.6-01:01:01").unwrap(),
        temporal
    );
    assert_eq!(
        DateTimeValue::from_str("2024-01-02T03:04:05.6-01:01:01").unwrap(),
        datetime
    );
}

#[test]
fn internal_temporal_helpers_cover_remaining_parse_and_format_edges() {
    assert_eq!(format_year(-12), "-0012");
    assert_eq!(format_year(12_345), "+12345");
    assert_eq!(days_in_month(2024, 2), 29);
    assert_eq!(days_in_month(2023, 2), 28);
    assert_eq!(days_in_month(2024, 13), 0);

    assert_eq!(
        DateTimeValue::new(2024, 1, 1, 0, 60, 0, 0, None),
        Err(TemporalValueError::InvalidMinute(60))
    );
    assert_eq!(
        DateTimeValue::new(2024, 1, 1, 0, 0, 60, 0, None),
        Err(TemporalValueError::InvalidSecond(60))
    );
    assert_eq!(
        DateTimeValue::new(2024, 1, 1, 0, 0, 0, 1_000_000_000, None),
        Err(TemporalValueError::InvalidNanosecond(1_000_000_000))
    );

    assert_eq!(parse_date(""), Err(TemporalValueError::InvalidFormat));
    assert_eq!(
        parse_date("123-01-02"),
        Err(TemporalValueError::InvalidFormat)
    );
    assert_eq!(
        parse_date("2024-01-02x"),
        Err(TemporalValueError::InvalidFormat)
    );

    assert_eq!(
        parse_time("03:04:0"),
        Err(TemporalValueError::InvalidFormat)
    );
    assert_eq!(
        parse_time("03:04:0x"),
        Err(TemporalValueError::InvalidFormat)
    );
    assert_eq!(
        parse_time("03:04:05Q"),
        Err(TemporalValueError::InvalidFormat)
    );

    let mut index = 0usize;
    assert_eq!(
        parse_offset(b"x", &mut index),
        Err(TemporalValueError::InvalidFormat)
    );

    let mut index = 0usize;
    assert_eq!(
        parse_u8_exact(b"7", &mut index, 2),
        Err(TemporalValueError::InvalidFormat)
    );

    let mut index = 0usize;
    assert_eq!(
        parse_u8_exact(b"7x", &mut index, 2),
        Err(TemporalValueError::InvalidFormat)
    );

    let mut index = 0usize;
    assert_eq!(
        expect_byte(b"x", &mut index, b'y'),
        Err(TemporalValueError::InvalidFormat)
    );
}
