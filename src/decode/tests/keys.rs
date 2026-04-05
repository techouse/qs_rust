use super::{
    DecodeOptions, Node, Value, find_recoverable_balanced_open, parse_keys, split_key_into_segments,
};

fn scalar(value: &str) -> Node {
    Node::scalar(Value::String(value.to_owned()))
}

#[test]
fn structured_key_helpers_cover_recovered_roots_and_trailing_segments() {
    assert_eq!(
        split_key_into_segments("[a[b]", false, 5, false).unwrap(),
        vec!["[a".to_owned(), "[b]".to_owned()]
    );
    assert_eq!(find_recoverable_balanced_open("[a[b]", 1), Some(2));

    assert_eq!(
        split_key_into_segments("a[b]tail", false, 5, false).unwrap(),
        vec!["a".to_owned(), "[b]".to_owned(), "[tail]".to_owned()]
    );
    assert!(split_key_into_segments("a[b]tail", false, 5, true).is_err());
}

#[test]
fn parse_keys_covers_empty_inputs_decoded_dots_and_list_limit_errors() {
    assert!(
        parse_keys("", scalar("x"), &DecodeOptions::new())
            .unwrap()
            .is_none()
    );

    let decoded = parse_keys(
        "user[%2Ehidden]",
        scalar("x"),
        &DecodeOptions::new()
            .with_allow_dots(true)
            .with_decode_dot_in_keys(true),
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        decoded,
        Node::Object(
            [(
                "user".to_owned(),
                Node::Object([(".hidden".to_owned(), scalar("x"))].into()),
            )]
            .into()
        )
    );

    let error = parse_keys(
        "list[2]",
        scalar("x"),
        &DecodeOptions::new()
            .with_list_limit(2)
            .with_throw_on_limit_exceeded(true),
    )
    .unwrap_err();
    assert!(error.is_list_limit_exceeded());
    assert_eq!(error.list_limit(), Some(2));
}
