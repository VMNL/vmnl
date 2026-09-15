# Cursor controls

## Public path and maturity

Methods on `vmnl::Window` plus `Context::is_raw_mouse_motion_supported`. Status: experimental,
operational where the active backend supports the requested mode.

## Purpose and use cases

Reads and updates cursor position, visibility, confinement, hover state, and raw-motion
configuration without exposing GLFW types.

## Public API

| Member | Contract |
|---|---|
| `Window::get_cursor_position` | Read content-area cursor coordinates. |
| `Window::set_cursor_position` | Request finite content-area cursor coordinates. |
| `Window::is_cursor_hovered` | Read GLFW's current content-area hover attribute. |
| `Window::get_cursor_mode`, `Window::set_cursor_mode` | Read/set [`CursorMode`](input/cursor_mode.md). |
| `Context::is_raw_mouse_motion_supported` | Read system/backend raw-motion availability. |
| `Window::is_raw_mouse_motion_enabled`, `Window::set_raw_mouse_motion` | Read/configure raw motion per window. |

## Construction, defaults, and validation

New windows start in `CursorMode::Normal` with raw motion disabled. Cursor coordinates must be
finite; invalid coordinates are rejected before GLFW is called. Enabling raw motion is rejected
with `GlfwUnsupportedPlatform` when the context reports it unavailable. Disabling it on an
unsupported system is a successful no-op.

## Units, coordinates, and valid ranges

The origin is the upper-left corner of the window content area. X increases rightward and Y
downward. Values are `f64` GLFW screen coordinates, not framebuffer pixels. Native system cursors
may quantize positions. Disabled mode instead uses an unbounded virtual position and preserves
GLFW's `f64` representation.

## Ownership, lifecycle, and threading

Position and mode are per window. Raw-motion availability belongs to the initialized context;
configuration belongs to each window. Calls remain constrained to GLFW's platform-compatible main
thread through the single-threaded `Context`/`Window` API.

## Errors, panics, and failure conditions

`set_cursor_position` returns `InvalidState` for NaN or infinite coordinates. An unfocused window
silently ignores position updates as specified by GLFW. Other native positioning/mode failures are
reported through the configured GLFW error callback. Getters can return GLFW sentinels or stored
configuration even when native behavior is unavailable.

`set_raw_mouse_motion(true)` returns `GlfwUnsupportedPlatform` when the availability query is
false. A true configured state only changes delivered deltas while `CursorMode::Disabled` is
effective.

## Allocation, transfers, synchronization, and GPU cost

These calls allocate no VMNL GPU resources and perform no GPU transfer or synchronization. Native
backends may update pointer constraints or cursor state; exact platform cost is unspecified.

## Platform, Vulkan, and display constraints

Disabled/captured confinement is effective only while focused. Wayland requires compositor
relative-pointer and pointer-constraint support; non-disabled cursor warping is unavailable.
Captured mode is not implemented by GLFW 3.4 on Cocoa, where raw motion is also unavailable. X11
raw motion requires XInput 2. See [platform compatibility](platform_compatibility.md).

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, CursorMode, Window};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::new(&context)?;
    window.set_cursor_mode(CursorMode::Disabled);
    if context.is_raw_mouse_motion_supported() {
        window.set_raw_mouse_motion(true)?;
    }
    let position = window.get_cursor_position();
    println!("cursor={position:?}, hovered={}", window.is_cursor_hovered());
    Ok(())
}
```

Related: [`EventKind::MouseMoved`](events/event_kind.md), [polling](polling.md), and the
[`events_input`](../../../../examples/window/events_input/src/main.rs) example.
