# `Uniform<TData>`

## Public path and maturity

Import path: `vmnl::raw::Uniform<TData>`. Status: experimental, operational initial-value buffer.

## Purpose and use cases

Owns typed uniform-buffer data later bound through `ResourcesBuilder`.

## Public API

`Uniform::builder(data) -> UniformBuilder<TData>`. Fields are private; no read/update/clone/default method.

## Construction, defaults, and validation

Initial data is required and moved into the builder. `TData: BufferContents` is required at build.

## Units, coordinates, and valid ranges

Defined by GLSL block layout and application semantics.

## Ownership, lifecycle, and threading

Owns a context-associated Vulkan subbuffer; resources clone a byte-view handle rather than borrowing it after build.

## Errors, panics, and failure conditions

Builder creation is infallible; buffer build/resource compatibility can fail.

## Allocation, transfers, synchronization, and GPU cost

Build allocates and writes one uniform buffer. No public update operation exists. Exact cost is unspecified.

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
    let uniform = Uniform::builder(Data { color: [1.0; 4] }).build(&context)?;
    drop(uniform);
    Ok(())
}
```

Related: [`UniformBuilder`](uniform_builder.md) and [`ResourcesBuilder`](../resources/resources_builder.md).

