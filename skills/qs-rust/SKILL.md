---
name: qs-rust
description: Use this skill whenever a user wants to install, configure, troubleshoot, or write Rust application code for encoding and decoding nested query strings with the qs_rust crate. This skill helps produce practical decode, decode_pairs, encode, serde from_str/to_string, Value/Object, DecodeOptions, and EncodeOptions snippets, choose option tradeoffs, and avoid qs_rust edge-case pitfalls around lists, dot notation, duplicates, null handling, charsets, serde scalar semantics, depth limits, untrusted input, and qs interoperability.
---

# qs_rust Usage Assistant

Help users parse and build query strings with the Rust `qs_rust` crate.
Focus on user application code and interoperability outcomes, not repository
maintenance, benchmarking, or release workflow.

## Start With Inputs

Before producing a final snippet, collect only the missing details that change
the code:

- Direction: decode an incoming query string, encode Rust data, merge structured
  pairs with `decode_pairs`, or use typed serde helpers.
- The actual query string, key/value pairs, `Value` tree, or Rust struct when
  available.
- Runtime context: web handler, CLI, tests, generated example, or library code.
- Target API convention for lists: indexed brackets, empty brackets, repeated
  keys, or comma-separated values.
- Whether the query may include a leading `?`, dot notation, literal dots in
  keys, duplicate keys, custom delimiters, comma-separated lists, `null` flags,
  ISO-8859-1/legacy charset behavior, serde typed models, temporal values, or
  untrusted user input.

Do not over-ask when the desired behavior is obvious. State assumptions in the
answer and give the user a concrete snippet they can paste.

## Installation

Use the core crate for the dynamic `Value` API:

```toml
[dependencies]
qs_rust = "<version>"
```

Enable typed serde helpers when the user wants to decode into structs or encode
serializable Rust data directly:

```toml
[dependencies]
qs_rust = { version = "<version>", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
```

Enable temporal adapters only when the user needs native `chrono` or `time`
integration:

```toml
[dependencies]
qs_rust = { version = "<version>", features = ["chrono", "time"] }
```

The crate-wide MSRV is Rust 1.88.

## Public API

Prefer the top-level re-exports:

```rust
use qs_rust::{decode, encode, DecodeOptions, EncodeOptions, ListFormat, Value};
```

Use `decode_pairs` when the caller already has structured key/value pairs:

```rust
use qs_rust::{decode_pairs, DecodeOptions, Value};
```

With the `serde` feature enabled, use:

```rust
use qs_rust::{from_str, from_value, to_string, to_value};
```

For application snippets, do not import from private or internal modules.

## Base Patterns

Decode a nested query string into the dynamic `Object`/`Value` model:

```rust
use qs_rust::{decode, DecodeOptions, Value};

let params = decode(
    "user[name]=Ada&tags[]=rust&tags[]=serde",
    &DecodeOptions::new(),
)
.unwrap();

assert_eq!(
    params.get("user"),
    Some(&Value::Object(
        [("name".to_owned(), Value::String("Ada".to_owned()))].into(),
    )),
);
assert_eq!(
    params.get("tags"),
    Some(&Value::Array(vec![
        Value::String("rust".to_owned()),
        Value::String("serde".to_owned()),
    ])),
);
```

Encode nested Rust values into a query string:

```rust
use qs_rust::{encode, EncodeOptions, ListFormat, Value};

let data = Value::Object(
    [
        (
            "user".to_owned(),
            Value::Object([("name".to_owned(), Value::String("Ada".to_owned()))].into()),
        ),
        (
            "tags".to_owned(),
            Value::Array(vec![
                Value::String("rust".to_owned()),
                Value::String("serde".to_owned()),
            ]),
        ),
    ]
    .into(),
);

let query = encode(
    &data,
    &EncodeOptions::new().with_list_format(ListFormat::Brackets),
)
.unwrap();

assert_eq!(query, "user%5Bname%5D=Ada&tags%5B%5D=rust&tags%5B%5D=serde");
```

Query-string decoding only produces `Value::Null`, `Value::String`,
`Value::Array`, and `Value::Object`. Structured inputs passed to `encode` or
`decode_pairs` may also contain booleans, numbers, bytes, and temporal leaves.

## Decode Recipes

Use these options with `decode(query, &DecodeOptions::new()...)`:

- Leading question mark: `with_ignore_query_prefix(true)`.
- Dot notation such as `a.b=c`: `with_allow_dots(true)`.
- Double-encoded literal dots in keys such as `name%252Eobj.first=John`:
  `with_decode_dot_in_keys(true)`.
