Run `npm ci` in this directory before executing the Node comparison tests.

The lockfile is pinned to `qs` 6.15.0, and the Rust comparison harness shells out to
`node tests/comparison/js/qs.js`.
