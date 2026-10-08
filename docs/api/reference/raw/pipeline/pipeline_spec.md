# `PipelineSpec<TVertex>`

## Public path and maturity

Import path: `vmnl::raw::PipelineSpec<TVertex>`. Status: experimental, operational within raw limits.

## Purpose and use cases

Configures shaders, topology, blending, triangle culling, front-face winding, polygon mode, line width, and vertex type before Vulkan pipeline construction.

## Public API

`vertex_shader`, `fragment_shader`, `topology`, `blend_mode`, `cull_mode`, `front_face`, `topology_value`, `blend_mode_value`, `cull_mode_value`, `front_face_value`, `polygon_mode`, `line_width`, `polygon_mode_value`, `line_width_value`, and `build(&Window)`. Implements `Default`; derives `Clone` and `Debug`.

## Construction, defaults, and validation

Defaults: no shaders, `TriangleList`, `Opaque`, `CullMode::None`, `FrontFace::CounterClockwise`, `PolygonMode::Fill`, line width `1.0`. Both shaders are required. Build requires `TVertex: BufferContents + Vertex + 'static`; entry point `main`; compatible vertex inputs; only single uniform-buffer descriptors; no descriptor arrays/push constants. All culling/winding combinations are valid and require no optional device feature. Setters/getters do not allocate or submit GPU work. `build` first validates rasterization against the window's logical device, before reading or compiling shaders. Non-solid modes require enabled `FillModeNonSolid`; any width other than `1.0` requires enabled `WideLines`, even for a topology that does not use that state. Support alone is insufficient. No feature is activated or GPU reselected.

## Units, coordinates, and valid ranges

Shader-defined. Descriptor set/binding indices are reflected from GLSL. Front-face winding is evaluated in framebuffer coordinates after shader/viewport transformations; culling applies to triangle primitives. Polygon mode changes triangle rasterization without changing topology; culling still applies before edges/vertices are rasterized. Line width is finite, strictly positive and within the inclusive range reported by `Context::line_width_limits()`. VMNL rejects out-of-range widths rather than clamping. In-range values may be rounded by the driver; exact pixel thickness is not promised. `gl_PointSize` remains shader-defined. These selections are fixed at pipeline construction; changing them requires another pipeline.

## Ownership, lifecycle, and threading

Owns shader sources and copied options; setters consume/return it; build consumes it and binds the result to the borrowed window's device/render pass.

## Errors, panics, and failure conditions

Returns `InvalidLineWidth { value, min, max }` for non-finite, non-positive or out-of-range widths; `DeviceFeatureNotEnabled { feature }` for disabled required features. Numeric validation precedes feature checks. `PolygonMode::Point` on portability-subset devices without enabled `pointPolygons` returns actionable `InvalidState` before shader compilation; VMNL cannot currently request that capability. Missing shaders/unsupported raw layouts also return `InvalidState`; shader read/compile, vertex validation and Vulkan layout/pipeline failures remain distinct.

## Allocation, transfers, synchronization, and GPU cost

Setters are CPU-only; source strings/paths may allocate. Build reads paths, runs shaderc, reflects layouts, and creates Vulkan objects. Cost is unspecified.

## Platform, Vulkan, and display constraints

Requires an operational window and driver support for the generated pipeline.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, ShaderSource, Window};
use vmnl::raw::{Pipeline, Pod, Vertex, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Vertex)]
struct V { #[format(R32G32_SFLOAT)] position: [f32; 2] }

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let window = Window::new(&context)?;
    let pipeline = Pipeline::<V>::builder()
        .vertex_shader(ShaderSource::Path("shader.vert".into()))
        .fragment_shader(ShaderSource::Path("shader.frag".into()))
        .build(&window)?;
    drop(pipeline);
    Ok(())
}
```

Related: [`Pipeline`](pipeline.md), [`PrimitiveTopology`](primitive_topology.md), [`BlendMode`](blend_mode.md), [`CullMode`](cull_mode.md), [`FrontFace`](front_face.md), [`PolygonMode`](polygon_mode.md), and [`LineWidthLimits`](../../line_width_limits.md).
