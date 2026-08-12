# Raw rendering API

The experimental raw layer is operational within documented limits. It provides typed custom graphics pipelines, vertex/index geometry, uniform buffers, descriptor resources, and ordered composition with 2D passes. VMNL still owns swapchains, render passes, command buffers, submission, and presentation.

| Area | Pages |
|---|---|
| Layout contracts | [Traits and derive macros](traits/README.md) |
| Shader pipeline | [Pipelines](pipeline/README.md) |
| Vertex/index buffers | [Geometry](geometry/README.md) |
| Uniform buffers | [Uniforms](uniforms/README.md) |
| Descriptor bindings | [Resources](resources/README.md) |

`vmnl::raw::__private` is excluded: it is a hidden macro-expansion implementation path, not client API.

