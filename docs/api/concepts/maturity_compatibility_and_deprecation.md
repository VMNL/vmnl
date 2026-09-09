# Maturity, compatibility, and deprecation

VMNL has no public release yet. The current public API is experimental and may change while the
initial `0.1.0` contract is prepared.

Publishing `0.1.0` starts the compatibility policy defined in
[`docs/deployment.md`](../../deployment.md): published Rust source APIs, documented behavior,
documented defaults, and Cargo feature names remain compatible throughout `0.x`. Backend
interoperability marked unstable, undocumented implementation details, and formats or ABIs not yet
declared stable are excluded.

| Area | Maturity |
|---|---|
| Context/window/input/monitors | Experimental, operational |
| 2D shapes/rendering | Experimental, operational |
| Raw pipelines/geometry/uniforms | Experimental, operational within documented limits |
| 3D types/resources | Scaffolded; frame submission unavailable |

After `0.1.0`, replacement APIs must precede deprecation. Deprecations appear in Rustdoc, this book,
the public API snapshot review, and the release notes. Deprecated `0.x` APIs remain until `1.0.0`;
their removal requires a migration guide and an explicit reviewed API change.

A change to documented defaults, behavior, ownership, errors, threading, or platform requirements
receives compatibility review even when the Rust signature does not change.
