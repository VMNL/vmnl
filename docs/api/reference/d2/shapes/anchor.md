# `Anchor`

## Public path and maturity

Import path: `vmnl::d2::Anchor`. Status: experimental.

## Purpose and use cases

Selects a predefined local pivot for rectangle rotation.

## Public API

Variants: `TopLeft` (default), `Top`, `TopRight`, `Left`, `Center`, `Right`, `BottomLeft`, `Bottom`, and `BottomRight`. Derives `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, and `Default`.

## Construction, defaults, and validation

Every variant is valid. `TopLeft` maps to `(0,0)`; edge/center variants map to the corresponding fractions of rectangle size.

## Units, coordinates, and valid ranges

Resolved pivot uses the rectangle's local coordinate units.

## Ownership, lifecycle, and threading

Plain copied builder configuration.

## Errors, panics, and failure conditions

None directly; rectangle numeric validation occurs at build.

## Allocation, transfers, synchronization, and GPU cost

None directly.

## Platform, Vulkan, and display constraints

None.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::d2::Anchor;

assert_eq!(Anchor::default(), Anchor::TopLeft);
```

Related: [`RectBuilder`](rect_builder.md).
