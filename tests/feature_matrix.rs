#[cfg(feature = "serde")]
#[test]
fn serde_feature_public_bridge_smoke_works() {
    use qs_rust::{DecodeOptions, EncodeOptions, from_str, to_string};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct Query {
        page: String,
        tags: Vec<String>,
    }

    let decoded: Query = from_str("page=2&tags[0]=rust&tags[1]=qs", &DecodeOptions::new()).unwrap();
    assert_eq!(
        decoded,
        Query {
            page: "2".to_owned(),
            tags: vec!["rust".to_owned(), "qs".to_owned()],
        }
    );

    let encoded = to_string(&decoded, &EncodeOptions::new().with_encode(false)).unwrap();
    assert_eq!(encoded, "page=2&tags[0]=rust&tags[1]=qs");
}

#[cfg(feature = "chrono")]
#[test]
fn chrono_feature_public_adapter_smoke_works() {
    use chrono::{TimeZone, Utc};
    use qs_rust::{EncodeOptions, TemporalSerializer, TemporalValue, Value, chrono_support};

    let timestamp = Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap();
    let options =
        EncodeOptions::new().with_temporal_serializer(Some(TemporalSerializer::new(|value| {
            match value {
                TemporalValue::DateTime(date) => Some(format!("wrapped:{}", date.year())),
            }
        })));

    assert_eq!(
        chrono_support::to_value(&timestamp),
        Value::Temporal(TemporalValue::from(timestamp))
    );
    assert_eq!(
        chrono_support::encode_at_key("at", &timestamp, &options).unwrap(),
        "at=wrapped%3A2024"
    );
}

#[cfg(feature = "time")]
#[test]
fn time_feature_public_adapter_smoke_works() {
    use qs_rust::{EncodeOptions, TemporalSerializer, TemporalValue, Value, time_support};
    use time::OffsetDateTime;

    let timestamp = OffsetDateTime::from_unix_timestamp(1_704_165_845).unwrap();
    let options =
        EncodeOptions::new().with_temporal_serializer(Some(TemporalSerializer::new(|value| {
            match value {
                TemporalValue::DateTime(date) => Some(format!("wrapped:{}", date.year())),
            }
        })));

    assert_eq!(
        time_support::to_value(&timestamp),
        Value::Temporal(TemporalValue::from(timestamp))
    );
    assert_eq!(
        time_support::encode_at_key("at", &timestamp, &options).unwrap(),
        "at=wrapped%3A2024"
    );
}

#[cfg(all(feature = "chrono", feature = "time"))]
#[test]
fn chrono_and_time_feature_serializers_can_coexist() {
    use chrono::{TimeZone, Utc};
    use qs_rust::{EncodeOptions, TemporalSerializer, Value, chrono_support, encode, time_support};
    use time::OffsetDateTime;

    let options = EncodeOptions::new()
        .with_encode(false)
        .with_temporal_serializer(Some(TemporalSerializer::new(|value| {
            Some(format!("seen:{}", value))
        })));

    let value = Value::Object(
        [
            (
                "chrono".to_owned(),
                chrono_support::to_value(&Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap()),
            ),
            (
                "time".to_owned(),
                time_support::to_value(
                    &OffsetDateTime::from_unix_timestamp(1_704_165_845).unwrap(),
                ),
            ),
        ]
        .into(),
    );

    assert_eq!(
        encode(&value, &options).unwrap(),
        "chrono=seen:2024-01-02T03:04:05Z&time=seen:2024-01-02T03:24:05Z"
    );
}
