use qs_rust::{DecodeOptions, EncodeOptions, Object, Value, decode, encode};

fn s(value: &str) -> Value {
    Value::String(value.to_owned())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Decode a nested query string into the dynamic `Value`/`Object` model.
    let decoded = decode(
        "user[name]=Ada&user[roles][]=admin&user[roles][]=editor",
        &DecodeOptions::new(),
    )?;
    println!("decoded:\n{decoded:#?}");

    // Build the same shape by hand and encode it back to query-string form.
    let mut user = Object::new();
    user.insert("name".to_owned(), s("Ada"));
    user.insert(
        "roles".to_owned(),
        Value::Array(vec![s("admin"), s("editor")]),
    );

    let mut root = Object::new();
    root.insert("user".to_owned(), Value::Object(user));

    // `encode(false)` keeps bracket syntax readable in the output.
    let encoded = encode(
        &Value::Object(root),
        &EncodeOptions::new().with_encode(false),
    )?;
    println!("encoded:\n{encoded}");

    assert_eq!(
        encoded,
        "user[name]=Ada&user[roles][0]=admin&user[roles][1]=editor"
    );
    Ok(())
}
