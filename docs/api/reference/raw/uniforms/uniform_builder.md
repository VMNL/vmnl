# `UniformBuilder<TData>`

## Public path and maturity

Import path: `vmnl::raw::UniformBuilder<TData>`. Status: experimental, operational.

## Purpose and use cases

Selects memory preference and uploads one typed initial uniform value.

## Public API

`buffer_memory_preference(preference)` and `build(&Context)` where `TData: BufferContents`.

## Construction, defaults, and validation

Created with required data by `Uniform::builder`; preference defaults to `Device`. Type/layout validity is compile-time; Vulkan allocation validity is checked at build.

## Units, coordinates, and valid ranges

Application/shader defined.

## Ownership, lifecycle, and threading

Owns/moves the data and is consumed by build; result is tied to the context device.

## Errors, panics, and failure conditions

Returns `VulkanFrameUboBufferCreationFailed` if allocation/upload fails. Later binding can reject device/layout mismatches.

## Allocation, transfers, synchronization, and GPU cost

Allocates and directly writes a uniform buffer. Placement is a preference; exact cost is unspecified.

## Platform, Vulkan, and display constraints

Depends on device memory and uniform-buffer layout/size limits.

## Example and related types

See [`Uniform`](uniform.md) and the existing [`examples/raw/uniform`](../../../../../examples/raw/uniform/src/main.rs).
