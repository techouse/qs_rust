#[cfg(feature = "chrono")]
mod chrono {
    use super::super::{
        TemporalValue, Value, ValueDeserializer, ValueSerializer, from_value, to_value,
    };
    use serde::{Deserialize, Serialize};

    #[test]
    fn chrono_naive_temporal_field_helpers_round_trip_directly() {
        let naive = chrono::NaiveDate::from_ymd_opt(2024, 1, 2)
            .unwrap()
            .and_hms_opt(3, 4, 5)
            .unwrap();

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Query {
            #[serde(with = "crate::serde::temporal::chrono_naive_datetime")]
            at: chrono::NaiveDateTime,
        }

        let query = Query { at: naive };
        let value = to_value(&query).unwrap();
        assert_eq!(
            value,
            Value::Object(
                [(
                    "at".to_owned(),
                    Value::Temporal(TemporalValue::from(query.at))
                )]
                .into()
            )
        );

        let decoded: Query = from_value(&value).unwrap();
        assert_eq!(decoded, query);
        assert_eq!(
            crate::serde::temporal::chrono_naive_datetime::serialize(&naive, ValueSerializer)
                .unwrap(),
            Value::Temporal(TemporalValue::from(naive))
        );
        assert_eq!(
            crate::serde::temporal::chrono_naive_datetime::deserialize(ValueDeserializer::new(
                &Value::Temporal(TemporalValue::from(naive)),
            ))
            .unwrap(),
            naive
        );
    }

    #[test]
    fn chrono_temporal_field_helpers_preserve_temporal_leaves() {
        use chrono::{FixedOffset, TimeZone};

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Query {
            #[serde(with = "crate::serde::temporal::chrono_datetime")]
            at: chrono::DateTime<chrono::FixedOffset>,
        }

        let query = Query {
            at: FixedOffset::east_opt(3_600)
                .unwrap()
                .with_ymd_and_hms(2024, 1, 2, 3, 4, 5)
                .unwrap(),
        };

        let value = to_value(&query).unwrap();
        assert_eq!(
            value,
            Value::Object(
                [(
                    "at".to_owned(),
                    Value::Temporal(TemporalValue::from(query.at))
                )]
                .into()
            )
        );

        let decoded: Query = from_value(&value).unwrap();
        assert_eq!(decoded, query);

        let decoded_from_string: Query = from_value(&Value::Object(
            [(
                "at".to_owned(),
                Value::String("2024-01-02T03:04:05+01:00".to_owned()),
            )]
            .into(),
        ))
        .unwrap();
        assert_eq!(decoded_from_string, query);
    }

    #[test]
    fn chrono_temporal_field_helpers_reject_mismatched_and_invalid_strings() {
        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct AwareQuery {
            #[serde(with = "crate::serde::temporal::chrono_datetime")]
            at: chrono::DateTime<chrono::FixedOffset>,
        }

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct NaiveQuery {
            #[serde(with = "crate::serde::temporal::chrono_naive_datetime")]
            at: chrono::NaiveDateTime,
        }

        let aware_err = from_value::<AwareQuery>(&Value::Object(
            [(
                "at".to_owned(),
                Value::String("2024-01-02T03:04:05".to_owned()),
            )]
            .into(),
        ))
        .unwrap_err();
        assert!(aware_err.to_string().contains("missing a UTC offset"));

        let naive_err = from_value::<NaiveQuery>(&Value::Object(
            [(
                "at".to_owned(),
                Value::String("2024-01-02T03:04:05+01:00".to_owned()),
            )]
            .into(),
        ))
        .unwrap_err();
        assert!(
            naive_err
                .to_string()
                .contains("unexpectedly contains a UTC offset")
        );

        let invalid_err = from_value::<AwareQuery>(&Value::Object(
            [("at".to_owned(), Value::String("not-a-datetime".to_owned()))].into(),
        ))
        .unwrap_err();
        assert!(invalid_err.to_string().contains("invalid datetime format"));
    }

    #[test]
    fn chrono_temporal_field_helpers_round_trip_nested_structs() {
        use chrono::{FixedOffset, TimeZone};

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Inner {
            #[serde(with = "crate::serde::temporal::chrono_datetime")]
            at: chrono::DateTime<chrono::FixedOffset>,
        }

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Outer {
            inner: Inner,
        }

        let query = Outer {
            inner: Inner {
                at: FixedOffset::east_opt(3_600)
                    .unwrap()
                    .with_ymd_and_hms(2024, 1, 2, 3, 4, 5)
                    .unwrap(),
            },
        };

        let value = to_value(&query).unwrap();
        assert_eq!(
            value,
            Value::Object(
                [(
                    "inner".to_owned(),
                    Value::Object(
                        [(
                            "at".to_owned(),
                            Value::Temporal(TemporalValue::from(query.inner.at))
                        )]
                        .into()
                    ),
                )]
                .into()
            )
        );

        let decoded: Outer = from_value(&value).unwrap();
        assert_eq!(decoded, query);
    }
}

