# API layers

VMNL exposes a progression of control:

1. `Shape` builders create predefined GPU-backed 2D resources.
2. `FrameRenderer` composes 2D and raw passes for one frame.
3. `raw` lets the client define typed vertex layouts, shaders, topology, blending, triangle culling/front-face winding, geometry, and uniform bindings.

The layers share a `Context`. Resources and pipelines must be used with the compatible context/window/device from which they were created. The raw layer is not a direct Vulkan handle API: VMNL still owns render passes, swapchains, command buffers, submission, and presentation.

In `0.1.0`, the `raw::BufferContents` and `raw::Vertex` adapters inherit Vulkano traits.
This backend coupling remains explicitly unstable and outside the compatibility commitment;
it is not a backend-independent stable contract. Removing or isolating it remains an architecture
requirement. The supported derives are re-exported through `raw` and require no direct client
dependency on Vulkano or `vmnl_macros`.

The `d3` layer currently provides data/resource scaffolding only. Calling
`draw3d(...).submit()` fails explicitly; no operational 3D workflow is documented.
