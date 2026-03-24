use super::{
    EncodeFilter, EncodeOptions, FilterResult, FunctionFilter, KeyPathNode, ListFormat, Value,
    WhitelistSelector, encode, try_encode_linear_map_chain,
};

#[test]
fn linear_chain_fast_path_matches_expected_output() {
    let value = Value::Object(
        [(
            "a".to_owned(),
            Value::Object([("leaf".to_owned(), Value::String("x".to_owned()))].into()),
        )]
        .into(),
    );
    let output = try_encode_linear_map_chain(
        &value,
        &KeyPathNode::from_raw("root"),
        &EncodeOptions::default(),
        0,
    )
    .unwrap();
    assert_eq!(output, "root%5Ba%5D%5Bleaf%5D=x");
}

#[test]
fn linear_chain_fast_path_supports_encode_false_output() {
    let value = Value::Object(
        [(
            "a".to_owned(),
            Value::Object([("leaf".to_owned(), Value::String("x".to_owned()))].into()),
        )]
        .into(),
    );
    let output = try_encode_linear_map_chain(
        &value,
        &KeyPathNode::from_raw("root"),
        &EncodeOptions::new().with_encode(false),
        0,
    )
    .unwrap();
    assert_eq!(output, "root[a][leaf]=x");
}

#[test]
fn linear_chain_fast_path_rejects_comma_lists() {
    let value = Value::Object(
        [(
            "a".to_owned(),
            Value::Object([("leaf".to_owned(), Value::String("x".to_owned()))].into()),
        )]
        .into(),
    );
    assert!(
        try_encode_linear_map_chain(
            &value,
            &KeyPathNode::from_raw("root"),
            &EncodeOptions::new().with_list_format(ListFormat::Comma),
            0,
        )
        .is_none()
    );
}

#[test]
fn linear_chain_fast_path_rejects_allow_dots() {
    let value = Value::Object(
        [(
            "a".to_owned(),
            Value::Object([("leaf".to_owned(), Value::String("x".to_owned()))].into()),
        )]
        .into(),
    );
    assert!(
        try_encode_linear_map_chain(
            &value,
            &KeyPathNode::from_raw("root"),
            &EncodeOptions::new().with_allow_dots(true),
            0,
        )
        .is_none()
    );
}

#[test]
fn linear_chain_fast_path_rejects_multi_key_nodes() {
    let value = Value::Object(
        [
            (
                "a".to_owned(),
                Value::Object([("leaf".to_owned(), Value::String("x".to_owned()))].into()),
            ),
            ("b".to_owned(), Value::String("y".to_owned())),
        ]
        .into(),
    );
    assert!(
        try_encode_linear_map_chain(
            &value,
            &KeyPathNode::from_raw("root"),
            &EncodeOptions::new(),
            0,
        )
        .is_none()
    );
}

#[test]
fn linear_chain_fast_path_matches_generic_encode_output() {
    let value = Value::Object(
        [(
            "root".to_owned(),
            Value::Object(
                [(
                    "child".to_owned(),
                    Value::Object([("leaf".to_owned(), Value::String("x".to_owned()))].into()),
                )]
                .into(),
            ),
        )]
        .into(),
    );

    let fast = encode(&value, &EncodeOptions::new()).unwrap();
    let generic = encode(
        &value,
        &EncodeOptions::new().with_whitelist(Some(vec![
            WhitelistSelector::Key("root".to_owned()),
            WhitelistSelector::Key("child".to_owned()),
            WhitelistSelector::Key("leaf".to_owned()),
        ])),
    )
    .unwrap();

    assert_eq!(fast, generic);
}

#[test]
fn linear_chain_encode_false_fast_path_matches_generic_output() {
    let value = Value::Object(
        [(
            "root".to_owned(),
            Value::Object(
                [(
                    "child".to_owned(),
                    Value::Object([("leaf".to_owned(), Value::String("x".to_owned()))].into()),
                )]
                .into(),
            ),
        )]
        .into(),
    );

    let fast = encode(&value, &EncodeOptions::new().with_encode(false)).unwrap();
    let generic = encode(
        &value,
        &EncodeOptions::new()
            .with_encode(false)
            .with_whitelist(Some(vec![
                WhitelistSelector::Key("root".to_owned()),
                WhitelistSelector::Key("child".to_owned()),
                WhitelistSelector::Key("leaf".to_owned()),
            ])),
    )
    .unwrap();

    assert_eq!(fast, generic);
}

#[test]
fn function_filter_replacement_bypasses_linear_chain_fast_path() {
    let value = Value::Object(
        [(
            "root".to_owned(),
            Value::Object([("leaf".to_owned(), Value::String("x".to_owned()))].into()),
        )]
        .into(),
    );

    let encoded = encode(
        &value,
        &EncodeOptions::new()
            .with_encode(false)
            .with_filter(Some(EncodeFilter::Function(FunctionFilter::new(
                |prefix, value| {
                    if prefix.ends_with("[leaf]") {
                        FilterResult::Replace(Value::String(format!(
                            "{}!",
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

    assert_eq!(encoded, "root[leaf]=x!");
}

#[test]
fn dot_encoding_matches_node_for_nested_object_paths() {
    let value = Value::Object(
        [(
            "a".to_owned(),
            Value::Object(
                [(
                    "b".to_owned(),
                    Value::Object([("c.d".to_owned(), Value::String("x".to_owned()))].into()),
                )]
                .into(),
            ),
        )]
        .into(),
    );

    let encoded = encode(
        &value,
        &EncodeOptions::new()
            .with_allow_dots(true)
            .with_encode_dot_in_keys(true),
    )
    .unwrap();
    assert_eq!(encoded, "a%252Eb.c%252Ed=x");
}
