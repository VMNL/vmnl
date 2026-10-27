# `RenderTargetConfig`

## Public path and maturity

Import path: `vmnl::RenderTargetConfig`. Status: experimental.

## Purpose and use cases

Configure an optional depth attachment and its per-frame clear value without changing the default color-only path.

## Public API

Methods: `depth(DepthFormat)`, `without_depth()`, `clear_depth(f32)`, `depth_value() -> Option<DepthFormat>`, `clear_depth_value() -> f32`, `validate() -> VMNLResult<()>`. Derives `Clone`, `Copy`, `Debug`, `PartialEq`; implements `Default`. Fields are private.

## Construction, defaults, and validation

Defaults to no depth attachment and clear depth `1.0`. Pass a reusable copy to `WindowBuilder::render_target`; another config replaces the whole previous configuration. Inspect the builder with `render_target_value` and a built window with `render_target_config`. Setters are infallible CPU-only configuration; `validate` checks numeric values headlessly, and build repeats validation before native window creation. Format support is checked before native creation on the selected context device; actual extent is checked before image allocation. Color format selection and single-sample rendering retain their existing policies.

## Units, coordinates, and valid ranges

Clear depth must be finite and within `[0, 1]`, including when depth is disabled. Attachment extent matches the actual swapchain image in framebuffer pixels, independently of logical window size and content scaling. `without_depth` retains the configured clear value but removes the attachment request.

## Ownership, lifecycle, and threading

The configuration is immutable on a built window. Framebuffers own separate image/views for every swapchain image; no depth image is shared between acquired images. Resize recreates those resources at the new extent and preserves format/clear value. Submitted command buffers retain their framebuffer resources through Vulkan ownership; old resources are released when no longer referenced.

## Errors, panics, and failure conditions

`validate` and build reject invalid clear values with `InvalidState`. Unsupported format/sample requests return `VulkanFormatNotSupported` without fallback or GPU reselection. Exceeded format extent limits return `InvalidState`. Device-query failures return `VulkanUnknownError`; image/view/framebuffer errors retain their existing categories. Minimized submission follows the existing out-of-date contract.

## Allocation, transfers, synchronization, and GPU cost

Setters/getters and successful numeric validation allocate nothing and perform no GPU work. Build and resize allocate one depth image/view per swapchain image when enabled, with device memory preferred; none are allocated when disabled. Depth is cleared each frame and contents are not preserved between frames. Before reusing an acquired image's depth attachment, submission waits on that image's previous frame fence and releases its resource locks. This can block the calling thread; it is not a device-wide wait. Color-only windows retain the existing synchronization path. No transfer, sampler or descriptor is introduced. Actual allocation bytes and timing are unspecified.

## Platform, Vulkan, and display constraints

Headless configuration needs no Vulkan/display; building needs an operational graphics context/window. High-level 2D always disables depth testing/writes. Raw pipelines default to disabled and can opt in with [`DepthState`](../raw/pipeline/depth_state.md). This does not make high-level 3D rendering operational.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, DepthFormat, RenderTargetConfig, Window};
fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let target = RenderTargetConfig::default()
        .depth(DepthFormat::D32Sfloat).clear_depth(1.0);
    let window = Window::builder().render_target(target).build(&context)?;
    assert_eq!(window.render_target_config(), target);
    Ok(())
}
```

Related: [`DepthFormat`](depth_format.md), [`WindowBuilder`](window_builder.md), [depth workflow](../../workflows/create_context_and_window.md#optional-depth-attachment).
