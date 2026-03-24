.DEFAULT_GOAL := help

CARGO ?= cargo
CARGO_MSRV ?= cargo +1.88.0
NPM ?= npm
PYTHON ?= env PYTHONPYCACHEPREFIX=/tmp/pycache /usr/bin/python3
RUSTDOCFLAGS_DOCS ?= -D warnings --cfg docsrs
PACKAGE_LIST ?= /tmp/qs_rust-package-list.txt

.PHONY: help build build-release clean fmt fmt-check clippy test test-all test-doc test-props \
	feature-matrix quality node-bootstrap parity msrv package-list package-check docs docs-pages \
	publish-dry-run ci release-check perf-compare perf-capture perf-cross-port fuzz-soak

help: ## Show available targets
	@awk 'BEGIN {FS = ":.*## "}; /^[a-zA-Z0-9_.-]+:.*## / {printf "%-18s %s\n", $$1, $$2}' $(MAKEFILE_LIST) | sort

build: ## Build the crate
	$(CARGO) build --locked

build-release: ## Build the crate in release mode
	$(CARGO) build --release --locked

clean: ## Remove Cargo build artifacts
	$(CARGO) clean

fmt: ## Format Rust sources
	$(CARGO) fmt --all

fmt-check: ## Check Rust formatting
	$(CARGO) fmt --all --check

clippy: ## Run clippy with CI warning policy
	$(CARGO) clippy --all-targets --all-features -- -D warnings

test: ## Run default-feature tests
	$(CARGO) test --locked

test-all: ## Run all-feature tests
	$(CARGO) test --all-features --locked

test-doc: ## Run documentation tests
	$(CARGO) test --doc --all-features --locked

test-props: ## Run the proptest-backed test targets
	$(CARGO) test --locked --test properties_decode --test properties_encode --test properties_roundtrip

feature-matrix: ## Run the feature-matrix checks from CI
	$(CARGO) test --locked
	$(CARGO) test --locked --features serde
	$(CARGO) test --locked --features chrono
	$(CARGO) test --locked --features time
	$(CARGO) test --locked --no-run --features "serde chrono"
	$(CARGO) test --locked --no-run --features "serde time"
	$(CARGO) test --locked --no-run --features "chrono time"

quality: ## Run formatting, clippy, and doc-test checks
	$(MAKE) fmt-check
	$(MAKE) clippy
	$(MAKE) test-doc

node-bootstrap: ## Install the Node fixture dependencies for parity tests
	$(NPM) --prefix tests/comparison/js ci

parity: ## Run the Node-backed parity tests (requires `make node-bootstrap`)
	$(CARGO) test --locked --test comparison --test parity_decode --test parity_encode

msrv: ## Run all-feature tests on the crate MSRV (requires toolchain 1.88.0)
	$(CARGO_MSRV) test --all-features --locked

package-list: ## List files included in the published crate package
	$(CARGO) package --locked --list > $(PACKAGE_LIST)
	@cat $(PACKAGE_LIST)

package-check: ## Run the packaging checks used by CI
	$(CARGO) package --locked --list > $(PACKAGE_LIST)
	! grep -E '^(fuzz/|tests/|scripts/|perf/|\.github/|\.vscode/|\.gitignore$$|AGENTS\.md|pyproject\.toml$$|src/bin/|src/.*/tests(/|\.rs$$))' $(PACKAGE_LIST)
	$(CARGO) package --locked

docs: ## Build library docs with docs.rs warning settings
	RUSTDOCFLAGS='$(RUSTDOCFLAGS_DOCS)' $(CARGO) doc --locked --no-deps --all-features --lib

docs-pages: ## Build docs and add a root redirect page for GitHub Pages
	$(MAKE) docs
	@printf '%s\n' \
		'<!doctype html>' \
		'<meta charset="utf-8">' \
		'<meta http-equiv="refresh" content="0; url=./qs_rust/index.html">' \
		'<link rel="canonical" href="./qs_rust/index.html">' \
		'<title>qs_rust docs</title>' \
		'<a href="./qs_rust/index.html">Open qs_rust docs</a>' \
		> target/doc/index.html

publish-dry-run: ## Verify crates.io packaging without uploading
	$(CARGO) publish --dry-run --locked

ci: ## Run the main local CI checks except MSRV
	$(MAKE) test-all
	$(MAKE) feature-matrix
	$(MAKE) quality
	$(MAKE) package-check

release-check: ## Run release-oriented local checks, including parity and publish dry-run
	$(MAKE) ci
	$(MAKE) parity
	$(MAKE) publish-dry-run

perf-compare: ## Compare committed Rust perf baselines
	$(PYTHON) scripts/compare_perf_baseline.py --scenario all

perf-capture: ## Refresh Rust perf baselines from a normal interactive shell
	$(PYTHON) scripts/capture_perf_baselines.py --scenario all

perf-cross-port: ## Capture the cross-port performance snapshot from a normal interactive shell
	$(PYTHON) scripts/cross_port_perf.py

fuzz-soak: ## Run the fuzz soak script from a normal interactive shell
	bash scripts/fuzz_soak.sh
