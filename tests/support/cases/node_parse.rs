use qs_rust::{Charset, DecodeOptions, Delimiter, Duplicates};
use regex::Regex;

use super::{CaseMeta, DecodeParityCase};

pub(crate) fn cases() -> Vec<DecodeParityCase> {
    vec![
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "parses a simple string",
                "basic",
                true,
            ),
            "a=b&c=d",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new("node-qs", "parse.js", "comma false", "comma", true),
            "a=b,c",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new("node-qs", "parse.js", "comma true", "comma", true),
            "a=b,c",
            DecodeOptions::new().with_comma(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new("node-qs", "parse.js", "allow dots", "dot notation", true),
            "user.name.first=alice",
            DecodeOptions::new().with_allow_dots(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "decode dot in keys",
                "dot notation",
                true,
            ),
            "name%252Eobj.first=John",
            DecodeOptions::new().with_decode_dot_in_keys(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "allows empty arrays in values",
                "empty arrays",
                true,
            ),
            "foo[]",
            DecodeOptions::new()
                .with_allow_empty_lists(true)
                .with_strict_null_handling(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "uses original key when depth is zero",
                "depth",
                true,
            ),
            "a[b][c]=d",
            DecodeOptions::new().with_depth(0),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "only parses one level when depth is one",
                "depth",
                true,
            ),
            "a[b][c][d]=e",
            DecodeOptions::new().with_depth(1),
        ),
        DecodeParityCase::new(
            CaseMeta::new("node-qs", "parse.js", "strict depth throws", "depth", true),
            "a[b][c][d]=e",
            DecodeOptions::new().with_depth(1).with_strict_depth(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "parses an explicit array",
                "arrays",
                true,
            ),
            "a[]=b&a[]=c",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "indexed array within limit",
                "arrays",
                true,
            ),
            "a[4]=b",
            DecodeOptions::new()
                .with_list_limit(5)
                .with_allow_sparse_lists(true)
                .with_throw_on_limit_exceeded(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "indexed array over limit converts to object",
                "arrays",
                true,
            ),
            "a[1001]=b",
            DecodeOptions::new().with_list_limit(1000),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "alternative string delimiter",
                "delimiter",
                true,
            ),
            "a=b;c=d",
            DecodeOptions::new().with_delimiter(Delimiter::String(";".to_owned())),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "alternative regex delimiter",
                "delimiter",
                true,
            ),
            "a=b;c=d,e=f",
            DecodeOptions::new().with_delimiter(Delimiter::Regex(Regex::new("[;,]").unwrap())),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "ignore query prefix",
                "delimiter",
                true,
            ),
            "?a=b&c=d",
            DecodeOptions::new().with_ignore_query_prefix(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new("node-qs", "parse.js", "parse arrays false", "arrays", true),
            "a[]=b&a[0]=c",
            DecodeOptions::new().with_parse_lists(false),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "compacts sparse arrays",
                "sparse arrays",
                true,
            ),
            "a[1]=b&a[3]=c",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "parses sparse arrays",
                "sparse arrays",
                true,
            ),
            "a[1]=b&a[3]=c",
            DecodeOptions::new().with_allow_sparse_lists(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "charset sentinel switches to latin1",
                "charset",
                true,
            ),
            "utf8=%26%2310003%3B&%F8=%F8",
            DecodeOptions::new()
                .with_charset(Charset::Utf8)
                .with_charset_sentinel(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "numeric entities in latin1",
                "charset",
                true,
            ),
            "name=%26%239786%3B",
            DecodeOptions::new()
                .with_charset(Charset::Iso88591)
                .with_interpret_numeric_entities(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "malformed uri characters",
                "decoding",
                true,
            ),
            "a=%E0%A4%A",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "duplicates first",
                "duplicates",
                true,
            ),
            "foo=bar&foo=baz",
            DecodeOptions::new().with_duplicates(Duplicates::First),
        ),
        DecodeParityCase::new(
            CaseMeta::new("node-qs", "parse.js", "duplicates last", "duplicates", true),
            "foo=bar&foo=baz",
            DecodeOptions::new().with_duplicates(Duplicates::Last),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "strict null handling",
                "null handling",
                true,
            ),
            "flag",
            DecodeOptions::new().with_strict_null_handling(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "parameter limit throws",
                "parameter limit",
                true,
            ),
            "a=1&b=2&c=3",
            DecodeOptions::new()
                .with_parameter_limit(2)
                .with_throw_on_limit_exceeded(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "empty keys skipped",
                "empty keys",
                true,
            ),
            "=x&=y&a=1",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new("node-qs", "parse.js", "root collisions", "merging", true),
            "a=1&a[b]=2",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "comma list within limit",
                "comma",
                true,
            ),
            "a=1,2,3",
            DecodeOptions::new().with_comma(true).with_list_limit(5),
        ),
        DecodeParityCase::new(
            CaseMeta::new("node-qs", "parse.js", "comma list at limit", "comma", true),
            "a=1,2,3",
            DecodeOptions::new().with_comma(true).with_list_limit(3),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "comma list over limit converts to object",
                "comma",
                true,
            ),
            "a=1,2,3,4",
            DecodeOptions::new().with_comma(true).with_list_limit(3),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "comma list over limit throws",
                "comma",
                true,
            ),
            "a=1,2,3,4",
            DecodeOptions::new()
                .with_comma(true)
                .with_list_limit(3)
                .with_throw_on_limit_exceeded(true),
        ),
    ]
}
