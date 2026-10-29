# `CornerRadii`

## Public path and maturity

Import path: `vmnl::d2::CornerRadii`. Status: experimental, operational.

## Purpose and use cases

Describes the four circular corner radii of a rectangle built through `Shape::rect`, including
sharp (`0.0`) corners. There is no separate rounded-rectangle shape type.

## Public API

Public fields `top_left`, `top_right`, `bottom_right`, and `bottom_left`. Constructors
`CornerRadii::all(radius)` and `CornerRadii::new(top_left, top_right, bottom_right, bottom_left)`.
Derives `Clone`, `Copy`, `Debug`, `Default`, and `PartialEq`; `Default` is four sharp corners.
Field and constructor order is clockwise from the top-left corner.

## Construction, defaults, and validation

`all(radius)` applies one radius to every corner. A radius of `0.0` keeps the corner sharp.
`RectBuilder::corner_radius(radius)` is equivalent to `corner_radii(CornerRadii::all(radius))`;
the last call to either method replaces the previous corner configuration. Validation happens
in `RectBuilder::build`: each radius must be finite and `>= 0`, the segment count must be at
least one, and adjacent corner radii must satisfy `top_left + top_right <= width`,
`bottom_left + bottom_right <= width`, `top_left + bottom_left <= height`, and
`top_right + bottom_right <= height`. Overlaps are rejected, never clamped or rescaled.

## Units, coordinates, and valid ranges

Radii use pixel-like 2D coordinates, matching the rectangle's size units.

## Ownership, lifecycle, and threading

Plain copied builder configuration consumed by `RectBuilder::corner_radii`.

## Errors, panics, and failure conditions

None directly; invalid or overlapping values return `InvalidState` from `RectBuilder::build`
before any GPU allocation.

## Allocation, transfers, synchronization, and GPU cost

None directly. See [`RectBuilder`](rect_builder.md) for the resulting tessellation cost.

## Platform, Vulkan, and display constraints

None beyond the owning rectangle build and draw requirements.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::d2::CornerRadii;

let uniform = CornerRadii::all(16.0);
let asymmetric = CornerRadii::new(24.0, 8.0, 32.0, 0.0);

assert_eq!(uniform.top_left, 16.0);
assert_eq!(asymmetric.bottom_left, 0.0);
```

Related: [`RectBuilder`](rect_builder.md), [`Anchor`](anchor.md), and [`Shape`](shape.md).
