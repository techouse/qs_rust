use qs_rust::{Charset, DecodeOptions, Delimiter, Duplicates};
use regex::Regex;

use super::{CaseMeta, DecodeParityCase};

pub(crate) fn cases() -> Vec<DecodeParityCase> {
    let mut cases = vec![
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
                "normalizes dots before applying depth zero",
                "depth",
                true,
            ),
            "a.b=c",
            DecodeOptions::new().with_allow_dots(true).with_depth(0),
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
                "literal empty brackets inside bracket group",
                "brackets",
                true,
            ),
            "search[withbracket[]]=foobar",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "single-level literal empty brackets inside bracket group",
                "brackets",
                true,
            ),
            "a[b[]]=c",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "outer array push with literal empty brackets inside child group",
                "brackets",
                true,
            ),
            "list[][x[]]=y",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "nested bracket pairs stay literal inside bracket group",
                "brackets",
                true,
            ),
            "a[b[c[]]]=d",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "depth limit preserves literal nested bracket group",
                "depth",
                true,
            ),
            "a[b[c[]]][d]=e",
            DecodeOptions::new().with_depth(1),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "unterminated bracket group stays one literal segment",
                "brackets",
                true,
            ),
            "a[[]b=c",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "trailing text after bracket group is ignored",
                "brackets",
                true,
            ),
            "a[b]tail=x",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "top-level percent-encoded bracket text is not mangled",
                "encoded brackets",
                true,
            ),
            "a%25255Bb=c",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "top-level percent-encoded closing bracket text is not mangled",
                "encoded brackets",
                true,
            ),
            "a%25255Db=c",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "stringify.js",
                "nested percent-encoded bracket text is not mangled",
                "encoded brackets",
                true,
            ),
            "a%5Bb%25255Bc%5D=d",
            DecodeOptions::new(),
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
                "bracket notation combines with duplicates first",
                "duplicates",
                true,
            ),
            "a=1&a=2&b[]=1&b[]=2",
            DecodeOptions::new().with_duplicates(Duplicates::First),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "bracket notation combines with duplicates last",
                "duplicates",
                true,
            ),
            "a=1&a=2&b[]=1&b[]=2",
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
                "strict merge wraps object then scalar conflicts by default",
                "merging",
                true,
            ),
            "a[b]=c&a=d",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "legacy merge adds scalar string keys when strict merge is disabled",
                "merging",
                true,
            ),
            "a[b]=c&a=d",
            DecodeOptions::new().with_strict_merge(false),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "legacy merge ignores empty assigned scalars",
                "merging",
                true,
            ),
            "a[b]=c&a=",
            DecodeOptions::new().with_strict_merge(false),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "node-qs",
                "parse.js",
                "legacy merge ignores empty missing-value scalars",
                "merging",
                true,
            ),
            "a[b]=c&a",
            DecodeOptions::new().with_strict_merge(false),
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
    ];
    cases.extend(qs_6_15_3_cases());
    cases
}

