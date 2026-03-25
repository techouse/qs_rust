use super::{array_all_map_like_or_undefined, map_entries, merge};
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
fn merge_scalar_targets_shift_canonical_overflow_keys_but_preserve_noncanonical_ones() {
    let overflow = Node::OverflowObject {
        entries: [
            ("0".to_owned(), scalar("b")),
            ("01".to_owned(), scalar("legacy")),
        ]
        .into(),
        max_index: 1,
    };

    let merged = merge(scalar("a"), overflow, &DecodeOptions::new()).unwrap();
    assert_eq!(
        merged,
        Node::OverflowObject {
            entries: [
                ("0".to_owned(), scalar("a")),
                ("1".to_owned(), scalar("b")),
                ("01".to_owned(), scalar("legacy")),
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

#[test]
fn merge_short_circuits_null_sources_and_undefined_targets() {
    let target = Node::Object([("field".to_owned(), scalar("value"))].into());
    assert_eq!(
        merge(
            target.clone(),
            Node::scalar(Value::Null),
            &DecodeOptions::new()
        )
        .unwrap(),
        target
    );

    let source = Node::Object([("field".to_owned(), scalar("value"))].into());
    assert_eq!(
        merge(Node::Undefined, source.clone(), &DecodeOptions::new()).unwrap(),
        source
    );
}

#[test]
fn merge_sparse_arrays_can_fall_back_to_numeric_objects_when_lists_are_disabled() {
    let merged = merge(
        Node::Array(vec![scalar("keep"), Node::Undefined]),
        Node::Array(vec![Node::Undefined]),
        &DecodeOptions::new().with_parse_lists(false),
    )
    .unwrap();

    assert_eq!(
        merged,
        Node::Object([("0".to_owned(), scalar("keep"))].into())
    );
}

#[test]
fn merge_arrays_append_non_undefined_values_and_scalars() {
    let merged_arrays = merge(
        Node::Array(vec![scalar("a")]),
        Node::Array(vec![Node::Undefined, scalar("b")]),
        &DecodeOptions::new(),
    )
    .unwrap();
    assert_eq!(merged_arrays, Node::Array(vec![scalar("a"), scalar("b")]));

    let merged_scalar = merge(
        Node::Array(vec![scalar("a")]),
        scalar("b"),
        &DecodeOptions::new(),
    )
    .unwrap();
    assert_eq!(merged_scalar, Node::Array(vec![scalar("a"), scalar("b")]));
}

#[test]
fn merge_object_with_array_merges_existing_numeric_children() {
    let merged = merge(
        Node::Object(
            [(
                "0".to_owned(),
                Node::Object([("left".to_owned(), scalar("a"))].into()),
            )]
            .into(),
        ),
        Node::Array(vec![
            Node::Object([("right".to_owned(), scalar("b"))].into()),
            Node::Undefined,
            scalar("c"),
        ]),
        &DecodeOptions::new(),
    )
    .unwrap();

    assert_eq!(
        merged,
        Node::Object(
            [
                (
                    "0".to_owned(),
                    Node::Object(
                        [
                            ("left".to_owned(), scalar("a")),
                            ("right".to_owned(), scalar("b")),
                        ]
                        .into(),
                    ),
                ),
                ("2".to_owned(), scalar("c")),
            ]
            .into()
        )
    );
}

#[test]
fn merge_overflow_objects_append_arrays_and_scalars() {
    let overflow = Node::OverflowObject {
        entries: [("0".to_owned(), scalar("a"))].into(),
        max_index: 0,
    };
    let merged_array = merge(
        overflow.clone(),
        Node::Array(vec![Node::Undefined, scalar("b")]),
        &DecodeOptions::new(),
    )
    .unwrap();
    assert_eq!(
        merged_array,
        Node::OverflowObject {
            entries: [("0".to_owned(), scalar("a")), ("1".to_owned(), scalar("b")),].into(),
            max_index: 1,
        }
    );

    let merged_scalar = merge(overflow, scalar("c"), &DecodeOptions::new()).unwrap();
    assert_eq!(
        merged_scalar,
        Node::OverflowObject {
            entries: [("0".to_owned(), scalar("a")), ("1".to_owned(), scalar("c")),].into(),
            max_index: 1,
        }
    );
}

#[test]
fn merge_arrays_into_overflow_sources_track_max_indices() {
    let merged = merge(
        Node::Array(vec![scalar("a")]),
        Node::OverflowObject {
            entries: [("2".to_owned(), scalar("c"))].into(),
            max_index: 2,
        },
        &DecodeOptions::new(),
    )
    .unwrap();

    assert_eq!(
        merged,
        Node::OverflowObject {
            entries: [("0".to_owned(), scalar("a")), ("2".to_owned(), scalar("c")),].into(),
            max_index: 2,
        }
    );
}

#[test]
fn merge_scalar_and_object_source_wraps_into_array() {
    let source = Node::Object([("field".to_owned(), scalar("value"))].into());
    let merged = merge(scalar("a"), source.clone(), &DecodeOptions::new()).unwrap();
    assert_eq!(merged, Node::Array(vec![scalar("a"), source]));
}

#[test]
fn merge_helpers_recognize_map_like_arrays_and_extract_entries() {
    assert!(array_all_map_like_or_undefined(&[
        Node::Object([("field".to_owned(), scalar("value"))].into()),
        Node::Undefined,
    ]));
    assert!(!array_all_map_like_or_undefined(&[scalar("value")]));

    let object_entries = map_entries(Node::Object([("field".to_owned(), scalar("value"))].into()));
    assert_eq!(
        object_entries.into_iter().collect::<Vec<_>>(),
        vec![("field".to_owned(), scalar("value"))]
    );

    let overflow_entries = map_entries(Node::OverflowObject {
        entries: [("1".to_owned(), scalar("value"))].into(),
        max_index: 1,
    });
    assert_eq!(
        overflow_entries.into_iter().collect::<Vec<_>>(),
        vec![("1".to_owned(), scalar("value"))]
    );

    assert!(map_entries(Node::Undefined).is_empty());
}

#[test]
fn merge_arrays_with_holes_append_scalar_sources_before_sparse_fallback() {
    let merged = merge(
        Node::Array(vec![Node::Undefined]),
        scalar("value"),
        &DecodeOptions::new(),
    )
    .unwrap();

    assert_eq!(merged, Node::Array(vec![Node::Undefined, scalar("value")]));
}

#[test]
fn merge_object_targets_ignore_undefined_sources() {
    let target = Node::Object([("field".to_owned(), scalar("value"))].into());

    let merged = merge(target.clone(), Node::Undefined, &DecodeOptions::new()).unwrap();
    assert_eq!(merged, target);
}

#[test]
fn merge_overflow_targets_ignore_undefined_sources() {
    let target = Node::OverflowObject {
        entries: [("2".to_owned(), scalar("value"))].into(),
        max_index: 2,
    };

    let merged = merge(target.clone(), Node::Undefined, &DecodeOptions::new()).unwrap();
    assert_eq!(merged, target);
}

#[test]
fn merge_scalar_targets_prepend_non_undefined_array_items() {
    let merged = merge(
        scalar("root"),
        Node::Array(vec![Node::Undefined, scalar("leaf")]),
        &DecodeOptions::new(),
    )
    .unwrap();

    assert_eq!(merged, Node::Array(vec![scalar("root"), scalar("leaf")]));
}

#[test]
fn merge_overflow_objects_with_plain_objects_track_max_indices_and_collisions() {
    let merged = merge(
        Node::OverflowObject {
            entries: [
                (
                    "0".to_owned(),
                    Node::Object([("left".to_owned(), scalar("a"))].into()),
                ),
                ("legacy".to_owned(), scalar("stale")),
            ]
            .into(),
            max_index: 0,
        },
        Node::Object(
            [
                ("1".to_owned(), scalar("b")),
                ("010".to_owned(), scalar("zero-ten")),
                (
                    "0".to_owned(),
                    Node::Object([("right".to_owned(), scalar("c"))].into()),
                ),
            ]
            .into(),
        ),
        &DecodeOptions::new(),
    )
    .unwrap();

    assert_eq!(
        merged,
        Node::OverflowObject {
            entries: [
                (
                    "0".to_owned(),
                    Node::Object(
                        [
                            ("left".to_owned(), scalar("a")),
                            ("right".to_owned(), scalar("c")),
                        ]
                        .into(),
                    ),
                ),
                ("legacy".to_owned(), scalar("stale")),
                ("1".to_owned(), scalar("b")),
                ("010".to_owned(), scalar("zero-ten")),
            ]
            .into(),
            max_index: 1,
        }
    );
}

#[test]
fn merge_overflow_objects_preserve_noncanonical_keys_from_other_overflows() {
    let merged = merge(
        Node::OverflowObject {
            entries: [("0".to_owned(), scalar("a"))].into(),
            max_index: 0,
        },
        Node::OverflowObject {
            entries: [
                ("5".to_owned(), scalar("b")),
                ("05".to_owned(), scalar("c")),
            ]
            .into(),
            max_index: 5,
        },
        &DecodeOptions::new(),
    )
    .unwrap();

    assert_eq!(
        merged,
        Node::OverflowObject {
            entries: [
                ("0".to_owned(), scalar("a")),
                ("5".to_owned(), scalar("b")),
                ("05".to_owned(), scalar("c")),
            ]
            .into(),
            max_index: 5,
        }
    );
}

#[test]
fn merge_array_targets_promote_to_objects_for_plain_map_sources() {
    let merged = merge(
        Node::Array(vec![scalar("a")]),
        Node::Object([("field".to_owned(), scalar("b"))].into()),
        &DecodeOptions::new(),
    )
    .unwrap();

    assert_eq!(
        merged,
        Node::Object(
            [
                ("0".to_owned(), scalar("a")),
                ("field".to_owned(), scalar("b")),
            ]
            .into()
        )
    );
}

#[test]
fn merge_empty_array_targets_keep_overflow_metadata_when_sources_are_overflow_maps() {
    let merged = merge(
        Node::Array(Vec::new()),
        Node::OverflowObject {
            entries: [("2".to_owned(), scalar("c"))].into(),
            max_index: 2,
        },
        &DecodeOptions::new(),
    )
    .unwrap();

    assert_eq!(
        merged,
        Node::OverflowObject {
            entries: [("2".to_owned(), scalar("c"))].into(),
            max_index: 2,
        }
    );
}

#[test]
fn merge_map_like_arrays_append_new_items_after_skipping_undefined_entries() {
    let merged = merge(
        Node::Array(vec![Node::Object(
            [("left".to_owned(), scalar("a"))].into(),
        )]),
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

#[test]
fn merge_map_like_arrays_can_fall_back_to_numeric_objects_when_holes_remain() {
    let merged = merge(
        Node::Array(vec![Node::Object(
            [("left".to_owned(), scalar("a"))].into(),
        )]),
        Node::Array(vec![Node::Undefined, Node::Undefined]),
        &DecodeOptions::new().with_parse_lists(false),
    )
    .unwrap();

    assert_eq!(
        merged,
        Node::Object(
            [(
                "0".to_owned(),
                Node::Object([("left".to_owned(), scalar("a"))].into())
            )]
            .into()
        )
    );
}
