# Copilot Project Instructions: qs_rust

Concise, project-specific guidance for AI coding agents working on this repo. Focus on preserving behavioral parity with Node `qs` while keeping the Rust port explicit, iterative, and measurement-first.

## 1. Project Purpose & Architecture
- Library: high-fidelity Rust port of the JavaScript `qs` query string encoder/decoder.
- Current crate-wide MSRV: Rust `1.88`.
- Public API is re-exported from `src/lib.rs`:
  - functions: `decode`, `decode_pairs`, `encode`
  - dynamic tree: `Value`, `Object`, `TemporalValue`, `DateTimeValue`
  - options/enums/hooks: `DecodeOptions`, `EncodeOptions`, `Charset`, `Format`, `ListFormat`, `Duplicates`, `SortMode`, `WhitelistSelector`, `DecodeDecoder`, `DecodeKind`, `EncodeFilter`, `FilterResult`, `FunctionFilter`, `EncodeToken`, `EncodeTokenEncoder`, `TemporalSerializer`
  - optional modules/features: `serde`, `chrono_support`, `time_support`
- Core implementation modules:
  - `src/decode.rs`: public decode façade; implementation lives under `src/decode/`
  - `src/encode.rs`: public encode façade; implementation lives under `src/encode/`
  - `src/merge.rs` and `src/compact.rs`: shared structural normalization and overflow/list handling
  - `src/structured_scan.rs` and `src/key_path.rs`: internal scanners/builders for hot paths
  - `src/options.rs`: public options façade; concrete option/callback types live under `src/options/`
- Internal tooling:
  - `src/bin/qs_perf/main.rs`: perf harness façade; implementation lives under `src/bin/qs_perf/`
- Onboarding examples:
  - `examples/`: runnable usage examples for the dynamic API, options, and the feature-gated serde bridge

## 2. Key Behavioral Invariants
- Treat Node `qs` `6.15.0` as the default parity baseline unless an intentional Rust divergence is documented in `docs/divergences.md`.
- Query-string `decode` only produces `Null`, `String`, `Array`, and `Object`. It must not infer booleans, numbers, or `Bytes`.
- `Bytes` are accepted by `encode` and `decode_pairs`, but raw query decoding never produces `Bytes`.
- `decode_pairs` is intentionally different from `decode`: it starts at the structured merge pipeline and bypasses delimiter parsing, query-prefix stripping, charset sentinel detection, and numeric-entity interpretation.
- Object insertion order is preserved unless `SortMode::LexicographicAsc` is selected.
- Depth, list, and parameter limits are safety features. Preserve `depth`, `strict_depth`, `list_limit`, `parameter_limit`, and their error behavior exactly when touching decode/encode internals.
- Keep traversal iterative. Deep decode/merge/compact/encode behavior should not rely on recursion.
- Public hook surfaces are explicit and typed. Do not reintroduce JS-style implicit host-object behavior, `Undefined`, implicit host-object date detection, or runtime-reflection semantics around the `Value::Temporal` core model.

## 3. Conventions & Rust Patterns
- Prefer borrowing over cloning, especially in hot encode/decode paths.
- Keep public options opaque and builder-style. Do not add public struct fields or encourage struct literals.
- Use typed errors and preserve the current `thiserror`-based surface in `src/error.rs`; avoid panic-style handling in public paths.
- Avoid magic strings when an enum or dedicated helper already exists.
- Preserve `IndexMap`-backed deterministic ordering semantics through `Object`.
- Keep code ASCII unless the file already requires Unicode.
- Comments should be sparse and high-signal; prefer tests and small helper names over explanatory noise.

## 4. Developer Workflow
- Rust correctness:
  - `cargo test --all-features`
  - `cargo clippy --all-targets --all-features -- -D warnings`
- Runnable examples:
  - `cargo run --example introduction`
  - `cargo run --example options`
  - `cargo run --example serde_bridge --features serde`
- Node-backed parity bootstrap:
  - `cd tests/comparison/js && npm ci`
- Perf workflow:
  - `cargo run --release --bin qs_perf -- --scenario encode --format json`
  - `cargo run --release --bin qs_perf -- --scenario decode --format json`
  - `python3 scripts/capture_perf_baselines.py --scenario all`
  - `python3 scripts/compare_perf_baseline.py --scenario all`
  - `python3 scripts/cross_port_perf.py`
- Treat cross-port performance as informative only. It is not a CI or release gate.

## 5. Adding / Modifying Features
- Update the relevant module under `src/options/` first when changing public behavior. Keep `src/options.rs` as the façade and keep defaults, builder methods, and getters aligned.
- If you alter shared semantics, update all three:
  - README examples
  - tests
  - `docs/divergences.md` when the change is an intentional Node-vs-port policy choice
- Keep merge/overflow behavior inside `src/merge.rs` and compaction behavior inside `src/compact.rs`; do not duplicate those rules ad hoc in decode/encode.
- For new sibling-port-style hooks, prefer explicit Rust wrappers (`FunctionFilter`, `TemporalSerializer`, etc.) over ad hoc closures embedded into unrelated structs.
- Do not change `decode_pairs` semantics just to match raw query-string behavior.

## 6. Testing Strategy
- Node-backed parity is the primary correctness oracle for shared public semantics:
  - `tests/comparison.rs`
  - `tests/parity_decode.rs`
  - `tests/parity_encode.rs`
- Rust-specific behavior belongs in:
  - `tests/regressions.rs`
  - module-local unit tests in `src/*`
- Runnable examples under `examples/` are part of the onboarding surface; keep them accurate and executable.
- Property tests live in:
  - `tests/properties_decode.rs`
  - `tests/properties_encode.rs`
  - `tests/properties_roundtrip.rs`
- Keep README examples and doctests valid. They are part of the public contract.
- When importing behavior from Python/Dart/Kotlin/C#/Swift, keep `tests/porting_ledger.md` aligned with what was ported, skipped, or intentionally diverged.

## 7. Performance & Measurement Rules
- Trust measurement before tuning.
- Do not optimize because a path "looks hot" if `qs_perf` or the capture environment is suspect.
- Keep the benchmark matrix stable unless there is a deliberate project-wide decision:
  - encode depths `2000`, `5000`, `12000`
  - decode cases `C1`, `C2`, `C3`
- Prefer machine-readable JSON for Rust-owned outputs.
- If a cross-port parser fails, record it explicitly; do not silently drop a language/runtime from the snapshot.
- If local shells show the old startup failure mode (`cargo test -- --list` or `qs_perf` hanging before useful output), treat timing as blocked and update docs/artifacts truthfully instead of "fixing" the repo around a broken shell.

## 8. Common Pitfalls To Avoid
- Accidentally changing default Node-compatible behavior when a sibling-port extension should remain opt-in.
- Reintroducing recursion or unnecessary cloning in deep encode/decode paths.
- Forgetting that `decode_dot_in_keys` and `encode_dot_in_keys` are constrained by `allow_dots`.
- Treating `decode_pairs` like raw query parsing.
- Writing perf summaries or baseline files that smooth over a partial or failed capture.
- Making cross-port benchmarking part of the default CI workflow.

## 9. When Unsure
- Check `README.md` first; crate docs are sourced from it.
- Check `docs/divergences.md` before changing behavior that might intentionally differ from Node.
- Check `tests/parity_*`, `tests/regressions.rs`, and `tests/porting_ledger.md` before refactoring semantics.
- Prefer preserving public signatures and builder patterns over introducing new surface area casually.

---
If any instruction here conflicts with measured behavior, tests, or documented divergence policy, follow the measured/tested behavior and update the docs explicitly.
