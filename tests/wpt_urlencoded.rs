use indexmap::map::Entry;
use qs_rust::{DecodeOptions, EncodeOptions, Format, ListFormat, Object, Value, decode, encode};

// Curated WPT subset from `wpt/url` covering only flat
// `application/x-www-form-urlencoded` behaviors that map cleanly to `qs_rust`.
// Full `URL`, `URLSearchParams`, `FormData`, and `urlpattern` object APIs remain
// out of scope; intentional WHATWG divergences are asserted in
// `tests/divergences.rs`.

#[derive(Debug)]
struct DecodeFixture {
    source_file: &'static str,
    title: &'static str,
    query: &'static str,
    expected_pairs: Vec<(&'static str, &'static str)>,
    options: DecodeOptions,
}

#[derive(Debug)]
struct EncodeFixture {
    source_file: &'static str,
    title: &'static str,
    value: Value,
    expected: &'static str,
    options: EncodeOptions,
}

fn s(value: &str) -> Value {
    Value::String(value.to_owned())
}

fn arr(values: Vec<Value>) -> Value {
    Value::Array(values)
}

fn obj(entries: Vec<(&str, Value)>) -> Value {
    Value::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect(),
    )
}

fn canonicalize_flat_pairs(pairs: &[(&str, &str)]) -> Object {
    let mut object = Object::default();

    for (key, value) in pairs {
        let value = s(value);
        match object.entry((*key).to_owned()) {
            Entry::Vacant(entry) => {
                entry.insert(value);
            }
            Entry::Occupied(mut entry) => match entry.get_mut() {
                Value::Array(items) => items.push(value),
                current => {
                    let first = current.clone();
                    *current = Value::Array(vec![first, value]);
                }
            },
        }
    }

    object
}

fn wpt_encode_options() -> EncodeOptions {
    EncodeOptions::new()
        .with_format(Format::Rfc1738)
        .with_list_format(ListFormat::Repeat)
}

