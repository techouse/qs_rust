use qs_rust::{
    Charset, DecodeOptions, Delimiter, Duplicates, EncodeOptions, Format, ListFormat, Object,
    Value, decode, encode,
};
use regex::Regex;

fn s(value: &str) -> Value {
    Value::String(value.to_owned())
}

fn arr(values: Vec<Value>) -> Value {
    Value::Array(values)
}

fn obj(entries: Vec<(&str, Value)>) -> Value {
    Value::Object(map(entries))
}

fn map(entries: Vec<(&str, Value)>) -> Object {
    entries
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect()
}

#[test]
fn simple_examples_work() {
    assert_eq!(
        decode("a=c", &DecodeOptions::new()).unwrap(),
        map(vec![("a", s("c"))])
    );
    assert_eq!(
        encode(&obj(vec![("a", s("c"))]), &EncodeOptions::new()).unwrap(),
        "a=c"
    );
}

#[test]
fn decoding_map_examples_work() {
    assert_eq!(
        decode("foo[bar]=baz", &DecodeOptions::new()).unwrap(),
        map(vec![("foo", obj(vec![("bar", s("baz"))]))]),
    );
    assert_eq!(
        decode("a%5Bb%5D=c", &DecodeOptions::new()).unwrap(),
        map(vec![("a", obj(vec![("b", s("c"))]))]),
    );
    assert_eq!(
        decode("foo[bar][baz]=foobarbaz", &DecodeOptions::new()).unwrap(),
        map(vec![(
            "foo",
            obj(vec![("bar", obj(vec![("baz", s("foobarbaz"))]))])
        )]),
    );
    assert_eq!(
        decode("a[b][c][d][e][f][g][h][i]=j", &DecodeOptions::new()).unwrap(),
        map(vec![(
            "a",
            obj(vec![(
                "b",
                obj(vec![(
                    "c",
                    obj(vec![(
                        "d",
                        obj(vec![(
                            "e",
                            obj(vec![("f", obj(vec![("[g][h][i]", s("j"))]))])
                        )]),
                    )]),
                )]),
            )]),
        )]),
    );
    assert_eq!(
        decode(
            "a[b][c][d][e][f][g][h][i]=j",
            &DecodeOptions::new().with_depth(1),
        )
        .unwrap(),
        map(vec![(
            "a",
            obj(vec![("b", obj(vec![("[c][d][e][f][g][h][i]", s("j"))]))])
        )]),
    );
    assert_eq!(
        decode("a=b&c=d", &DecodeOptions::new().with_parameter_limit(1)).unwrap(),
        map(vec![("a", s("b"))]),
    );
    assert_eq!(
        decode(
            "?a=b&c=d",
            &DecodeOptions::new().with_ignore_query_prefix(true),
        )
        .unwrap(),
        map(vec![("a", s("b")), ("c", s("d"))]),
    );
    assert_eq!(
        decode(
            "a=b;c=d",
            &DecodeOptions::new().with_delimiter(Delimiter::String(";".to_owned())),
        )
        .unwrap(),
        map(vec![("a", s("b")), ("c", s("d"))]),
    );
    assert_eq!(
        decode(
            "a=b;c=d",
            &DecodeOptions::new().with_delimiter(Delimiter::Regex(Regex::new("[;,]").unwrap())),
        )
        .unwrap(),
        map(vec![("a", s("b")), ("c", s("d"))]),
    );
    assert_eq!(
        decode("a.b=c", &DecodeOptions::new().with_allow_dots(true)).unwrap(),
        map(vec![("a", obj(vec![("b", s("c"))]))]),
    );
    assert_eq!(
        decode(
            "name%252Eobj.first=John&name%252Eobj.last=Doe",
            &DecodeOptions::new().with_decode_dot_in_keys(true),
        )
        .unwrap(),
        map(vec![(
            "name.obj",
            obj(vec![("first", s("John")), ("last", s("Doe"))])
        )]),
    );
    assert_eq!(
        decode(
            "foo[]&bar=baz",
            &DecodeOptions::new().with_allow_empty_lists(true),
        )
        .unwrap(),
        map(vec![("foo", arr(vec![])), ("bar", s("baz"))]),
    );
    assert_eq!(
        decode(
            "foo=bar&foo=baz",
            &DecodeOptions::new().with_duplicates(Duplicates::First)
        )
        .unwrap(),
        map(vec![("foo", s("bar"))]),
    );
    assert_eq!(
        decode(
            "foo=bar&foo=baz",
            &DecodeOptions::new().with_duplicates(Duplicates::Last)
        )
        .unwrap(),
        map(vec![("foo", s("baz"))]),
    );
}

