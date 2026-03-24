use std::rc::Rc;

use super::KeyPathNode;

#[test]
fn append_empty_reuses_the_same_underlying_node() {
    let root = KeyPathNode::from_raw("user.name");
    let same = root.append("");

    assert!(Rc::ptr_eq(&root.0, &same.0));
}

#[test]
fn append_preserves_materialized_path_segments() {
    let nested = KeyPathNode::from_raw("user.name")
        .append("[first.last]")
        .append("[0]");

    assert_eq!(nested.materialize(), "user.name[first.last][0]");
}

#[test]
fn dot_encoded_cache_reuses_nodes_when_segments_have_no_dots() {
    let nested = KeyPathNode::from_raw("user.name")
        .append("[first.last]")
        .append("[0]");
    let encoded_once = nested.as_dot_encoded("%2E");
    let encoded_twice = nested.as_dot_encoded("%2E");

    assert_eq!(encoded_once.materialize(), "user%2Ename[first%2Elast][0]");
    assert!(Rc::ptr_eq(&encoded_once.0, &encoded_twice.0));
}
