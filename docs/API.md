# VMNL API

The exhaustive client-facing API book is under [`docs/api/`](api/README.md).

- [Concepts and contracts](api/concepts/README.md)
- [Public reference](api/reference/README.md)
- [Workflows](api/workflows/README.md)
- [Coverage and generated inventories](api/maintenance/README.md)

Rustdoc is canonical for exact signatures and item-local contracts. The API book is canonical for navigation, concepts, workflows, maturity, and cross-cutting matrices.

The independent audio crate is under development and is not yet exposed by the facade.
Its [acceptance contract](audio.md) records the target; generate its current item-level Rustdoc
with `cargo doc -p vmnl_audio --no-deps`, including defaults, ownership, and known limitations.
