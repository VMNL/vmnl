# `LineCap`

## Public path and maturity

Import path: `vmnl::d2::LineCap`. Status: experimental.

## Purpose and use cases

Selects generated geometry at both endpoints of a thick line.

## Public API

Variants: `Butt` (default, no extension), `Round` (semicircular geometry), and `Square` (half-width extension). Derives `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, and `Default`.

## Construction, defaults, and validation

Every variant is valid; geometry validation belongs to `LineBuilder::build`.

## Units, coordinates, and valid ranges

Square/round extensions are based on half the configured line width.

## Ownership, lifecycle, and threading

Plain copied configuration.

## Errors, panics, and failure conditions

None directly.

## Allocation, transfers, synchronization, and GPU cost

Round caps create additional vertices/indices; exact cost is unspecified.

## Platform, Vulkan, and display constraints

None beyond line rendering.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::d2::LineCap;

assert_eq!(LineCap::default(), LineCap::Butt);
```

Related: [`LineBuilder`](line_builder.md).
