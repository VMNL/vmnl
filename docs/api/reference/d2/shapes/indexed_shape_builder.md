# `IndexedShapeBuilder`

## Public path and maturity

Import path: `vmnl_graphics::d2::IndexedShapeBuilder`; created by `Shape::indexed`. Status: experimental, operational.

## Purpose and use cases

Builds arbitrary 2D triangle-list geometry from public colored vertices and `u32` indices.

## Public API

`Shape::indexed(vertices, indices)` returns this builder and is annotated `#[must_use]`;
discarding the result triggers `unused_must_use`. Use `let _ = Shape::indexed(...)` for an
intentional discard. Builder methods are `buffer_memory_preference(preference)` and
`build(&Context) -> VMNLResult<Shape>`.

## Construction, defaults, and validation

Vertices and indices are required at `Shape::indexed`, which accepts inputs convertible to
owned vectors. Memory preference defaults to `Device`. Build requires at least three vertices,
a non-empty index count divisible by three, in-bounds indices, counts representable as `u32`,
and finite X/Y positions for every supplied vertex, including unused vertices. Structural,
count, and index errors take precedence over position errors; NaN positions take precedence
over infinities.

This explicit triangle-list API permits repeated indices, duplicate positions, unused finite
vertices, either winding, and zero-area/degenerate triangles. Its validation does not impose
the stricter primitive-triangle area rule.

## Units, coordinates, and valid ranges

Positions follow 2D coordinates; indices reference the supplied vertex vector and are interpreted in groups of three.

## Ownership, lifecycle, and threading

Builder owns CPU vectors and is consumed by `build`; the resulting shape owns context-associated GPU buffers.

## Errors, panics, and failure conditions

Returns `VMNLErrorKind::InvalidState` for triangle-list structure, count, index-bound, or
non-finite-position failures. Structural/count/index failures are reported before position
failures. Vulkan vertex/index-buffer allocation and upload errors propagate from `build`.

## Allocation, transfers, synchronization, and GPU cost

Converting array or other inputs into the builder's owned vectors may allocate. `build` checks
triangle structure and every position before allocating/uploading the vertex and index buffers.
Placement is a preference; exact timing and performance are unspecified.

## Platform, Vulkan, and display constraints

Requires a supported Vulkan context; visual rendering requires a compatible window/display.

## Example and related types

```rust,no_run
# extern crate vmnl_graphics;
use vmnl_graphics::Context;
use vmnl_graphics::common::Rgba;
use vmnl_graphics::d2::{Shape, Vector2f, Vertex2D};

fn main() -> vmnl_graphics::VMNLResult<()> {
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
