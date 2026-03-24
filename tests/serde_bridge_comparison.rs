#![cfg(feature = "serde")]

use std::collections::BTreeMap;

use qs_rust::{
    DecodeError, DecodeOptions, Duplicates, EncodeOptions, ListFormat, from_str, to_string,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Address {
    city: String,
    postcode: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct NestedQuery {
    name: String,
    address: Address,
    user_ids: Vec<String>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct TagQuery {
    tags: Vec<String>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct OptionalQuery {
    name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    alias: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    note: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct UserName(String);

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct RenamedMapQuery {
    user_name: UserName,
    scores: BTreeMap<String, String>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct ScalarQuery {
    page: u32,
    admin: bool,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct ScalarDuplicateQuery {
    tag: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct SequenceDuplicateQuery {
    tag: Vec<String>,
}

#[test]
fn serde_qs_overlap_nested_struct_decode_and_encode() {
    let decoded: NestedQuery = from_str(
        "address[postcode]=12345&user_ids[1]=2&name=Acme&address[city]=CarrotCity&user_ids[0]=1",
        &DecodeOptions::new(),
    )
    .unwrap();

    assert_eq!(
        decoded,
        NestedQuery {
            name: "Acme".to_owned(),
            address: Address {
                city: "CarrotCity".to_owned(),
                postcode: "12345".to_owned(),
            },
            user_ids: vec!["1".to_owned(), "2".to_owned()],
        }
    );

    let encoded = to_string(&decoded, &EncodeOptions::new()).unwrap();
    assert_eq!(
        encoded,
        "name=Acme&address%5Bcity%5D=CarrotCity&address%5Bpostcode%5D=12345&user_ids%5B0%5D=1&user_ids%5B1%5D=2"
    );
}

#[test]
fn serde_qs_overlap_vector_fields_accept_indexed_and_bracket_inputs() {
    let indexed: TagQuery = from_str("tags[0]=rust&tags[1]=serde", &DecodeOptions::new()).unwrap();
    assert_eq!(
        indexed,
        TagQuery {
            tags: vec!["rust".to_owned(), "serde".to_owned()],
        }
    );

    let bracketed: TagQuery = from_str("tags[]=rust&tags[]=serde", &DecodeOptions::new()).unwrap();
    assert_eq!(bracketed, indexed);

    let encoded = to_string(
        &indexed,
        &EncodeOptions::new().with_list_format(ListFormat::Brackets),
    )
    .unwrap();
    assert_eq!(encoded, "tags%5B%5D=rust&tags%5B%5D=serde");
}

#[test]
fn serde_qs_overlap_option_default_and_skip_fields_round_trip() {
    let decoded: OptionalQuery = from_str("name=alice", &DecodeOptions::new()).unwrap();
    assert_eq!(
        decoded,
        OptionalQuery {
            name: "alice".to_owned(),
            alias: None,
            tags: vec![],
            note: String::new(),
        }
    );

    let encoded = to_string(&decoded, &EncodeOptions::new()).unwrap();
    assert_eq!(encoded, "name=alice");

    let explicit = OptionalQuery {
        name: "alice".to_owned(),
        alias: Some("ally".to_owned()),
        tags: vec!["x".to_owned(), "y".to_owned()],
        note: "ready".to_owned(),
    };
    let explicit_encoded = to_string(
        &explicit,
        &EncodeOptions::new().with_list_format(ListFormat::Brackets),
    )
    .unwrap();
    assert_eq!(
        explicit_encoded,
        "name=alice&alias=ally&tags%5B%5D=x&tags%5B%5D=y&note=ready"
    );
}

#[test]
fn serde_qs_overlap_rename_newtype_and_map_fields_round_trip() {
    let decoded: RenamedMapQuery = from_str(
        "scores[quality]=87&userName=alice&scores[relevance]=95",
        &DecodeOptions::new(),
    )
    .unwrap();

    let mut scores = BTreeMap::new();
    scores.insert("quality".to_owned(), "87".to_owned());
    scores.insert("relevance".to_owned(), "95".to_owned());

    assert_eq!(
        decoded,
        RenamedMapQuery {
            user_name: UserName("alice".to_owned()),
            scores,
        }
    );

    let encoded = to_string(&decoded, &EncodeOptions::new()).unwrap();
    assert_eq!(
        encoded,
        "userName=alice&scores%5Bquality%5D=87&scores%5Brelevance%5D=95"
    );
}

#[test]
fn serde_qs_divergence_plain_scalar_decode_remains_stringly() {
    let error = from_str::<ScalarQuery>("page=2&admin=true", &DecodeOptions::new()).unwrap_err();
    assert!(matches!(error, DecodeError::Serde(_)));
}

#[test]
fn serde_qs_divergence_duplicate_scalar_policy_is_option_driven_not_type_driven() {
    let sequence: SequenceDuplicateQuery = from_str("tag=a&tag=b", &DecodeOptions::new()).unwrap();
    assert_eq!(
        sequence,
        SequenceDuplicateQuery {
            tag: vec!["a".to_owned(), "b".to_owned()],
        }
    );

    let scalar_error =
        from_str::<ScalarDuplicateQuery>("tag=a&tag=b", &DecodeOptions::new()).unwrap_err();
    assert!(matches!(scalar_error, DecodeError::Serde(_)));

    let scalar_last: ScalarDuplicateQuery = from_str(
        "tag=a&tag=b",
        &DecodeOptions::new().with_duplicates(Duplicates::Last),
    )
    .unwrap();
    assert_eq!(
        scalar_last,
        ScalarDuplicateQuery {
            tag: "b".to_owned(),
        }
    );
}
