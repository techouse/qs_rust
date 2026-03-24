# `qs_rust` vs `serde_qs` on the typed serde bridge

`qs_rust` and `serde_qs` both expose typed serde entrypoints, but they are not the
same kind of library.

- `qs_rust` is `qs`-semantics-first and keeps a dynamic [`Value`](../src/value.rs) core.
- `serde_qs` is a serde-first querystring library with a dedicated typed parser,
  serializer, helpers, and framework integrations.

This document only compares the overlapping typed bridge surface in `qs_rust`:
[`from_str`](../src/serde.rs), [`to_string`](../src/serde.rs),
[`from_value`](../src/serde.rs), and [`to_value`](../src/serde.rs).

## Validated overlap

The checked-in serde bridge comparison suite covers the typed-core cases that
`qs_rust` and `serde_qs` genuinely share:

- nested structs with bracket notation
- order-independent field decoding
- vector fields decoded from indexed and bracketed inputs
- `Option<T>` / `#[serde(default)]` / `#[serde(skip_serializing_if = ...)]`
  behavior on compatible shapes
- renamed fields, string-backed newtypes, and string-backed map values

Those cases live in `tests/serde_bridge_comparison.rs`.

## Intentional divergences

Some `serde_qs` behavior does not match `qs_rust`'s bridge contract by design.

- Plain query-string scalars stay stringly on decode. `qs_rust::from_str()` first
  decodes into the same dynamic `Value` tree used by `decode()`, so `page=2` and
  `admin=true` arrive as strings unless the serde side adds its own conversion layer.
- Duplicate-key handling is driven by [`DecodeOptions`](../src/options/decode.rs),
  not by the destination Rust type. With the default `Duplicates::Combine` policy,
  repeated scalar keys become arrays and may fail typed scalar deserialization.
- Generic typed serde remains stringly for datetime-like fields too. Preserving
  temporal leaves requires the explicit `qs_rust::serde::temporal::*` helper
  modules rather than implicit inference.
- `serde_qs::Config` is not a 1:1 equivalent of `DecodeOptions` or `EncodeOptions`.
  The two crates expose different configuration surfaces because they sit on
  different semantic cores.

These differences are covered by tests and are not treated as bugs unless they
contradict the current `qs_rust` serde documentation.

## Out of scope for this comparison

This tranche does not treat `serde_qs`-specific extras as parity targets:

- helper attributes such as comma-, pipe-, space-, or generic-delimited serde helpers
- actix, axum, or warp integrations
- the broader `serde_qs` typed-config surface beyond what cleanly maps to the
  current `qs_rust` bridge

If you want a serde-first typed querystring library with those extras,
`serde_qs` is likely the better fit. If you want `qs`-style semantics, the
dynamic `Value` API, and an optional typed bridge on top of that core,
`qs_rust` is the intended fit.
