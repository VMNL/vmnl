# `Drawable3D` — scaffolded

## Public path and maturity

Import path: `vmnl::d3::Drawable3D`. Status: experimental scaffold; 3D rendering is not operational.

## Purpose and use cases

Converts a high-level 3D resource into an opaque `RenderItem3D` for the future backend. `Mesh` implements it.

## Public API

Required method: `render_item_3d(&self) -> RenderItem3D`.

## Construction, defaults, and validation

No default. External implementations are practically constrained because `RenderItem3D` has no public constructor/fields. A returned item still cannot be rendered today.

## Units, coordinates, and valid ranges

Not specified while 3D is scaffolded.

## Ownership, lifecycle, and threading

Borrows the drawable and returns an owned descriptor with shared buffer handles.

## Errors, panics, and failure conditions

The method is infallible, but frame submission always rejects a recorded 3D pass.

## Allocation, transfers, synchronization, and GPU cost

`Mesh` clones shared handles. No 3D commands are submitted.

## Platform, Vulkan, and display constraints

No operational Vulkan 3D backend.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::Context;
use vmnl::common::Rgba;
use vmnl::d3::{Drawable3D, Mesh, Vector3f, Vertex3D};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let vertices = [
        Vertex3D { position: Vector3f { x: 0.0, y: 0.0, z: 0.0 }, color: Rgba::RED },
        Vertex3D { position: Vector3f { x: 1.0, y: 0.0, z: 0.0 }, color: Rgba::GREEN },
        Vertex3D { position: Vector3f { x: 0.0, y: 1.0, z: 0.0 }, color: Rgba::BLUE },
    ];
    let mesh = Mesh::indexed(vertices, [0, 1, 2]).build(&context)?;
    let _item = mesh.render_item_3d();
    Ok(())
}
```

Related: [`RenderItem3D`](render_item_3d.md) and [`Mesh`](mesh/mesh.md).

