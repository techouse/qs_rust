use super::{
    EncodeInput, apply_filter_result, encode_node_filtered, filter_root_value, has_filter_control,
    has_function_filter,
};
use crate::key_path::KeyPathNode;
use crate::options::{
    EncodeFilter, EncodeOptions, FilterResult, FunctionFilter, ListFormat, WhitelistSelector,
};
use crate::value::Value;

#[test]
fn filtered_encode_fast_path_traverses_linear_objects() {
    let fast = encode_node_filtered(
        EncodeInput::Borrowed(&Value::Object(
            [(
                "b".to_owned(),
                Value::Object([("c".to_owned(), Value::String("x".to_owned()))].into()),
            )]
            .into(),
        )),
        KeyPathNode::from_raw("a"),
        &EncodeOptions::new().with_encode(false),
        0,
    )
    .unwrap();
    assert_eq!(fast, vec!["a[b][c]=x".to_owned()]);
}

#[test]
fn filtered_encode_returns_empty_output_for_omitted_input() {
    let omitted = encode_node_filtered(
        EncodeInput::Omitted,
        KeyPathNode::from_raw("a"),
        &EncodeOptions::new().with_encode(false),
        0,
    )
    .unwrap();
    assert!(omitted.is_empty());
}

#[test]
fn filtered_encode_reports_max_depth_errors() {
    let error = encode_node_filtered(
        EncodeInput::Borrowed(&Value::Object(
            [
                ("a".to_owned(), Value::String("x".to_owned())),
                ("b".to_owned(), Value::String("y".to_owned())),
            ]
            .into(),
        )),
        KeyPathNode::from_raw("root"),
        &EncodeOptions::new()
            .with_encode(false)
            .with_max_depth(Some(0)),
        1,
    )
    .unwrap_err();
    assert!(error.is_depth_exceeded());
}

#[test]
fn filtered_encode_emits_empty_list_suffixes_when_allowed() {
    let empty_list = encode_node_filtered(
        EncodeInput::Borrowed(&Value::Array(Vec::new())),
        KeyPathNode::from_raw("items"),
        &EncodeOptions::new()
            .with_encode(false)
            .with_allow_empty_lists(true)
            .with_list_format(ListFormat::Indices),
        0,
    )
    .unwrap();
    assert_eq!(empty_list, vec!["items[]".to_owned()]);
}

#[test]
fn filtered_encode_uses_dot_encoded_keys_and_skips_null_object_children() {
    let dotted = encode_node_filtered(
        EncodeInput::Borrowed(&Value::Object(
            [
                ("dot.key".to_owned(), Value::String("x".to_owned())),
                ("skip".to_owned(), Value::Null),
            ]
            .into(),
        )),
        KeyPathNode::from_raw("root"),
        &EncodeOptions::new()
            .with_encode(false)
            .with_allow_dots(true)
            .with_encode_dot_in_keys(true)
            .with_skip_nulls(true),
        0,
    )
    .unwrap();
    assert_eq!(dotted, vec!["root.dot%2Ekey=x".to_owned()]);
}

#[test]
fn filtered_encode_skips_null_array_items() {
    let skipped_array_null = encode_node_filtered(
        EncodeInput::Borrowed(&Value::Array(vec![
            Value::Null,
            Value::String("y".to_owned()),
        ])),
        KeyPathNode::from_raw("list"),
        &EncodeOptions::new()
            .with_encode(false)
            .with_skip_nulls(true),
        0,
    )
    .unwrap();
    assert_eq!(skipped_array_null, vec!["list[1]=y".to_owned()]);
}

#[test]
fn filter_helpers_cover_root_replacements_and_omitted_inputs() {
    let original = Value::String("root".to_owned());
    let replacement_options = EncodeOptions::new().with_filter(Some(EncodeFilter::Function(
        FunctionFilter::new(|prefix, _| {
            if prefix.is_empty() {
                FilterResult::Replace(Value::Object(
                    [("answer".to_owned(), Value::String("42".to_owned()))].into(),
                ))
            } else {
                FilterResult::Keep
            }
        }),
    )));
    let replaced_root = filter_root_value(&original, &replacement_options);
    assert!(matches!(
        replaced_root,
        EncodeInput::Owned(Value::Object(entries))
            if matches!(entries.get("answer"), Some(Value::String(text)) if text == "42")
    ));

    let non_container_options = EncodeOptions::new().with_filter(Some(EncodeFilter::Function(
        FunctionFilter::new(|_, _| FilterResult::Replace(Value::String("ignored".to_owned()))),
    )));
    assert!(matches!(
        filter_root_value(&original, &non_container_options),
        EncodeInput::Borrowed(Value::String(text)) if text == "root"
    ));

    let omitted = apply_filter_result(EncodeInput::Omitted, "ignored", &replacement_options);
    assert!(matches!(omitted, EncodeInput::Omitted));

    let function_options = EncodeOptions::new().with_filter(Some(EncodeFilter::Function(
        FunctionFilter::new(|_, _| FilterResult::Keep),
    )));
    assert!(has_function_filter(&function_options));
    assert!(has_filter_control(&function_options));

    let whitelist_options = EncodeOptions::new()
        .with_whitelist(Some(vec![WhitelistSelector::Key("answer".to_owned())]));
    assert!(!has_function_filter(&whitelist_options));
    assert!(has_filter_control(&whitelist_options));
}
