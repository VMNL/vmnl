# Raw rendering API

The raw layer in `0.1.0` is operational within documented limits. It provides typed custom graphics pipelines, vertex/index geometry, direct and frame-varying uniform buffers, descriptor resources, and ordered composition with 2D passes. VMNL still owns swapchains, render passes, command buffers, submission, and presentation.

| Area | Pages |
|---|---|
| Layout contracts | [Traits and derive macros](traits/README.md) |
| Shader pipeline | [Pipelines](pipeline/README.md) |
| Vertex/index buffers | [Geometry](geometry/README.md) |
| Uniform buffers | [Uniforms](uniforms/README.md) |
| Descriptor bindings | [Resources](resources/README.md) |

`vmnl_graphics::raw::__private` is excluded: it is a hidden macro-expansion implementation path, not client API.
