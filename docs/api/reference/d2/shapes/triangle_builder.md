# `TriangleBuilder`

## Public path and maturity

Import path: `vmnl::d2::TriangleBuilder`; created by `Shape::triangle` or `Shape::triangle_from_vertices`. Status: experimental, operational.

## Purpose and use cases

Builds one non-indexed colored triangle.

## Public API

`color(color)`, `vertex_colors(a, b, c)`, `buffer_memory_preference(preference)`, and `build(&Context)`. Later color calls replace earlier color configuration.

## Construction, defaults, and validation

Position constructor defaults all colors to opaque white. The vertex constructor preserves supplied colors. Memory preference defaults to `Device`. Build requires pairwise-distinct positions; it does not reject all collinear/NaN/infinite coordinates explicitly.

## Units, coordinates, and valid ranges

Positions are 2D pixel-like `f32`; colors use `Rgba`.

## Ownership, lifecycle, and threading

Owns three positions/colors and is consumed by build; resulting GPU resource belongs to the context/device.

## Errors, panics, and failure conditions

Duplicate positions return `InvalidState`; GPU vertex-buffer allocation/upload may fail. Other geometric degeneracy is not specified as invalid.

## Allocation, transfers, synchronization, and GPU cost

Build allocates/uploads a three-vertex buffer. No index buffer. Cost guarantees are not specified.

## Platform, Vulkan, and display constraints

Requires Vulkan for build and a compatible display/window for drawing.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::Context;
use vmnl::d2::{Shape, Vector2f};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let triangle = Shape::triangle(
        Vector2f { x: 0.0, y: 0.0 },
        Vector2f { x: 1.0, y: 0.0 },
        Vector2f { x: 0.0, y: 1.0 },
    ).color([255, 0, 0]).build(&context)?;
    drop(triangle);
    Ok(())
}
```

Related: [`Shape`](shape.md), [`Vertex2D`](../vertex_2d.md), and [`Rgba`](../../common/rgba.md).
