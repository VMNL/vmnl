# `DepthFormat`

## Public path and maturity

Import path: `vmnl::DepthFormat`. Status: experimental.

## Purpose and use cases

Request an explicit depth-only image format for a window's render target.

## Public API

Variants: `D16Unorm`, `D32Sfloat`. Derives `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`. No `Default`.

## Construction, defaults, and validation

Select with `RenderTargetConfig::depth`. The default render target requests no depth image. Window construction checks optimal-tiling depth-attachment support on the context's already selected device before creating a native window; allocation additionally checks single-sample support and the actual image extent. There is no format fallback or device reselection.

## Units, coordinates, and valid ranges

`D16Unorm` stores normalized unsigned 16-bit depth; `D32Sfloat` stores 32-bit floating-point depth. Neither format includes stencil. Clear values use finite normalized `[0, 1]` values for both formats.

## Ownership, lifecycle, and threading

Copied immutable configuration. Each framebuffer owns its own depth image/view; these resources are replaced when the swapchain is recreated. The format remains fixed for the window lifetime.

## Errors, panics, and failure conditions

`VulkanFormatNotSupported` rejects an unsupported request or unsupported single-sample attachment. Device-query failure, invalid actual extent, image/view allocation, and framebuffer creation have their separate error paths. No optional feature is enabled to satisfy a request.

## Allocation, transfers, synchronization, and GPU cost

Choosing or copying a variant allocates nothing. Enabling it allocates one image per swapchain image with device memory preferred. Actual allocation size and driver cost are unspecified; no upload or device-wide wait is added.

## Platform, Vulkan, and display constraints

Follows [Vulkan depth formats](https://docs.vulkan.org/guide/latest/depth.html). Device support is checked explicitly. Attachment creation alone does not enable depth testing/writes; raw pipelines opt in through `PipelineSpec::depth`, while high-level 2D always disables them. High-level 3D rendering remains scaffolded.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::{DepthFormat, RenderTargetConfig};
let config = RenderTargetConfig::default().depth(DepthFormat::D32Sfloat);
assert_eq!(config.depth_value(), Some(DepthFormat::D32Sfloat));
```

Related: [`RenderTargetConfig`](render_target_config.md), [`WindowBuilder`](window_builder.md).
