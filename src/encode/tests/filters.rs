use super::{
    EncodeFilter, EncodeOptions, FilterResult, FunctionFilter, ListFormat, Value,
    WhitelistSelector, encode,
};

#[test]
fn function_filter_can_omit_or_replace_values() {
    let value = Value::Object(
        [
            ("keep".to_owned(), Value::String("x".to_owned())),
            ("drop".to_owned(), Value::String("y".to_owned())),
            ("rename".to_owned(), Value::String("z".to_owned())),
        ]
        .into(),
    );

    let encoded = encode(
        &value,
        &EncodeOptions::new()
            .with_encode(false)
            .with_filter(Some(EncodeFilter::Function(FunctionFilter::new(
                |prefix, value| {
                    if prefix.ends_with("drop") {
                        FilterResult::Omit
                    } else if prefix.ends_with("rename") {
                        FilterResult::Replace(Value::String(format!(
                            "{}-mutated",
                            match value {
                                Value::String(text) => text,
                                _ => "",
                            }
                        )))
                    } else {
                        FilterResult::Keep
                    }
                },
            )))),
    )
    .unwrap();

    assert_eq!(encoded, "keep=x&rename=z-mutated");
}

#[test]
fn filter_result_omit_differs_from_replace_null() {
    let value = Value::Object(
        [
            ("keep".to_owned(), Value::String("x".to_owned())),
            ("omit".to_owned(), Value::String("y".to_owned())),
            ("nullify".to_owned(), Value::String("z".to_owned())),
        ]
        .into(),
    );

    let encoded = encode(
        &value,
        &EncodeOptions::new()
            .with_encode(false)
            .with_filter(Some(EncodeFilter::Function(FunctionFilter::new(
                |prefix, _| {
                    if prefix.ends_with("omit") {
                        FilterResult::Omit
                    } else if prefix.ends_with("nullify") {
                        FilterResult::Replace(Value::Null)
                    } else {
                        FilterResult::Keep
                    }
                },
            )))),
    )
    .unwrap();

    assert_eq!(encoded, "keep=x&nullify=");
}

#[test]
fn function_filter_omission_matches_list_format_semantics() {
    let value = Value::Object(
        [(
            "a".to_owned(),
            Value::Array(vec![
                Value::String("x".to_owned()),
                Value::String("y".to_owned()),
                Value::String("z".to_owned()),
            ]),
        )]
        .into(),
    );

    let filtered = |list_format| {
        encode(
            &value,
            &EncodeOptions::new()
                .with_encode(false)
                .with_list_format(list_format)
                .with_filter(Some(EncodeFilter::Function(FunctionFilter::new(
                    |_, value| {
                        if matches!(value, Value::String(text) if text == "y") {
                            FilterResult::Omit
                        } else {
                            FilterResult::Keep
                        }
                    },
                )))),
        )
        .unwrap()
    };

    assert_eq!(filtered(ListFormat::Indices), "a[0]=x&a[2]=z");
    assert_eq!(filtered(ListFormat::Brackets), "a[]=x&a[]=z");
    assert_eq!(filtered(ListFormat::Repeat), "a=x&a=z");
    assert_eq!(filtered(ListFormat::Comma), "a=x,z");
}

#[test]
fn whitelist_out_of_range_indices_are_omitted() {
    let value = Value::Object(
        [(
            "a".to_owned(),
            Value::Array(vec![
                Value::String("x".to_owned()),
                Value::String("y".to_owned()),
            ]),
        )]
        .into(),
    );

    let encoded = encode(
        &value,
        &EncodeOptions::new()
            .with_encode(false)
            .with_whitelist(Some(vec![
                WhitelistSelector::Key("a".to_owned()),
                WhitelistSelector::Index(0),
                WhitelistSelector::Index(4),
            ])),
    )
    .unwrap();

    assert_eq!(encoded, "a[0]=x");
}

#[test]
fn omitted_children_do_not_emit_empty_parent_containers() {
    let value = Value::Object(
        [(
            "outer".to_owned(),
            Value::Object([("secret".to_owned(), Value::String("x".to_owned()))].into()),
        )]
        .into(),
    );

    let encoded = encode(
        &value,
        &EncodeOptions::new()
            .with_encode(false)
            .with_filter(Some(EncodeFilter::Function(FunctionFilter::new(
                |prefix, _| {
                    if prefix.ends_with("[secret]") {
                        FilterResult::Omit
                    } else {
                        FilterResult::Keep
                    }
                },
            )))),
    )
    .unwrap();

    assert_eq!(encoded, "");
}

#[test]
fn root_function_filter_can_omit_entire_output() {
    let value = Value::Object([("a".to_owned(), Value::String("x".to_owned()))].into());

    let encoded = encode(
        &value,
        &EncodeOptions::new()
            .with_encode(false)
            .with_filter(Some(EncodeFilter::Function(FunctionFilter::new(
                |prefix, _| {
                    if prefix.is_empty() {
                        FilterResult::Omit
                    } else {
                        FilterResult::Keep
                    }
                },
            )))),
    )
    .unwrap();

    assert_eq!(encoded, "");
}

#[test]
fn root_function_filter_non_container_reuses_original_root() {
    let value = Value::Object([("a".to_owned(), Value::String("x".to_owned()))].into());
    let encoded = encode(
        &value,
        &EncodeOptions::new()
            .with_encode(false)
            .with_filter(Some(EncodeFilter::Function(FunctionFilter::new(
                |prefix, _| {
                    if prefix.is_empty() {
                        FilterResult::Replace(Value::String("not-a-container".to_owned()))
                    } else {
                        FilterResult::Keep
                    }
                },
            )))),
    )
    .unwrap();

    assert_eq!(encoded, "a=x");
}
