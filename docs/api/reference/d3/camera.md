# `Camera` — scaffolded

## Public path and maturity

Import path: `vmnl::d3::Camera`. Status: experimental data scaffold; 3D rendering is not operational.

## Purpose and use cases

Stores position, target, and up parameters accepted by `draw3d`. No view/projection matrix or rendering behavior is currently produced.

## Public API

Fields: `position`, `target`, and `up`, all `Vector3f`. Method: `new(position, target, up)`. Implements `Default`; derives `Clone`, `Copy`, `Debug`, and `PartialEq`.

## Construction, defaults, and validation

Default position `(0,0,1)`, target `(0,0,0)`, up `(0,1,0)`. No validation rejects coincident points, zero/parallel up vectors, NaN, or infinity because the backend is scaffolded.

## Units, coordinates, and valid ranges

World units, handedness, clip space, field of view, near/far planes, and projection convention are not specified.

## Ownership, lifecycle, and threading

Plain copied data borrowed by a pending 3D pass until submission.

## Errors, panics, and failure conditions

Construction is infallible. Submitting any pass using it returns the scaffolded 3D `InvalidState` error.

## Allocation, transfers, synchronization, and GPU cost

No allocation/GPU work in this type. No camera data is uploaded by the current backend.

## Platform, Vulkan, and display constraints

Platform-independent data; there is no operational Vulkan 3D path.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::d3::{Camera, Vector3f};

let camera = Camera::new(
    Vector3f { x: 0.0, y: 0.0, z: 5.0 },
    Vector3f::default(),
    Vector3f { x: 0.0, y: 1.0, z: 0.0 },
);
assert_eq!(camera.position.z, 5.0);
```

Related: [`FrameRenderer::draw3d`](../window/rendering/frame_renderer.md) and [`Vector3f`](vector_3f.md).
