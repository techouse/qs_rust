---
name: Bug report
about: Report incorrect parsing/stringifying behavior, a regression, or a performance issue.
title: ""
labels: bug
assignees: techouse
---

<!--
    qs_rust is a port of Node qs with a few explicit sibling-port extensions and
    divergences. Before filing:

    - check upstream qs issues: https://github.com/ljharb/qs/issues
    - check documented Rust divergence decisions:
      https://github.com/techouse/qs_rust/blob/master/docs/divergences.md

    If you find a relevant upstream or sibling-port issue, please link it below.
-->

## Problem Summary

<!--
Describe the bug clearly:
- what input/value/options you used
- what you expected
- what happened instead
-->

## Reproduction

<!--
Please include a minimal reproducible example. Prefer a tiny Rust snippet that can
be pasted into a test or `cargo run`.
-->

```rust
use qs_rust::{decode, encode, DecodeOptions, EncodeOptions, Value};

fn main() {
    // Minimal repro here
}
```

## Expected Behavior

<!-- What should have happened? -->

## Actual Behavior

<!-- What happened instead? Include the exact output or error when possible. -->

## Inputs And Options

<!--
Fill in whichever parts apply.
-->

- Query string:
- Structured input / `Value` payload:
- `DecodeOptions`:
- `EncodeOptions`:
- Relevant feature flags: `serde`, `chrono`, `time`, or none

## Parity Context

<!--
If this is a parity issue, please say what you compared against.
-->

- [ ] Matches upstream Node `qs`
- [ ] Matches another port
- [ ] This may be an intentional Rust divergence

Relevant links:

- Upstream Node `qs` issue or behavior:
- Sibling-port reference:
- `docs/divergences.md` entry, if applicable:

## Performance Impact

<!-- Delete this section if not relevant. -->

- [ ] This is a correctness bug
- [ ] This is a performance regression

If this is a perf issue, include the command(s) and results you used:

```bash
cargo run --release --bin qs_perf -- --scenario decode --format json
python3 scripts/compare_perf_baseline.py --scenario all
python3 scripts/cross_port_perf.py
```

Observed numbers or summary:

```text
```

## Environment

<!-- Please provide the actual outputs when possible. -->

```bash
rustc --version
cargo --version
uname -a
```

```text
```

## Additional Context

<!--
Add logs, screenshots, comparison output, or anything else that helps reproduce or
classify the issue.
-->