fn decode_fixtures() -> Vec<DecodeFixture> {
    vec![
        DecodeFixture {
            source_file: "wpt/url/urlencoded-parser.any.js",
            title: "empty input",
            query: "",
            expected_pairs: vec![],
            options: DecodeOptions::new(),
        },
        DecodeFixture {
            source_file: "wpt/url/urlencoded-parser.any.js",
            title: "bare key becomes empty value",
            query: "test",
            expected_pairs: vec![("test", "")],
            options: DecodeOptions::new(),
        },
        DecodeFixture {
            source_file: "wpt/url/urlencoded-parser.any.js",
            title: "simple assignment",
            query: "a=b",
            expected_pairs: vec![("a", "b")],
            options: DecodeOptions::new(),
        },
        DecodeFixture {
            source_file: "wpt/url/urlencoded-parser.any.js",
            title: "empty assigned value",
            query: "a=",
            expected_pairs: vec![("a", "")],
            options: DecodeOptions::new(),
        },
        DecodeFixture {
            source_file: "wpt/url/urlencoded-parser.any.js",
            title: "extra equals stays in the value",
            query: "a==a",
            expected_pairs: vec![("a", "=a")],
            options: DecodeOptions::new(),
        },
        DecodeFixture {
            source_file: "wpt/url/urlencoded-parser.any.js",
            title: "repeated separators collapse to real pairs",
            query: "&&&a=b&&&&c=d&",
            expected_pairs: vec![("a", "b"), ("c", "d")],
            options: DecodeOptions::new(),
        },
        DecodeFixture {
            source_file: "wpt/url/urlencoded-parser.any.js",
            title: "duplicates preserve encounter order",
            query: "a=a&a=b&a=c",
            expected_pairs: vec![("a", "a"), ("a", "b"), ("a", "c")],
            options: DecodeOptions::new(),
        },
        DecodeFixture {
            source_file: "wpt/url/urlsearchparams-constructor.any.js",
            title: "plus decodes in values",
            query: "a=b+c",
            expected_pairs: vec![("a", "b c")],
            options: DecodeOptions::new(),
        },
        DecodeFixture {
            source_file: "wpt/url/urlsearchparams-constructor.any.js",
            title: "plus decodes in keys",
            query: "a+b=c",
            expected_pairs: vec![("a b", "c")],
            options: DecodeOptions::new(),
        },
        DecodeFixture {
            source_file: "wpt/url/urlsearchparams-constructor.any.js",
            title: "leading question mark is ignored only when opted in",
            query: "?a=b",
            expected_pairs: vec![("a", "b")],
            options: DecodeOptions::new().with_ignore_query_prefix(true),
        },
        DecodeFixture {
            source_file: "wpt/url/urlencoded-parser.any.js",
            title: "ascii percent-decoding applies in keys",
            query: "%61+%4d%4D=",
            expected_pairs: vec![("a MM", "")],
            options: DecodeOptions::new(),
        },
        DecodeFixture {
            source_file: "wpt/url/urlsearchparams-constructor.any.js",
            title: "percent twenty decodes in keys",
            query: "a%20b=c",
            expected_pairs: vec![("a b", "c")],
            options: DecodeOptions::new(),
        },
        DecodeFixture {
            source_file: "wpt/url/urlsearchparams-constructor.any.js",
            title: "percent zero zero decodes NUL in keys",
            query: "a%00b=c",
            expected_pairs: vec![("a\0b", "c")],
            options: DecodeOptions::new(),
        },
        DecodeFixture {
            source_file: "wpt/url/urlsearchparams-constructor.any.js",
            title: "unicode percent-decoding applies in keys",
            query: "a%f0%9f%92%a9b=c",
            expected_pairs: vec![("a💩b", "c")],
            options: DecodeOptions::new(),
        },
        DecodeFixture {
            source_file: "wpt/url/urlencoded-parser.any.js",
            title: "lone percent stays literal",
            query: "id=0&value=%",
            expected_pairs: vec![("id", "0"), ("value", "%")],
            options: DecodeOptions::new(),
        },
        DecodeFixture {
            source_file: "wpt/url/urlencoded-parser.any.js",
            title: "malformed percent nibble stays literal before valid escape",
            query: "b=%2sf%2a",
            expected_pairs: vec![("b", "%2sf*")],
            options: DecodeOptions::new(),
        },
        DecodeFixture {
            source_file: "wpt/url/urlencoded-parser.any.js",
            title: "partial percent prefix stays literal around valid escapes",
            query: "b=%2%2af%2a",
            expected_pairs: vec![("b", "%2*f*")],
            options: DecodeOptions::new(),
        },
        DecodeFixture {
            source_file: "wpt/url/urlencoded-parser.any.js",
            title: "double percent prefix round-trips literally",
            query: "b=%%2a",
            expected_pairs: vec![("b", "%*")],
            options: DecodeOptions::new(),
        },
    ]
}

