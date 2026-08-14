# VMNL public API

This book documents the client-visible API re-exported by the `vmnl` crate. It covers the `common`, `d2`, `d3`, `raw`, window, input, monitor, rendering, and error surfaces. Crate-local items, restricted-visibility items, blanket implementations, and `raw::__private` are outside its scope.

Use [concepts](concepts/README.md) for cross-cutting contracts, [reference](reference/README.md) for public items, and [workflows](workflows/README.md) for end-to-end usage. Rustdoc remains canonical for exact signatures and item-local contracts; this book is canonical for navigation, concepts, workflows, maturity, and cross-cutting matrices.

All public APIs are currently experimental. The 2D and raw paths render; the 3D types are scaffolding and a submitted 3D pass returns an error.
