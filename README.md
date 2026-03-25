# qs_rust

![qs_rust](https://github.com/techouse/qs_rust/blob/main/logo.png?raw=true)

A query string encoding and decoding library for Rust.

Ported from [qs](https://www.npmjs.com/package/qs) for JavaScript.

![Crates.io Version](https://img.shields.io/crates/v/qs_rust)
![Crates.io MSRV](https://img.shields.io/crates/msrv/qs_rust)
![Crates.io Size](https://img.shields.io/crates/size/qs_rust)
![Crates.io Downloads (recent)](https://img.shields.io/crates/dr/qs_rust)
[![Test](https://github.com/techouse/qs_rust/actions/workflows/test.yml/badge.svg)](https://github.com/techouse/qs_rust/actions/workflows/test.yml)
[![codecov](https://codecov.io/gh/techouse/qs_rust/graph/badge.svg?token=DHq7RZTFAn)](https://codecov.io/gh/techouse/qs_rust)
[![Codacy Badge](https://app.codacy.com/project/badge/Grade/51927280c1814424b844cad1eec67180)](https://app.codacy.com/gh/techouse/qs_rust/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade)
[![GitHub](https://img.shields.io/github/license/techouse/qs_rust)](https://github.com/techouse/qs_rust/blob/main/LICENSE)
[![GitHub Sponsors](https://img.shields.io/github/sponsors/techouse)](https://github.com/sponsors/techouse)
[![GitHub Repo stars](https://img.shields.io/github/stars/techouse/qs_rust)](https://github.com/techouse/qs_rust/stargazers)

## Highlights

- Nested object and list support: `foo[bar][baz]=qux` ⇄ nested `Value::Object` / `Value::Array`
- Multiple list formats: indices, brackets, repeat, and comma
- Dot-notation support plus `decode_dot_in_keys` / `encode_dot_in_keys`
- UTF-8 and Latin-1 charsets, optional charset sentinel support, and numeric-entity decoding
- Explicit Rust hook surfaces for custom decoding, filtering, sorting, scalar encoding, and temporal serialization
- Iterative decode, merge, compact, and encode paths for deep-input safety
- Node-backed parity tests plus cross-port regressions and perf tooling checked into the repo

## Installation

```toml
[dependencies]
qs_rust = "1.0.0"
```

Optional `serde` support:

```toml
[dependencies]
qs_rust = { version = "1.0.0", features = ["serde"] }
```

Optional temporal adapters:

```toml
[dependencies]
qs_rust = { version = "1.0.0", features = ["chrono", "time"] }
```

## Quick Start

```rust
use qs_rust::{decode, encode, DecodeOptions, EncodeOptions, ListFormat, Value};

let decoded = decode(
    "user[name]=alice&tags[]=x&tags[]=y",
    &DecodeOptions::new(),
)
.unwrap();

assert!(decoded.contains_key("user"));
assert!(decoded.contains_key("tags"));

let value = Value::Object(
    [
        (
            "user".to_owned(),
            Value::Object([("name".to_owned(), Value::String("alice".to_owned()))].into()),
        ),
        (
            "tags".to_owned(),
            Value::Array(vec![
                Value::String("x".to_owned()),
                Value::String("y".to_owned()),
            ]),
        ),
    ]
    .into(),
);

let encoded = encode(
    &value,
    &EncodeOptions::new().with_list_format(ListFormat::Brackets),
)
.unwrap();

assert_eq!(encoded, "user%5Bname%5D=alice&tags%5B%5D=x&tags%5B%5D=y");
```

Query-string decoding only produces `Null`, `String`, `Array`, and `Object`. Structured inputs passed to `encode` or `decode_pairs` may also contain `Bool`, numeric variants, and `Bytes`.

## Decoding

### Nested Objects, Depth, Prefixes, and Delimiters

```rust
use qs_rust::{decode, DecodeOptions, Delimiter, Value};

let nested = decode("foo[bar][baz]=qux", &DecodeOptions::new()).unwrap();
assert_eq!(
    nested.get("foo"),
    Some(&Value::Object(
        [(
            "bar".to_owned(),
            Value::Object([("baz".to_owned(), Value::String("qux".to_owned()))].into()),
        )]
        .into(),
    )),
);

let depth_limited = decode(
    "a[b][c][d][e][f][g]=x",
    &DecodeOptions::new().with_depth(1),
)
.unwrap();
assert_eq!(
    depth_limited.get("a"),
    Some(&Value::Object(
        [(
            "b".to_owned(),
            Value::Object([("[c][d][e][f][g]".to_owned(), Value::String("x".to_owned()))].into()),
        )]
        .into(),
    )),
);

let prefixed = decode(
    "?a=b&c=d",
    &DecodeOptions::new().with_ignore_query_prefix(true),
)
.unwrap();
assert_eq!(prefixed.get("a"), Some(&Value::String("b".to_owned())));
assert_eq!(prefixed.get("c"), Some(&Value::String("d".to_owned())));

let custom_delimiter = decode(
    "a=b;c=d",
    &DecodeOptions::new().with_delimiter(Delimiter::String(";".to_owned())),
)
.unwrap();
assert_eq!(custom_delimiter.get("a"), Some(&Value::String("b".to_owned())));
assert_eq!(custom_delimiter.get("c"), Some(&Value::String("d".to_owned())));
```

By default, decoding depth is `5`, parameter limit is `1000`, lists are compacted, and duplicate keys are combined into arrays.

### Dots, Lists, Duplicates, and Scalar Values

```rust
use qs_rust::{decode, DecodeOptions, Duplicates, Value};

let dotted = decode("a.b=c", &DecodeOptions::new().with_allow_dots(true)).unwrap();
assert_eq!(
    dotted.get("a"),
    Some(&Value::Object([("b".to_owned(), Value::String("c".to_owned()))].into())),
);

let decoded_dot_key = decode(
    "name%252Eobj.first=John&name%252Eobj.last=Doe",
    &DecodeOptions::new().with_decode_dot_in_keys(true),
)
.unwrap();
assert_eq!(
    decoded_dot_key.get("name.obj"),
    Some(&Value::Object(
        [
            ("first".to_owned(), Value::String("John".to_owned())),
            ("last".to_owned(), Value::String("Doe".to_owned())),
        ]
        .into(),
    )),
);

let list = decode("a[]=b&a[]=c", &DecodeOptions::new()).unwrap();
assert_eq!(
    list.get("a"),
    Some(&Value::Array(vec![
        Value::String("b".to_owned()),
        Value::String("c".to_owned()),
    ])),
);

let empty_list = decode(
    "foo[]&bar=baz",
    &DecodeOptions::new().with_allow_empty_lists(true),
)
.unwrap();
assert_eq!(empty_list.get("foo"), Some(&Value::Array(vec![])));

let first = decode(
    "foo=bar&foo=baz",
    &DecodeOptions::new().with_duplicates(Duplicates::First),
)
.unwrap();
assert_eq!(first.get("foo"), Some(&Value::String("bar".to_owned())));

let comma = decode("a=b,c", &DecodeOptions::new().with_comma(true)).unwrap();
assert_eq!(
    comma.get("a"),
    Some(&Value::Array(vec![
        Value::String("b".to_owned()),
        Value::String("c".to_owned()),
    ])),
);

let scalars = decode("a=15&b=true&c=null", &DecodeOptions::new()).unwrap();
assert_eq!(scalars.get("a"), Some(&Value::String("15".to_owned())));
assert_eq!(scalars.get("b"), Some(&Value::String("true".to_owned())));
assert_eq!(scalars.get("c"), Some(&Value::String("null".to_owned())));
```

### Charset Sentinels, Numeric Entities, and Strict Null Handling

```rust
use qs_rust::{decode, Charset, DecodeOptions, Value};

let latin1 = decode(
    "a=%A7",
    &DecodeOptions::new().with_charset(Charset::Iso88591),
)
.unwrap();
assert_eq!(latin1.get("a"), Some(&Value::String("§".to_owned())));

let utf8_sentinel = decode(
    "utf8=%E2%9C%93&a=%C3%B8",
    &DecodeOptions::new()
        .with_charset(Charset::Iso88591)
        .with_charset_sentinel(true),
)
.unwrap();
assert_eq!(utf8_sentinel.get("a"), Some(&Value::String("ø".to_owned())));

let numeric_entities = decode(
    "a=%26%239786%3B",
    &DecodeOptions::new()
        .with_charset(Charset::Iso88591)
        .with_interpret_numeric_entities(true),
)
.unwrap();
assert_eq!(numeric_entities.get("a"), Some(&Value::String("☺".to_owned())));

let strict_null = decode(
    "a&b=",
    &DecodeOptions::new().with_strict_null_handling(true),
)
.unwrap();
assert_eq!(strict_null.get("a"), Some(&Value::Null));
assert_eq!(strict_null.get("b"), Some(&Value::String(String::new())));
```

### Structured Input With `decode_pairs`

```rust
use qs_rust::{decode_pairs, DecodeOptions, Value};

let decoded = decode_pairs(
    vec![
        ("a[b]".to_owned(), Value::String("1".to_owned())),
        ("a[b]".to_owned(), Value::String("2".to_owned())),
    ],
    &DecodeOptions::new(),
)
.unwrap();

assert_eq!(
    decoded.get("a"),
    Some(&Value::Object([(
        "b".to_owned(),
        Value::Array(vec![
            Value::String("1".to_owned()),
            Value::String("2".to_owned()),
        ]),
    )]
    .into())),
);
```

`decode_pairs` starts at the structured merge pipeline and intentionally bypasses raw query-string behaviors such as delimiter splitting, query-prefix stripping, charset sentinel detection, and numeric-entity interpretation.

## Encoding

### Basics and Nested Objects

```rust
use qs_rust::{encode, EncodeOptions, Value};

let simple = Value::Object([("a".to_owned(), Value::String("b".to_owned()))].into());
assert_eq!(encode(&simple, &EncodeOptions::new()).unwrap(), "a=b");

let nested = Value::Object(
    [(
        "a".to_owned(),
        Value::Object([("b".to_owned(), Value::String("c".to_owned()))].into()),
    )]
    .into(),
);
assert_eq!(encode(&nested, &EncodeOptions::new()).unwrap(), "a%5Bb%5D=c");
assert_eq!(
    encode(&nested, &EncodeOptions::new().with_encode(false)).unwrap(),
    "a[b]=c"
);
```

### List Formats

```rust
use qs_rust::{encode, EncodeOptions, ListFormat, Value};

let data = Value::Object(
    [(
        "a".to_owned(),
        Value::Array(vec![
            Value::String("b".to_owned()),
            Value::String("c".to_owned()),
        ]),
    )]
    .into(),
);

assert_eq!(
    encode(&data, &EncodeOptions::new().with_encode(false)).unwrap(),
    "a[0]=b&a[1]=c"
);
assert_eq!(
    encode(
        &data,
        &EncodeOptions::new()
            .with_encode(false)
            .with_list_format(ListFormat::Brackets),
    )
    .unwrap(),
    "a[]=b&a[]=c"
);
assert_eq!(
    encode(
        &data,
        &EncodeOptions::new()
            .with_encode(false)
            .with_list_format(ListFormat::Repeat),
    )
    .unwrap(),
    "a=b&a=c"
);
assert_eq!(
    encode(
        &data,
        &EncodeOptions::new()
            .with_encode(false)
            .with_list_format(ListFormat::Comma),
    )
    .unwrap(),
    "a=b,c"
);
```

### Dot Notation, Empty Lists, Prefixes, and Delimiters

```rust
use qs_rust::{encode, EncodeOptions, Value};

let dotted = Value::Object(
    [(
        "a".to_owned(),
        Value::Object(
            [(
                "b".to_owned(),
                Value::Object([("c".to_owned(), Value::String("d".to_owned()))].into()),
            )]
            .into(),
        ),
    )]
    .into(),
);
assert_eq!(
    encode(
        &dotted,
        &EncodeOptions::new()
            .with_encode(false)
            .with_allow_dots(true),
    )
    .unwrap(),
    "a.b.c=d"
);

let encoded_dot_key = Value::Object(
    [(
        "name.obj".to_owned(),
        Value::Object(
            [
                ("first".to_owned(), Value::String("John".to_owned())),
                ("last".to_owned(), Value::String("Doe".to_owned())),
            ]
            .into(),
        ),
    )]
    .into(),
);
assert_eq!(
    encode(
        &encoded_dot_key,
        &EncodeOptions::new()
            .with_allow_dots(true)
            .with_encode_dot_in_keys(true),
    )
    .unwrap(),
    "name%252Eobj.first=John&name%252Eobj.last=Doe"
);

let empty_list = Value::Object(
    [
        ("foo".to_owned(), Value::Array(vec![])),
        ("bar".to_owned(), Value::String("baz".to_owned())),
    ]
    .into(),
);
assert_eq!(
    encode(
        &empty_list,
        &EncodeOptions::new()
            .with_encode(false)
            .with_allow_empty_lists(true),
    )
    .unwrap(),
    "foo[]&bar=baz"
);

let prefixed = Value::Object(
    [
        ("a".to_owned(), Value::String("b".to_owned())),
        ("c".to_owned(), Value::String("d".to_owned())),
    ]
    .into(),
);
assert_eq!(
    encode(&prefixed, &EncodeOptions::new().with_add_query_prefix(true)).unwrap(),
    "?a=b&c=d"
);
assert_eq!(
    encode(&prefixed, &EncodeOptions::new().with_delimiter(";")).unwrap(),
    "a=b;c=d"
);
```

### Nulls, Bytes, Charset Sentinels, and RFC 1738 Formatting

```rust
use qs_rust::{decode, encode, Charset, DecodeOptions, EncodeOptions, Format, Value};

let with_nulls = Value::Object(
    [
        ("a".to_owned(), Value::Null),
        ("b".to_owned(), Value::String(String::new())),
    ]
    .into(),
);
assert_eq!(encode(&with_nulls, &EncodeOptions::new()).unwrap(), "a=&b=");
assert_eq!(
    encode(
        &with_nulls,
        &EncodeOptions::new().with_strict_null_handling(true),
    )
    .unwrap(),
    "a&b="
);

let skip_nulls = Value::Object(
    [
        ("a".to_owned(), Value::String("b".to_owned())),
        ("c".to_owned(), Value::Null),
    ]
    .into(),
);
assert_eq!(
    encode(&skip_nulls, &EncodeOptions::new().with_skip_nulls(true)).unwrap(),
    "a=b"
);

let bytes = Value::Object([("data".to_owned(), Value::Bytes(vec![0x41, 0x20, 0xFF]))].into());
assert_eq!(
    encode(
        &bytes,
        &EncodeOptions::new().with_charset(Charset::Iso88591),
    )
    .unwrap(),
    "data=A%20%FF"
);

let latin1 = Value::Object([("æ".to_owned(), Value::String("æ".to_owned()))].into());
assert_eq!(
    encode(
        &latin1,
        &EncodeOptions::new().with_charset(Charset::Iso88591),
    )
    .unwrap(),
    "%E6=%E6"
);

let sentinel = Value::Object([("a".to_owned(), Value::String("☺".to_owned()))].into());
assert_eq!(
    encode(&sentinel, &EncodeOptions::new().with_charset_sentinel(true)).unwrap(),
    "utf8=%E2%9C%93&a=%E2%98%BA"
);

let rfc1738 = Value::Object([("a".to_owned(), Value::String("b c".to_owned()))].into());
assert_eq!(encode(&rfc1738, &EncodeOptions::new()).unwrap(), "a=b%20c");
assert_eq!(
    encode(&rfc1738, &EncodeOptions::new().with_format(Format::Rfc1738)).unwrap(),
    "a=b+c"
);

let round_trip = decode("a&b=", &DecodeOptions::new().with_strict_null_handling(true)).unwrap();
assert_eq!(round_trip.get("a"), Some(&Value::Null));
assert_eq!(round_trip.get("b"), Some(&Value::String(String::new())));
```

## Customization

The sibling ports expose callback-heavy surfaces. In Rust those are available through explicit, typed hooks.
Rust does not expose a standalone public `Undefined` value; the sibling omission behavior is represented by
`FilterResult::Omit` in encode callbacks.

The callback-free convenience layer is also part of the public encode surface:

- `EncodeOptions::with_whitelist(...)` uses `WhitelistSelector::{Key, Index}` for key/index selection
- `EncodeOptions::with_sort(...)` uses `SortMode::{Preserve, LexicographicAsc}` for built-in ordering
- `Value::Object` uses the public `Object` alias, which is an ordered `IndexMap<String, Value>`

### Custom Decode, Filter, Sort, and Encode Hooks

`EncodeTokenEncoder` receives explicit `EncodeToken::{Key, Value, TextValue}` variants so Rust callers can distinguish key-path tokens from normal values and joined comma-list text.

```rust
use qs_rust::{
    decode, encode, DecodeDecoder, DecodeKind, DecodeOptions, EncodeFilter, EncodeOptions,
    EncodeToken, EncodeTokenEncoder, FilterResult, FunctionFilter, Sorter, Value,
};

let decode_options = DecodeOptions::new().with_decoder(Some(DecodeDecoder::new(
    |raw, _charset, kind| match kind {
        DecodeKind::Key => raw.to_owned(),
        DecodeKind::Value => raw.to_ascii_uppercase(),
    },
)));
let decoded = decode("a=hello", &decode_options).unwrap();
assert_eq!(decoded.get("a"), Some(&Value::String("HELLO".to_owned())));

let filtered = Value::Object(
    [
        ("b".to_owned(), Value::String("2".to_owned())),
        ("secret".to_owned(), Value::String("x".to_owned())),
        ("a".to_owned(), Value::String("1".to_owned())),
    ]
    .into(),
);
let encoded = encode(
    &filtered,
    &EncodeOptions::new()
        .with_encode(false)
        .with_filter(Some(EncodeFilter::Function(FunctionFilter::new(
            |prefix, _| {
                if prefix.ends_with("secret") {
                    FilterResult::Omit
                } else {
                    FilterResult::Keep
                }
            },
        ))))
        .with_sorter(Some(Sorter::new(|left, right| left.cmp(right)))),
)
.unwrap();
assert_eq!(encoded, "a=1&b=2");

let numbers = Value::Object(
    [
        ("b".to_owned(), Value::I64(2)),
        ("a".to_owned(), Value::I64(1)),
    ]
    .into(),
);
let encoded_numbers = encode(
    &numbers,
    &EncodeOptions::new()
        .with_encode(false)
        .with_encoder(Some(EncodeTokenEncoder::new(|token, _, _| match token {
            EncodeToken::Key(key) => key.to_owned(),
            EncodeToken::Value(Value::I64(number)) => format!("n:{number}"),
            EncodeToken::Value(Value::String(text)) => text.clone(),
            EncodeToken::TextValue(text) => text.to_owned(),
            EncodeToken::Value(_) => String::new(),
        })))
        .with_sorter(Some(Sorter::new(|left, right| right.cmp(left)))),
)
.unwrap();
assert_eq!(encoded_numbers, "b=n:2&a=n:1");
```

### Temporal Values

`qs_rust` now has a core temporal leaf:

- `Value::Temporal(TemporalValue)`

The default formatter emits canonical ISO-8601 datetime text:

- offset-aware values: `YYYY-MM-DDTHH:MM:SS[.fraction](Z|±HH:MM)`
- naive values: `YYYY-MM-DDTHH:MM:SS[.fraction]`

For custom temporal output, use the core serializer hook:

- `EncodeOptions::with_temporal_serializer(Some(TemporalSerializer::new(...)))`

Feature-gated adapters remain available for converting native runtime types into
that core temporal model:

- `chrono_support` behind the `chrono` feature
- `time_support` behind the `time` feature

Those helpers now produce `Value::Temporal(...)` directly, so temporal leaves can
live inside arbitrary nested arrays or objects without being pre-stringified.

### Serde Bridge and Errors

With the `serde` feature enabled, `from_str(...)`, `to_string(...)`,
`from_value(...)`, and `to_value(...)` all route typed data through the same
semantic core as the dynamic `Value` API.

That means plain query-string scalars arrive with the same semantics as `Value`: values such as `page=2` and `admin=true` decode as strings unless your serde model adds its own conversion layer.

Generic typed serde remains stringly for ordinary datetime-like fields too. If
you want typed models to preserve temporal leaves instead of collapsing them to
strings, use the opt-in helper modules under `qs_rust::serde::temporal::*`.

For a runnable typed example, use:

```bash
cargo run --example serde_bridge --features serde
```

Compared with `serde_qs`, `qs_rust` keeps the dynamic `qs` semantic core and
layers serde on top of it. For validated overlap cases, intentional
divergences such as stringly scalar decode and duplicate-key handling, and
`serde_qs`-only extras that are out of scope for this bridge, see
[docs/serde_comparison.md](https://github.com/techouse/qs_rust/blob/main/docs/serde_comparison.md).

`DecodeError` and `EncodeError` are `#[non_exhaustive]`. Match them with a catch-all arm and prefer the stable inspector helpers (`is_*`, `*_limit()`) when you need durable error introspection.

## Testing and Parity

The repository includes two Node-backed comparison layers:

- `tests/comparison.rs` runs the checked-in smoke corpus from `tests/comparison/test_cases.json`
- typed parity suites shell out to Node `qs` for per-case comparisons

Before running the Node-backed tests, bootstrap the fixture environment:

```bash
cd tests/comparison/js
npm ci
```

The checked-in `package-lock.json` pins `qs` to `6.15.0`.

Rust-specific behavior lives alongside that parity layer:

- `tests/regressions.rs` covers `decode_pairs`, `Bytes`, serde boundaries, deep stack-safety, and sibling-port-specific edge cases
- `tests/properties_*.rs` cover randomized encode/decode/round-trip invariants
- `tests/porting_ledger.md` records which Node/Python/Dart/Kotlin/C#/Swift cases were ported, skipped, or intentionally diverged

## Fuzzing

The repository also includes a local-only `cargo-fuzz` harness for hostile-input hardening of the three public entrypoints:

- `decode`
- `encode`
- `decode_pairs`

The fuzz targets are intentionally crash-focused for the first pass: successful results and clean `Err(...)` values are both acceptable. The goal is to catch panics, sanitizer failures, or obvious hang/regression cases on bounded inputs.

Install the tooling once:

```bash
rustup toolchain install nightly
cargo install cargo-fuzz
```

Build the fuzz targets:

```bash
cargo +nightly fuzz build decode
cargo +nightly fuzz build encode
cargo +nightly fuzz build decode_pairs
```

Run short local smoke sessions against a disposable copy of the committed corpus so libFuzzer does not spray generated inputs back into the tracked `fuzz/corpus/` tree:

```bash
tmpdir="$(mktemp -d /tmp/qs_rust_fuzz_decode.XXXXXX)"
cp -R fuzz/corpus/decode/. "$tmpdir"/
cargo +nightly fuzz run decode "$tmpdir" -- -max_total_time=60 -verbosity=0 -print_final_stats=1
rm -rf "$tmpdir"

tmpdir="$(mktemp -d /tmp/qs_rust_fuzz_encode.XXXXXX)"
cp -R fuzz/corpus/encode/. "$tmpdir"/
cargo +nightly fuzz run encode "$tmpdir" -- -max_total_time=60 -verbosity=0 -print_final_stats=1
rm -rf "$tmpdir"

tmpdir="$(mktemp -d /tmp/qs_rust_fuzz_decode_pairs.XXXXXX)"
cp -R fuzz/corpus/decode_pairs/. "$tmpdir"/
cargo +nightly fuzz run decode_pairs "$tmpdir" -- -max_total_time=60 -verbosity=0 -print_final_stats=1
rm -rf "$tmpdir"
```

Run a longer balanced soak with the checked-in helper script. By default it runs each target sequentially for `900` seconds, prints the exact command and temp paths it uses, and stops on the first non-zero exit:

```bash
./scripts/fuzz_soak.sh
```

The helper script keeps generated corpora and crash artifacts under a disposable `/tmp` root instead of the tracked `fuzz/corpus/` tree. Useful knobs:

```bash
# Shorter local sanity pass.
QS_FUZZ_SECONDS=60 ./scripts/fuzz_soak.sh

# Target subset.
QS_FUZZ_TARGETS="decode encode" ./scripts/fuzz_soak.sh

# Extra libFuzzer arguments, appended after the default balanced soak args.
QS_FUZZ_ARGS="-jobs=1 -workers=1" ./scripts/fuzz_soak.sh

# Remove the temporary /tmp work tree after a successful run.
QS_FUZZ_CLEANUP=1 ./scripts/fuzz_soak.sh
```

The default balanced soak takes about `45` minutes across all three targets. The committed corpora live under `fuzz/corpus/` and use small JSON envelopes so new seeds can be added directly from README examples, parity cases, and regressions. Generated crashes and coverage output stay local in ignored paths under `fuzz/artifacts/` and `fuzz/coverage/`; disposable working corpora should stay in `/tmp` or another untracked directory.

If fuzzing finds a real issue, minimize it first with `cargo +nightly fuzz tmin ...`, then promote the minimized reproducer into a normal checked-in regression test before considering the bug closed.

## Performance

The repository includes a local release-mode perf snapshot binary and checked-in baseline artifacts:

```bash
cargo run --release --bin qs_perf
cargo run --release --bin qs_perf -- --scenario encode --format json
cargo run --release --bin qs_perf -- --scenario decode --format json
python3 scripts/capture_perf_baselines.py --scenario all
python3 scripts/compare_perf_baseline.py --scenario all
python3 scripts/cross_port_perf.py
```

The harness, checked-in Rust baselines, and latest cross-port comparison snapshot all live in the repo now. Refresh those artifacts from a normal interactive shell when you want new numbers, and see [docs/performance.md](https://github.com/techouse/qs_rust/blob/main/docs/performance.md) for the trust-first capture workflow and failure-mode checks.

## Stability Policy

This repository now tracks the published `1.0.0` contract. The intended `1.x` contract is the current public surface re-exported from `src/lib.rs`; changes to that surface should stay semver-compatible and only correct clear contract bugs or add clearly intended behavior.

After `1.0.0`, changes should stay focused on bug fixes, test additions, documentation improvements, measurement-backed performance work, and additive features that keep the current `1.x` non-goals explicit.

- Node `qs` `6.15.0` remains the semantic baseline for shared public query-string behavior.
- C# remains the architectural reference for internal design decisions. Other sibling ports are informative, not normative.
- The semantic core is shared across the dynamic API, the typed option/enums, the callback wrappers, and the optional `serde` bridge (`from_str` / `to_string`).
- [docs/divergences.md](https://github.com/techouse/qs_rust/blob/main/docs/divergences.md) records the intentional `1.x` boundaries: host-object reflection, cycles, runtime bridge behavior, and other non-goals remain unsupported by design.
- [docs/python_backend_readiness.md](https://github.com/techouse/qs_rust/blob/main/docs/python_backend_readiness.md) defines how the future `qs_codec` native backend should consume this crate and how the Python suite should validate `pure`, `rust`, and `auto` backends.
- Merge, compact, finalization, and encode traversal are implemented iteratively to avoid recursion limits on deep inputs.

## Support Policy

- The crate-wide MSRV is Rust `1.88`.
- The support target for `1.x` is latest stable Rust plus the MSRV on Linux, macOS, and Windows.
- Optional features (`serde`, `chrono`, and `time`) follow the same support policy as the core crate. If a feature ever needs a newer compiler, the crate-wide MSRV should move with it instead of splitting policy.
