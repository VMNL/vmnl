# `RenderItem2D`

## Public path and maturity

Import path: `vmnl::d2::RenderItem2D`. Status: experimental opaque backend descriptor.

## Purpose and use cases

Transfers a drawable's pipeline/material/blend selection, buffers, and counts into VMNL's frame renderer.

## Public API

Derives `Clone`. All fields are private and no public constructor or accessor exists. Values are returned by `Drawable2D::render_item_2d`.

## Construction, defaults, and validation

Clients cannot directly construct or validate the descriptor. `Shape` creates a consistent item internally.

## Units, coordinates, and valid ranges

Counts are internal Vulkan draw counts; coordinate interpretation belongs to the buffers/pipeline.

## Ownership, lifecycle, and threading

Owns cloned shared GPU-buffer handles; dropping it releases only those shares.

## Errors, panics, and failure conditions

Creation through `Shape`'s trait method is infallible. Submission can reject incompatible state.

## Allocation, transfers, synchronization, and GPU cost

Cloning handles does not upload geometry. Rendering the item records and submits draw commands; exact cost is not specified.

## Platform, Vulkan, and display constraints

Usable only with VMNL's compatible 2D render backend.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::Context;
use vmnl::d2::{Drawable2D, Shape};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let shape = Shape::rect(2.0, 2.0).build(&context)?;
    let item = shape.render_item_2d();
    let cloned = item.clone();
    drop((item, cloned));
    Ok(())
}
```

Related: [`Drawable2D`](drawable_2d.md).
