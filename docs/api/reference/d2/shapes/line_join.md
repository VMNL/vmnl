# `LineJoin`

## Public path and maturity

Import path: `vmnl::d2::LineJoin`. Status: experimental.

## Purpose and use cases

Selects generated geometry where two adjacent segments of a polyline meet.

## Public API

Variants: `Bevel` (default), `Miter`, and `Round`. The enum derives `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, and `Default`.

## Construction, defaults, and validation

Every variant is valid. `PolylineBuilder::join` defaults to `Bevel`. `Miter` uses the builder's positive finite miter limit and falls back to bevel when its outer corner exceeds that bound.

## Units, coordinates, and valid ranges

The miter limit is a multiple of half the configured polyline width. `Round` joins use 12 fixed arc sectors. Bevel and miter joins use straight triangle partitions.

## Ownership, lifecycle, and threading

Plain copied builder configuration; no independent resource or lifecycle.

## Errors, panics, and failure conditions

The enum itself has no errors. Invalid miter-limit values are rejected by `PolylineBuilder::build`, including when another join style is selected.

## Allocation, transfers, synchronization, and GPU cost

Round joins add a fixed 12-triangle arc per non-collinear joined point. Bevel and miter joins add two triangles. No runtime synchronization is introduced.

## Platform, Vulkan, and display constraints

Not applicable beyond the polyline's graphics requirements.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::d2::LineJoin;

assert_eq!(LineJoin::default(), LineJoin::Bevel);
```

Related: [`PolylineBuilder`](polyline_builder.md).
