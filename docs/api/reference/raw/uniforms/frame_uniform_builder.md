# `FrameUniformBuilder<TData>`

## Public path and maturity

Import path: `vmnl::raw::FrameUniformBuilder<TData>`. Status: experimental, operational.

## Purpose and use cases

Selects memory preference and initializes one typed uniform slot for every current swapchain image.

## Public API

`buffer_memory_preference(preference)` and `build(&Window)` where `TData: BufferContents + Clone`.

## Construction, defaults, and validation

Created with required data by `FrameUniform::builder`; preference defaults to `Device`. Type/layout validity is compile-time; swapchain image availability and Vulkan allocation validity are checked at build.

## Units, coordinates, and valid ranges

Application/shader defined.

## Ownership, lifecycle, and threading

Owns/moves the data and is consumed by build; result is tied to the window context device and current swapchain image count.

## Errors, panics, and failure conditions

Returns `InvalidState` if the window has no swapchain image, or `VulkanFrameUboBufferCreationFailed` if initial allocation/upload fails.

## Allocation, transfers, synchronization, and GPU cost

Initializes one uniform slot per current swapchain image. Later frame writes may allocate or reuse additional subbuffers. Placement is a preference; exact cost is unspecified.

## Platform, Vulkan, and display constraints

Depends on the window swapchain and device uniform-buffer layout/size limits.

## Example and related types

See [`FrameUniform`](frame_uniform.md) and [`examples/raw/uniform`](../../../../../examples/raw/uniform/src/main.rs).
