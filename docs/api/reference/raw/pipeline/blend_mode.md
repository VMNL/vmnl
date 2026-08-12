# `BlendMode`

## Public path and maturity

Import path: `vmnl::raw::BlendMode`. Status: experimental.

## Purpose and use cases

Selects raw pipeline color-attachment blending.

## Public API

Variants: `Opaque` and `Alpha`. Derives `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`.

## Construction, defaults, and validation

`PipelineSpec` defaults to `Opaque`. `Alpha` uses Vulkano's standard source-alpha attachment blend state.

## Units, coordinates, and valid ranges

Shader color values are application-defined; the blend factors operate on color attachment values.

## Ownership, lifecycle, and threading

Copied pipeline-build configuration.

## Errors, panics, and failure conditions

Selection is infallible; incompatible pipeline/device creation may fail later.

## Allocation, transfers, synchronization, and GPU cost

No direct allocation. Blending can change GPU work; no performance guarantee is specified.

## Platform, Vulkan, and display constraints

Requires a compatible color attachment; VMNL creates the state for its window render pass.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::raw::{BlendMode, PipelineSpec};

let spec = PipelineSpec::<[f32; 2]>::default().blend_mode(BlendMode::Alpha);
assert_eq!(spec.blend_mode_value(), BlendMode::Alpha);
```

Related: [`PipelineSpec`](pipeline_spec.md).

