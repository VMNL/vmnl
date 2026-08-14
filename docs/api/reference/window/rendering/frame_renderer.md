# `FrameRenderer`

## Public path and maturity

Import path: `vmnl::FrameRenderer<'w, 'g>`; created by `Window::render()`. Status: experimental; 2D/raw operational, 3D scaffolded.

## Purpose and use cases

Records ordered logical passes for one frame and submits them through the borrowed window.

## Public API

| Method | Contract |
|---|---|
| `mode(RenderMode)` | Select object submission strategy. |
| `draw2d([&D; N])` | Append a pass from `Drawable2D` values. |
| `draw3d(&Camera, [&D; N])` | Append scaffolded `Drawable3D` pass. |
| `draw_raw(&Pipeline<T>, [&Geometry<T>; N])` | Append raw pass without descriptors. |
| `draw_raw_with(&Pipeline<T>, &Resources, [&Geometry<T>; N])` | Append raw pass with descriptors. |
| `submit()` | Consume the builder, acquire/record/submit/present, optionally poll events. |

Passes execute in append order. Empty arrays create empty logical passes; a frame with no pass still clears and presents.

## Construction, defaults, and validation

`Window::render()` starts with `RenderMode::PerObject` and no passes. Compatibility of raw pipeline, geometry, resources, window render pass, and device is checked during submission/build paths. Any recorded 3D pass causes `submit` to return `InvalidState("3D rendering is not implemented yet")` before swapchain acquisition.

## Units, coordinates, and valid ranges

Pass element count is the const-generic `N`. Rendering coordinates follow each layer's shader contract.

## Ownership, lifecycle, and threading

The builder mutably borrows `Window` and borrows draw resources for `'g`; those values must outlive submission. Setters consume and return the builder; `submit` consumes it.

## Errors, panics, and failure conditions

`submit` can fail for 3D use, incompatible devices/layouts/render passes, zero-size/out-of-date swapchains, command recording, acquisition, device loss, queue submission, or presentation.

## Allocation, transfers, synchronization, and GPU cost

Recording passes allocates CPU vectors and clones shared GPU handles. `submit` performs swapchain acquisition, command recording, queue submission, synchronization, and presentation. Exact batching/performance guarantees are not specified.

## Platform, Vulkan, and display constraints

Requires a ready window and renderable framebuffer. Resize can trigger swapchain/framebuffer recreation. `RenderMode::Batched` currently behaves like `PerObject`.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, Window};
use vmnl::d2::Shape;

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::new(&context)?;
    let shape = Shape::rect(100.0, 50.0).build(&context)?;
    window.render().draw2d([&shape]).submit()?;
    Ok(())
}
```

Related: [`RenderMode`](render_mode.md), [`Drawable2D`](../../d2/drawable_2d.md), and [raw pipelines](../../raw/pipeline/README.md).