fn encode_fixtures() -> Vec<EncodeFixture> {
    vec![
        EncodeFixture {
            source_file: "wpt/url/urlsearchparams-stringifier.any.js",
            title: "serialize spaces in values using RFC1738",
            value: obj(vec![("a", s("b c"))]),
            expected: "a=b+c",
            options: wpt_encode_options(),
        },
        EncodeFixture {
            source_file: "wpt/url/urlsearchparams-stringifier.any.js",
            title: "serialize spaces in keys using RFC1738",
            value: obj(vec![("a b", s("c"))]),
            expected: "a+b=c",
            options: wpt_encode_options(),
        },
        EncodeFixture {
            source_file: "wpt/url/urlencoded-parser.any.js",
            title: "repeat arrays preserve flat duplicate order",
            value: obj(vec![("a", arr(vec![s("a"), s("b"), s("c")]))]),
            expected: "a=a&a=b&a=c",
            options: wpt_encode_options(),
        },
        EncodeFixture {
            source_file: "wpt/url/urlsearchparams-stringifier.any.js",
            title: "empty values and empty keys stay representable in repeat mode",
            value: obj(vec![
                ("a", arr(vec![s(""), s("")])),
                ("", arr(vec![s("b"), s(""), s("")])),
            ]),
            expected: "a=&a=&=b&=&=",
            options: wpt_encode_options(),
        },
        EncodeFixture {
            source_file: "wpt/url/urlsearchparams-stringifier.any.js",
            title: "plus signs are percent-escaped",
            value: obj(vec![("a", s("b+c")), ("a+b", s("c"))]),
            expected: "a=b%2Bc&a%2Bb=c",
            options: wpt_encode_options(),
        },
        EncodeFixture {
            source_file: "wpt/url/urlsearchparams-stringifier.any.js",
            title: "equals signs are percent-escaped",
            value: obj(vec![("=", s("a")), ("b", s("="))]),
            expected: "%3D=a&b=%3D",
            options: wpt_encode_options(),
        },
        EncodeFixture {
            source_file: "wpt/url/urlsearchparams-stringifier.any.js",
            title: "ampersands are percent-escaped",
            value: obj(vec![("&", s("a")), ("b", s("&"))]),
            expected: "%26=a&b=%26",
            options: wpt_encode_options(),
        },
        EncodeFixture {
            source_file: "wpt/url/urlsearchparams-stringifier.any.js",
            title: "percent signs are percent-escaped",
            value: obj(vec![("id", s("0")), ("value", s("%"))]),
            expected: "id=0&value=%25",
            options: wpt_encode_options(),
        },
        EncodeFixture {
            source_file: "wpt/url/urlsearchparams-stringifier.any.js",
            title: "NUL bytes are preserved without normalization",
            value: obj(vec![("a\0b", s("c")), ("d", s("e\0f"))]),
            expected: "a%00b=c&d=e%00f",
            options: wpt_encode_options(),
        },
        EncodeFixture {
            source_file: "wpt/url/urlsearchparams-stringifier.any.js",
            title: "unicode scalar values are UTF-8 encoded",
            value: obj(vec![("a", s("b💩c"))]),
            expected: "a=b%F0%9F%92%A9c",
            options: wpt_encode_options(),
        },
        EncodeFixture {
            source_file: "wpt/url/urlsearchparams-stringifier.any.js",
            title: "newline bytes are preserved byte-for-byte",
            value: obj(vec![("a\nb", s("c\rd")), ("e\n\rf", s("g\r\nh"))]),
            expected: "a%0Ab=c%0Dd&e%0A%0Df=g%0D%0Ah",
            options: wpt_encode_options(),
        },
    ]
}

#[test]
fn curated_wpt_flat_decode_cases_match_qs_rust_public_surface() {
    for fixture in decode_fixtures() {
        let decoded = decode(fixture.query, &fixture.options).unwrap_or_else(|error| {
            panic!(
                "{} [{}] decode unexpectedly failed for {:?}: {error:?}",
                fixture.source_file, fixture.title, fixture.query
            )
        });
        let expected = canonicalize_flat_pairs(&fixture.expected_pairs);
        assert_eq!(
            decoded, expected,
            "{} [{}] decode mismatch for {:?}",
            fixture.source_file, fixture.title, fixture.query
        );
    }
}

#[test]
fn curated_wpt_flat_encode_cases_match_wpt_compatible_preset() {
    for fixture in encode_fixtures() {
        let encoded = encode(&fixture.value, &fixture.options).unwrap_or_else(|error| {
            panic!(
                "{} [{}] encode unexpectedly failed for {:?}: {error:?}",
                fixture.source_file, fixture.title, fixture.value
            )
        });
        assert_eq!(
            encoded, fixture.expected,
            "{} [{}] encode mismatch for {:?}",
            fixture.source_file, fixture.title, fixture.value
        );
    }
}
