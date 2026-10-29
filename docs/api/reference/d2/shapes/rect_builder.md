# `RectBuilder`

## Public path and maturity

Import path: `vmnl::d2::RectBuilder`; created by `Shape::rect`. Status: experimental, operational.

## Purpose and use cases

Builds a positioned, colored, optionally rotated indexed rectangle with sharp or rounded
circular corners.

## Public API

`position(x, y)`, `color(color)`, `rotation(degrees)`, `anchor(Anchor)`, `origin(x, y)`,
`corner_radius(radius)`, `corner_radii(CornerRadii)`, `corner_segments(segments)`,
`buffer_memory_preference(preference)`, and `build(&Context)`. `anchor` and `origin` replace
each other. `corner_radius(radius)` is equivalent to `corner_radii(CornerRadii::all(radius))`;
the last call to either corner method replaces the previous corner configuration.

## Construction, defaults, and validation

Width/height are required. Defaults: position `(0,0)`, opaque white, rotation `0`,
`Anchor::TopLeft`, sharp corners (`CornerRadii::default()`), `8` corner segments,
`Device` preference. Size must be finite and strictly positive; position, rotation, custom
origin, and computed bounds must be finite/not NaN. Each corner radius must be finite and
`>= 0`; the segment count must be at least one; adjacent corner radii must satisfy
`top_left + top_right <= width`, `bottom_left + bottom_right <= width`,
`top_left + bottom_left <= height`, and `top_right + bottom_right <= height`. Overlapping
or otherwise invalid corner configurations are rejected before GPU allocation, never clamped
or rescaled. Computed arc centers, contour vertices, transformed vertices, and generated
counts must remain finite and representable.

## Units, coordinates, and valid ranges

Position, size, origin, and radii are 2D pixel-like values. Rotation is degrees and reduced
modulo 360 for geometry. Segments are `u16`, at least one; zero-radius corners ignore the
segment setting.

## Ownership, lifecycle, and threading

CPU-only builder consumed by build; resulting shape owns context-associated buffers. The
shape is immutable after build; rebuild to change geometry.

## Errors, panics, and failure conditions

Invalid numeric geometry, invalid or overlapping corner radii, zero segments, and
unrepresentable generated geometry return `InvalidState`; GPU buffer creation can return
Vulkan errors. Building never panics on invalid input.

## Allocation, transfers, synchronization, and GPU cost

With all four radii zero, build keeps the existing four-vertex/six-index sharp-rectangle
geometry and allocates/uploads one vertex and one index buffer. With any non-zero radius,
build performs CPU tessellation: one clockwise boundary of straight edges between tangent
points, one quarter arc per non-zero corner, and one sharp point per zero-radius corner,
triangulated as one triangle fan around the rectangle center. For four non-zero corners and
`S` segments per corner the shape has `4 * (S + 1) + 1` vertices and `3 * 4 * (S + 1)`
indices; rounded generation uses checked count arithmetic and fallible CPU reservation.
Rotation around the configured anchor or custom origin is applied after local boundary
generation. Drawing reuses the immutable buffers with no per-frame allocation. `corner_segments`
controls the geometric approximation only and does not provide anti-aliasing. Exact cost
is unspecified.

## Platform, Vulkan, and display constraints

Requires Vulkan for build and compatible display/window for drawing.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::Context;
use vmnl::d2::{Anchor, CornerRadii, Shape};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let sharp = Shape::rect(100.0, 50.0)
        .anchor(Anchor::Center)
        .rotation(30.0)
        .build(&context)?;
    let rounded = Shape::rect(240.0, 120.0)
        .position(100.0, 80.0)
        .corner_radius(16.0)
        .corner_segments(8)
        .anchor(Anchor::Center)
        .rotation(20.0)
        .build(&context)?;
    let asymmetric = Shape::rect(240.0, 120.0)
        .corner_radii(CornerRadii::new(24.0, 8.0, 32.0, 0.0))
        .build(&context)?;
    drop((sharp, rounded, asymmetric));
    Ok(())
}
```

Related: [`CornerRadii`](corner_radii.md), [`Anchor`](anchor.md), [`Shape`](shape.md), and
[`BufferMemoryPreference`](../../common/buffer_memory_preference.md).