#[test]
fn decoding_list_and_scalar_examples_work() {
    assert_eq!(
        decode("a[]=b&a[]=c", &DecodeOptions::new()).unwrap(),
        map(vec![("a", arr(vec![s("b"), s("c")]))]),
    );
    assert_eq!(
        decode("a[1]=c&a[0]=b", &DecodeOptions::new()).unwrap(),
        map(vec![("a", arr(vec![s("b"), s("c")]))]),
    );
    assert_eq!(
        decode("a[1]=b&a[15]=c", &DecodeOptions::new()).unwrap(),
        map(vec![("a", arr(vec![s("b"), s("c")]))]),
    );
    assert_eq!(
        decode("a[]=&a[]=b", &DecodeOptions::new()).unwrap(),
        map(vec![("a", arr(vec![s(""), s("b")]))]),
    );
    assert_eq!(
        decode("a[0]=b&a[1]=&a[2]=c", &DecodeOptions::new()).unwrap(),
        map(vec![("a", arr(vec![s("b"), s(""), s("c")]))]),
    );
    assert_eq!(
        decode("a[100]=b", &DecodeOptions::new()).unwrap(),
        map(vec![("a", obj(vec![("100", s("b"))]))]),
    );
    assert_eq!(
        decode("a[1]=b", &DecodeOptions::new().with_list_limit(0)).unwrap(),
        map(vec![("a", obj(vec![("1", s("b"))]))]),
    );
    assert_eq!(
        decode("a[]=b", &DecodeOptions::new().with_parse_lists(false)).unwrap(),
        map(vec![("a", obj(vec![("0", s("b"))]))]),
    );
    assert_eq!(
        decode("a[0]=b&a[b]=c", &DecodeOptions::new()).unwrap(),
        map(vec![("a", obj(vec![("0", s("b")), ("b", s("c"))]))]),
    );
    assert_eq!(
        decode("a[][b]=c", &DecodeOptions::new()).unwrap(),
        map(vec![("a", arr(vec![obj(vec![("b", s("c"))])]))]),
    );
    assert_eq!(
        decode("a=b,c", &DecodeOptions::new().with_comma(true)).unwrap(),
        map(vec![("a", arr(vec![s("b"), s("c")]))]),
    );
    assert_eq!(
        decode("a=15&b=true&c=null", &DecodeOptions::new()).unwrap(),
        map(vec![("a", s("15")), ("b", s("true")), ("c", s("null"))]),
    );
}

#[test]
fn decoding_null_and_charset_examples_work() {
    assert_eq!(
        decode(
            "a=%A7",
            &DecodeOptions::new().with_charset(Charset::Iso88591)
        )
        .unwrap(),
        map(vec![("a", s("§"))]),
    );
    assert_eq!(
        decode(
            "utf8=%E2%9C%93&a=%C3%B8",
            &DecodeOptions::new()
                .with_charset(Charset::Iso88591)
                .with_charset_sentinel(true),
        )
        .unwrap(),
        map(vec![("a", s("ø"))]),
    );
    assert_eq!(
        decode(
            "utf8=%26%2310003%3B&a=%F8",
            &DecodeOptions::new()
                .with_charset(Charset::Utf8)
                .with_charset_sentinel(true),
        )
        .unwrap(),
        map(vec![("a", s("ø"))]),
    );
    assert_eq!(
        decode(
            "a=%26%239786%3B",
            &DecodeOptions::new()
                .with_charset(Charset::Iso88591)
                .with_interpret_numeric_entities(true),
        )
        .unwrap(),
        map(vec![("a", s("☺"))]),
    );
    assert_eq!(
        encode(
            &obj(vec![("a", Value::Null), ("b", s(""))]),
            &EncodeOptions::new(),
        )
        .unwrap(),
        "a=&b="
    );
    assert_eq!(
        encode(
            &obj(vec![("a", Value::Null), ("b", s(""))]),
            &EncodeOptions::new().with_strict_null_handling(true),
        )
        .unwrap(),
        "a&b="
    );
    assert_eq!(
        decode(
            "a&b=",
            &DecodeOptions::new().with_strict_null_handling(true),
        )
        .unwrap(),
        map(vec![("a", Value::Null), ("b", s(""))]),
    );
    assert_eq!(
        encode(
            &obj(vec![("a", s("b")), ("c", Value::Null)]),
            &EncodeOptions::new().with_skip_nulls(true),
        )
        .unwrap(),
        "a=b"
    );
}

