# Deployment

## Release Policy

VMNL releases are performed manually. The repository has no automatic release or publication workflow, and one must not be added unless this policy is explicitly changed.

Manual execution does not authorize a release by itself. Publishing crates, creating or pushing tags, creating a GitHub release, and using release credentials must each be explicitly authorized.

Required secret for an authorized crates.io publication:

- `CARGO_REGISTRY_TOKEN`: crates.io API token with publish permission.

## Current Publication Blocker

`vmnl-graphics` depends on `vmnl-macros` through a path-only dependency, and `vmnl-macros` is currently `publish = false`. The package graph cannot yet be resolved entirely through crates.io.

Do not publish crates until this graph is made crates.io-compatible.

## Manual Release Preconditions

After resolving the publication blocker, run the non-mutating validation suite:

```bash
cargo build --workspace --all-targets
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
./run -d
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
./run -ut
./run -at
./run -st
cargo test -p vmnl-gpu-tests --no-run
./run -gt # when the environment supports it
```

Then inspect the packaged files:

```bash
cargo package -p vmnl-macros --list
cargo package -p vmnl-graphics --list
cargo package -p vmnl --list
```

Make the dependency graph publishable before any dry-run:

```text
1. Make `vmnl-macros` publishable by removing `publish = false`.
2. Replace every publishable path-only dependency with a versioned crates.io dependency.
3. Verify that each packaged manifest resolves its published dependencies by version.
```

Run dry-runs in dependency order:

```bash
cargo publish -p vmnl-macros --dry-run
cargo publish -p vmnl-graphics --dry-run
cargo publish -p vmnl --dry-run
```

Only after all validations and dry-runs succeed may an explicitly authorized maintainer publish the crates manually in the same dependency order, then create the release tag and GitHub release manually.

Invariant:

```text
Every published crate dependency must resolve from crates.io by version, not only by local path.
```
