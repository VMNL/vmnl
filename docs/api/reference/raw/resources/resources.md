# `Resources`

## Public path and maturity

Import path: `vmnl::raw::Resources`. Status: experimental, operational for uniform buffers.

## Purpose and use cases

Owns descriptor resources compatible with one raw pipeline layout for `draw_raw_with`. Resources can be static, or frame-varying when they bind a `FrameUniform`.

## Public API

`Resources::builder(&Pipeline<TVertex>) -> ResourcesBuilder`. Fields are private; no clone/default/accessors.

## Construction, defaults, and validation

Builder captures the pipeline device/layout. Every required supported binding must be supplied before build. If any binding uses `FrameUniform`, every `FrameUniform` in the same `Resources` must have the same swapchain image count.

## Units, coordinates, and valid ranges

Descriptor set/binding indices are `u32` and must match shader declarations.

## Ownership, lifecycle, and threading

Owns shared descriptor/device/layout state; borrowed during frame recording. It is logically tied to its pipeline layout/device. Frame-uniform resources are also tied to the swapchain image count observed when their frame uniforms were built.

## Errors, panics, and failure conditions

Builder entry is infallible; resource build/submission can reject device, layout, binding, or swapchain image-count mismatches.

## Allocation, transfers, synchronization, and GPU cost

Static uniform resources allocate descriptor sets at build. `FrameUniform` resources keep shared frame-uniform slots and allocate descriptor sets during frame recording so the acquired image uses the latest slot. Exact cost is unspecified.

## Platform, Vulkan, and display constraints

Current surface supports single uniform-buffer descriptors only; no arrays, textures/samplers, storage buffers, or push constants.

## Example and related types

See [`ResourcesBuilder`](resources_builder.md) and the [uniform-binding workflow](../../../workflows/bind_raw_uniforms.md).
