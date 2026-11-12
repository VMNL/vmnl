# `BufferMemoryPreference`

## Public path and maturity

Import path: `vmnl_graphics::common::BufferMemoryPreference`. Status: 0.1.0.

## Purpose and use cases

Requests CPU-friendly or GPU-friendly memory for buffers uploaded directly by VMNL.

## Public API

Variants: `Host` and `Device`. The type derives `Debug`, `Clone`, `Copy`, `Default`, `PartialEq`, `Eq`, and `Hash`; `Device` is the default.

## Construction, defaults, and validation

Select a variant and pass it to a shape, mesh, geometry, or uniform builder. The value is a preference, not a guarantee. VMNL requires host-visible sequential-write memory for current direct uploads.

## Units, coordinates, and valid ranges

Not applicable.

## Ownership, lifecycle, and threading

Plain copied value with no resource ownership.

## Errors, panics, and failure conditions

Selecting a preference is infallible. A later `build` may fail if no compatible memory type/allocation is available.

## Allocation, transfers, synchronization, and GPU cost

`Host` prefers CPU-friendly memory; `Device` prefers GPU-friendly memory while retaining host-write requirements. Exact placement, transfer strategy, caching, and performance are not specified.

## Platform, Vulkan, and display constraints

Actual placement depends on Vulkan memory types exposed by the selected device/driver.

## Example and related types

```rust
# extern crate vmnl_graphics;
use vmnl_graphics::common::BufferMemoryPreference;

assert_eq!(BufferMemoryPreference::default(), BufferMemoryPreference::Device);
```

Related: [`GeometryBuilder`](../raw/geometry/geometry_builder.md) and [`UniformBuilder`](../raw/uniforms/uniform_builder.md).
