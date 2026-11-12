# Maturity, compatibility, and deprecation

The current version of `vmnl_graphics` is `0.1.0`, the initial graphics API baseline. Reference
page status labels identify this version and retain each item's operational limits.

The `0.1.0` baseline follows the compatibility policy defined in
[`docs/deployment.md`](../../deployment.md): published Rust source APIs, documented behavior,
documented defaults, and Cargo feature names remain compatible throughout `0.x`. Backend
interoperability marked unstable, undocumented implementation details, and formats or ABIs not yet
declared stable are excluded.

| Area | Maturity |
|---|---|
| Context/window/input/monitors | 0.1.0, operational within platform limits |
| 2D shapes/rendering | 0.1.0, operational |
| Raw pipelines/geometry/uniforms | 0.1.0, operational within documented limits; backend-coupled traits remain unstable |
| 3D types/resources | Scaffolded; frame submission unavailable |

From `0.1.0`, replacement APIs must precede deprecation. Deprecations appear in Rustdoc, this book,
the public API snapshot review, and the release notes. Deprecated `0.x` APIs remain until `1.0.0`;
their removal requires a migration guide and an explicit reviewed API change.

A change to documented defaults, behavior, ownership, errors, threading, or platform requirements
receives compatibility review even when the Rust signature does not change.
