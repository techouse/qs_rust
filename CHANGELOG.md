## 1.0.3

* [FEAT] add `DecodeOptions::with_strict_merge` / `strict_merge` for Node `qs` `strictMerge` parity, defaulting to modern object/scalar array wrapping while supporting the legacy scalar-key merge mode
* [FIX] align decode duplicate handling with Node `qs` by combining bracket-array assignments under `Duplicates::First` and `Duplicates::Last` while preserving normal duplicate behavior for plain keys
* [FIX] align decode key parsing with Node `qs` 6.15.2, including depth-zero dot normalization, unterminated and nested bracket groups, literal bracket text, trailing text after closed brackets, and encoded-bracket cases
* [FIX] align stringify behavior with Node `qs` 6.15.2 for charset-sentinel delimiters, strict-null RFC1738 key-only formatting, and comma-array null handling
* [CHORE] update the Node comparison fixture to `qs` 6.15.2 and expand Node-backed parse/stringify parity coverage for the upstream 6.15.0-6.15.2 changes
* [DOCS] document the `qs` 6.15.2 semantic baseline, pnpm-based comparison fixture setup, bracket duplicate behavior, and `strict_merge` legacy-mode decode output

## 1.0.2

* [CHORE] update `rand` from 0.9.2 to 0.9.4
* [CHORE] bump `indexmap` from 2.13.1 to 2.14.0
* [CHORE] bump Node `qs` from 6.15.0 to 6.15.1 in

## 1.0.1

* [FIX] remove `serde` temporal helper warnings by cfg-gating imports that are only needed when `chrono` or `time` support is enabled
* [CHORE] expand decode, encode, temporal, and public-surface regression coverage, including WPT URL-encoded compatibility cases and broader internal helper tests
* [CHORE] move inline unit tests into dedicated `tests.rs` modules to keep production modules smaller and test coverage easier to maintain
* [CHORE] improve CI and release verification by deriving the MSRV from `Cargo.toml`, adding an aggregate required job, pinning the `dtolnay/rust-toolchain` action, and hardening package-list validation
* [CHORE] update dependencies and packaging metadata, including `codecov-action` v6, `indexmap` 2.13.1, removal of the unused `encoding_rs` dependency, and excluding `.codacy.yml` from the published crate
* [CHORE] refresh repository metadata and local tooling support with README badge/license cleanup, Codacy test-file exclusions, and macOS-specific ignore rules

## 1.0.0

Initial stable release.
