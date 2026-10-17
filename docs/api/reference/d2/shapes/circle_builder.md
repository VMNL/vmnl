# `CircleBuilder`

## Public path and maturity

Entry point: `vmnl::d2::Shape::circle(radius)`. Status: experimental, operational geometry builder.
The returned type currently has no public import path: keep its type inferred. In particular,
`vmnl::d2::CircleBuilder` is not exported, and Rustdoc does not expose a standalone builder page.
The accessible Rustdoc entry is [`Shape::circle`](../../../../../target/doc/vmnl/d2/struct.Shape.html#method.circle).

## Purpose and use cases

Builds filled circles and ellipses, outlined rings, filled sectors, and outlined arcs through one
builder. Ellipse strokes follow the ellipse normal rather than subtracting a second pair of radii.

## Public API

All setters consume and return the builder. Its fields and constructor are private; it has no
public `Default`, `Clone`, or `Copy` implementation.

| Method | Effect |
| --- | --- |
| `position(x, y)` | Set the center |
| `ellipse(radius)` | Replace the vertical radius; preserve the horizontal radius |
| `filled(bool)` | Select filled or outlined geometry; preserve stored widths |
| `inner(w)` | Set the inward offset; disable filling; preserve the outer offset |
| `outer(w)` | Set the outward offset; disable filling; preserve the inner offset |
| `width(w)` | Set both offsets to `w / 2`; disable filling |
| `sector(start, sweep)` | Set the signed angular interval; preserve fill mode |
| `cap(LineCap)` | Select sector endpoint geometry |
| `segments(u16)` | Set subdivisions across the complete circle or requested sweep |
| `color(C: Into<Rgba>)` | Set one color for every generated vertex |
| `buffer_memory_preference(preference)` | Set the vertex/index buffer placement preference |
| `build(&Context)` | Consume the builder and return `VMNLResult<Shape>` |

## Construction, defaults, and validation

`Shape::circle(r)` starts with horizontal and vertical radii equal to `r`, center `(0, 0)`, filled
geometry, opaque white, 32 segments, inner and outer offsets of `0.5`, no sector, no cap, and
`BufferMemoryPreference::Device`.

`Shape::ellipse(rx, ry)` and the exported `EllipseBuilder` have been removed. Use
`Shape::circle(rx).ellipse(ry)`; remove explicit `EllipseBuilder` type annotations.

Setter order matters: `width`, `inner`, and `outer` switch to an outline; a later `filled(true)`
restores filling without clearing either width. Filled geometry omits the inner boundary but
still offsets the outer boundary. Consequently, a default filled circle with reference radius
`50.0` has boundary radius `50.5`. For a filled circle with boundary radius `50.0`, use
`Shape::circle(50.0).outer(0.0).filled(true)`; zero outer width is accepted only in filled mode.
For ellipses, the outer offset follows the unit normal and is not generally another exact ellipse.

Validation occurs on `build`, before GPU allocation:

| Parameter | Accepted values |
| --- | --- |
| Center | Finite X and Y |
| Both radii | Finite and strictly positive |
| Inner and outer offsets, when unfilled | Both finite and strictly positive |
| Offsets, when filled | Not validated; the outer offset still affects the boundary |
| Segment count | `1..=65535`; no minimum-three or nondegeneracy check |
| Sector start | Finite, in `0..=360` |
| Sector sweep | Finite, `0 < abs(sweep) <= 360`; fractional degrees are accepted |
| Sector/cap pair | Either both absent or both present, including filled/full-turn sectors |

Without a sector, geometry covers a full turn and cannot have caps. An explicit sector with a
`360` or `-360` sweep still requires a cap and receives cap geometry when unfilled. Cap selection
is mandatory but has no geometric effect when filled. `Butt` leaves the endpoints unchanged;
`Square` extends each endpoint tangentially by `(inner + outer) / 2`; `Round` adds a semicircle of
that radius at each end, with its center shifted for asymmetric offsets. Round caps use 32 fixed
subdivisions each, independently of `segments`.

## Units, coordinates, and valid ranges

Centers, radii, and widths are `f32` pixel-like 2D values. Position is the center, not a bounding-box
corner. Angles are degrees: zero is the top, `90` the right, `180` the bottom, and `270` the left.
Positive sweeps are clockwise in the built-in X-right/Y-down coordinates; negative sweeps reverse
direction. Start angles are checked rather than normalized. For ellipses, these are parametric
ellipse angles, not equal arc-length steps.

Offsets are not constrained relative to the radii. Large inward offsets, low segment counts, or
explicit full-turn caps can produce degenerate or overlapping geometry. There is no
self-intersection or single-coverage guarantee.

## Ownership, lifecycle, and threading

The builder holds CPU configuration and no context borrow. `build` consumes it; the resulting
immutable shape owns shared vertex/index buffer handles associated with the supplied context.
Use it with a window from the same context. Rebuild to change geometry or style. No worker thread
is created; dropping the shape releases its ownership shares.

## Errors, panics, and failure conditions

Invalid parameters and invalid triangle topology/counts return `VMNLErrorKind::InvalidState`.
GPU buffer creation/upload errors propagate from `build`.

Current limitation: generated vertex positions are not checked again for finiteness, degeneracy,
or self-intersections. Finite inputs can overflow during geometry generation; filled geometry
also uses its unvalidated outer offset. Parameter acceptance does not guarantee usable geometry
for extreme values or oversized inward offsets. Allocation exhaustion is not covered by a
recoverable-error guarantee.

## Allocation, transfers, synchronization, and GPU cost

Setters configure CPU values only. `build` allocates CPU vertex/index vectors, tessellates, then
allocates/uploads one vertex buffer and one index buffer through the indexed-shape path.

| Geometry, for `n` segments | Vertices | Indices | Triangles |
| --- | --- | --- | --- |
| Filled circle, ellipse, or sector | `n + 2` | `3n` | `n` |
| Outline without round caps | `2(n + 1)` | `6n` | `2n` |
| Outlined sector with round caps | `2(n + 1) + 68` | `6n + 192` | `2n + 64` |

At the default 32 segments, a filled circle has 34 vertices and 96 indices; a ring has 66 vertices
and 192 indices. Square caps reuse endpoint vertices. Drawing reuses the buffers. Memory placement
is a preference, not a guarantee; exact timing, internal allocations, and synchronization costs
are unspecified.

## Platform, Vulkan, and display constraints

Builder configuration is headless. Building requires a Vulkan context; drawing additionally
requires a compatible window/render backend. Compilation and CPU geometry tests do not establish
correct pixels or platform qualification.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, d2::{LineCap, Shape}};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let ellipse = Shape::circle(70.0)
        .ellipse(40.0)
        .position(100.0, 120.0)
        .build(&context)?;
    let ring = Shape::circle(50.0).width(8.0).build(&context)?;
    let arc = Shape::circle(70.0)
        .ellipse(40.0)
        .width(8.0)
        .sector(0.0, -90.0)
        .cap(LineCap::Round)
        .build(&context)?;
    drop((ellipse, ring, arc));
    Ok(())
}
```

Full visual program: [`d2_circles`](../../../../../examples/d2/circles/src/main.rs), run with
`just run d2_circles`. CPU geometry/validation evidence lives in the
[`circle` unit tests](../../../../../crates/vmnl_graphics/src/2d/shape/circle.rs).
Related: [`Shape`](shape.md), [`LineCap`](line_cap.md),
[`BufferMemoryPreference`](../../common/buffer_memory_preference.md), and the
[draw-2D workflow](../../../workflows/draw_2d.md).
