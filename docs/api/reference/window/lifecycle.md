# `Window` lifecycle methods

## Public path and maturity

Methods on `vmnl::Window`. Status: experimental, operational.

## Purpose and use cases

Controls visibility, focus, minimized/maximized state, and application-driven closure.

## Public API

`iconify`, `is_iconified`, `restore`, `maximize`, `is_maximized`, `show`, `hide`, `is_visible`, `focus`, `is_focused`, `is_open`, `is_ready`, and `close`.

## Construction, defaults, and validation

New windows start open; visibility/readiness are finalized by platform creation. No builder value is required for lifecycle operations.

## Units, coordinates, and valid ranges

Not applicable.

## Ownership, lifecycle, and threading

Mutating operations require `&mut Window`. `is_open` also requires mutable access because it consults/synchronizes close state. `close` marks the window for loop termination; dropping the window releases owned native/Vulkan state.

## Errors, panics, and failure conditions

These methods expose no typed result. Unsupported features are reported through the configured
error callback and have no effect. Platform refusal or asynchronous window-manager behavior may
mean an observed state changes later than the call.

## Allocation, transfers, synchronization, and GPU cost

No promised GPU work. Restoring/resizing can cause later swapchain recreation. Exact window-manager and synchronization cost is not specified.

## Platform, Vulkan, and display constraints

Visibility, focus, iconification, and maximization are requests to the platform/window manager and
may be constrained or ignored. Focus is never guaranteed. Wayland iconification/restoration is
compositor-dependent; X11 focus and maximization depend on the EWMH window manager. See
[platform compatibility](platform_compatibility.md).

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, Window};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::new(&context)?;
    if window.is_open() {
        window.close();
    }
    Ok(())
}
```

Related: [`Event::Closed`](events/event.md) and [event loops](../../workflows/window_event_loop.md).
