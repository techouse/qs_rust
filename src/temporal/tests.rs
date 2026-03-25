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
