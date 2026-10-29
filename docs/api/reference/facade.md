# `vmnl` facade

## Public path and maturity

Import path: `vmnl`. Status: experimental facade; all client-visible items documented by this book are re-exported here.

## Purpose and use cases

The facade is a dependency boundary for VMNL clients; independent domain crates remain supported.
Graphics keeps `common`, `d2`, `d3`, and `raw`; optional audio is grouped under `audio`.

## Public API

`graphics` root re-exports: `Context`, `Window`, `WindowBuilder`, `FrameRenderer`, `RenderMode`, `PresentMode`,
`Event`, `EventKind`, `Cursor`, `CursorBuilder`, `CursorMode`, `StandardCursor`,
`StandardCursorBuilder`, `Input`, `Key`, `KeyboardState`, `Modifiers`, `MouseButton`, `MouseState`,
`InputRuntime`, `InputRuntimeBuilder`, `InputRuntimeConfig`, `Monitors`, `MonitorInfo`, `VideoMode`,
`JoystickId`, `JoystickSample`, `JoystickButtonState`, `JoystickHatState`, `GamepadButton`,
`GamepadAxis`, `GamepadState`, `ShaderSource`, `VMNLError`, `VMNLErrorKind`, `VMNLErrorLocation`,
and `VMNLResult`. Modules:
`common`, `d2`, `d3`, and `raw`.

## Construction, defaults, and validation

The facade has no value to construct.

| Cargo features | Surface |
|---|---|
| Default | `graphics`, retaining the existing root exports |
| No defaults, `graphics` | Graphics only |
| No defaults, `audio` | `vmnl::audio`, without graphics/Vulkan/GLFW dependencies |
| `graphics,audio` | Both independent domains |
| No features | No domain |

Default audio control is manual; see [Audio](audio.md). Feature selection creates no runtime.

## Units, coordinates, and valid ranges

See [coordinates, units, and colors](../concepts/coordinates_units_and_colors.md).

## Ownership, lifecycle, and threading

The facade adds no ownership or threading behavior. See [lifecycle, ownership, and threads](../concepts/lifecycle_ownership_and_threads.md).

## Errors, panics, and failure conditions

The facade itself is infallible. Graphics operations use `VMNLResult`; audio operations use `audio::AudioResult`.

## Allocation, transfers, synchronization, and GPU cost

Re-exporting symbols performs no operation at runtime. Costs of the re-exported API are documented by the referenced items; other performance characteristics are not specified.

## Platform, Vulkan, and display constraints

Audio-only clients need no graphics dependencies or display. Playback still needs a usable
audio device. Compilation does not qualify a Vulkan device, GLFW window, or output device.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::common::Rgba;
use vmnl::d2::Vector2f;

let point = Vector2f { x: 1.0, y: 2.0 };
let color = Rgba::RED;
assert_eq!((point.x, color.r), (1.0, 255));
```

Related: [`Context`](context.md), [`Window`](window/window.md), [`InputRuntime`](input_runtime.md),
[joystick/gamepad samples](joystick_gamepad.md), and [`VMNLResult`](errors/vmnl_result.md).
