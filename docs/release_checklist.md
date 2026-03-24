# Release Checklist

Before cutting any later `1.x` release:

- bootstrap the Node-backed comparison environment with `cd tests/comparison/js && npm ci`
- run the feature-slice checks from CI:
  - `cargo test --locked`
  - `cargo test --locked --features serde`
  - `cargo test --locked --features chrono`
  - `cargo test --locked --features time`
  - `cargo test --locked --no-run --features "serde chrono"`
  - `cargo test --locked --no-run --features "serde time"`
  - `cargo test --locked --no-run --features "chrono time"`
- run `cargo test --all-features --locked`
- run `cargo test --doc --all-features --locked`
- run `cargo clippy --all-targets --all-features -- -D warnings`
- run `RUSTDOCFLAGS="-D warnings --cfg docsrs" cargo doc --no-deps --all-features --locked`
- keep the Node-backed parity suites green with `cargo test --locked --test comparison --test parity_decode --test parity_encode`
- run `cargo package --locked`
- run `cargo publish --dry-run --locked`
- keep the published crate consumer-lean: repo-only tooling, parity fixtures, perf artifacts, and fuzz infrastructure stay in the repository but out of the crates.io package
- verify the current Rust baselines with `python3 scripts/compare_perf_baseline.py --scenario all`
- if encode/decode logic changed since the last trustworthy perf capture, refresh the Rust baselines and cross-port snapshot from a normal interactive shell with `python3 scripts/capture_perf_baselines.py --scenario all` and `python3 scripts/cross_port_perf.py`
- if encode/decode logic changed since the last clean full fuzz soak, rerun `./scripts/fuzz_soak.sh` from a normal interactive shell; otherwise keep the last clean soak as supporting evidence
- confirm `perf/comparison/latest_snapshot.json` is truthful and [docs/performance_comparison.md](https://github.com/techouse/qs_rust/blob/main/docs/performance_comparison.md) matches it exactly
- keep the current non-goals explicit: host-object reflection, cycles, and language-runtime bridge behavior remain unsupported
