# `Vector3f` — scaffolded 3D data

## Public path and maturity

Import path: `vmnl::d3::Vector3f`. Status: experimental value type; 3D rendering is not operational.

## Purpose and use cases

Stores camera/vertex coordinates and application-defined 3D values.

## Public API

Fields: `x`, `y`, `z` (`f32`). `#[repr(C)]`; derives `Clone`, `Copy`, `Debug`, `Default`, `Pod`, `Zeroable`, `PartialEq`; explicit `Eq`, total `Ord`/`PartialOrd`, `Sub`, `SubAssign`, `AddAssign`, `Mul<f32>`.

## Construction, defaults, and validation

Literal/default construction; zero is default. All `f32` patterns are representable.

## Units, coordinates, and valid ranges

World units/handedness are not specified. Ordering uses component-wise `f32::total_cmp`.

## Ownership, lifecycle, and threading

Plain copied POD.

## Errors, panics, and failure conditions

Operators are infallible; IEEE-754 behavior applies.

## Allocation, transfers, synchronization, and GPU cost

No standalone allocation. The current 3D backend submits no draw commands.

## Platform, Vulkan, and display constraints

None as data; 3D Vulkan rendering is unavailable.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::d3::Vector3f;

let scaled = Vector3f { x: 1.0, y: 2.0, z: 3.0 } * 2.0;
assert_eq!(scaled.z, 6.0);
```

Related: [`Camera`](camera.md) and [`Vertex3D`](vertex_3d.md).

