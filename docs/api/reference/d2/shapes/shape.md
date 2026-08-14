# `Shape`

## Public path and maturity

Import path: `vmnl::d2::Shape`. Status: experimental, operational GPU resource.

## Purpose and use cases

Owns GPU-backed 2D geometry accepted by `FrameRenderer::draw2d`.

## Public API

Builder entry points: `rect(w, h)`, `indexed(vertices, indices)`, `triangle(a, b, c)`, `triangle_from_vertices([Vertex2D; 3])`, and `line(from, to)`. Implements `Drawable2D` and `AsRef<Shape>`; fields and direct construction are private.

## Construction, defaults, and validation

Each entry point captures required CPU data; its builder validates and allocates on `build(&Context)`. Alpha values select opaque versus alpha blending internally.

## Units, coordinates, and valid ranges

Shape dimensions, positions, endpoints, widths, origins, and vertices are `f32` pixel-like 2D coordinates; detailed rules are builder-specific.

## Ownership, lifecycle, and threading

Owns shared Vulkan buffer handles associated with its context/device. It is not cloneable. Borrow it through a frame submission; drop releases its ownership shares.

## Errors, panics, and failure conditions

Builder entry points are infallible. `build` can reject invalid geometry/counts and GPU buffer creation failures.

## Allocation, transfers, synchronization, and GPU cost

Build allocates and directly uploads vertex data and, when indexed, index data. Drawing reuses these buffers. Exact allocation count/performance is not specified.

## Platform, Vulkan, and display constraints

Build requires a Vulkan context; drawing additionally requires a compatible window/render backend.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::Context;
use vmnl::d2::Shape;

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let rect = Shape::rect(100.0, 50.0).position(20.0, 30.0).build(&context)?;
    drop(rect);
    Ok(())
}
```

Related: [`Drawable2D`](../drawable_2d.md) and the [draw-2D workflow](../../../workflows/draw_2d.md).