#[test]
fn encoding_examples_work() {
    assert_eq!(
        encode(&obj(vec![("a", s("b"))]), &EncodeOptions::new()).unwrap(),
        "a=b"
    );
    assert_eq!(
        encode(
            &obj(vec![("a", obj(vec![("b", s("c"))]))]),
            &EncodeOptions::new()
        )
        .unwrap(),
        "a%5Bb%5D=c"
    );
    assert_eq!(
        encode(
            &obj(vec![("a", obj(vec![("b", s("c"))]))]),
            &EncodeOptions::new().with_encode(false),
        )
        .unwrap(),
        "a[b]=c"
    );
    assert_eq!(
        encode(
            &obj(vec![
                ("a", s("b")),
                ("c", arr(vec![s("d"), s("e=f")])),
                ("f", arr(vec![arr(vec![s("g")]), arr(vec![s("h")])])),
            ]),
            &EncodeOptions::new().with_encode_values_only(true),
        )
        .unwrap(),
        "a=b&c[0]=d&c[1]=e%3Df&f[0][0]=g&f[1][0]=h"
    );
    assert_eq!(
        encode(
            &obj(vec![("a", arr(vec![s("b"), s("c"), s("d")]))]),
            &EncodeOptions::new().with_encode(false),
        )
        .unwrap(),
        "a[0]=b&a[1]=c&a[2]=d"
    );
    assert_eq!(
        encode(
            &obj(vec![("a", arr(vec![s("b"), s("c"), s("d")]))]),
            &EncodeOptions::new()
                .with_encode(false)
                .with_list_format(ListFormat::Repeat),
        )
        .unwrap(),
        "a=b&a=c&a=d"
    );
    assert_eq!(
        encode(
            &obj(vec![("a", arr(vec![s("b"), s("c")]))]),
            &EncodeOptions::new()
                .with_encode(false)
                .with_list_format(ListFormat::Brackets),
        )
        .unwrap(),
        "a[]=b&a[]=c"
    );
    assert_eq!(
        encode(
            &obj(vec![("a", arr(vec![s("b"), s("c")]))]),
            &EncodeOptions::new()
                .with_encode(false)
                .with_list_format(ListFormat::Comma),
        )
        .unwrap(),
        "a=b,c"
    );
    assert_eq!(
        encode(
            &obj(vec![(
                "a",
                obj(vec![("b", obj(vec![("c", s("d")), ("e", s("f"))]))])
            )]),
            &EncodeOptions::new().with_encode(false),
        )
        .unwrap(),
        "a[b][c]=d&a[b][e]=f"
    );
    assert_eq!(
        encode(
            &obj(vec![(
                "a",
                obj(vec![("b", obj(vec![("c", s("d")), ("e", s("f"))]))])
            )]),
            &EncodeOptions::new()
                .with_encode(false)
                .with_allow_dots(true),
        )
        .unwrap(),
        "a.b.c=d&a.b.e=f"
    );
    assert_eq!(
        encode(
            &obj(vec![(
                "name.obj",
                obj(vec![("first", s("John")), ("last", s("Doe"))])
            )]),
            &EncodeOptions::new()
                .with_allow_dots(true)
                .with_encode_dot_in_keys(true),
        )
        .unwrap(),
        "name%252Eobj.first=John&name%252Eobj.last=Doe"
    );
    assert_eq!(
        encode(
            &obj(vec![("foo", arr(vec![])), ("bar", s("baz"))]),
            &EncodeOptions::new()
                .with_encode(false)
                .with_allow_empty_lists(true),
        )
        .unwrap(),
        "foo[]&bar=baz"
    );
    assert_eq!(
        encode(
            &obj(vec![("a", s("b")), ("c", s("d"))]),
            &EncodeOptions::new().with_add_query_prefix(true),
        )
        .unwrap(),
        "?a=b&c=d"
    );
    assert_eq!(
        encode(
            &obj(vec![("a", s("b")), ("c", s("d"))]),
            &EncodeOptions::new().with_delimiter(";"),
        )
        .unwrap(),
        "a=b;c=d"
    );
}

#[test]
fn encoding_charset_and_format_examples_work() {
    assert_eq!(
        encode(
            &obj(vec![("æ", s("æ"))]),
            &EncodeOptions::new().with_charset(Charset::Iso88591),
        )
        .unwrap(),
        "%E6=%E6"
    );
    assert_eq!(
        encode(
            &obj(vec![("a", s("☺"))]),
            &EncodeOptions::new().with_charset(Charset::Iso88591),
        )
        .unwrap(),
        "a=%26%239786%3B"
    );
    assert_eq!(
        encode(
            &obj(vec![("a", s("☺"))]),
            &EncodeOptions::new().with_charset_sentinel(true),
        )
        .unwrap(),
        "utf8=%E2%9C%93&a=%E2%98%BA"
    );
    assert_eq!(
        encode(
            &obj(vec![("a", s("æ"))]),
            &EncodeOptions::new()
                .with_charset(Charset::Iso88591)
                .with_charset_sentinel(true),
        )
        .unwrap(),
        "utf8=%26%2310003%3B&a=%E6"
    );
    assert_eq!(
        encode(&obj(vec![("a", s("b c"))]), &EncodeOptions::new(),).unwrap(),
        "a=b%20c"
    );
    assert_eq!(
        encode(
            &obj(vec![("a", s("b c"))]),
            &EncodeOptions::new().with_format(Format::Rfc1738),
        )
        .unwrap(),
        "a=b+c"
    );
}
