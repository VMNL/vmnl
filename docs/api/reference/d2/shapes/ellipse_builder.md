# `EllipseBuilder`

## Public path and maturity

Import path: `vmnl::d2::EllipseBuilder`; created by `Shape::ellipse`.
Status: experimental, operational.

## Purpose and use cases

Builds a filled ellipse centered at a specified position.

An ellipse is defined by two radiuses: a horizontal radius and a vertical
radius. The ellipse is contained within a bounding rectangle whose width is
twice the horizontal radius and whose height is twice the vertical radius.

A circle is represented as an ellipse whose horizontal and vertical radiuses
are equal.

## Public API

`position(x, y)`, `color(color)`, `buffer_memory_preference(preference)`,
and `build(&Context)`.

The ellipse's radiuses are specified when creating the builder through
`Shape::ellipse`.

## Construction, defaults, and validation

Radiuses are required. The center defaults to `(0, 0)`, color to opaque
white, tessellation to 32 triangles, and memory preference to `Device`.

All radiuses must be finite and strictly positive. The center and all bounds
formed by `center ± radiuses` must be finite and not NaN.

## Units, coordinates, and valid ranges

Center coordinates and radiuses use pixel-like `f32` 2D values.

The builder's `position` specifies the geometric center of the ellipse, not
the top-left corner of its bounding rectangle.

For horizontal radius `rx` and vertical radius `ry`, the bounding rectangle
is:

- left: `center.x - rx`
- right: `center.x + rx`
- top: `center.y - ry`
- bottom: `center.y + ry`

## Ellipse and circle

A circle is a special case of an ellipse where both radiuses are equal.

For example, an ellipse with horizontal and vertical radiuses of `50.0`
is a circle with a diameter of `100.0`.

`Shape::circle(50.0)` therefore produces a circle when the builder's
horizontal and vertical radiuses are both `50.0`. (n.d: `Shape::circle(50.0)` is translated in the backend as `Shape::ellipse(50.0, 50.0)`)

## Ownership, lifecycle, and threading

The CPU-only builder is consumed by `build`. The resulting shape owns
context-associated vertex and index buffer handles.

## Errors, panics, and failure conditions

Invalid numeric geometry returns `InvalidState`. GPU buffer creation can
return Vulkan errors.

The builder does not panic for invalid user-provided geometry.

## Allocation, transfers, synchronization, and GPU cost

Building creates 33 vertices and 96 indices, then allocates and uploads one
vertex buffer and one index buffer.

Drawing reuses those buffers. Exact allocation and performance costs are
unspecified.

## Platform, Vulkan, and display constraints

Building requires a Vulkan context. Drawing requires a compatible
display/window backend.

## Example

```rust,no_run
# extern crate vmnl;
use vmnl::Context;
use vmnl::d2::Shape;

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;


    let circle = Shape::circle(50.0)
        .position(100.0, 120.0)
        .build(&context)?;

    let ellipse = Shape::ellipse(70.0, 40.0)
        .position(100.0, 120.0)
        .build(&context)?;

    drop(circle);
    drop(ellipse);

    Ok(())
}
```


Related: [`Shape`](shape.md), [`BufferMemoryPreference`](../../common/buffer_memory_preference.md), and the [draw-2D workflow](../../../workflows/draw_2d.md).