fn qs_6_15_3_cases() -> Vec<DecodeParityCase> {
    vec![
        node_parse_case(
            "mixed scalar then index enforces cumulative list limit",
            "list limits",
            "a=x&a[0]=y",
            DecodeOptions::new()
                .with_list_limit(1)
                .with_throw_on_limit_exceeded(true),
        ),
        node_parse_case(
            "mixed index then append enforces cumulative list limit",
            "list limits",
            "a[0]=x&a[]=y",
            DecodeOptions::new()
                .with_list_limit(1)
                .with_throw_on_limit_exceeded(true),
        ),
        node_parse_case(
            "duplicate scalars enforce cumulative list limit",
            "list limits",
            "a=x&a=y",
            DecodeOptions::new()
                .with_list_limit(1)
                .with_throw_on_limit_exceeded(true),
        ),
        node_parse_case(
            "duplicate bracket values enforce cumulative list limit",
            "list limits",
            "a[]=x&a[]=y",
            DecodeOptions::new()
                .with_list_limit(1)
                .with_throw_on_limit_exceeded(true),
        ),
        node_parse_case(
            "indexed list then scalar enforces cumulative list limit",
            "list limits",
            "a[0]=1&a[1]=2&a=3",
            DecodeOptions::new()
                .with_list_limit(1)
                .with_throw_on_limit_exceeded(true),
        ),
        node_parse_case(
            "mixed index then scalar promotes on cumulative overflow",
            "list limits",
            "a[0]=x&a=y",
            DecodeOptions::new().with_list_limit(1),
        ),
        node_parse_case(
            "mixed index then append promotes on cumulative overflow",
            "list limits",
            "a[0]=x&a[]=y",
            DecodeOptions::new().with_list_limit(1),
        ),
        node_parse_case(
            "mixed overflow keeps later append indices",
            "list limits",
            "a[0]=x&a=y&a[]=z",
            DecodeOptions::new().with_list_limit(1),
        ),
        node_parse_case(
            "sparse overflow allows a later null to fill an omitted index",
            "list limits",
            "a[1]=x&a=y&a[0]",
            DecodeOptions::new()
                .with_list_limit(2)
                .with_strict_null_handling(true),
        ),
        node_parse_case(
            "nested mixed list growth enforces cumulative limit",
            "list limits",
            "a[b][0]=x&a[b][]=y",
            DecodeOptions::new()
                .with_list_limit(1)
                .with_throw_on_limit_exceeded(true),
        ),
        node_parse_case(
            "cumulative comma growth succeeds at limit",
            "comma",
            "a=1,2,3&a=4,5",
            DecodeOptions::new()
                .with_comma(true)
                .with_list_limit(5)
                .with_throw_on_limit_exceeded(true),
        ),
        node_parse_case(
            "cumulative comma growth throws over limit",
            "comma",
            "a=1,2,3&a=4,5,6",
            DecodeOptions::new()
                .with_comma(true)
                .with_list_limit(5)
                .with_throw_on_limit_exceeded(true),
        ),
        node_parse_case(
            "cumulative comma growth softly promotes over limit",
            "comma",
            "a=1,2,3&a=4,5,6",
            DecodeOptions::new().with_comma(true).with_list_limit(5),
        ),
        node_parse_case(
            "later comma token throws after earlier cumulative overflow",
            "comma",
            "a=v,v,v,v,v&a=v,v,v,v,v&a=v,v,v,v,v",
            DecodeOptions::new()
                .with_comma(true)
                .with_list_limit(5)
                .with_throw_on_limit_exceeded(true),
        ),
        node_parse_case(
            "nested flat comma value throws before splitting",
            "comma",
            "a[b]=1,2,3,4,5,6",
            DecodeOptions::new()
                .with_comma(true)
                .with_list_limit(5)
                .with_throw_on_limit_exceeded(true),
        ),
        node_parse_case(
            "bracketed comma group counts as one outer item",
            "comma",
            "a[]=1,2,3,4,5,6",
            DecodeOptions::new()
                .with_comma(true)
                .with_list_limit(1)
                .with_throw_on_limit_exceeded(true),
        ),
        node_parse_case(
            "multiple bracketed comma groups count as outer items",
            "comma",
            "a[]=1,2,3&a[]=4,5,6",
            DecodeOptions::new()
                .with_comma(true)
                .with_list_limit(5)
                .with_throw_on_limit_exceeded(true),
        ),
        node_parse_case(
            "bracketed comma group softly overflows at zero outer items",
            "comma",
            "a[]=1,2",
            DecodeOptions::new().with_comma(true).with_list_limit(0),
        ),
        node_parse_case(
            "bracketed comma group throws at zero outer items",
            "comma",
            "a[]=1,2",
            DecodeOptions::new()
                .with_comma(true)
                .with_list_limit(0)
                .with_throw_on_limit_exceeded(true),
        ),
        node_parse_case(
            "unclosed group after a parent",
            "unbalanced brackets",
            "a[bc=v",
            DecodeOptions::new(),
        ),
        node_parse_case(
            "bare unclosed bracket after a parent",
            "unbalanced brackets",
            "a[=v",
            DecodeOptions::new(),
        ),
        node_parse_case(
            "unclosed group after a valid group",
            "unbalanced brackets",
            "a[b][c=v",
            DecodeOptions::new(),
        ),
        node_parse_case(
            "unclosed group after trailing text",
            "unbalanced brackets",
            "a[b]c[d=v",
            DecodeOptions::new(),
        ),
        node_parse_case(
            "issue 558 custom tag reproduction",
            "unbalanced brackets",
            "filters[customtags:Env: Prod=v",
            DecodeOptions::new(),
        ),
        node_parse_case(
            "stray close before unclosed group",
            "unbalanced brackets",
            "][a=v",
            DecodeOptions::new(),
        ),
        node_parse_case(
            "stray close inside parent before unclosed group",
            "unbalanced brackets",
            "a][b=v",
            DecodeOptions::new(),
        ),
        node_parse_case(
            "unclosed group containing inner bracket",
            "unbalanced brackets",
            "a[b[c=v",
            DecodeOptions::new(),
        ),
        node_parse_case(
            "unbalanced group containing inner close bracket",
            "unbalanced brackets",
            "a[b[c]=v",
            DecodeOptions::new(),
        ),
        node_parse_case(
            "unclosed inner bracket group after valid group",
            "unbalanced brackets",
            "a[b][c[d=v",
            DecodeOptions::new(),
        ),
        node_parse_case(
            "key starts with unclosed bracket",
            "unbalanced brackets",
            "[abc=v",
            DecodeOptions::new(),
        ),
        node_parse_case(
            "key starts with unbalanced bracket group",
            "unbalanced brackets",
            "[[]b=v",
            DecodeOptions::new(),
        ),
        node_parse_case(
            "depth budget leaves final unclosed remainder literal",
            "unbalanced brackets",
            "a[b]c[d]e[f=v",
            DecodeOptions::new().with_depth(5),
        ),
        node_parse_case(
            "low depth leaves more unclosed remainder literal",
            "unbalanced brackets",
            "a[b]c[d]e[f=v",
            DecodeOptions::new().with_depth(1),
        ),
        node_parse_case(
            "depth zero keeps unbalanced key literal",
            "unbalanced brackets",
            "a[bc=v",
            DecodeOptions::new().with_depth(0),
        ),
        node_parse_case(
            "allow dots preserves trailing unclosed bracket",
            "unbalanced brackets",
            "a.b[c=v",
            DecodeOptions::new().with_allow_dots(true),
        ),
        node_parse_case(
            "stray close without open bracket stays flat",
            "unbalanced brackets",
            "a]b=v",
            DecodeOptions::new(),
        ),
        node_parse_case(
            "text after balanced group remains ignored",
            "unbalanced brackets",
            "a[b]extra=v",
            DecodeOptions::new(),
        ),
    ]
}

fn node_parse_case(
    title: &'static str,
    family: &'static str,
    query: &'static str,
    options: DecodeOptions,
) -> DecodeParityCase {
    DecodeParityCase::new(
        CaseMeta::new("node-qs", "parse.js", title, family, true),
        query,
        options,
    )
}
