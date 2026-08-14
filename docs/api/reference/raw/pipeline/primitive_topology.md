# `PrimitiveTopology`

## Public path and maturity

Import path: `vmnl::raw::PrimitiveTopology`. Status: experimental.

## Purpose and use cases

Selects Vulkan input-assembly interpretation for raw geometry.

## Public API

Variants: `PointList`, `LineList`, `LineStrip`, `TriangleList`, and `TriangleStrip`. Derives `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`.

## Construction, defaults, and validation

`PipelineSpec` defaults to `TriangleList`. Geometry validation checks only non-empty/bounds/count representation, not topology-specific primitive completeness.

## Units, coordinates, and valid ranges

Not applicable.

## Ownership, lifecycle, and threading

Copied pipeline-build configuration.

## Errors, panics, and failure conditions

Selection is infallible; pipeline creation/device validation may fail later.

## Allocation, transfers, synchronization, and GPU cost

No direct cost; topology changes assembly semantics and may affect performance, which is not specified.

## Platform, Vulkan, and display constraints

Mapped directly to the corresponding Vulkano topology supported by the graphics pipeline.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::raw::{PipelineSpec, PrimitiveTopology};

let spec = PipelineSpec::<[f32; 2]>::default().topology(PrimitiveTopology::LineList);
assert_eq!(spec.topology_value(), PrimitiveTopology::LineList);
```

Related: [`PipelineSpec`](pipeline_spec.md).
