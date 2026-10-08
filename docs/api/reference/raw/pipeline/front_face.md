# `FrontFace`

## Public path and maturity

Import path: `vmnl::raw::FrontFace`. Status: experimental.

## Purpose and use cases

Selects which triangle winding counts as front-facing for culling and the fragment shader's `gl_FrontFacing` input.

## Public API

Variants: `CounterClockwise` and `Clockwise`. Derives `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`.

## Construction, defaults, and validation

`PipelineSpec` defaults to `CounterClockwise`. Both variants are valid with every `CullMode`. Front/back classification remains shader-visible with `CullMode::None`.

## Units, coordinates, and valid ranges

Winding is evaluated after shader and viewport transformations, in framebuffer coordinates. Positive signed triangle area is front-facing for `CounterClockwise`; negative signed area is front-facing for `Clockwise`. Application-space winding alone does not determine the result. Zero-area triangles are classified as back-facing.

## Ownership, lifecycle, and threading

Copied configuration, fixed when the pipeline is built.

## Errors, panics, and failure conditions

Selection is infallible. Pipeline construction can still fail for shader/device/render-pass errors.

## Allocation, transfers, synchronization, and GPU cost

Setters/getters do not allocate or submit GPU work. Pipeline construction creates backend objects. No performance guarantee is specified.

## Platform, Vulkan, and display constraints

Requires no optional Vulkan device feature. Semantics follow [Vulkan front-face classification](https://docs.vulkan.org/refpages/latest/refpages/source/VkFrontFace.html).

## Example and related types

```rust
# extern crate vmnl;
use vmnl::raw::{FrontFace, PipelineSpec};

let spec = PipelineSpec::<[f32; 2]>::default().front_face(FrontFace::Clockwise);
assert_eq!(spec.front_face_value(), FrontFace::Clockwise);
```

Related: [`CullMode`](cull_mode.md), [`PipelineSpec`](pipeline_spec.md), and the [raw pipeline workflow](../../../workflows/create_raw_pipeline.md).
