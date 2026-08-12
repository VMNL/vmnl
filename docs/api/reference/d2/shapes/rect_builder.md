# `RectBuilder`

## Public path and maturity

Import path: `vmnl::d2::RectBuilder`; created by `Shape::rect`. Status: experimental, operational.

## Purpose and use cases

Builds a positioned, colored, optionally rotated indexed rectangle.

## Public API

`position(x, y)`, `color(color)`, `rotation(degrees)`, `anchor(Anchor)`, `origin(x, y)`, `buffer_memory_preference(preference)`, and `build(&Context)`. `anchor` and `origin` replace each other.

## Construction, defaults, and validation

Width/height are required. Defaults: position `(0,0)`, opaque white, rotation `0`, `Anchor::TopLeft`, `Device` preference. Size must be finite and strictly positive; position, rotation, custom origin, and computed bounds must be finite/not NaN.

## Units, coordinates, and valid ranges

Position, size, and origin are 2D pixel-like values. Rotation is degrees and reduced modulo 360 for geometry.

## Ownership, lifecycle, and threading

CPU-only builder consumed by build; resulting shape owns context-associated buffers.

## Errors, panics, and failure conditions

Invalid numeric geometry returns `InvalidState`; GPU buffer creation can return Vulkan errors.

## Allocation, transfers, synchronization, and GPU cost

Build creates four vertices and six indices, then allocates/uploads vertex/index buffers. Exact cost is unspecified.

## Platform, Vulkan, and display constraints

Requires Vulkan for build and compatible display/window for drawing.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::Context;
use vmnl::d2::{Anchor, Shape};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let rect = Shape::rect(100.0, 50.0)
        .anchor(Anchor::Center)
        .rotation(30.0)
        .build(&context)?;
    drop(rect);
    Ok(())
}
```

Related: [`Anchor`](anchor.md), [`Shape`](shape.md), and [`BufferMemoryPreference`](../../common/buffer_memory_preference.md).

