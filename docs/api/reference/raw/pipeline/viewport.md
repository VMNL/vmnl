# `Viewport`

## Public path and maturity

Import path: `vmnl::raw::Viewport`. Status: experimental.

## Purpose and use cases

Describe one viewport transform independently of the clipping rectangle.

## Public API

Public fields: `offset: [f32; 2]`, `extent: [f32; 2]`, `depth_range: [f32; 2]`. Derives `Clone`, `Copy`, `Debug`, `PartialEq`; no `Default`.

## Construction, defaults, and validation

Construct a value and select `ViewportPolicy::Fixed`. Policy resolution checks numeric validity; pipeline build and recording additionally check device limits. `PipelineSpec` defaults to the full framebuffer policy.

## Units, coordinates, and valid ranges

Offset and extent use framebuffer pixels, independently of logical window size and DPI. Offsets and endpoints must be finite; width/height finite and strictly positive. Negative offsets and rectangles extending beyond the image are allowed within `viewportBoundsRange`; dimensions must not exceed `maxViewportDimensions`. Negative-height viewports are unsupported. Both depth endpoints must be finite and within `[0, 1]`; reversed order is valid. This transform adds no depth attachment/test.

## Ownership, lifecycle, and threading

Copied CPU data, immutable on a built pipeline. Fixed values persist across resize without scaling or clamping.

## Errors, panics, and failure conditions

Invalid numeric values or device limits return `InvalidState` at resolution/build/recording. Device limits are not checked by headless resolution.

## Allocation, transfers, synchronization, and GPU cost

Constructing/copying the value performs no allocation or GPU work. The renderer applies one dynamic viewport per raw draw; no pipeline rebuild is needed for resize.

## Platform, Vulkan, and display constraints

Follows [Vulkan viewport limits](https://docs.vulkan.org/refpages/latest/refpages/source/VkViewport.html). Only one viewport is supported; no optional device feature is requested.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::raw::{Viewport, ViewportPolicy};
let value = Viewport { offset: [100.0, 50.0], extent: [320.0, 200.0], depth_range: [0.0, 1.0] };
assert_eq!(ViewportPolicy::Fixed(value).resolve([800, 600])?, value);
# Ok::<(), vmnl::VMNLError>(())
```

Related: [`ViewportPolicy`](viewport_policy.md), [`Scissor`](scissor.md), [`PipelineSpec`](pipeline_spec.md).
