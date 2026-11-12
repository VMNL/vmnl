# Crate and modules

Consumers depend directly on package `vmnl-graphics` and import `vmnl_graphics`. This crate owns
the public graphics surface and its implementation; backend modules remain private. Public
procedural macros are re-exported through `raw`, so clients need no macros dependency.

| Public path | Responsibility | Status |
|---|---|---|
| `vmnl_graphics::{Context, Window, ...}` | Context, window, errors, event loop | 0.1.0 |
| `vmnl_graphics::common` | Colors, shader sources, memory preferences | 0.1.0 |
| `vmnl_graphics::d2` | GPU-backed 2D shapes and draw contracts | 0.1.0, operational |
| `vmnl_graphics::d3` | 3D data contracts | Scaffolded; rendering unavailable |
| `vmnl_graphics::raw` | Typed custom pipelines, geometry, uniforms | 0.1.0, operational within documented limits |

`pub(crate)`, `pub(super)`, implementation-only re-exports, and `vmnl_graphics::raw::__private` are not client API. `__private` exists only so derive macro expansions can name dependencies.
