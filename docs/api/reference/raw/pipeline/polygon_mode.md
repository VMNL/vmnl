# `PolygonMode`

## Public path and maturity

Import path: `vmnl::raw::PolygonMode`. Status: experimental.

## Purpose and use cases

Choose filled triangles, triangle edges or triangle vertices without changing primitive topology. Culling still precedes rasterization. Line/point input topologies are unaffected by this mode.

## Public API

Variants: `Fill`, `Line`, `Point`. Derives `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`.

## Construction, defaults, and validation

`PipelineSpec` defaults to `Fill`. `Line` and `Point` require enabled `DeviceFeature::FillModeNonSolid`, even with a line/point topology. Selecting a mode is infallible; the device is checked at pipeline build.

## Units, coordinates, and valid ranges

Edges use `PipelineSpec::line_width` in framebuffer units. Vertices use the shader's `gl_PointSize`; polygon mode does not set their size. Wireframe shows triangle edges, including internal triangulation edges; it does not generate an outer contour.

## Ownership, lifecycle, and threading

Copied configuration, immutable on a built pipeline and inspectable with `polygon_mode_value`. Changing it requires a new pipeline.

## Errors, panics, and failure conditions

Build returns `DeviceFeatureNotEnabled` when non-solid rasterization was not activated. On portability-subset devices, `Point` additionally needs enabled `pointPolygons`, which `DeviceConfig` cannot currently request; build returns an explicit `InvalidState` before shader compilation.

## Allocation, transfers, synchronization, and GPU cost

Selection and inspection allocate nothing and perform no GPU work. Building a pipeline compiles shaders and creates Vulkan objects; no performance guarantee is specified.

## Platform, Vulkan, and display constraints

Follows [Vulkan polygon rasterization](https://docs.vulkan.org/spec/latest/chapters/primsrast.html#primsrast-polygons) and [portability-subset constraints](https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineRasterizationStateCreateInfo.html).

## Example and related types

```rust
# extern crate vmnl;
use vmnl::raw::{PipelineSpec, PolygonMode};
let spec = PipelineSpec::<[f32; 2]>::default().polygon_mode(PolygonMode::Line);
assert_eq!(spec.polygon_mode_value(), PolygonMode::Line);
```

Related: [`PipelineSpec`](pipeline_spec.md), [`CullMode`](cull_mode.md), and [`DeviceFeature`](../../device_feature.md).
