# Divergence Matrix

`qs_rust` still uses Node `qs` `6.15.1` as the baseline for shared public query-string semantics, but it now tracks sibling-port behavior explicitly in three buckets:

- `shared-port default`: Rust adopts the sibling-port extension or fix directly.
- `Node-compatible default`: Rust intentionally keeps Node behavior even when another port diverges.
- `unsupported in Rust`: the behavior depends on host-object/runtime features outside the Rust public API.

## Matrix

| Case | Classification | Rust status | Coverage |
| --- | --- | --- | --- |
| Parameter counting includes skipped charset sentinel parameters and empty-key pairs | Node-compatible default | Kept | `tests/divergences.rs`, `tests/parity_decode.rs` |
| Top-level dotted keys remain raw at `depth = 0` even with `allow_dots = true` | Node-compatible default | Kept | `tests/divergences.rs`, `tests/porting_ledger.md` |
| Negative or infinite limits (`depth`, `listLimit`, `parameterLimit`) from dynamic ports | Unsupported in Rust | Rejected at the type level (`usize`) | `tests/porting_ledger.md` |
| Prototype/plain-object/null-prototype host behavior from Node | Unsupported in Rust | Not modeled | `tests/porting_ledger.md` |
| Arbitrary host-object graphs, cycles, reflection-heavy map/object coercions | Unsupported in Rust | Not modeled | `tests/porting_ledger.md` |
| Encode-side function filtering without a public `Undefined` value | Shared-port default | Added as `EncodeFilter::Function` + `FilterResult` | `tests/divergences.rs`, `src/encode/tests/filters.rs` |
| Key-aware custom query decoding | Shared-port default | Added as `DecodeDecoder` + `DecodeKind` | `tests/divergences.rs`, `src/decode/tests/flat.rs` |
| Encode-side custom sorting | Shared-port default | Added as `Sorter`; `SortMode` remains the non-callback convenience layer | `src/encode/tests/helpers.rs` |
| Encode-side custom key/value token encoding | Shared-port default | Added as `EncodeTokenEncoder` + `EncodeToken` | `src/encode/tests/helpers.rs` |
| Comma-list null compaction | Shared-port default | Added as `comma_compact_nulls` opt-in behavior | `tests/divergences.rs`, `src/encode/tests/helpers.rs` |
| Encode depth guard for iterative traversal | Shared-port default | Added as `max_depth` with stable `EncodeError::DepthExceeded` | `src/encode/tests/iterative.rs`, `tests/regressions.rs` |
| Core temporal leaves plus native feature-adapter conversions | Shared-port default | Added as `Value::Temporal`, `TemporalValue`, `TemporalSerializer`, and `chrono_support` / `time_support` conversion helpers | `src/temporal.rs`, `src/chrono_support.rs`, `src/time_support.rs`, `src/serde.rs` |

## Notes

- For `1.x`, Node `qs` `6.15.1` remains the semantic baseline for shared public behavior, while the C# port remains the architectural reference for internal design choices. Other sibling ports are informative only.
- The public contract for `1.x` is the re-exported surface from `src/lib.rs` together with the intentional boundaries recorded in this matrix and the support/stability policy in `README.md`.
- Optional features (`serde`, `chrono`, and `time`) follow the same MSRV and platform support policy as the core crate.
- Rust does not silently inherit every sibling-port divergence. When sibling ports disagree and there is no clear shared correction, the crate stays Node-compatible by default.
- Swift-derived encode/decode edge cases are imported selectively:
  - portable public query-string behavior goes into the Node-backed parity suites
  - sentinel/numeric-entity/key-protection and runtime-bridge hardening can stay in local Rust tests when the same behavior is already covered there
- The callback/customization surface is intentionally Rust-specific:
  - no public `Undefined`
  - no arbitrary host-object reflection
  - no cyclic public value graphs
- `decode_pairs` remains a structured-input helper and intentionally bypasses raw query-string callback behavior such as delimiter tokenization, charset sentinel detection, and numeric-entity interpretation.
