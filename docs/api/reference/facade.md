# `vmnl` facade

## Public path and maturity

Import path: `vmnl`. Status: experimental facade; all client-visible items documented by this book are re-exported here.

## Purpose and use cases

The facade is the supported dependency boundary for VMNL clients. It hides the internal crate/module organization while grouping rendering primitives into `common`, `d2`, `d3`, and `raw`.

## Public API

Root re-exports: `Context`, `Window`, `WindowBuilder`, `FrameRenderer`, `RenderMode`, `PresentMode`, `Event`, `Input`, `Key`, `KeyboardState`, `MouseButton`, `MouseState`, `Monitors`, `MonitorInfo`, `VideoMode`, `ShaderSource`, `VMNLError`, `VMNLErrorKind`, `VMNLErrorLocation`, and `VMNLResult`. Modules: `common`, `d2`, `d3`, and `raw`.

## Construction, defaults, and validation

The facade has no value to construct. Defaults and validation belong to the re-exported items.

## Units, coordinates, and valid ranges

See [coordinates, units, and colors](../concepts/coordinates_units_and_colors.md).

## Ownership, lifecycle, and threading

The facade adds no ownership or threading behavior. See [lifecycle, ownership, and threads](../concepts/lifecycle_ownership_and_threads.md).

## Errors, panics, and failure conditions

The facade itself is infallible. Re-exported operations use `VMNLResult` where failure is represented.

## Allocation, transfers, synchronization, and GPU cost

Re-exporting symbols performs no operation at runtime. Costs of the re-exported API are documented by the referenced items; other performance characteristics are not specified.

## Platform, Vulkan, and display constraints

Compiling the facade is not proof that a Vulkan device, GLFW window, or display is available at runtime.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::common::Rgba;
use vmnl::d2::Vector2f;

let point = Vector2f { x: 1.0, y: 2.0 };
let color = Rgba::RED;
assert_eq!((point.x, color.r), (1.0, 255));
```

Related: [`Context`](context.md), [`Window`](window/window.md), and [`VMNLResult`](errors/vmnl_result.md).
