# `LineBuilder`

## Public path and maturity

Import path: `vmnl::d2::LineBuilder`; created by `Shape::line`. Status: experimental, operational.

## Purpose and use cases

Generates indexed thick-line geometry with butt, square, or approximated round caps.

## Public API

`width(width)`, `cap(LineCap)`, `color(color)`, `buffer_memory_preference(preference)`, and `build(&Context)`.

## Construction, defaults, and validation

Endpoints are required. Defaults: width `1.0`, `LineCap::Butt`, opaque white, and `Device` preference. Endpoints must be distinct, finite, and not NaN; width must be finite and strictly positive.

## Units, coordinates, and valid ranges

Endpoints and width use 2D pixel-like units. Round caps use 12 fixed internal segments; this is current behavior, not a tessellation-quality guarantee.

## Ownership, lifecycle, and threading

CPU builder consumed by build; resulting shape owns context-associated vertex/index buffers.

## Errors, panics, and failure conditions

Invalid geometry returns `InvalidState`; count or buffer allocation/upload can fail.

## Allocation, transfers, synchronization, and GPU cost

Build generates CPU vertices/indices then allocates/uploads two buffers. Round caps generate more geometry than butt/square caps; exact performance is unspecified.

## Platform, Vulkan, and display constraints

Requires Vulkan for build and compatible display/window for drawing.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::Context;
use vmnl::d2::{LineCap, Shape, Vector2f};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let line = Shape::line(
        Vector2f { x: 0.0, y: 0.0 },
        Vector2f { x: 100.0, y: 0.0 },
    ).width(4.0).cap(LineCap::Round).build(&context)?;
    drop(line);
    Ok(())
}
```

Related: [`LineCap`](line_cap.md), [`Shape`](shape.md), and [`Vector2f`](../vector_2f.md).
