# `BufferContents`

## Public path and maturity

Import path: `vmnl::raw::BufferContents`. Status: experimental marker trait.

## Purpose and use cases

Marks values that Vulkano can safely copy into typed raw vertex or uniform buffers.

## Public API

No methods. Supertrait: Vulkano's `BufferContents`. VMNL provides a blanket adapter for every type implementing that underlying trait; the blanket impl is omitted from inventory.

## Construction, defaults, and validation

Normally obtained through compatible derives such as `raw::Pod`/`raw::Zeroable` plus Vulkano contracts. There is no runtime validation.

## Units, coordinates, and valid ranges

Defined by the data/shader contract.

## Ownership, lifecycle, and threading

The marker adds no ownership. Buffer builders consume/move values into GPU allocations.

## Errors, panics, and failure conditions

An incorrect unsafe underlying implementation can violate memory-layout safety. VMNL cannot detect such a violation at runtime.

## Allocation, transfers, synchronization, and GPU cost

The marker itself performs no runtime work; buffer creation may copy the represented bytes.

## Platform, Vulkan, and display constraints

Layout must be compatible with the Vulkan buffer/shader use selected by the client.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::raw::BufferContents;

fn accepts<T: BufferContents>(_value: T) {}
accepts([1.0_f32, 2.0]);
```

Related: [`Pod`](pod.md), [`Vertex`](vertex.md), and [`UniformBuilder`](../uniforms/uniform_builder.md).
