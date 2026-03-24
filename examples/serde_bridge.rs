use qs_rust::{DecodeOptions, EncodeOptions, from_str, to_string};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct User {
    name: String,
    admin: String,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Query {
    page: String,
    tags: Vec<String>,
    user: User,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The serde bridge routes through the crate's dynamic `Value` model, so
    // plain query scalars deserialize as strings unless your serde types add
    // their own conversion layer.
    let query: Query = from_str(
        "page=2&tags[0]=rust&tags[1]=serde&user[name]=Ada&user[admin]=true",
        &DecodeOptions::new(),
    )?;
    println!("typed decode:\n{query:#?}");

    assert_eq!(
        query,
        Query {
            page: "2".to_owned(),
            tags: vec!["rust".to_owned(), "serde".to_owned()],
            user: User {
                name: "Ada".to_owned(),
                admin: "true".to_owned(),
            },
        }
    );

    // Encoding the typed value goes back through the same semantic core.
    let encoded = to_string(&query, &EncodeOptions::new().with_encode(false))?;
    println!("typed encode:\n{encoded}");

    assert_eq!(
        encoded,
        "page=2&tags[0]=rust&tags[1]=serde&user[name]=Ada&user[admin]=true"
    );
    Ok(())
}
