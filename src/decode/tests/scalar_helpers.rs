use super::{
    Charset, DecodeDecoder, DecodeKind, DecodeOptions, Node, Value, decode_component,
    interpret_numeric_entities, interpret_numeric_entities_in_node,
};

#[test]
fn scalar_helpers_cover_custom_decoders_and_recursive_numeric_entity_nodes() {
    let options = DecodeOptions::new().with_decoder(Some(DecodeDecoder::new(
        |input, _charset, kind| match kind {
            DecodeKind::Value => input.to_ascii_uppercase(),
            DecodeKind::Key => input.to_owned(),
        },
    )));
    assert_eq!(
        decode_component("plain", Charset::Utf8, DecodeKind::Value, &options),
        "PLAIN"
    );
    assert_eq!(interpret_numeric_entities("plain"), "plain");

    let interpreted = interpret_numeric_entities_in_node(Node::Array(vec![
        Node::scalar(Value::String("&#65;".to_owned())),
        Node::scalar(Value::String("plain".to_owned())),
    ]));
    assert_eq!(
        interpreted,
        Node::Array(vec![
            Node::scalar(Value::String("A".to_owned())),
            Node::scalar(Value::String("plain".to_owned())),
        ])
    );

    let untouched_object = Node::Object([("field".to_owned(), Node::scalar(Value::Null))].into());
    assert_eq!(
        interpret_numeric_entities_in_node(untouched_object.clone()),
        untouched_object
    );
}
