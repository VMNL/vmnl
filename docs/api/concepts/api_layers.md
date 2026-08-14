# API layers

VMNL exposes a progression of control:

1. `Shape` builders create predefined GPU-backed 2D resources.
2. `FrameRenderer` composes 2D and raw passes for one frame.
3. `raw` lets the client define typed vertex layouts, shaders, topology, blending, geometry, and uniform bindings.

The layers share a `Context`. Resources and pipelines must be used with the compatible context/window/device from which they were created. The raw layer is not a direct Vulkan handle API: VMNL still owns render passes, swapchains, command buffers, submission, and presentation.

The `d3` layer currently provides data/resource scaffolding only. Calling `draw3d(...).submit()` fails explicitly; no operational 3D workflow is documented.
