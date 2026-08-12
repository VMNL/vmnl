# `IndexedShapeBuilder`

## Public path and maturity

Import path: `vmnl::d2::IndexedShapeBuilder`; created by `Shape::indexed`. Status: experimental, operational.

## Purpose and use cases

Builds arbitrary 2D triangle-list geometry from public colored vertices and `u32` indices.

## Public API

`buffer_memory_preference(preference)` and `build(&Context) -> VMNLResult<Shape>`.

## Construction, defaults, and validation

Vertices/indices are required at `Shape::indexed`. Memory preference defaults to `Device`. Build requires at least three vertices, a non-empty index count divisible by three, in-bounds indices, and counts representable as `u32`.

## Units, coordinates, and valid ranges

Positions follow 2D coordinates; indices reference the supplied vertex vector and are interpreted in groups of three.

## Ownership, lifecycle, and threading

Builder owns CPU vectors and is consumed by `build`; the resulting shape owns context-associated GPU buffers.

## Errors, panics, and failure conditions

Returns `InvalidState` for geometry/count failures and Vulkan vertex/index-buffer errors for allocation/upload failures.

## Allocation, transfers, synchronization, and GPU cost

Construction may allocate vectors through `Into<Vec<_>>`; build allocates/uploads vertex and index buffers. Placement is a preference and performance is unspecified.

## Platform, Vulkan, and display constraints

Requires a supported Vulkan context; visual rendering requires a compatible window/display.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::Context;
use vmnl::common::Rgba;
use vmnl::d2::{Shape, Vector2f, Vertex2D};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let vertices = [
        Vertex2D { position: Vector2f { x: 0.0, y: 0.0 }, color: Rgba::RED },
        Vertex2D { position: Vector2f { x: 1.0, y: 0.0 }, color: Rgba::GREEN },
        Vertex2D { position: Vector2f { x: 0.0, y: 1.0 }, color: Rgba::BLUE },
    ];
    let shape = Shape::indexed(vertices, [0, 1, 2]).build(&context)?;
    drop(shape);
    Ok(())
}
```

Related: [`Vertex2D`](../vertex_2d.md) and [`BufferMemoryPreference`](../../common/buffer_memory_preference.md).

