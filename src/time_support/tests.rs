use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

use super::{encode_at_key, to_value};
use crate::{
    EncodeOptions, ListFormat, TemporalSerializer, TemporalValue, TemporalValueError, Value, encode,
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
    let value = Value::Object([("at".to_owned(), Value::Array(vec![to_value(&timestamp)]))].into());

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
