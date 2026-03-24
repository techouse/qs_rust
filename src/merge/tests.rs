use super::merge;
use crate::internal::node::Node;
use crate::options::DecodeOptions;
use crate::value::Value;

fn scalar(value: &str) -> Node {
    Node::scalar(Value::String(value.to_owned()))
}

#[test]
fn merge_overflow_object_into_primitive_shifts_indices() {
    let overflow = Node::OverflowObject {
        entries: [("0".to_owned(), scalar("b")), ("1".to_owned(), scalar("c"))].into(),
        max_index: 1,
    };

    let merged = merge(scalar("a"), overflow, &DecodeOptions::new()).unwrap();
    assert_eq!(
        merged,
        Node::OverflowObject {
            entries: [
                ("0".to_owned(), scalar("a")),
                ("1".to_owned(), scalar("b")),
                ("2".to_owned(), scalar("c")),
            ]
            .into(),
            max_index: 2,
        }
    );
}

#[test]
fn merge_overflow_preserves_noncanonical_keys_without_advancing_max_index() {
    let target = Node::Object([("010".to_owned(), scalar("x"))].into());
    let overflow = Node::OverflowObject {
        entries: [("0".to_owned(), scalar("a")), ("1".to_owned(), scalar("b"))].into(),
        max_index: 1,
    };

    let merged = merge(target, overflow, &DecodeOptions::new()).unwrap();
    assert_eq!(
        merged,
        Node::OverflowObject {
            entries: [
                ("010".to_owned(), scalar("x")),
                ("0".to_owned(), scalar("a")),
                ("1".to_owned(), scalar("b")),
            ]
            .into(),
            max_index: 1,
        }
    );
}

#[test]
fn merge_array_map_like_children_reuses_undefined_holes_before_appending() {
    let merged = merge(
        Node::Array(vec![
            Node::Object([("left".to_owned(), scalar("a"))].into()),
            Node::Undefined,
        ]),
        Node::Array(vec![
            Node::Undefined,
            Node::Object([("right".to_owned(), scalar("b"))].into()),
        ]),
        &DecodeOptions::new(),
    )
    .unwrap();

    assert_eq!(
        merged,
        Node::Array(vec![
            Node::Object([("left".to_owned(), scalar("a"))].into()),
            Node::Object([("right".to_owned(), scalar("b"))].into()),
        ])
    );
}
