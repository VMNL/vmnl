# `Vertex2D`

## Public path and maturity

Import path: `vmnl::d2::Vertex2D`. Status: experimental value type.

## Purpose and use cases

Defines one public 2D vertex with position and 8-bit color for indexed/triangle shape builders.

## Public API

Fields: `position: Vector2f`, `color: Rgba`. `#[repr(C)]`; derives `Clone`, `Copy`, `Debug`, `Default`, `Pod`, `Zeroable`, `PartialEq`; explicit `Eq`, total `Ord`/`PartialOrd`, component-wise `Sub`, `SubAssign`, `AddAssign`, and `Mul<f32>`.

## Construction, defaults, and validation

Construct with a literal. Default is zero position and transparent black. Geometry builders validate counts/indices/shape-specific positions, not the standalone value.

## Units, coordinates, and valid ranges

Position units follow the 2D shader/shape path. Color channels are `0..=255`. Ordering compares position by `total_cmp` then channels.

## Ownership, lifecycle, and threading

Plain copied POD value.

## Errors, panics, and failure conditions

Standalone construction/operators are infallible; IEEE-754 behavior and color operator contracts apply.

## Allocation, transfers, synchronization, and GPU cost

At shape build, VMNL converts color to normalized `f32` and uploads a private GPU vertex format. No standalone allocation.

## Platform, Vulkan, and display constraints

The public layout is stable for Rust FFI/layout purposes through `repr(C)`, but the actual GPU format is a VMNL implementation detail.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::common::Rgba;
use vmnl::d2::{Vector2f, Vertex2D};

let vertex = Vertex2D {
    position: Vector2f { x: 10.0, y: 20.0 },
    color: Rgba::RED,
};
assert_eq!(vertex.color.a, 255);
```

Related: [`Vector2f`](vector_2f.md), [`Rgba`](../common/rgba.md), and [`IndexedShapeBuilder`](shapes/indexed_shape_builder.md).
