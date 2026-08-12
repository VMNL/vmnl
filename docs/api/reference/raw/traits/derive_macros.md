# Raw derive macros

## Public path and maturity

Macro-namespace paths: `vmnl::raw::Pod`, `vmnl::raw::Zeroable`, and `vmnl::raw::Vertex`. Status: experimental procedural macros. They are distinct from the same-named traits.

## Purpose and use cases

Generate underlying bytemuck/Vulkano unsafe trait implementations without requiring direct client dependencies on those crates.

## Public API

- `#[derive(Pod)]`: structs only; requires `#[repr(C)]` or `#[repr(transparent)]`; every field must be `Pod`; non-generic structs receive a compile-time no-padding assertion.
- `#[derive(Zeroable)]`: structs only; every field must be `Zeroable`.
- `#[derive(Vertex)]`: named-field structs only; every field requires `#[format(VULKAN_FORMAT)]`; optional `#[name("shader_name", ...)]` maps one field to shader member names. It generates per-vertex and per-instance Vulkano descriptions.

## Construction, defaults, and validation

Derives run at compile time and emit diagnostics for unsupported input. The `Vertex` macro calculates aligned field offsets and asserts each field size is a multiple of its Vulkan format block size. Generic `Pod` types do not receive the macro's concrete padding assertion; the unsafe contract still applies to every instantiation.

## Units, coordinates, and valid ranges

Formats define byte widths/component interpretation, not application units.

## Ownership, lifecycle, and threading

Generated impls are static type metadata and own no runtime state.

## Errors, panics, and failure conditions

Invalid derives are compile errors. Some generated `Vertex` layout assertions execute when the description is evaluated and can panic for incompatible field/format sizes. A type that bypasses/defeats safety preconditions can cause unsound byte interpretation.

## Allocation, transfers, synchronization, and GPU cost

Derivation is compile-time. Vertex description builds a member map when requested; GPU transfers occur only through resource builders.

## Platform, Vulkan, and display constraints

Format identifiers must be valid Vulkano `Format` variants and compatible with the device/shader input.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::raw::{Pod, Vertex, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Vertex)]
struct VertexData {
    #[name("in_position")]
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
}
```

Related: [`BufferContents`](buffer_contents.md), [`Vertex` trait](vertex.md), and [shader/layout safety](../../../concepts/shaders_vertex_layouts_and_safety.md).

