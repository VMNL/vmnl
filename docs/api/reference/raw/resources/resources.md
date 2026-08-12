# `Resources`

## Public path and maturity

Import path: `vmnl::raw::Resources`. Status: experimental, operational for uniform buffers.

## Purpose and use cases

Owns descriptor sets compatible with one raw pipeline layout for `draw_raw_with`.

## Public API

`Resources::builder(&Pipeline<TVertex>) -> ResourcesBuilder`. Fields are private; no clone/default/accessors.

## Construction, defaults, and validation

Builder captures the pipeline device/layout. Every required supported binding must be supplied before build.

## Units, coordinates, and valid ranges

Descriptor set/binding indices are `u32` and must match shader declarations.

## Ownership, lifecycle, and threading

Owns shared descriptor sets/device/layout; borrowed during frame recording. It is logically tied to its pipeline layout/device.

## Errors, panics, and failure conditions

Builder entry is infallible; resource build/submission can reject mismatches.

## Allocation, transfers, synchronization, and GPU cost

Build allocates descriptor sets and clones buffer handles. It does not upload uniform values beyond the prior `Uniform` build. Exact cost is unspecified.

## Platform, Vulkan, and display constraints

Current surface supports single uniform-buffer descriptors only; no arrays, textures/samplers, storage buffers, or push constants.

## Example and related types

See [`ResourcesBuilder`](resources_builder.md) and the [uniform-binding workflow](../../../workflows/bind_raw_uniforms.md).

