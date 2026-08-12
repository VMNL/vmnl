# Validation

Prerequisites and exact versions are listed in [`docs/build.md`](../../build.md). Documentation-only edits require structure, generated-file, snippet, and local-link checks. Source or tooling edits require the repository completion sequence.

Run:

```bash
cargo build --workspace --all-targets
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
just doctest
just docs
just docs-api-check
just test-unit
just test-api
just test-smoke
```

The order is intentional. GPU execution and human visual validation are required only when GPU-facing behavior changes; documentation compilation is not runtime or visual evidence.
