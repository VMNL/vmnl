# `CircleBuilder`

## Public path and maturity

Import path: `vmnl::d2::CircleBuilder`; created by `Shape::circle`. Status: experimental, operational.

## Purpose and use cases

Builds a filled circle from a center point and radius.

## Public API

`position(x, y)`, `color(color)`, `buffer_memory_preference(preference)`, and `build(&Context)`.

## Construction, defaults, and validation

Radius is required. The center defaults to `(0,0)`, color to opaque white, tessellation to 32 triangles, and memory preference to `Device`. Radius must be finite and strictly positive. The center and all bounds formed by `center ± radius` must be finite and not NaN.

## Units, coordinates, and valid ranges

Center coordinates and radius use pixel-like `f32` 2D values. The builder's `position` is the geometric center, not the bounding box's top-left corner.

## Ownership, lifecycle, and threading

The CPU-only builder is consumed by `build`; its resulting shape owns context-associated vertex and index buffer handles.

## Errors, panics, and failure conditions

Invalid numeric geometry returns `InvalidState`; GPU buffer creation can return Vulkan errors. The builder does not panic for invalid user geometry.

## Allocation, transfers, synchronization, and GPU cost

Build creates 33 vertices and 96 indices, then allocates/uploads one vertex and one index buffer. Drawing reuses those buffers. Exact allocation and performance costs are unspecified.

## Platform, Vulkan, and display constraints

Building requires a Vulkan context; drawing requires a compatible display/window backend.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::Context;
use vmnl::d2::Shape;

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let circle = Shape::circle(50.0).position(100.0, 120.0).build(&context)?;
    drop(circle);
    Ok(())
}
```

Related: [`Shape`](shape.md), [`BufferMemoryPreference`](../../common/buffer_memory_preference.md), and the [draw-2D workflow](../../../workflows/draw_2d.md).
