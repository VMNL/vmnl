# `Pipeline<TVertex>`

## Public path and maturity

Import path: `vmnl::raw::Pipeline<TVertex>`. Status: experimental, operational within raw limits.

## Purpose and use cases

Owns a typed Vulkan graphics pipeline created for one window's device and render pass.

## Public API

`Pipeline::<TVertex>::builder() -> PipelineSpec<TVertex>`, `polygon_mode_value() -> PolygonMode`, `line_width_value() -> f32`, `viewport_value() -> ViewportPolicy`, `scissor_value() -> ScissorPolicy`, `depth_value() -> DepthState`, `render_target_info() -> RenderTargetInfo`, `validate_target(&Window) -> VMNLResult<()>`. Fields are private; no `Clone`/`Default`.

## Construction, defaults, and validation

Use the spec to provide both shaders and build against a `Window`. `TVertex` must implement `BufferContents + Vertex + 'static` at build.

## Units, coordinates, and valid ranges

Coordinates are defined by the shader and vertex layout. The rasterization getters return the immutable mode and validated width passed at creation. Width is in framebuffer units; the getter reports configuration, not measured pixel coverage or driver rounding. It performs no allocation or GPU operation.

The depth getter returns the immutable [`DepthState`](depth_state.md) passed at construction, without GPU access; `Disabled` is the default even on a depth-enabled target.

The viewport/scissor getters return immutable policies. Call `resolve(framebuffer_extent)` to inspect values for a supplied pixel extent; it does not inspect the last submitted image or device limits. Full policies follow the acquired image on each draw; fixed values persist across resize without scaling/clamping. See [`ViewportPolicy`](viewport_policy.md) and [`ScissorPolicy`](scissor_policy.md).

## Ownership, lifecycle, and threading

Owns shared Vulkan pipeline/device/render-pass handles and a copied resolved target description. It is tied to the exact window render pass/device used at build and borrowed by raw frame passes. `render_target_info` reports color format, samples and depth without allocation or GPU access. Matching descriptions do not authorize cross-window use, even on the same context.

`validate_target(&window)` checks the same identity predicate as submission before recording a frame. Resize preserves the originating window's render pass and preflight success. The check does not validate geometry, resources, viewport/scissor limits or framebuffer readiness; it does not create or submit work.

## Errors, panics, and failure conditions

`validate_target` returns `InvalidState` for another device or render-pass object, including another window with equal attachment properties. Build a separate pipeline for that window. Errors also occur in `PipelineSpec::build`; combining a pipeline with incompatible geometry/resources or a different target remains rejected at submission, even if preflight was omitted.

## Allocation, transfers, synchronization, and GPU cost

Pipeline build compiles shaders and allocates Vulkan shader modules/layout/pipeline. Reuse the pipeline; exact compilation/cache cost is unspecified.

Target inspection and successful preflight allocate nothing and perform no GPU query, shader work or synchronization. An error message may allocate. Preflight does not rebuild or retarget a pipeline.

## Platform, Vulkan, and display constraints

Requires a built window/render pass and device support for selected shader/layout/topology/blending.

## Example and related types

See the existing [`examples/raw/pipeline`](../../../../../examples/raw/pipeline/src/main.rs) workflow; the [pipeline workflow](../../../workflows/create_raw_pipeline.md) explains the steps without duplicating it.

Related: [`PipelineSpec`](pipeline_spec.md), [`Geometry`](../geometry/geometry.md), and [`FrameRenderer`](../../window/rendering/frame_renderer.md).
