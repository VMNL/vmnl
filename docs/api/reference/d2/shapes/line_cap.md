# `LineCap`

## Public path and maturity

Import path: `vmnl::d2::LineCap`. Status: experimental.

## Purpose and use cases

Selects generated geometry at both endpoints of a thick line, open polyline, or outlined
circle/ellipse sector. Closed polylines have no endpoint caps.

## Public API

Variants: `Butt` (default, no extension), `Round` (semicircular geometry), and `Square` (half-width extension). Derives `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, and `Default`.

## Construction, defaults, and validation

Every variant is valid; geometry validation belongs to the consuming builder. A circle-family
sector requires an explicit cap, even when filled or sweeping a full turn. A cap without a sector
is rejected. Filled sectors ignore cap geometry; unfilled explicit full sweeps still receive caps.

## Units, coordinates, and valid ranges

Square/round extensions are based on half the configured line width.
For circle-family sectors, this is `(inner + outer) / 2`; asymmetric offsets shift round-cap centers.

## Ownership, lifecycle, and threading

Plain copied configuration.

## Errors, panics, and failure conditions

None directly.

## Allocation, transfers, synchronization, and GPU cost

Round caps create additional vertices/indices; exact cost is unspecified.
Circle-family round caps use 32 subdivisions per endpoint, adding 68 vertices and 192 indices
in total, independently of the sector's segment count. Square caps reuse its endpoint vertices.

## Platform, Vulkan, and display constraints

None beyond the consuming shape's rendering requirements.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::d2::LineCap;

assert_eq!(LineCap::default(), LineCap::Butt);
```

Related: [`CircleBuilder`](circle_builder.md), [`LineBuilder`](line_builder.md), and [`PolylineBuilder`](polyline_builder.md).
