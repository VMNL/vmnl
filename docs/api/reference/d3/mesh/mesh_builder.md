# `MeshBuilder` — scaffolded

## Public path and maturity

Import path: `vmnl::d3::MeshBuilder`; created by `Mesh::indexed`. Status: experimental GPU-resource scaffold; 3D rendering is not operational.

## Purpose and use cases

Validates indexed triangle data and builds buffers for the future 3D backend.

## Public API

`buffer_memory_preference(preference)` and `build(&Context) -> VMNLResult<Mesh>`.

## Construction, defaults, and validation

Vertices/indices are required. Preference defaults to `Device`. Build requires at least three vertices, non-empty index count divisible by three, in-bounds indices, and `u32`-representable counts.

## Units, coordinates, and valid ranges

3D coordinate convention is unspecified. Indices are groups of three.

## Ownership, lifecycle, and threading

Owns CPU vectors and is consumed by build; result owns context-associated buffers.

## Errors, panics, and failure conditions

Invalid geometry/counts return `InvalidState`; allocation/upload returns Vulkan errors. A successfully built mesh still fails at 3D frame submission.

## Allocation, transfers, synchronization, and GPU cost

Build allocates/uploads vertex and index buffers. Exact placement/performance is unspecified; no 3D commands are submitted.

## Platform, Vulkan, and display constraints

Requires Vulkan to build. No operational 3D display backend.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::Context;
use vmnl::common::Rgba;
use vmnl::d3::{Mesh, Vector3f, Vertex3D};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let vertices = [
        Vertex3D { position: Vector3f { x: 0.0, y: 0.0, z: 0.0 }, color: Rgba::RED },
        Vertex3D { position: Vector3f { x: 1.0, y: 0.0, z: 0.0 }, color: Rgba::GREEN },
        Vertex3D { position: Vector3f { x: 0.0, y: 1.0, z: 0.0 }, color: Rgba::BLUE },
    ];
    let mesh = Mesh::indexed(vertices, [0, 1, 2]).build(&context)?;
    drop(mesh); // construction works; rendering does not
    Ok(())
}
```

Related: [`Mesh`](mesh.md), [`Vertex3D`](../vertex_3d.md), and [`BufferMemoryPreference`](../../common/buffer_memory_preference.md).

