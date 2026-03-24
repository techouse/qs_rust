# Performance Workflow

`qs_rust` now includes the same high-level snapshot scenarios used by the sibling ports:

- Encode deep snapshot:
  - depths: `2000`, `5000`, `12000`
  - iterations: `20`, `20`, `8`
- Decode snapshot:
  - `C1`: `count=100`, `comma=false`, `utf8=false`, `value_len=8`, `iterations=120`
  - `C2`: `count=1000`, `comma=false`, `utf8=false`, `value_len=40`, `iterations=16`
  - `C3`: `count=1000`, `comma=true`, `utf8=true`, `value_len=40`, `iterations=16`

The harness lives in [src/bin/qs_perf/main.rs](../src/bin/qs_perf/main.rs).

## Run locally

```bash
cargo run --release --bin qs_perf
cargo run --release --bin qs_perf -- --scenario encode --format json
```

Useful flags:

- `--warmups N`
- `--samples N`
- `--max-encode-depth N`
- `--output /path/to/file`

## Baselines

Baseline file locations:

- [perf/baselines/encode_deep_snapshot_baseline.json](../perf/baselines/encode_deep_snapshot_baseline.json)
- [perf/baselines/decode_snapshot_baseline.json](../perf/baselines/decode_snapshot_baseline.json)

These files now contain real captured measurements and serve as the checked-in Rust self-baselines for the current harness shape. Refresh them only from a normal interactive host shell after confirming that `qs_perf` starts cleanly and emits stable JSON.

If the old local startup failure mode reappears in an agent-driven shell, treat it as an execution-environment problem rather than a trustworthy perf result. In particular, if `cargo test -- --list`, `cargo run --release --bin qs_perf ...`, or a direct launch like `./target/debug/qs_perf --help` stalls after Cargo prints `Running ...`, do not recapture baselines from that environment.

## Compare command

```bash
python3 scripts/capture_perf_baselines.py --scenario all
python3 scripts/compare_perf_baseline.py --scenario all
```

If a baseline file is still marked `pending`, the compare script exits with a clear message instead of pretending there is a meaningful threshold to enforce. If `qs_perf` times out, the compare script now exits with an explicit timeout error instead of hanging indefinitely.

## Cross-port snapshots

Use the local orchestration script to run the Rust, Python, Dart, Kotlin, Swift, and C# snapshot harnesses and normalize the output into checked-in artifacts:

```bash
python3 scripts/cross_port_perf.py
```

Artifact locations:

- [perf/comparison/latest_snapshot.json](../perf/comparison/latest_snapshot.json)
- [docs/performance_comparison.md](./performance_comparison.md)

The checked-in snapshot is informative only and should be refreshed on the same machine when you want a new comparison point. If any sibling harness fails, times out, or changes output format, prefer an explicitly `partial` snapshot over silently dropping that language.
