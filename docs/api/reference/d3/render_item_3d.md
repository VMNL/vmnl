# `RenderItem3D` — scaffolded

## Public path and maturity

Import path: `vmnl::d3::RenderItem3D`. Status: experimental opaque scaffold; 3D rendering is not operational.

## Purpose and use cases

Carries future pipeline/material/buffer/count data from a `Drawable3D` toward the renderer.

## Public API

Derives `Clone`; all fields are private and there is no public constructor or accessor.

## Construction, defaults, and validation

Created internally by `Mesh::render_item_3d`. Clients cannot directly construct it.

## Units, coordinates, and valid ranges

Internal counts/coordinates have no operational rendering contract yet.

## Ownership, lifecycle, and threading

Owns shared buffer handles; dropping releases those shares.

## Errors, panics, and failure conditions

Cloning is infallible. Any frame containing the value's originating 3D pass fails at submission.

## Allocation, transfers, synchronization, and GPU cost

Cloning handles does not upload. No 3D GPU commands are submitted.

## Platform, Vulkan, and display constraints

No operational Vulkan 3D backend.

## Example and related types

Values are only obtainable from a `Drawable3D`; see [`Drawable3D`](drawable_3d.md). No executable rendering example exists because the backend is scaffolded.

