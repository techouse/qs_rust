# AGENTS.md

This repository expects agent work to be measurement-first and explicit about uncertainty.

## Working Style

- Give your opinion first before starting work when the user asks for a change, suggestion, or review-driven follow-up.
- Double-check instructions for logic errors, hidden assumptions, or cases where the requested action would produce misleading results.
- Prefer discussing or clarifying the task only when the ambiguity is real and cannot be resolved by inspecting the repo.
- Keep the current crate-wide MSRV (`1.88`) truthful across `Cargo.toml`, README, and CI whenever support policy changes.

## Harness Engineering Defaults

- Trust measurement before tuning.
- Do not optimize "because it seems hot" if the harness output is not trustworthy yet.
- Do not fabricate baselines, comparison snapshots, or perf summaries.
- If a capture is partial, failed, or blocked, write that state down explicitly instead of smoothing it over.
- Cross-port performance is informative only. It is not a CI gate and it is not a release gate.

## Canonical Perf Workflow

Rust-only workflow:

```bash
cargo test --all-features --no-run
cargo clippy --all-targets --all-features -- -D warnings
cargo run --release --bin qs_perf -- --scenario encode --format json
cargo run --release --bin qs_perf -- --scenario decode --format json
python3 scripts/capture_perf_baselines.py --scenario all
python3 scripts/compare_perf_baseline.py --scenario all
```

Cross-port workflow:

```bash
python3 scripts/cross_port_perf.py
```

Primary docs and artifacts:

- `docs/performance.md`
- `docs/performance_comparison.md`
- `perf/baselines/encode_deep_snapshot_baseline.json`
- `perf/baselines/decode_snapshot_baseline.json`
- `perf/comparison/latest_snapshot.json`

## Artifact Rules

- `perf/baselines/*.json` must contain either:
  - real captured numbers and capture metadata, or
  - a `pending` state with a concrete reason.
- `perf/comparison/latest_snapshot.json` must contain a truthful top-level status:
  - `complete`
  - `partial`
  - `pending`
- `docs/performance_comparison.md` must match the actual snapshot state. Do not write a success-looking Markdown summary if the JSON capture is pending or partial.
- If a parser cannot understand sibling output, record a parse failure explicitly. Do not silently drop that language.

## When A Perf Run Is Not Trustworthy

Treat the run as blocked, not successful, if any of these happen:

- `cargo` prints `Running ...` and the process never reaches useful output
- `cargo test -- --list` hangs
- direct binary launches like `./target/debug/qs_perf --help` hang
- the shell environment is known to interfere with local binaries or shims

When that happens:

1. Reproduce with the smallest command possible.
2. Distinguish repo bug vs environment issue before editing code.
3. If it is environment-specific, stop changing harness logic to "fix" it.
4. Update docs/artifacts to say capture must be done from a normal interactive host shell.

## Known Local Failure Mode To Recheck

Agent-executed shells have previously shown a local startup failure mode where Rust binaries can stall before `main`, even for commands such as:

- `cargo test -- --list`
- `cargo run --release --bin qs_perf -- --scenario encode --format json`
- `./target/debug/qs_perf --help`

If you observe that behavior again, do not treat any timing result from that environment as valid. Capture baselines and cross-port snapshots from a normal interactive shell on the host machine instead.

## Editing Harnesses And Scripts

- Prefer machine-readable JSON for Rust-owned outputs.
- Parse text output only for sibling ports that do not expose stable JSON already.
- The perf harness source now lives under `src/bin/qs_perf/` with `main.rs` as the façade entrypoint.
- Keep the snapshot scenario matrix aligned with the sibling repos:
  - encode depths `2000`, `5000`, `12000`
  - decode cases `C1`, `C2`, `C3`
- Preserve explicit timeout handling in perf scripts.
- Preserve explicit failure reporting for timeouts, non-zero exits, and parse failures.
- Do not make cross-port capture part of default CI.

## Examples And Docs

- Keep runnable examples under `examples/` aligned with the current public contract.
- Treat README examples, `examples/`, and rustdoc as user-facing surfaces that should agree on semantics.

## Python In Restricted Shells

- If the Python shim or bytecode cache path misbehaves in a restricted shell, prefer:

```bash
env PYTHONPYCACHEPREFIX=/tmp/pycache /usr/bin/python3 ...
```

- Do not "fix" the repo just because a local shim is broken.

## Before Declaring Perf Work Done

- Rust baseline JSON files are no longer `pending`.
- `scripts/compare_perf_baseline.py` succeeds against committed baselines.
- `perf/comparison/latest_snapshot.json` is real and current, or explicitly `partial`/`pending`.
- `docs/performance_comparison.md` matches the actual snapshot state.
- Any remaining blockers are documented as environment or tooling issues, not left implicit.
