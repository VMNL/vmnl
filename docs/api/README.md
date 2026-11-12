# VMNL Graphics public API

This book documents version `0.1.0` of the `vmnl-graphics` package, imported as `vmnl_graphics`. It covers the `common`, `d2`, `d3`, `raw`, window, input, monitor, rendering, and error surfaces. Crate-local items, restricted-visibility items, blanket implementations, and `raw::__private` are outside its scope.

Use [concepts](concepts/README.md) for cross-cutting contracts, [reference](reference/README.md) for public items, and [workflows](workflows/README.md) for end-to-end usage. Rustdoc remains canonical for exact signatures and item-local contracts; this book is canonical for navigation, concepts, workflows, maturity, and cross-cutting matrices.

`0.1.0` is the initial API baseline under the [compatibility policy](concepts/maturity_compatibility_and_deprecation.md). The 2D and raw paths render within documented limits; the 3D types are scaffolding and a submitted 3D pass returns an error. A version number does not establish platform or hardware qualification.
