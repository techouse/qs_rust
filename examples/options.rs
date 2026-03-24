use qs_rust::{
    DecodeOptions, Duplicates, EncodeOptions, ListFormat, Object, Value, decode, encode,
};

fn s(value: &str) -> Value {
    Value::String(value.to_owned())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Show a few decode options working together:
    // - ignore a leading '?'
    // - treat dots as path separators
    // - keep the last duplicate
    // - parse comma-separated values as arrays
    let decoded = decode(
        "?user.name=Ada&user.name=Grace&tags=rust,qs",
        &DecodeOptions::new()
            .with_ignore_query_prefix(true)
            .with_allow_dots(true)
            .with_duplicates(Duplicates::Last)
            .with_comma(true),
    )?;
    println!("decoded with options:\n{decoded:#?}");

    let mut user = Object::new();
    user.insert("name".to_owned(), s("Grace"));

    let mut root = Object::new();
    root.insert("user".to_owned(), Value::Object(user));
    root.insert("tags".to_owned(), Value::Array(vec![s("rust"), s("qs")]));

    // Use the matching encode-side options to produce a compact dotted query.
    let encoded = encode(
        &Value::Object(root),
        &EncodeOptions::new()
            .with_encode(false)
            .with_allow_dots(true)
            .with_list_format(ListFormat::Comma)
            .with_add_query_prefix(true),
    )?;
    println!("encoded with options:\n{encoded}");

    assert_eq!(encoded, "?user.name=Grace&tags=rust,qs");
    Ok(())
}
