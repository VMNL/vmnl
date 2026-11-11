# Crate and modules

Consumers depend directly on package `vmnl-graphics` and import `vmnl_graphics`. This crate owns
the public graphics surface and its implementation; backend modules remain private. Public
procedural macros are re-exported through `raw`, so clients need no macros dependency.

| Public path | Responsibility | Status |
|---|---|---|
| `vmnl_graphics::{Context, Window, ...}` | Context, window, errors, event loop | Experimental |
| `vmnl_graphics::common` | Colors, shader sources, memory preferences | Experimental |
| `vmnl_graphics::d2` | GPU-backed 2D shapes and draw contracts | Operational, experimental |
| `vmnl_graphics::d3` | 3D data contracts | Scaffolded; rendering unavailable |
| `vmnl_graphics::raw` | Typed custom pipelines, geometry, uniforms | Operational, experimental |

`pub(crate)`, `pub(super)`, implementation-only re-exports, and `vmnl_graphics::raw::__private` are not client API. `__private` exists only so derive macro expansions can name dependencies.
