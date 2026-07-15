# Deployment

## crates.io release

Required secret:

- `CARGO_REGISTRY_TOKEN`: crates.io API token with publish permission.

## Current Status

Automatic crates.io publishing is disabled in `.github/workflows/release.yml`.

`vmnl-graphics` depends on `vmnl-macros` through a path dependency, and `vmnl-macros` is currently `publish = false`. The package graph cannot yet be resolved entirely through crates.io.

Do not publish crates until this graph is made crates.io-compatible.

## Release Preconditions

Run before enabling publication:

```bash
./run -w
./run -t
./run -d
cargo doc --workspace --no-deps
cargo package -p vmnl-macros --list
cargo package -p vmnl-graphics --list
cargo package -p vmnl --list
```

Then:

```text
1. Make `vmnl-macros` publishable by removing `publish = false`.
2. Replace every publishable path-only dependency with a versioned crates.io dependency.
3. Re-enable the release workflow only after all dry-runs succeed.
```

Expected publish order and dry-runs:

```bash
cargo publish -p vmnl-macros --dry-run
cargo publish -p vmnl-graphics --dry-run
cargo publish -p vmnl --dry-run
```

Invariant:

```text
Every published crate dependency must resolve from crates.io by version, not only by local path.
```
