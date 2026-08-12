# `Drawable2D`

## Public path and maturity

Import path: `vmnl::d2::Drawable2D`. Status: experimental trait, operational for VMNL-provided implementations.

## Purpose and use cases

Converts a high-level 2D drawable into the opaque backend item consumed by `FrameRenderer::draw2d`.

## Public API

Required method: `render_item_2d(&self) -> RenderItem2D`. `Shape` implements the trait. The generic implementation contract is intentionally not a blanket impl.

## Construction, defaults, and validation

No default. External implementations are practically constrained because `RenderItem2D` exposes no public constructor or fields; clients should currently use `Shape`.

## Units, coordinates, and valid ranges

Defined by the concrete drawable and active 2D pipeline.

## Ownership, lifecycle, and threading

The method borrows the drawable and returns an owned descriptor holding shared GPU handles.

## Errors, panics, and failure conditions

The method is infallible by signature. Invalid GPU/context combinations can fail later at frame submission.

## Allocation, transfers, synchronization, and GPU cost

VMNL's `Shape` implementation clones shared handles and does not upload new geometry. Exact custom-implementation cost is unspecified.

## Platform, Vulkan, and display constraints

Returned items target VMNL's 2D Vulkan pipeline and require a compatible rendering window.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::Context;
use vmnl::d2::{Drawable2D, Shape};

fn item<D: Drawable2D>(drawable: &D) -> vmnl::d2::RenderItem2D {
    drawable.render_item_2d()
}

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let shape = Shape::rect(1.0, 1.0).build(&context)?;
    let _ = item(&shape);
    Ok(())
}
```

Related: [`RenderItem2D`](render_item_2d.md), [`Shape`](shapes/shape.md), and [`FrameRenderer`](../window/rendering/frame_renderer.md).

