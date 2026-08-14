# Facade and modules

Consumers depend on `vmnl`; `vmnl_graphics` is the implementation crate. The facade re-exports the public graphics surface without exposing its private module layout.

| Public path | Responsibility | Status |
|---|---|---|
| `vmnl::{Context, Window, ...}` | Context, window, errors, event loop | Experimental |
| `vmnl::common` | Colors, shader sources, memory preferences | Experimental |
| `vmnl::d2` | GPU-backed 2D shapes and draw contracts | Operational, experimental |
| `vmnl::d3` | 3D data contracts | Scaffolded; rendering unavailable |
| `vmnl::raw` | Typed custom pipelines, geometry, uniforms | Operational, experimental |

`pub(crate)`, `pub(super)`, implementation-only re-exports, and `vmnl::raw::__private` are not client API. `__private` exists only so derive macro expansions can name dependencies.
