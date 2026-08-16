# API change protocol

For every public API change:

1. Change the facade contract and implementation together; do not expose implementation-only modules.
2. Add or update the narrowest deterministic test for the observable contract.
3. Update the item's Rustdoc and its reference page, including defaults, validation, failures, ownership, GPU cost, and platform constraints.
4. Update workflows, examples, matrices, and navigation only where the public contract changes.
5. If the change adds or modifies a GLFW operation, complete the
   [GLFW portability protocol](glfw_portability_protocol.md).
6. Run `just docs-api-update`; review all generated files and the coverage matrix.
7. Add an `Unreleased` changelog entry.
8. Run `just docs-api-check`, then the repository validation sequence in [validation](validation.md).

`docs-api-update` is intentionally mutating. `docs-api-check` is non-mutating and fails on a
missing/duplicate symbol, unknown status or inventory syntax, missing source/proof/page/method
anchor, dependency version drift, direct raw GLFW call outside the adapter, or stale generated
file.
