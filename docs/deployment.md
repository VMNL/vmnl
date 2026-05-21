# Deployment

## crates.io release

Required secret:

- `CARGO_REGISTRY_TOKEN`: crates.io API token with publish permission.

Manual checks before tagging:

```bash
cargo test --workspace
cargo doc -p vmnl --no-deps
cargo publish -p vmnl_native --dry-run
cargo package -p vmnl --list
```

Publish order:

```bash
cargo publish -p vmnl_native
sleep 60
cargo publish -p vmnl --dry-run
cargo publish -p vmnl
```

`vmnl_native` must be visible in the crates.io index before `vmnl` can be
published, because `vmnl` depends on `vmnl_native` by version.