- Duplicate keys: `Duplicates::Combine` keeps all values as an array where
  possible; use `Duplicates::First` or `Duplicates::Last` to collapse.
- Bracket lists: enabled by default; set `with_parse_lists(false)` to treat
  list syntax as object keys.
- Empty list tokens such as `foo[]`: `with_allow_empty_lists(true)`.
- Sparse list indices: use `with_allow_sparse_lists(true)` to preserve gaps.
- List representation threshold: default `list_limit` is `20`; numeric indices
  at or above the threshold become object keys. Soft overflow preserves every
  value in a numeric-keyed object, while `throw_on_limit_exceeded` reports a
  `DecodeError`. The threshold applies to cumulative list growth across
  duplicate keys, comma values, bracket notation, mixed index/scalar/append
  forms, sparse lists, and nested merges. Sparse gaps count toward list length.
- Comma-separated values such as `a=b,c`: `with_comma(true)`. A comma group
  assigned through `a[]=` counts as one outer list element.
- Tokens without `=` as null: `with_strict_null_handling(true)`.
- Custom delimiters: `with_delimiter(Delimiter::String(";".to_owned()))`, or
  `with_delimiter(Delimiter::Regex(regex))` when the app also depends on
  `regex`.
- Legacy charset input: `with_charset(Charset::Iso88591)`; use
  `with_charset_sentinel(true)` when a form may include `utf8=...` to signal
  the real charset.
- HTML numeric entities: `with_interpret_numeric_entities(true)`, usually with
  ISO-8859-1 or charset sentinel handling.
- Untrusted input: bound the raw input size and keep `depth`, `parameter_limit`,
  and `list_limit` controlled. Use `with_strict_depth(true)` and
  `with_throw_on_limit_exceeded(true)` when callers need hard failures;
  `list_limit` alone changes representation without capping accepted values.

Example for a request query:

```rust
use qs_rust::{decode, DecodeOptions, Duplicates, Value};

let params = decode(
    "?filter.status=open&tag=rust&tag=serde",
    &DecodeOptions::new()
        .with_ignore_query_prefix(true)
        .with_allow_dots(true)
        .with_duplicates(Duplicates::Combine),
)
.unwrap();

assert_eq!(
    params.get("filter"),
    Some(&Value::Object(
        [("status".to_owned(), Value::String("open".to_owned()))].into(),
    )),
);
assert_eq!(
    params.get("tag"),
    Some(&Value::Array(vec![
        Value::String("rust".to_owned()),
        Value::String("serde".to_owned()),
    ])),
);
```

Use `decode_pairs` only when splitting, prefix stripping, charset sentinel
detection, and numeric entity interpretation have already happened elsewhere:

```rust
use qs_rust::{decode_pairs, DecodeOptions, Value};

let params = decode_pairs(
    vec![
        ("a[b]".to_owned(), Value::String("1".to_owned())),
        ("a[b]".to_owned(), Value::String("2".to_owned())),
    ],
    &DecodeOptions::new(),
)
.unwrap();

assert_eq!(
    params.get("a"),
    Some(&Value::Object(
        [(
            "b".to_owned(),
            Value::Array(vec![
                Value::String("1".to_owned()),
                Value::String("2".to_owned()),
            ]),
        )]
        .into(),
    )),
);
```

## Encode Recipes

Use these options with `encode(&value, &EncodeOptions::new()...)`:

- List style defaults to `ListFormat::Indices`:
  `tags%5B0%5D=rust&tags%5B1%5D=serde`.
- Empty brackets: `with_list_format(ListFormat::Brackets)`.
- Repeated keys: `with_list_format(ListFormat::Repeat)`.
- Comma-separated values: `with_list_format(ListFormat::Comma)`.
- Single-item comma lists that must round-trip as lists:
  `with_comma_round_trip(true)`.
- Drop null items before comma-joining lists:
  `with_comma_compact_nulls(true)`.
- Dot notation for nested objects: `with_allow_dots(true)`.
- Literal dots in keys: `with_encode_dot_in_keys(true)`.
- Add a leading `?`: `with_add_query_prefix(true)`.
- Custom pair delimiter: `with_delimiter(";")`.
- Preserve readable bracket/dot keys while encoding values:
  `with_encode_values_only(true)`.
- Disable percent encoding entirely for debugging or documented examples:
  `with_encode(false)`.
