# Deployment

## crates.io release

Required secret:

- `CARGO_REGISTRY_TOKEN`: crates.io API token with publish permission.

Manual checks before release work:

```bash
./run -t
./run -d
cargo doc --workspace --no-deps
cargo package -p vmnl --list
```

Current publishing constraint:

`vmnl-graphics` depends on `vmnl-macros` through a path dependency, and `vmnl-macros` is currently `publish = false`.

Do not publish crates until the workspace publish graph is made crates.io-compatible.

Expected publish order after that cleanup:

```text
vmnl-macros
vmnl-graphics
vmnl
```

Then run dry-runs before publishing:

```bash
cargo publish -p vmnl-macros --dry-run
cargo publish -p vmnl-graphics --dry-run
cargo publish -p vmnl --dry-run
```

Invariant:

```text
Every published crate dependency must resolve from crates.io by version, not only by local path.
```
