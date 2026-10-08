# `Pipeline<TVertex>`

## Public path and maturity

Import path: `vmnl::raw::Pipeline<TVertex>`. Status: experimental, operational within raw limits.

## Purpose and use cases

Owns a typed Vulkan graphics pipeline created for one window's device and render pass.

## Public API

`Pipeline::<TVertex>::builder() -> PipelineSpec<TVertex>`, `polygon_mode_value() -> PolygonMode`, `line_width_value() -> f32`. Fields are private; no `Clone`/`Default`.

## Construction, defaults, and validation

Use the spec to provide both shaders and build against a `Window`. `TVertex` must implement `BufferContents + Vertex + 'static` at build.

## Units, coordinates, and valid ranges

Coordinates are defined by the shader and vertex layout. The rasterization getters return the immutable mode and validated width passed at creation. Width is in framebuffer units; the getter reports configuration, not measured pixel coverage or driver rounding. It performs no allocation or GPU operation.

## Ownership, lifecycle, and threading

Owns shared Vulkan pipeline/device/render-pass handles. It is logically tied to the window render pass/device used at build and borrowed by raw frame passes.

## Errors, panics, and failure conditions

Errors occur in `PipelineSpec::build`; combining it with incompatible window/geometry/resources fails during build/submission.

## Allocation, transfers, synchronization, and GPU cost

Pipeline build compiles shaders and allocates Vulkan shader modules/layout/pipeline. Reuse the pipeline; exact compilation/cache cost is unspecified.

## Platform, Vulkan, and display constraints

Requires a built window/render pass and device support for selected shader/layout/topology/blending.

## Example and related types

See the existing [`examples/raw/pipeline`](../../../../../examples/raw/pipeline/src/main.rs) workflow; the [pipeline workflow](../../../workflows/create_raw_pipeline.md) explains the steps without duplicating it.

Related: [`PipelineSpec`](pipeline_spec.md), [`Geometry`](../geometry/geometry.md), and [`FrameRenderer`](../../window/rendering/frame_renderer.md).
