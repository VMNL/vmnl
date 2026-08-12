# `Mesh` — scaffolded

## Public path and maturity

Import path: `vmnl::d3::Mesh`. Status: experimental GPU-resource scaffold; 3D rendering is not operational.

## Purpose and use cases

Stores indexed GPU geometry for the future 3D backend and allows compile-time wiring through `Drawable3D`.

## Public API

`Mesh::indexed(vertices, indices) -> MeshBuilder`. Implements `Drawable3D` and `AsRef<Mesh>`; fields/direct construction are private.

## Construction, defaults, and validation

No default. `indexed` captures required data; validation/allocation occur in `MeshBuilder::build`.

## Units, coordinates, and valid ranges

Vertex coordinate conventions are not specified while rendering is scaffolded. Indices are `u32` triangle lists.

## Ownership, lifecycle, and threading

Owns context-associated shared Vulkan buffers; not cloneable.

## Errors, panics, and failure conditions

`indexed` is infallible. Build may fail; any later frame submission with the mesh fails with the explicit 3D `InvalidState`.

## Allocation, transfers, synchronization, and GPU cost

Mesh build allocates/uploads buffers even though no 3D command is submitted. Avoid building meshes solely to render until the backend exists.

## Platform, Vulkan, and display constraints

Build needs Vulkan. No operational Vulkan 3D/display path.

## Example and related types

The compile-only construction example is in [`MeshBuilder`](mesh_builder.md). No successful render example exists.

