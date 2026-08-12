# `Pod` trait

## Public path and maturity

Type-namespace path: `vmnl::raw::Pod`. Status: experimental marker trait.

## Purpose and use cases

Exposes the bytemuck plain-old-data contract through the VMNL facade for raw buffer layouts.

## Public API

No methods. Supertrait: `bytemuck::Pod`; blanket-adapted for underlying implementations and omitted from inventory.

## Construction, defaults, and validation

Prefer the same-named derive. `Pod` requires every bit pattern to be valid, no padding bytes, stable layout, `Copy`, and compatible field types according to bytemuck's safety contract.

## Units, coordinates, and valid ranges

Not applicable to the marker.

## Ownership, lifecycle, and threading

POD values contain no invalid uninitialized/pointer-like state under the underlying contract.

## Errors, panics, and failure conditions

False unsafe implementations are unsound. The derive rejects obvious unsupported representation/field/padding cases at compile time.

## Allocation, transfers, synchronization, and GPU cost

The marker itself performs no runtime work; buffer creation can use the proven byte representation for uploads.

## Platform, Vulkan, and display constraints

Rust layout and shader layout must still agree; `Pod` alone does not prove shader compatibility.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::raw::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct UniformData { value: [f32; 4] }
```

Related: [`Zeroable`](zeroable.md) and [derive macros](derive_macros.md).
