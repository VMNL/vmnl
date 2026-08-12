# `GeometryBuilder<TVertex>`

## Public path and maturity

Import path: `vmnl::raw::GeometryBuilder<TVertex>`. Status: experimental, operational.

## Purpose and use cases

Configures optional indexing and memory preference, validates counts/bounds, and uploads typed raw geometry.

## Public API

`indices(indices)`, `buffer_memory_preference(preference)`, and `build(&Context)` where `TVertex: BufferContents`.

## Construction, defaults, and validation

Vertices are required. Default has no indices and prefers `Device`. Build rejects zero vertices, an explicitly empty index list, out-of-bounds indices, or counts not representable as `u32`. It does not validate topology-specific group sizes.

## Units, coordinates, and valid ranges

Application/shader defined; indices are `u32`.

## Ownership, lifecycle, and threading

Owns CPU vectors and is consumed by build; resulting geometry is tied to the context device.

## Errors, panics, and failure conditions

Returns `InvalidState` for validation/count errors and Vulkan buffer errors for allocation/upload.

## Allocation, transfers, synchronization, and GPU cost

`Into<Vec>` and optional indices may allocate. Build directly uploads host-written vertex/index buffers using the requested memory preference. Exact placement/cost is unspecified.

## Platform, Vulkan, and display constraints

Depends on device memory types and buffer size limits.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::Context;
use vmnl::common::BufferMemoryPreference;
use vmnl::raw::Geometry;

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let geometry = Geometry::<[f32; 2]>::builder([
        [0.0, 0.0], [1.0, 0.0], [0.0, 1.0],
    ]).indices([0, 1, 2])
      .buffer_memory_preference(BufferMemoryPreference::Host)
      .build(&context)?;
    drop(geometry);
    Ok(())
}
```

Related: [`Geometry`](geometry.md) and [`BufferMemoryPreference`](../../common/buffer_memory_preference.md).

