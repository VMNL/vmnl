# Cursor controls

## Public path and maturity

`vmnl::Cursor`, `vmnl::CursorBuilder`, `vmnl::StandardCursor`,
`vmnl::StandardCursorBuilder`, methods on `vmnl::Window`, plus
`Context::is_raw_mouse_motion_supported`. Status: experimental, operational where the active
backend supports the requested cursor or mode.

## Purpose and use cases

Creates and assigns cursor resources, and reads or updates cursor position, visibility,
confinement, hover state, and raw-motion configuration without exposing GLFW types.

## Public API

| Member | Contract |
|---|---|
| `Cursor::standard` | Configure a [standard cursor builder](input/standard_cursor_builder.md). |
| `Cursor::rgba8` | Configure a custom [RGBA8 cursor builder](input/cursor_builder.md). |
| `CursorBuilder::hotspot_marker` | Replace the final hotspot pixel with a client-selected diagnostic color. |
| Builder `build` methods | Validate configuration and allocate the native cursor using a `Context`. |
| `Window::cursor`, `Window::set_cursor` | Inspect/assign a shared cursor; `None` restores the backend default. |
| `Window::get_cursor_position` | Read content-area cursor coordinates. |
| `Window::set_cursor_position` | Request finite content-area cursor coordinates. |
| `Window::is_cursor_hovered` | Read GLFW's current content-area hover attribute. |
| `Window::get_cursor_mode`, `Window::set_cursor_mode` | Read/set [`CursorMode`](input/cursor_mode.md). |
| `Context::is_raw_mouse_motion_supported` | Read system/backend raw-motion availability. |
| `Window::is_raw_mouse_motion_enabled`, `Window::set_raw_mouse_motion` | Read/configure raw motion per window. |

## Construction, defaults, and validation

New windows start in `CursorMode::Normal`, with the backend default cursor and raw motion disabled.
Custom cursor dimensions must be positive and fit `c_int`; the byte slice must contain exactly
`width * height * 4` bytes and the hotspot must be inside the image. These conditions and overflow
are checked during `CursorBuilder::build` before GLFW. The custom hotspot defaults to `(0, 0)`.
Its diagnostic marker is disabled by default and does not choose a contrast color implicitly.
Cursor coordinates must be finite. Enabling raw motion is rejected with
`GlfwUnsupportedPlatform` when the context reports it unavailable. Disabling it on an unsupported
system is a successful no-op.

## Units, coordinates, and valid ranges

Position and custom-cursor hotspot origins are upper-left, with X rightward and Y downward.
Positions are `f64` GLFW screen coordinates relative to the content area, not framebuffer pixels.
Hotspots and cursor dimensions are integer image pixels. Native system cursors may quantize
positions. Disabled mode instead uses an unbounded virtual position and preserves GLFW's `f64`
representation.

## Ownership, lifecycle, and threading

`Cursor` clones share one native handle through `Rc`; equality means resource identity. Each window
retains its own clone until replacement, removal, or window destruction, so one cursor can be used
by several windows and dropping the client clone is safe. The last owner destroys the handle by
RAII. A retained GLFW token keeps GLFW initialized through destruction. `Cursor` is neither `Send`
nor `Sync`.

Position, assigned cursor and mode are per window. Raw-motion availability belongs to the
initialized context; configuration belongs to each window. Calls remain constrained to GLFW's
platform-compatible main thread through the single-threaded `Context`/`Window` API.

## Errors, panics, and failure conditions

Custom cursor building returns `InvalidState` for invalid image parameters,
`GlfwUnsupportedPlatform` when a standard shape is unavailable, or the corresponding GLFW category
for another native failure. `Window::set_cursor` preserves the previous resource when GLFW reports
an error. Destruction errors can only be reported through the configured GLFW callback because
destruction occurs during `Drop`.

`set_cursor_position` returns `InvalidState` for NaN or infinite coordinates. An unfocused window
silently ignores position updates as specified by GLFW. Other native positioning/mode failures are
reported through the configured GLFW error callback. Getters can return GLFW sentinels or stored
configuration even when native behavior is unavailable.

`set_raw_mouse_motion(true)` returns `GlfwUnsupportedPlatform` when the availability query is
false. A true configured state only changes delivered deltas while `CursorMode::Disabled` is
effective.

## Allocation, transfers, synchronization, and GPU cost

Cursor factories and setters allocate no native resource. Each builder `build` performs one native
allocation. `CursorBuilder::build` synchronously copies the borrowed pixel slice before returning.
An enabled hotspot marker adds one temporary full-image VMNL allocation; the unmarked path adds
none. Cloning and assignment only increment an `Rc` owner count; they allocate no VMNL resource.
Cursor operations allocate no GPU resource and perform no GPU transfer or synchronization. Native
pointer/theme costs are unspecified.

## Platform, Vulkan, and display constraints

Disabled/captured confinement is effective only while focused. Wayland requires compositor
relative-pointer and pointer-constraint support; non-disabled cursor warping is unavailable.
Captured mode is not implemented by GLFW 3.4 on Cocoa, where raw motion is also unavailable. X11
raw motion requires XInput 2. Diagonal resize and not-allowed standard cursors can be unavailable
with some X11 or Wayland cursor themes. See [platform compatibility](platform_compatibility.md).

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, Cursor, CursorMode, StandardCursor, Window};

fn main() -> vmnl::VMNLResult<()> {
let context = Context::new()?;
let mut window = Window::new(&context)?;
let pointer = Cursor::standard(StandardCursor::PointingHand).build(&context)?;
window.set_cursor(Some(&pointer))?;
window.set_cursor_mode(CursorMode::Disabled);
    if context.is_raw_mouse_motion_supported() {
        window.set_raw_mouse_motion(true)?;
    }
    let position = window.get_cursor_position();
    println!("cursor={position:?}, hovered={}", window.is_cursor_hovered());
    Ok(())
}
```

Related: [`Cursor`](input/cursor.md), [`CursorBuilder`](input/cursor_builder.md),
[`StandardCursor`](input/standard_cursor.md),
[`StandardCursorBuilder`](input/standard_cursor_builder.md),
[`EventKind::MouseMoved`](events/event_kind.md), [polling](polling.md), and the
[`events_input`](../../../../examples/window/events_input/src/main.rs) example.
