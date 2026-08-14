# `Geometry<TVertex>`

## Public path and maturity

Import path: `vmnl::raw::Geometry<TVertex>`. Status: experimental, operational.

## Purpose and use cases

Owns typed GPU vertex data plus optional indices for a raw pipeline with the same `TVertex`.

## Public API

`Geometry::<TVertex>::builder(vertices) -> GeometryBuilder<TVertex>`. Fields are private; no `Clone`/`Default`.

## Construction, defaults, and validation

Vertices are required at builder creation; validation/allocation occur in `build`.

## Units, coordinates, and valid ranges

Shader/application defined. Indices are `u32` references into vertices.

## Ownership, lifecycle, and threading

Owns context-associated Vulkan buffer handles and is borrowed by raw frame passes.

## Errors, panics, and failure conditions

Builder entry is infallible; build and frame compatibility checks can fail.

## Allocation, transfers, synchronization, and GPU cost

Builder construction may allocate a vector; build uploads buffers. Draw reuses them. Exact cost is unspecified.

## Platform, Vulkan, and display constraints

Build requires Vulkan; rendering requires a same-type compatible pipeline/window/device.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::Context;
use vmnl::raw::Geometry;

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let geometry = Geometry::<[f32; 2]>::builder([[0.0, 0.0], [1.0, 0.0]])
        .build(&context)?;
    drop(geometry);
    Ok(())
}
```

Related: [`GeometryBuilder`](geometry_builder.md), [`Pipeline`](../pipeline/pipeline.md), and [`BufferContents`](../traits/buffer_contents.md).
