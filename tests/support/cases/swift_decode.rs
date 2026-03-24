use qs_rust::{Charset, DecodeOptions, Delimiter};
use regex::Regex;

use super::{CaseMeta, DecodeParityCase};

pub(crate) fn decode_cases() -> Vec<DecodeParityCase> {
    vec![
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "allow dots parses real dotted keys as nested objects",
                "dot notation",
                true,
            ),
            "a.b=c",
            DecodeOptions::new().with_allow_dots(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "allowDots with decodeDotInKeys false keeps encoded dots literal in parent keys",
                "dot notation",
                true,
            ),
            "name%252Eobj.first=John&name%252Eobj.last=Doe",
            DecodeOptions::new().with_allow_dots(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "empty bracket key defaults to an empty-string list element",
                "empty arrays",
                true,
            ),
            "foo[]&bar=baz",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "allowEmptyLists true keeps an empty list instead of a single empty string",
                "empty arrays",
                true,
            ),
            "foo[]&bar=baz",
            DecodeOptions::new().with_allow_empty_lists(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "decodeDotInKeys implies allowDots for deep encoded-dot paths",
                "dot notation",
                true,
            ),
            "name%252Eobj%252Esubobject.first%252Egodly%252Ename=John&name%252Eobj%252Esubobject.last=Doe",
            DecodeOptions::new().with_decode_dot_in_keys(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "allowEmptyLists with strictNullHandling still produces an empty list for bare brackets",
                "empty arrays",
                true,
            ),
            "testEmptyList[]",
            DecodeOptions::new()
                .with_allow_empty_lists(true)
                .with_strict_null_handling(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "depth zero preserves fully bracketed keys literally",
                "depth",
                true,
            ),
            "a[0][0]=b&a[0][1]=c&a[1]=d&e=2",
            DecodeOptions::new().with_depth(0),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "default depth preserves the remaining suffix as a single bracketed key",
                "depth",
                true,
            ),
            "a[b][c][d][e][f][g][h]=i",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "array limit zero keeps indexed entries as object keys",
                "arrays",
                true,
            ),
            "a[1]=c",
            DecodeOptions::new().with_list_limit(0),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "parse lists false keeps numeric indices as string keys",
                "arrays",
                true,
            ),
            "a[0]=b&a[2]=c",
            DecodeOptions::new().with_parse_lists(false),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "mixed numeric and named keys promote arrays to ordered objects",
                "arrays",
                true,
            ),
            "foo[bad]=baz&foo[]=bar&foo[]=foo",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "lists of maps decode through empty bracket children",
                "arrays",
                true,
            ),
            "a[][b]=c",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "encoded equals signs decode in both key and value",
                "decoding",
                true,
            ),
            "he%3Dllo=th%3Dere",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "brackets inside values remain scalar text",
                "decoding",
                true,
            ),
            "pets=[\"tobi\"]",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "special operator child keys remain plain object keys",
                "objects",
                true,
            ),
            "a[>=]=25",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "ignore query prefix false keeps the leading question mark in the key",
                "delimiter",
                true,
            ),
            "?foo=bar",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "bare ampersand does not create a phantom key",
                "delimiter",
                true,
            ),
            "&",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "trailing ampersand does not create a phantom key",
                "delimiter",
                true,
            ),
            "_r=1&",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "comma parsing respects percent-encoded commas before splitting",
                "comma",
                true,
            ),
            "foo=a%2C%20b,d",
            DecodeOptions::new().with_comma(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "keys beginning with digits inside brackets stay object keys",
                "arrays",
                true,
            ),
            "a[12b]=c",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "bracketed numeric top-level keys become string indices",
                "root collisions",
                true,
            ),
            "[0]=a&[1]=b",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "empty strings and bare keys stay distinguishable in object fallback mode",
                "null handling",
                true,
            ),
            "a[]=b&a[]&a[]=c&a[]=",
            DecodeOptions::new()
                .with_list_limit(0)
                .with_strict_null_handling(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "root-level empty brackets continue parsing as key zero",
                "root collisions",
                true,
            ),
            "[]=&a=b",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "root-level empty brackets respect strictNullHandling",
                "null handling",
                true,
            ),
            "[]&a=b",
            DecodeOptions::new().with_strict_null_handling(true),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "root-level bracketed keys become plain top-level keys",
                "root collisions",
                true,
            ),
            "[foo]=bar",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "params starting with a closing bracket stay plain keys",
                "root collisions",
                true,
            ),
            "]=toString",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "params starting with a starting bracket stay plain keys",
                "root collisions",
                true,
            ),
            "[=toString",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "scalar roots combine with later object leaves under the same key",
                "root collisions",
                true,
            ),
            "a[b]=c&a=d",
            DecodeOptions::new(),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "regex delimiters can consume optional whitespace",
                "delimiter",
                true,
            ),
            "a=b; c=d",
            DecodeOptions::new().with_delimiter(Delimiter::Regex(Regex::new("[;,] *").unwrap())),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "percent-u escapes remain literal in ISO-8859-1 mode",
                "charset",
                true,
            ),
            "%u263A=%u263A",
            DecodeOptions::new().with_charset(Charset::Iso88591),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "parameterLimit truncates later pairs when limit is reached",
                "parameter limit",
                true,
            ),
            "a=b&c=d",
            DecodeOptions::new().with_parameter_limit(1),
        ),
        DecodeParityCase::new(
            CaseMeta::new(
                "swift-qsswift",
                "DecodeTests.swift",
                "jquery style indexed filters decode into nested arrays",
                "arrays",
                true,
            ),
            "filter%5B0%5D%5B%5D=int1&filter%5B0%5D%5B%5D=%3D&filter%5B0%5D%5B%5D=77&filter%5B%5D=and&filter%5B2%5D%5B%5D=int2&filter%5B2%5D%5B%5D=%3D&filter%5B2%5D%5B%5D=8",
            DecodeOptions::new(),
        ),
    ]
}
