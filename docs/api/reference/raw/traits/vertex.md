# `Vertex` trait

## Public path and maturity

Type-namespace path: `vmnl::raw::Vertex`. Status: experimental marker trait.

## Purpose and use cases

Marks Rust structs that expose a Vulkano vertex-buffer description compatible with raw pipeline shader inputs.

## Public API

No VMNL-specific methods. Supertrait: Vulkano's `Vertex`; VMNL blanket-adapts implementing types, and that generic impl is omitted from inventory.

## Construction, defaults, and validation

Prefer the same-named [`Vertex` derive macro](derive_macros.md). Pipeline build compares the generated definition with the vertex shader interface.

## Units, coordinates, and valid ranges

Application/shader defined.

## Ownership, lifecycle, and threading

Static type contract; no runtime ownership.

## Errors, panics, and failure conditions

Pipeline build returns `VulkanValidationFailed` when the Rust layout and shader interface cannot be combined. Incorrect unsafe implementations can violate layout invariants.

## Allocation, transfers, synchronization, and GPU cost

Trait dispatch is static. Layout reflection/pipeline validation occurs during pipeline build; vertex upload occurs in `GeometryBuilder`.

## Platform, Vulkan, and display constraints

Formats and shader interfaces must be supported by Vulkano/Vulkan and match the target device.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::raw::{Pod, Vertex, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Vertex)]
struct VertexData {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
}
```

Related: [`Vertex` derive](derive_macros.md), [`PipelineSpec`](../pipeline/pipeline_spec.md), and [`Geometry`](../geometry/geometry.md).

