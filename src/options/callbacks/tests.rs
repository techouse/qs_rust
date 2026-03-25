use std::cmp::Ordering;

use super::{
    DecodeDecoder, EncodeFilter, EncodeToken, EncodeTokenEncoder, FilterResult, FunctionFilter,
    Sorter, TemporalSerializer,
};
use crate::temporal::{DateTimeValue, TemporalValue};
use crate::value::Value;
use crate::{Charset, DecodeKind, Format, WhitelistSelector};

fn sample_temporal() -> TemporalValue {
    TemporalValue::from(DateTimeValue::new(2024, 1, 2, 3, 4, 5, 0, Some(0)).unwrap())
}

#[test]
fn callback_wrappers_invoke_closures_and_expose_debug_placeholders() {
    let decoder =
        DecodeDecoder::new(|input, charset, kind| format!("{input}:{charset:?}:{kind:?}"));
    assert_eq!(
        decoder.decode("a", Charset::Utf8, DecodeKind::Key),
        "a:Utf8:Key"
    );
    assert_eq!(format!("{decoder:?}"), "DecodeDecoder(<closure>)");

    let encoder = EncodeTokenEncoder::new(|token, charset, format| match token {
        EncodeToken::Key(key) => format!("{key}:{charset:?}:{format:?}"),
        EncodeToken::Value(Value::String(text)) => format!("{text}:{charset:?}:{format:?}"),
        EncodeToken::Value(_) => "value".to_owned(),
        EncodeToken::TextValue(text) => format!("{text}:{charset:?}:{format:?}"),
    });
    assert_eq!(
        encoder.encode(EncodeToken::Key("field"), Charset::Utf8, Format::Rfc3986),
        "field:Utf8:Rfc3986"
    );
    assert_eq!(
        encoder.encode(
            EncodeToken::Value(&Value::String("value".to_owned())),
            Charset::Iso88591,
            Format::Rfc1738,
        ),
        "value:Iso88591:Rfc1738"
    );
    assert_eq!(
        encoder.encode(
            EncodeToken::TextValue("joined"),
            Charset::Utf8,
            Format::Rfc3986
        ),
        "joined:Utf8:Rfc3986"
    );
    assert_eq!(format!("{encoder:?}"), "EncodeTokenEncoder(<closure>)");

    let filter = FunctionFilter::new(|prefix, value| {
        if prefix == "drop" {
            FilterResult::Omit
        } else {
            FilterResult::Replace(Value::String(format!("{prefix}:{value:?}")))
        }
    });
    assert_eq!(filter.apply("drop", &Value::Null), FilterResult::Omit);
    assert_eq!(
        filter.apply("keep", &Value::Bool(true)),
        FilterResult::Replace(Value::String("keep:Bool(true)".to_owned()))
    );
    assert_eq!(format!("{filter:?}"), "FunctionFilter(<closure>)");

    let whitelist = EncodeFilter::Whitelist(vec![
        WhitelistSelector::Key("field".to_owned()),
        WhitelistSelector::Index(1),
    ]);
    match whitelist {
        EncodeFilter::Whitelist(entries) => {
            assert_eq!(
                entries,
                vec![
                    WhitelistSelector::Key("field".to_owned()),
                    WhitelistSelector::Index(1),
                ]
            );
        }
        EncodeFilter::Function(_) => panic!("expected whitelist"),
    }

    let sorter = Sorter::new(|left, right| left.len().cmp(&right.len()));
    assert_eq!(sorter.compare("a", "bbb"), Ordering::Less);
    assert_eq!(format!("{sorter:?}"), "Sorter(<closure>)");

    let temporal_serializer = TemporalSerializer::new(|value| Some(format!("ts:{value}")));
    assert_eq!(
        temporal_serializer.serialize(&sample_temporal()),
        Some("ts:2024-01-02T03:04:05Z".to_owned())
    );
    assert_eq!(
        format!("{temporal_serializer:?}"),
        "TemporalSerializer(<closure>)"
    );
}
