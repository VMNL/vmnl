# `RenderTargetInfo`

## Public path and maturity

Import path: `vmnl::RenderTargetInfo`. Status: experimental.

## Purpose and use cases

Inspect resolved render-pass attachments and compare their properties. This description is separate from requested `RenderTargetConfig` and from target identity.

## Public API

Methods: `color_format() -> ColorFormat`, `sample_count() -> u32`, `depth_format() -> Option<DepthFormat>`. Derives `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`. Fields are private; no public constructor or `Default`.

## Construction, defaults, and validation

Window construction derives this information from the created render pass. Read a copy with `Window::render_target_info`; `Pipeline::render_target_info` returns the description captured from its build window. Current targets have one color attachment and one sample per pixel, plus the explicitly requested optional D16/D32 depth attachment. An unexpected internal attachment shape returns `InvalidState` during window construction. This API does not enable MSAA or change automatic color selection.

## Units, coordinates, and valid ranges

`sample_count` is samples per pixel, currently `1`. `depth_format` is `None` for color-only targets and reports an attachment independently of any pipeline depth state. The description excludes extent, clear value, color space, device, image count and render-pass identity. Equality compares the included properties only.

## Ownership, lifecycle, and threading

Copied CPU snapshot without resource ownership or thread affinity. Resize preserves the render pass and its description; extent is not part of the snapshot. Values remain valid after dropping the window/pipeline, but retain no live target handle.

## Errors, panics, and failure conditions

Getters do not fail. Matching descriptions do not make pipelines shareable between windows. `Pipeline::validate_target(&window)` checks the same exact device/render-pass identity predicate as raw submission and returns `InvalidState` for another window, even on the same context with equal properties. Geometry, resources, viewport/scissor bounds and framebuffer readiness remain separate submission checks.

## Allocation, transfers, synchronization, and GPU cost

Window construction records these properties without creating additional GPU resources. Getters copy values and perform no allocation, query or wait. Successful target preflight is CPU-only and allocation-free; its error message may allocate. No pipeline rebuild or state transition is triggered.

## Platform, Vulkan, and display constraints

Obtaining a window snapshot requires a built Vulkan window. Comparing existing copies is headless. VMNL deliberately retains its per-window render-pass restriction; this description does not implement the broader [Vulkan render-pass compatibility rules](https://docs.vulkan.org/spec/latest/chapters/renderpass.html#renderpass-compatibility).

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, DepthFormat, RenderTargetConfig, Window};
fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let window = Window::builder()
        .render_target(RenderTargetConfig::default().depth(DepthFormat::D16Unorm))
        .build(&context)?;
    let info = window.render_target_info();
    println!("color: {}, samples: {}, depth: {:?}",
        info.color_format(), info.sample_count(), info.depth_format());
    assert_eq!(info.sample_count(), 1);
    assert_eq!(info.depth_format(), Some(DepthFormat::D16Unorm));
    Ok(())
}
```

Related: [`ColorFormat`](color_format.md), [`RenderTargetConfig`](render_target_config.md), [`Pipeline`](../raw/pipeline/pipeline.md).
