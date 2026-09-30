# Node `qs` 6.15.3 → 6.16.0: Rust parity review

## Conclusion

The review identified two behavior updates needed to keep the supported Rust surface aligned with Node `qs` 6.16.0. Both are now implemented:

1. In strict limit mode, reject oversized comma groups even when assigned through `[]=`.
2. Apply `encode_dot_in_keys` to top-level primitive-valued keys as well as arrays and objects.

The upstream changes for stringify depth, filtered dates, and overflow-array appends are already represented in Rust. Two other fixes concern JavaScript object capabilities that the Rust `Value` model does not provide.

The Node-backed decode, encode, and comparison suites and a direct public-API smoke pass. The package manifest and lockfile both resolve `qs` 6.16.0.

## Behavior-by-behavior assessment

### 1. `[]=` comma-group list-limit bypass — implemented

Upstream now checks each comma-split group against `arrayLimit` before splitting whenever `throwOnLimitExceeded` is enabled, including `a[]=...`. The inner group still contributes one element to the outer `[]` list; it must also fit the configured strict threshold. The non-throwing path is unchanged. This closes the bracket-key bypass described in [GHSA-x5fp-wj9c-mxmx](https://github.com/ljharb/qs/security/advisories/GHSA-x5fp-wj9c-mxmx); the implementation change is [upstream commit 8859c37](https://github.com/ljharb/qs/commit/8859c37470e11b42b547b275e4e9bd0bc8cc5464).

Both comma builders in `src/decode/accumulate/build.rs` now check the inner group's element count in strict mode before allocating/splitting it. The flat-value flag and bracket-suffix exemption were removed. `build_direct_value` and `build_custom_value` still wrap the comma values in the outer list afterward. In custom-decoder mode, the error precedes calls to the value decoder.

Before the fix, the Node-backed parity suite reproduced the regression:

```text
rtk cargo test --test parity_decode typed_decode_parity_matches_node_qs -- --nocapture
```

That pre-fix run failed at `node-qs:parse.js [comma] bracketed comma group counts as one outer item`: Rust succeeded with six inner values at `list_limit = 1`, while Node 6.16.0 returned `list_limit_exceeded`.

Conflicting prior-bug characterizations were removed. Coverage now checks strict rejection, at-limit nesting, soft-mode preservation, encoded brackets and commas, nested paths, trailing empty items, and cumulative outer limits. Custom-decoder tests assert no value calls for oversized groups and ordered calls for a valid group. `src/options/decode.rs` now documents the strict inner-group threshold separately from the outer-list shape.

This is not a default allocation cap: `list_limit` is a representation threshold unless `throw_on_limit_exceeded` is set. A comma group is one query parameter, so `parameter_limit` does not bound the number of values created by splitting it. Preserve the existing non-throwing semantics; do not describe this fix as a universal comma-allocation cap. The upstream clarification is in [commit 68118da](https://github.com/ljharb/qs/commit/68118daba18722fe8b30b524ca9ddb1bd7e3f9c4).

### 2. Dotted top-level primitive keys — implemented

Upstream now encodes dots in a top-level key with a primitive value when `encodeDotInKeys` is set, fixing a stringify/parse round-trip loss ([commit 708faded](https://github.com/ljharb/qs/commit/708fadedc91d68788cd02dd24803eed3c18427df)). For example, Node emits `a%252Eb=c` for `{ 'a.b': 'c' }` with `allowDots: true, encodeDotInKeys: true`.

Root object keys now use `src/encode.rs::raw_key_component` for all value kinds; the obsolete container-only root helper was removed. Node-backed cases cover scalar and strict-null leaves, `encode_values_only`, and `encode = false`. Rust regressions cover literal-key round trips and function-filter prefixes. The supported `allow_dots = true` configuration is unchanged.

Scope note: `EncodeOptions::validate` rejects `encode_dot_in_keys = true` when `allow_dots = false`, while Node accepts that combination. The repository instructions explicitly constrain this option by `allow_dots`; that broader, pre-existing API difference is not caused by 6.16.0 and should not be changed silently as part of this update.

### 3. Stringify `depth` option — already represented

Node adds an optional `stringify` depth guard, defaulting to unlimited, and throws a deterministic error beyond the configured depth ([commit f3dc5b0](https://github.com/ljharb/qs/commit/f3dc5b026c944906b4d132bd6ae5485c0617c07d)). Rust already exposes `EncodeOptions::with_max_depth`; `None` is unlimited and the iterative encoder returns `EncodeError::DepthExceeded` when the limit is crossed. This is the Rust-typed equivalent; no new option or behavior change is needed.

### 4. Date serialization with a filter — already represented

Node now serializes Date values after a function filter has run ([commit 62fd254](https://github.com/ljharb/qs/commit/62fd25480b0b0d9c0a667ee67e13608a363f5d0e)). Rust applies the filter before matching the value in `src/encode/filter.rs`; `Value::Temporal` then reaches the scalar path, where `TemporalSerializer` is applied. Existing temporal/filter tests cover replacement and serialization. No implementation change is indicated.

### 5. Appending collections to an overflowed array — already represented

Node’s `combine` now spreads one appended collection level into successive numeric indices rather than nesting it under one index ([commit d56f48c](https://github.com/ljharb/qs/commit/d56f48ca137b1bf6385da749b1044246ae142f19)). Rust’s `combine_with_limit` flattens the next `Node::Array` before appending it, and `src/decode/tests/duplicates.rs::combine_with_limit_flattens_overflow_appends_and_respects_throw_on_limit_exceeded` covers this shape. No change is needed.

### 6. `isBuffer` guard and empty-array own properties/cycles — no Rust equivalent

The `isBuffer` fix prevents calling a truthy non-function `constructor.isBuffer` member; the upstream issue is documented in [GHSA-4mjr-xmp4-gh2g](https://github.com/ljharb/qs/security/advisories/GHSA-4mjr-xmp4-gh2g) and [commit e83d321](https://github.com/ljharb/qs/commit/e83d321ffafb38cf210683ac31714fce6ce1c6c6). Rust serializes its typed `Value` tree and never duck-types user objects as buffers. Likewise, Rust arrays are `Vec<Value>` entries with no own string properties, and the owned value tree cannot contain JavaScript-style cycles. These fixes do not require Rust behavior changes.

Upstream’s `allowEmptyArrays` fix also preserves own enumerable keys on an empty JavaScript array and still detects a cycle through those keys ([commit b433a9b](https://github.com/ljharb/qs/commit/b433a9b1633e1c3348aa53c513589a5bfe47f113)); those shapes are outside the Rust value model.

## Baseline and coverage maintenance

Current baseline references in README, divergence docs, comparison docs, the porting ledger, and repository instructions now use 6.16.0. CHANGELOG records the update under Unreleased and retains historical release entries. `qs_6_15_3_cases` remains named for its source version; new decode cases live in `qs_6_16_0_cases`.

## Primary sources

- [GitHub compare: v6.15.3…v6.16.0](https://github.com/ljharb/qs/compare/v6.15.3...v6.16.0)
- [6.16.0 changelog](https://github.com/ljharb/qs/blob/v6.16.0/CHANGELOG.md)
- [6.16.0 parser](https://github.com/ljharb/qs/blob/v6.16.0/lib/parse.js)
- [6.16.0 stringifier](https://github.com/ljharb/qs/blob/v6.16.0/lib/stringify.js)
- [6.16.0 utility functions](https://github.com/ljharb/qs/blob/v6.16.0/lib/utils.js)
- [6.16.0 stringify tests](https://github.com/ljharb/qs/blob/v6.16.0/test/stringify.js)
