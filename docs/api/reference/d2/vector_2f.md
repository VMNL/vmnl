# `Vector2f`

## Public path and maturity

Import path: `vmnl::d2::Vector2f`. Status: experimental value type.

## Purpose and use cases

Stores 2D positions, dimensions, offsets, and application-defined vector values.

## Public API

Public fields: `x: f32`, `y: f32`. `#[repr(C)]`; derives `Clone`, `Copy`, `Debug`, `Default`, `Pod`, `Zeroable`, `PartialEq`; explicit `Eq`, total `Ord`/`PartialOrd`, `Sub`, `SubAssign`, `AddAssign`, and `Mul<f32>` implementations.

## Construction, defaults, and validation

Construct with a literal. Default/zeroed value is `(0.0, 0.0)`. Every `f32` bit pattern is representable; builders apply their own finite/range validation.

## Units, coordinates, and valid ranges

Units are determined by the consuming API; 2D shape builders use pixel-like coordinates. Ordering uses `f32::total_cmp`, including deterministic ordering of NaNs and signed zero.

## Ownership, lifecycle, and threading

Plain copied POD value.

## Errors, panics, and failure conditions

Operators are infallible but IEEE-754 overflow/NaN propagation applies.

## Allocation, transfers, synchronization, and GPU cost

No allocation or synchronization. `repr(C)`/`Pod` permits compatible byte transfer.

## Platform, Vulkan, and display constraints

None; rendering interpretation depends on shaders and framebuffer state.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::d2::Vector2f;

let mut value = Vector2f { x: 1.0, y: 2.0 };
value += Vector2f { x: 3.0, y: 4.0 };
assert_eq!(value, Vector2f { x: 4.0, y: 6.0 });
```

Related: [`Vertex2D`](vertex_2d.md) and [`Shape`](shapes/shape.md).

