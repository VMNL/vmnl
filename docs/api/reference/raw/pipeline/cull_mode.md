# `CullMode`

## Public path and maturity

Import path: `vmnl::raw::CullMode`. Status: experimental.

## Purpose and use cases

Selects which triangle faces a raw pipeline discards, using the configured `FrontFace` to classify them.

## Public API

Variants: `None`, `Front`, `Back`, and `FrontAndBack`. Derives `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`.

## Construction, defaults, and validation

`PipelineSpec` defaults to `None`, preserving both triangle orientations. `Front` discards front-facing triangles; `Back` discards back-facing triangles; `FrontAndBack` discards all triangles. All variants are valid with either `FrontFace` value.

## Units, coordinates, and valid ranges

Classification uses triangle winding in framebuffer coordinates. Point and line primitives are not discarded by this setting.

## Ownership, lifecycle, and threading

Copied configuration, fixed when the pipeline is built. Reuse pipelines for subsequent draws.

## Errors, panics, and failure conditions

Selection is infallible. Pipeline construction can still fail for shader/device/render-pass errors.

## Allocation, transfers, synchronization, and GPU cost

Setters/getters do not allocate or submit GPU work. Pipeline construction creates backend objects. No performance guarantee is specified.

## Platform, Vulkan, and display constraints

Requires no optional Vulkan device feature. Semantics follow [Vulkan triangle culling](https://docs.vulkan.org/refpages/latest/refpages/source/VkCullModeFlagBits.html).

## Example and related types

```rust
# extern crate vmnl;
use vmnl::raw::{CullMode, PipelineSpec};

let spec = PipelineSpec::<[f32; 2]>::default().cull_mode(CullMode::Back);
assert_eq!(spec.cull_mode_value(), CullMode::Back);
```

Related: [`FrontFace`](front_face.md), [`PipelineSpec`](pipeline_spec.md), and the [raw pipeline workflow](../../../workflows/create_raw_pipeline.md).