- Emit null without `=`: `with_strict_null_handling(true)`.
- Omit null values: `with_skip_nulls(true)`.
- Emit empty lists as `foo[]`: `with_allow_empty_lists(true)`.
- Legacy form spaces as `+`: `with_format(Format::Rfc1738)`; the default
  is `Format::Rfc3986`, which emits spaces as `%20`.
- Legacy charset output: `with_charset(Charset::Iso88591)`; use
  `with_charset_sentinel(true)` to prepend the `utf8=...` sentinel.
- Custom behavior: use `with_whitelist`, `with_sort`,
  `with_filter`, `with_sorter`, `with_encoder`, or
  `with_temporal_serializer` when the target API needs selected fields,
  stable ordering, special scalar encoding, or custom datetime formatting.

Example for an API that expects repeated keys:

```rust
use qs_rust::{encode, EncodeOptions, ListFormat, Value};

let data = Value::Object(
    [
        ("q".to_owned(), Value::String("query strings".to_owned())),
        (
            "tag".to_owned(),
            Value::Array(vec![
                Value::String("rust".to_owned()),
                Value::String("serde".to_owned()),
            ]),
        ),
    ]
    .into(),
);

let query = encode(
    &data,
    &EncodeOptions::new()
        .with_list_format(ListFormat::Repeat)
        .with_add_query_prefix(true),
)
.unwrap();

assert_eq!(query, "?q=query%20strings&tag=rust&tag=serde");
```

## Serde Bridge

Use the `serde` feature when the user wants typed structs at the boundary:

```rust
use qs_rust::{from_str, to_string, DecodeOptions, EncodeOptions};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Query {
    page: String,
    tags: Vec<String>,
}

let query: Query = from_str(
    "page=2&tags[0]=rust&tags[1]=serde",
    &DecodeOptions::new(),
)
.unwrap();
assert_eq!(
    query,
    Query {
        page: "2".to_owned(),
        tags: vec!["rust".to_owned(), "serde".to_owned()],
    },
);

let encoded = to_string(&query, &EncodeOptions::new().with_encode(false)).unwrap();
assert_eq!(encoded, "page=2&tags[0]=rust&tags[1]=serde");
```

The serde bridge routes through the same dynamic `Value` core. Plain query
scalars such as `page=2` and `admin=true` decode as strings unless the user's
serde model adds its own conversion layer. For native datetime preservation,
point users to `qs_rust::serde::temporal::*` helpers behind the relevant
`chrono` or `time` feature.

## Combinations To Check

Warn or adjust before giving code for these cases:

- `with_parameter_limit(0)` is invalid.
- Empty string delimiters are invalid for both decode and encode.
- `with_decode_dot_in_keys(true)` and `with_encode_dot_in_keys(true)` imply dot
  notation; turning dot notation off afterward clears the dot-key option.
- `with_throw_on_limit_exceeded(true)` turns parameter and cumulative list
  limit overflows into `DecodeError` values; without it, parameter parsing
  truncates and list growth falls back to numeric-keyed objects.
- `with_strict_depth(true)` errors on well-formed decode depth overflow; with
  the default `false`, the remainder beyond `depth` is kept as a trailing key
  segment.
- `with_max_depth(Some(n))` limits encode traversal and can return
  `EncodeError::DepthExceeded`.
- Built-in charset handling supports UTF-8 and ISO-8859-1.
- `with_comma(true)` parses simple comma-separated values, but does not decode
  nested object syntax inside comma items. Flat comma values are measured by
  item count against the list threshold, while each `[]=` comma group is one
  outer list item.
- `encode` of scalar roots, empty objects, and empty containers generally
  produces an empty string.
- Standard URL extractors and many web frameworks flatten duplicates or nested
  query syntax. Prefer `decode` on the raw query string when qs-style nested or
  repeated values matter.
- `DecodeError` and `EncodeError` are non-exhaustive. Match them with a catch-all
  arm and prefer stable inspector helpers for durable limit/depth checks.

## Response Shape

For code-generation requests, answer with:

1. A short statement of assumptions, especially Rust feature flags, list format,
   null handling, charset, prefix handling, serde use, and whether input is
   trusted.
2. One concrete Rust snippet using `decode`, `decode_pairs`, `encode`,
   `from_str`, `to_string`, `from_value`, or `to_value`.
3. A brief explanation of only the options used.
4. A small verification example, such as `assert_eq!`, expected `Value`, expected
   query string, or a typed struct round trip.

Keep snippets application-oriented. Prefer public API imports from `qs_rust`;
do not ask users to import from `qs_rust` private modules.
