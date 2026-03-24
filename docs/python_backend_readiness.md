# Python Backend Readiness

This note records how `qs_rust` is expected to serve as the frozen engine for a
future native backend in [`qs_codec`](https://github.com/techouse/qs_codec).

The goal is not to replace the Python package's contract. The goal is to let the
Python package use this crate as an optional accelerator while `qs.py` remains
the authority for Python-facing behavior.

## Backend Contract

- `qs.py` remains the source of truth for Python-visible semantics.
- The future native backend should target the current `1.0.0` Rust public
  surface: `Value`, `TemporalValue`, encode/decode options, callback contracts,
  and the typed serde/value bridge where relevant to Python conversion layers.
- The Rust backend is optional. Pure Python remains the canonical fallback.
- The user-facing default backend mode should be `auto`, meaning "use Rust when
  importable, otherwise use pure Python".

## Expected Backend Modes

The future Python package should expose an explicit, testable backend selector:

- `pure`: force the current Python implementation
- `rust`: force the native backend and fail clearly if the extension is absent
- `auto`: prefer the native backend and fall back to pure Python

Tests must force `pure` and `rust` explicitly. `auto` is a runtime convenience,
not the primary validation mode.

## Test Authority

Do not port the full Python suite into Rust.

The future validation model is:

- run Python public-contract tests against both `pure` and `rust`
- keep Python-internal/helper/cache tests pure-only
- add a small bridge-specific suite for Python↔Rust conversion behavior

This keeps one authoritative Python contract instead of two drifting test
universes.

## Future Python Test Split

### Run Twice: `pure` and `rust`

These are the tests that define Python-visible behavior and should pass against
both backends:

- `tests/unit/encode_test.py`
- `tests/unit/decode_test.py`
- `tests/unit/example_test.py`
- `tests/unit/fixed_qs_issues_test.py`
- `tests/e2e/e2e_test.py`
- `tests/comparison/*`
- the public encode/decode portions of `tests/unit/thread_safety_test.py`

### Pure-Only

These are Python-internal tests and should stay bound to the pure-Python
implementation unless the future backend intentionally exposes matching internals:

- helper tests that import underscored functions from `qs_codec.encode` or
  `qs_codec.decode`
- cache/model tests such as `encode_internal_helpers_test.py`,
  `key_path_node_test.py`, and `weakref_test.py`
- utility/helper tests that exercise Python-only support code

### Bridge-Specific

Add a small future suite specifically for the Python↔Rust boundary:

- backend selection and fallback (`pure`, `rust`, `auto`)
- Python value conversion into Rust `Value` / `TemporalValue`
- error mapping and exception surfaces
- datetime/temporal conversion behavior
- extension import failure behavior

## Non-Goals

- No promise that the Rust backend will reproduce Python-private helper APIs.
- No requirement to port all Python tests into Rust.
- No promise that language-runtime bridge behavior becomes part of the Rust
  crate's own public contract.
