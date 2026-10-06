# Create a raw pipeline

1. Define a `#[repr(C)]` vertex type and derive `Pod`, `Zeroable`, and `Vertex`.
2. Annotate every vertex field with its Vulkan `#[format(...)]` and optional shader `#[name(...)]`.
3. Build a window first because the pipeline is bound to its device/render pass.
4. Supply both shader stages and optionally topology, blending, culling, and front-face winding.
5. Build and retain the pipeline for compatible geometry submissions.

The canonical implementation is [`examples/raw/pipeline`](../../../examples/raw/pipeline/src/main.rs); do not copy its full shaders into documentation. Review [`PipelineSpec`](../reference/raw/pipeline/pipeline_spec.md) and [shader/layout safety](../concepts/shaders_vertex_layouts_and_safety.md).

Current layout limits: only descriptor-count-one uniform buffers, no descriptor arrays, no other descriptor types, and no push constants.

## Inspect triangle culling

Run `just run raw_pipeline`. The square on the right contains two triangles with opposite windings. Pipelines for all culling/winding combinations are built at startup and reused; key presses select an existing pipeline.

- `C` cycles `None`, `Front`, `Back`, and `FrontAndBack`.
- `F` switches `CounterClockwise` and `Clockwise`; the title shows the current selection.
- With `None`, both square halves remain visible. With `Front` or `Back`, only one half remains; switching the winding swaps the visible half. With `FrontAndBack`, the square disappears.
- Points, lines, and the triangle strip retain their separate default pipelines throughout.

An operator should check these observations and run the existing `raw_triangle`, `raw_uniform`, and `raw_d2_composition` examples for visible regressions. GPU submission tests do not verify the displayed pixels.
