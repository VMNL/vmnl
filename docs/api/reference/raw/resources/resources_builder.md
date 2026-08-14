# `ResourcesBuilder`

## Public path and maturity

Import path: `vmnl::raw::ResourcesBuilder`. Status: experimental, operational for uniform buffers.

## Purpose and use cases

Binds typed `Uniform` buffers to reflected descriptor set/binding positions and allocates compatible descriptor sets.

## Public API

`uniform(set, binding, &Uniform<TData>)` and `build(&Context) -> VMNLResult<Resources>`.

## Construction, defaults, and validation

Created for a required pipeline. Reusing the same set/binding is remembered as an error. Build requires the same context/device as the pipeline and every uniform; supplied/required sets and bindings must match. Only `UniformBuffer` with descriptor count one is supported; missing, extra, duplicate, array, unsupported-type, and push-constant contracts are rejected.

## Units, coordinates, and valid ranges

Set/binding numbers are shader-declared `u32` indices.

## Ownership, lifecycle, and threading

Builder stores cloned buffer handles and is consumed by build. `Resources` remains tied to pipeline layout/device but does not borrow the pipeline/uniform values.

## Errors, panics, and failure conditions

Returns `InvalidState` with set/binding detail for contract mismatches, `VulkanValidationFailed` for conversion/internal validation, or `VulkanDescriptorSetCreationFailed` for allocation.

## Allocation, transfers, synchronization, and GPU cost

Binding updates ordered CPU maps; build allocates vectors and Vulkan descriptor sets. No quantitative performance/synchronization guarantee is specified.

## Platform, Vulkan, and display constraints

Subject to Vulkan descriptor limits and the current restricted raw layout model.

## Example and related types

```rust,no_run
# extern crate vmnl;
# use vmnl::{Context, ShaderSource, Window};
# use vmnl::raw::{Pipeline, Pod, Resources, Uniform, Vertex, Zeroable};
# #[repr(C)] #[derive(Clone, Copy, Pod, Zeroable, Vertex)]
# struct V { #[format(R32G32_SFLOAT)] position: [f32; 2] }
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Data { color: [f32; 4] }

# fn main() -> vmnl::VMNLResult<()> {
# let context = Context::new()?;
# let window = Window::new(&context)?;
# let pipeline = Pipeline::<V>::builder()
#   .vertex_shader(ShaderSource::Path("shader.vert".into()))
#   .fragment_shader(ShaderSource::Path("shader.frag".into()))
#   .build(&window)?;
let uniform = Uniform::builder(Data { color: [1.0; 4] }).build(&context)?;
let resources = Resources::builder(&pipeline)
    .uniform(0, 0, &uniform)
    .build(&context)?;
# drop(resources);
# Ok(())
# }
```

Related: [`Resources`](resources.md), [`Uniform`](../uniforms/uniform.md), and [`Pipeline`](../pipeline/pipeline.md).
