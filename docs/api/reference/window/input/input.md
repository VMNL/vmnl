# `Input`

## Public path and maturity

Import path: `vmnl::Input`. Status: experimental, operational.

## Purpose and use cases

Groups keyboard, mouse, and multi-device joystick state updated by `Window::poll_events`.

## Public API

`new()`, `keyboard() -> &KeyboardState`, `mouse() -> &MouseState`, and
`joystick(id: JoystickId) -> &JoystickState`, and `set_stick_settings(id, joystick, settings) -> VMNLResult<()>`.
`joystick_mut(id)` allows settings, reset, and typed application data.
`Default` delegates to `new`. Stick settings and validation are described in [joystick input](joysticks.md).

## Construction, defaults, and validation

New/default state has every key and button up, centered sticks, and no
pressed/released transitions. Joystick presence initially assumes an absent device.

## Units, coordinates, and valid ranges

Joystick directions are degrees in `[0, 360)` or `None` when centered; see
[joystick input](joysticks.md). Cursor positions and scroll deltas are represented by `Event`.

## Ownership, lifecycle, and threading

Owned by `Window`; `Window::input()` returns a shared borrow. Manual `Input::new` creates an independent snapshot not connected to GLFW.

## Errors, panics, and failure conditions

Construction/accessors are infallible. Invalid stick settings return `InvalidState` without mutation.

## Allocation, transfers, synchronization, and GPU cost

Construction starts with empty raw vectors. Raw polling may allocate vectors;
getters borrow stored state without allocation. No GPU work is performed by input sampling.

## Platform, Vulkan, and display constraints

Keyboard/mouse state depends on enabled polling, platform focus, and processed events.
Joystick sampling is independent of those polling flags. All 16 GLFW slots are
tracked; sticks require a gamepad mapping. Presence does not require a mapping.
All raw axes, buttons, and hats are available without a mapping. Complete mapped
buttons and axes are available through `gamepad()` when a mapping exists.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::{Input, Key};

let input = Input::new();
assert!(!input.keyboard().is_down(Key::Escape));
```

Related: [`KeyboardState`](keyboard_state.md), [`MouseState`](mouse_state.md), and [`Window::input`](../events/event_processing_and_timers.md).
