# `PipelineSpec<TVertex>`

## Public path and maturity

Import path: `vmnl::raw::PipelineSpec<TVertex>`. Status: experimental, operational within raw limits.

## Purpose and use cases

Configures shaders, topology, blending, and vertex type before Vulkan pipeline construction.

## Public API

`vertex_shader`, `fragment_shader`, `topology`, `blend_mode`, `topology_value`, `blend_mode_value`, and `build(&Window)`. Implements `Default`; derives `Clone` and `Debug`.

## Construction, defaults, and validation

Defaults: no shaders, `TriangleList`, `Opaque`. Both shaders are required. Build requires `TVertex: BufferContents + Vertex + 'static`; entry point `main`; compatible vertex inputs; only single uniform-buffer descriptors; no descriptor arrays/push constants.

## Units, coordinates, and valid ranges

Shader-defined. Descriptor set/binding indices are reflected from GLSL.

## Ownership, lifecycle, and threading

Owns shader sources and copied options; setters consume/return it; build consumes it and binds the result to the borrowed window's device/render pass.

## Errors, panics, and failure conditions

Returns `InvalidState` for missing shaders/unsupported raw layouts, shader read/compile errors, vertex validation errors, and Vulkan layout/pipeline creation failures.

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

Related: [`Pipeline`](pipeline.md), [`PrimitiveTopology`](primitive_topology.md), and [`BlendMode`](blend_mode.md).

