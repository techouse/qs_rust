use chrono::{FixedOffset, NaiveDate, TimeZone};

use super::{encode_at_key, to_value};
use crate::{EncodeOptions, TemporalSerializer, TemporalValue, TemporalValueError, Value, encode};

#[test]
fn chrono_values_use_the_core_temporal_serializer_when_present() {
    let timestamp = FixedOffset::east_opt(3_600)
        .unwrap()
        .timestamp_opt(42, 0)
        .unwrap();
    let options =
        EncodeOptions::new().with_temporal_serializer(Some(TemporalSerializer::new(|value| {
            match value {
                TemporalValue::DateTime(date) => Some(format!(
                    "offset:{}:{}",
                    date.offset_seconds().unwrap_or_default(),
                    date.second()
                )),
            }
        })));

    assert_eq!(
        to_value(&timestamp),
        Value::Temporal(TemporalValue::from(timestamp))
    );
    assert_eq!(
        encode_at_key("at", &timestamp, &options).unwrap(),
        "at=offset%3A3600%3A42"
    );
}

#[test]
fn chrono_nested_arrays_preserve_temporal_output() {
    let timestamp = FixedOffset::east_opt(0)
        .unwrap()
        .timestamp_opt(42, 0)
        .unwrap();
    let options = EncodeOptions::new().with_encode(false);
    let value = Value::Object([("at".to_owned(), Value::Array(vec![to_value(&timestamp)]))].into());

    assert_eq!(
        encode(&value, &options).unwrap(),
        "at[0]=1970-01-01T00:00:42Z"
    );
}

#[test]
fn chrono_round_trips_through_core_temporal_value() {
    let aware = FixedOffset::east_opt(3_600)
        .unwrap()
        .with_ymd_and_hms(2024, 1, 2, 3, 4, 5)
        .unwrap();
    let temporal = TemporalValue::from(aware);
    let round_trip = chrono::DateTime::<FixedOffset>::try_from(&temporal).unwrap();
    assert_eq!(round_trip, aware);

    let naive = NaiveDate::from_ymd_opt(2024, 1, 2)
        .unwrap()
        .and_hms_opt(3, 4, 5)
        .unwrap();
    let temporal = TemporalValue::from(naive);
    let round_trip = chrono::NaiveDateTime::try_from(&temporal).unwrap();
    assert_eq!(round_trip, naive);
}

#[test]
fn chrono_aware_conversion_requires_a_temporal_offset() {
    let naive = NaiveDate::from_ymd_opt(2024, 1, 2)
        .unwrap()
        .and_hms_opt(3, 4, 5)
        .unwrap();
    let temporal = TemporalValue::from(naive);

    assert_eq!(
        chrono::DateTime::<FixedOffset>::try_from(&temporal),
        Err(TemporalValueError::MissingOffset)
    );
}

#[test]
fn chrono_naive_conversion_rejects_offset_aware_temporals() {
    let aware = FixedOffset::east_opt(3_600)
        .unwrap()
        .with_ymd_and_hms(2024, 1, 2, 3, 4, 5)
        .unwrap();
    let temporal = TemporalValue::from(aware);

    assert_eq!(
        chrono::NaiveDateTime::try_from(&temporal),
        Err(TemporalValueError::UnexpectedOffset)
    );
}
