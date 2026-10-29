# Facade and modules

Consumers can use `vmnl` or an independent domain crate. The facade reexports domain types
without wrapping them: `graphics` is the default and preserves the graphics API, while `audio`
adds `vmnl::audio`. Disable defaults for audio without graphics, Vulkan or GLFW. With neither
feature the facade exposes no domain. See [feature selection](../reference/facade.md).

| Public path | Responsibility | Status |
|---|---|---|
| `vmnl::{Context, Window, ...}` | Context, window, errors, event loop | Experimental |
| `vmnl::common` | Colors, shader sources, memory preferences | Experimental |
| `vmnl::d2` | GPU-backed 2D shapes and draw contracts | Operational, experimental |
| `vmnl::d3` | 3D data contracts | Scaffolded; rendering unavailable |
| `vmnl::raw` | Typed custom pipelines, geometry, uniforms | Operational, experimental |
| `vmnl::audio` | Independent devices, resources, controls and errors | Experimental; native qualification separate |

`pub(crate)`, `pub(super)`, implementation-only re-exports, and `vmnl::raw::__private` are not client API. `__private` exists only so derive macro expansions can name dependencies.
