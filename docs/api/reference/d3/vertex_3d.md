# `Vertex3D` — scaffolded 3D data

## Public path and maturity

Import path: `vmnl::d3::Vertex3D`. Status: experimental value type; 3D rendering is not operational.

## Purpose and use cases

Defines a public mesh vertex with position and 8-bit RGBA color.

## Public API

Fields: `position: Vector3f`, `color: Rgba`. `#[repr(C)]`; derives `Clone`, `Copy`, `Debug`, `Default`, `Pod`, `Zeroable`, `PartialEq`; explicit `Eq`, total `Ord`/`PartialOrd`.

## Construction, defaults, and validation

Literal/default construction. Mesh building validates index structure/counts, not coordinate finiteness or geometric degeneracy.

## Units, coordinates, and valid ranges

3D units/handedness are unspecified. Color channels are `0..=255`.

## Ownership, lifecycle, and threading

Plain copied POD.

## Errors, panics, and failure conditions

Standalone construction is infallible. Rendering remains unavailable.

## Allocation, transfers, synchronization, and GPU cost

Mesh build normalizes color and uploads a private GPU vertex format, although those buffers cannot currently be rendered in 3D.

## Platform, Vulkan, and display constraints

Buffer creation requires Vulkan; no operational 3D display path.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::common::Rgba;
use vmnl::d3::{Vector3f, Vertex3D};

let vertex = Vertex3D { position: Vector3f::default(), color: Rgba::WHITE };
assert_eq!(vertex.color.a, 255);
```

Related: [`Vector3f`](vector_3f.md) and [`MeshBuilder`](mesh/mesh_builder.md).

