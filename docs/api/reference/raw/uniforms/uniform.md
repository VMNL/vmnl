# `Uniform<TData>`

## Public path and maturity

Import path: `vmnl::raw::Uniform<TData>`. Status: experimental, operational direct-write buffer.

## Purpose and use cases

Owns one typed uniform-buffer value later bound through `ResourcesBuilder`. Use it for stable data or explicit one-off updates. Use [`FrameUniform`](frame_uniform.md) for values that change every frame.

## Public API

- `Uniform::builder(data) -> UniformBuilder<TData>`.
- `write(&mut self, data) -> VMNLResult<()>` where `TData: BufferContents`.

Fields are private; no read/clone/default method.

## Construction, defaults, and validation

Initial data is required and moved into the builder. `TData: BufferContents` is required at build.

## Units, coordinates, and valid ranges

Defined by GLSL block layout and application semantics.

## Ownership, lifecycle, and threading

Owns a context-associated Vulkan subbuffer. Resources clone a byte-view handle to the same buffer rather than borrowing the `Uniform` after build. `write` updates that existing buffer, so already-built resources remain usable.

## Errors, panics, and failure conditions

Builder creation is infallible; buffer build/resource compatibility can fail. `write` returns `InvalidState` when active CPU or GPU access conflicts with the write, and a Vulkan validation error for other backend write failures.

## Allocation, transfers, synchronization, and GPU cost

Build allocates and writes one uniform buffer. `write` maps and writes the existing buffer; it does not recreate the buffer, rebuild descriptor sets, submit GPU work, or wait for in-flight GPU access. It can therefore fail if the same buffer is still in use by an in-flight frame. Exact cost is unspecified.

## Platform, Vulkan, and display constraints

Size/alignment/layout must match shader and device requirements; current raw descriptors accept only single uniform buffers.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::Context;
use vmnl::raw::{Pod, Uniform, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Data { color: [f32; 4] }

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let mut uniform = Uniform::builder(Data { color: [1.0; 4] }).build(&context)?;
    uniform.write(Data { color: [0.5; 4] })?;
    drop(uniform);
    Ok(())
}
```

Related: [`UniformBuilder`](uniform_builder.md), [`FrameUniform`](frame_uniform.md), and [`ResourcesBuilder`](../resources/resources_builder.md).
