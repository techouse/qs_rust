use std::cmp::Ordering;

use super::EncodeOptions;
use crate::error::EncodeError;
use crate::temporal::TemporalValue;
use crate::value::Value;
use crate::{
    Charset, EncodeFilter, EncodeToken, EncodeTokenEncoder, FilterResult, Format, FunctionFilter,
    SortMode, Sorter, TemporalSerializer, WhitelistSelector,
};

#[test]
fn getters_and_builders_cover_encode_specific_configuration() {
    let options = EncodeOptions::new()
        .with_skip_nulls(true)
        .with_comma_round_trip(true)
        .with_comma_compact_nulls(true)
        .with_encode_values_only(true)
        .with_add_query_prefix(true)
        .with_encode_dot_in_keys(true)
        .with_filter(Some(EncodeFilter::Function(FunctionFilter::new(|_, _| {
            FilterResult::Keep
        }))))
        .with_sort(SortMode::LexicographicAsc)
        .with_sorter(Some(Sorter::new(|left, right| {
            left.len().cmp(&right.len())
        })))
        .with_encoder(Some(EncodeTokenEncoder::new(|token, _, _| match token {
            EncodeToken::Key(text) | EncodeToken::TextValue(text) => format!("enc:{text}"),
            EncodeToken::Value(Value::String(text)) => format!("enc:{text}"),
            EncodeToken::Value(_) => "enc:other".to_owned(),
        })))
        .with_temporal_serializer(Some(TemporalSerializer::new(|_| {
            Some("temporal".to_owned())
        })))
        .with_max_depth(Some(3));

    assert!(options.skip_nulls());
    assert!(options.comma_round_trip());
    assert!(options.comma_compact_nulls());
    assert!(options.encode_values_only());
    assert!(options.add_query_prefix());
    assert!(options.allow_dots());
    assert!(options.encode_dot_in_keys());
    assert!(matches!(options.filter(), Some(EncodeFilter::Function(_))));
    assert_eq!(options.sort(), SortMode::LexicographicAsc);
    assert!(options.sorter().is_some());
    assert!(options.encoder().is_some());
    assert!(options.temporal_serializer().is_some());
    assert_eq!(options.max_depth(), Some(3));
    assert!(options.has_temporal_serializer());

    let cleared = options.clone().with_allow_dots(false);
    assert!(!cleared.allow_dots());
    assert!(!cleared.encode_dot_in_keys());

    let whitelist = EncodeOptions::new().with_whitelist(Some(vec![
        WhitelistSelector::Key("a".to_owned()),
        WhitelistSelector::Index(1),
    ]));
    assert!(matches!(
        whitelist.filter(),
        Some(EncodeFilter::Whitelist(entries)) if entries.len() == 2
    ));
    assert_eq!(
        whitelist.whitelist(),
        Some(
            &[
                WhitelistSelector::Key("a".to_owned()),
                WhitelistSelector::Index(1),
            ][..]
        )
    );

    let no_whitelist = EncodeOptions::new().with_filter(Some(EncodeFilter::Function(
        FunctionFilter::new(|_, _| FilterResult::Keep),
    )));
    assert!(no_whitelist.whitelist().is_none());

    let sorter = options.sorter().unwrap();
    assert_eq!(sorter.compare("aa", "b"), Ordering::Greater);
    let filter = match options.filter().unwrap() {
        EncodeFilter::Function(filter) => filter,
        other => panic!("expected function filter, got {other:?}"),
    };
    assert_eq!(filter.apply("root", &Value::Null), FilterResult::Keep);

    let encoder = options.encoder().unwrap();
    assert_eq!(
        encoder.encode(EncodeToken::Key("dot"), Charset::Utf8, Format::Rfc3986),
        "enc:dot"
    );
    assert_eq!(
        encoder.encode(
            EncodeToken::TextValue("joined"),
            Charset::Utf8,
            Format::Rfc3986,
        ),
        "enc:joined"
    );
    assert_eq!(
        encoder.encode(
            EncodeToken::Value(&Value::String("leaf".to_owned())),
            Charset::Utf8,
            Format::Rfc3986,
        ),
        "enc:leaf"
    );
    assert_eq!(
        encoder.encode(
            EncodeToken::Value(&Value::Bool(true)),
            Charset::Utf8,
            Format::Rfc3986,
        ),
        "enc:other"
    );
    let temporal = TemporalValue::datetime(2024, 1, 2, 3, 4, 5, 0, None).unwrap();
    assert_eq!(
        options.temporal_serializer().unwrap().serialize(&temporal),
        Some("temporal".to_owned())
    );
}

#[test]
fn validate_rejects_impossible_dot_configuration() {
    let options = EncodeOptions {
        allow_dots: false,
        encode_dot_in_keys: true,
        ..EncodeOptions::new()
    };

    assert!(matches!(
        options.validate(),
        Err(EncodeError::EncodeDotInKeysRequiresAllowDots)
    ));
}
