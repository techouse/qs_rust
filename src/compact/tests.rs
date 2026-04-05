use super::{compact, node_to_object, node_to_value};
use crate::internal::node::Node;
use crate::value::Value;

fn scalar(value: &str) -> Node {
    Node::scalar(Value::String(value.to_owned()))
}

#[test]
fn compact_collapses_sparse_arrays_when_sparse_lists_are_disabled() {
    let compacted = compact(
        Node::Array(vec![
            Node::Undefined,
            scalar("a"),
            Node::Undefined,
            scalar("b"),
        ]),
        false,
    );
    assert_eq!(
        node_to_value(compacted),
        Value::Array(vec![
            Value::String("a".to_owned()),
            Value::String("b".to_owned())
        ])
    );
}

#[test]
fn compact_preserves_sparse_offsets_as_nulls_when_sparse_lists_are_enabled() {
    let compacted = compact(
        Node::Array(vec![
            Node::Undefined,
            scalar("a"),
            Node::Undefined,
            scalar("b"),
        ]),
        true,
    );
    assert_eq!(
        node_to_value(compacted),
        Value::Array(vec![
            Value::Null,
            Value::String("a".to_owned()),
            Value::Null,
            Value::String("b".to_owned()),
        ])
    );
}

#[test]
fn compact_removes_undefined_entries_from_objects_and_nested_lists() {
    let compacted = compact(
        Node::Object(
            [
                ("a".to_owned(), Node::Undefined),
                (
                    "b".to_owned(),
                    Node::Array(vec![Node::Undefined, scalar("value")]),
                ),
            ]
            .into(),
        ),
        false,
    );

    assert_eq!(
        node_to_value(compacted),
        Value::Object(
            [(
                "b".to_owned(),
                Value::Array(vec![Value::String("value".to_owned())]),
            )]
            .into()
        )
    );
}

#[test]
fn compact_preserves_named_overflow_keys_while_dropping_undefined_children() {
    let compacted = compact(
        Node::OverflowObject {
            entries: [
                ("0".to_owned(), Node::Undefined),
                ("1".to_owned(), scalar("a")),
                ("name".to_owned(), scalar("b")),
            ]
            .into(),
            max_index: 1,
        },
        false,
    );

    assert_eq!(
        node_to_value(compacted),
        Value::Object(
            [
                ("1".to_owned(), Value::String("a".to_owned())),
                ("name".to_owned(), Value::String("b".to_owned())),
            ]
            .into()
        )
    );
}

#[test]
fn node_conversion_helpers_cover_arrays_scalars_and_undefined_values() {
    assert_eq!(node_to_value(Node::Undefined), Value::Null);

    assert_eq!(
        node_to_object(Node::Array(vec![scalar("a"), Node::Undefined])),
        [
            ("0".to_owned(), Value::String("a".to_owned())),
            ("1".to_owned(), Value::Null),
        ]
        .into()
    );

    assert_eq!(
        node_to_object(scalar("root")),
        [("0".to_owned(), Value::String("root".to_owned()))].into()
    );
}
