# `Shape`

## Public path and maturity

Import path: `vmnl::d2::Shape`. Status: experimental, operational GPU resource.

## Purpose and use cases

Owns GPU-backed 2D geometry accepted by `FrameRenderer::draw2d`.

## Public API

Builder entry points: `circle(radius)`, `rect(w, h)`, `indexed(vertices, indices)`, `triangle(a, b, c)`, `triangle_from_vertices([Vertex2D; 3])`, `polygon(points)`, `polygon_from_vertices(vertices)`, `line(from, to)`, and `polyline(points)`. Implements `Drawable2D` and `AsRef<Shape>`; fields and direct construction are private.

`circle` returns the inferred [`CircleBuilder`](circle_builder.md), with filled geometry by default.
Use `.ellipse(radius_y)` for a second radius, `.width(w)` for a ring, and
`.sector(start, sweep).cap(LineCap::...)` for a sector or arc. `Shape::ellipse` and `EllipseBuilder`
have been removed; the replacement builder currently has no public import path.

## Construction, defaults, and validation

Each entry point captures required CPU data; its builder validates and allocates on `build(&Context)`. Alpha values select opaque versus alpha blending internally.

Circle-family width setters disable filling. A later `.filled(true)` preserves widths, and the
outer offset still affects the filled boundary. See the [circle builder contract](circle_builder.md)
for defaults, setter order, sector/cap validation, and numeric limitations.

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

Related: [`PolygonBuilder`](polygon_builder.md), [`Drawable2D`](../drawable_2d.md) and the [draw-2D workflow](../../../workflows/draw_2d.md).