#[cfg(feature = "time")]
mod time {
    use super::super::{
        TemporalValue, Value, ValueDeserializer, ValueSerializer, from_value, to_value,
    };
    use serde::{Deserialize, Serialize};

    #[test]
    fn time_primitive_temporal_field_helpers_round_trip_directly() {
        use time::{Date, Month};

        let primitive = Date::from_calendar_date(2024, Month::January, 2)
            .unwrap()
            .with_hms(3, 4, 5)
            .unwrap();

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Query {
            #[serde(with = "crate::serde::temporal::time_primitive_datetime")]
            at: time::PrimitiveDateTime,
        }

        let query = Query { at: primitive };
        let value = to_value(&query).unwrap();
        assert_eq!(
            value,
            Value::Object(
                [(
                    "at".to_owned(),
                    Value::Temporal(TemporalValue::from(query.at))
                )]
                .into()
            )
        );

        let decoded: Query = from_value(&value).unwrap();
        assert_eq!(decoded, query);
        assert_eq!(
            crate::serde::temporal::time_primitive_datetime::serialize(&primitive, ValueSerializer)
                .unwrap(),
            Value::Temporal(TemporalValue::from(primitive))
        );
        assert_eq!(
            crate::serde::temporal::time_primitive_datetime::deserialize(ValueDeserializer::new(
                &Value::Temporal(TemporalValue::from(primitive)),
            ))
            .unwrap(),
            primitive
        );
    }

    #[test]
    fn time_temporal_field_helpers_preserve_temporal_leaves() {
        use time::{Date, Month, UtcOffset};

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Query {
            #[serde(with = "crate::serde::temporal::time_offset_datetime")]
            at: time::OffsetDateTime,
        }

        let query = Query {
            at: Date::from_calendar_date(2024, Month::January, 2)
                .unwrap()
                .with_hms(3, 4, 5)
                .unwrap()
                .assume_offset(UtcOffset::from_hms(1, 0, 0).unwrap()),
        };

        let value = to_value(&query).unwrap();
        assert_eq!(
            value,
            Value::Object(
                [(
                    "at".to_owned(),
                    Value::Temporal(TemporalValue::from(query.at))
                )]
                .into()
            )
        );

        let decoded: Query = from_value(&value).unwrap();
        assert_eq!(decoded, query);

        let decoded_from_string: Query = from_value(&Value::Object(
            [(
                "at".to_owned(),
                Value::String("2024-01-02T03:04:05+01:00".to_owned()),
            )]
            .into(),
        ))
        .unwrap();
        assert_eq!(decoded_from_string, query);
    }

    #[test]
    fn time_temporal_field_helpers_reject_mismatched_and_invalid_strings() {
        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct AwareQuery {
            #[serde(with = "crate::serde::temporal::time_offset_datetime")]
            at: time::OffsetDateTime,
        }

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct NaiveQuery {
            #[serde(with = "crate::serde::temporal::time_primitive_datetime")]
            at: time::PrimitiveDateTime,
        }

        let aware_err = from_value::<AwareQuery>(&Value::Object(
            [(
                "at".to_owned(),
                Value::String("2024-01-02T03:04:05".to_owned()),
            )]
            .into(),
        ))
        .unwrap_err();
        assert!(aware_err.to_string().contains("missing a UTC offset"));

        let naive_err = from_value::<NaiveQuery>(&Value::Object(
            [(
                "at".to_owned(),
                Value::String("2024-01-02T03:04:05+01:00".to_owned()),
            )]
            .into(),
        ))
        .unwrap_err();
        assert!(
            naive_err
                .to_string()
                .contains("unexpectedly contains a UTC offset")
        );

        let invalid_err = from_value::<AwareQuery>(&Value::Object(
            [("at".to_owned(), Value::String("not-a-datetime".to_owned()))].into(),
        ))
        .unwrap_err();
        assert!(invalid_err.to_string().contains("invalid datetime format"));
    }

    #[test]
    fn time_temporal_field_helpers_round_trip_nested_structs() {
        use time::{Date, Month, UtcOffset};

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Inner {
            #[serde(with = "crate::serde::temporal::time_offset_datetime")]
            at: time::OffsetDateTime,
        }

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Outer {
            inner: Inner,
        }

        let query = Outer {
            inner: Inner {
                at: Date::from_calendar_date(2024, Month::January, 2)
                    .unwrap()
                    .with_hms(3, 4, 5)
                    .unwrap()
                    .assume_offset(UtcOffset::from_hms(1, 0, 0).unwrap()),
            },
        };

        let value = to_value(&query).unwrap();
        assert_eq!(
            value,
            Value::Object(
                [(
                    "inner".to_owned(),
                    Value::Object(
                        [(
                            "at".to_owned(),
                            Value::Temporal(TemporalValue::from(query.inner.at))
                        )]
                        .into()
                    ),
                )]
                .into()
            )
        );

        let decoded: Outer = from_value(&value).unwrap();
        assert_eq!(decoded, query);
    }
}
