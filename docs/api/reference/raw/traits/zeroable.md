# `Zeroable` trait

## Public path and maturity

Type-namespace path: `vmnl::raw::Zeroable`. Status: experimental marker trait.

## Purpose and use cases

Exposes the bytemuck contract that an all-zero byte pattern is valid for a type.

## Public API

No methods. Supertrait: `bytemuck::Zeroable`; blanket-adapted for underlying implementations and omitted from inventory.

## Construction, defaults, and validation

Prefer the derive. Every field must be zeroable. `Zeroable` does not itself guarantee absence of padding or general POD safety.

## Units, coordinates, and valid ranges

Not applicable.

## Ownership, lifecycle, and threading

Static type contract.

## Errors, panics, and failure conditions

False unsafe implementations are unsound. The derive validates supported input/fields at compile time.

## Allocation, transfers, synchronization, and GPU cost

The marker trait itself performs no allocation, transfer, or synchronization. The performance of code using the trait is not specified.

## Platform, Vulkan, and display constraints

Does not establish Vulkan/shader compatibility by itself.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::raw::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Data { value: u32 }
```

Related: [`Pod`](pod.md) and [derive macros](derive_macros.md).
