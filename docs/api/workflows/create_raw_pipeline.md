# Create a raw pipeline

1. Define a `#[repr(C)]` vertex type and derive `Pod`, `Zeroable`, and `Vertex`.
2. Annotate every vertex field with its Vulkan `#[format(...)]` and optional shader `#[name(...)]`.
3. Build a window first because the pipeline is bound to its device/render pass.
4. Supply both shader stages and optionally topology/blending.
5. Build and retain the pipeline for compatible geometry submissions.

The canonical implementation is [`examples/raw/pipeline`](../../../examples/raw/pipeline/src/main.rs); do not copy its full shaders into documentation. Review [`PipelineSpec`](../reference/raw/pipeline/pipeline_spec.md) and [shader/layout safety](../concepts/shaders_vertex_layouts_and_safety.md).

Current layout limits: only descriptor-count-one uniform buffers, no descriptor arrays, no other descriptor types, and no push constants.

