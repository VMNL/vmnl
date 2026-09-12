# `Input`

## Public path and maturity

Import path: `vmnl::Input`. Status: experimental, operational.

## Purpose and use cases

Groups keyboard, mouse, and slot-1 joystick state updated by `Window::poll_events`.

## Public API

`new()`, `keyboard() -> &KeyboardState`, `mouse() -> &MouseState`, and
`joystick() -> &JoystickState`, and `set_stick_settings(joystick, settings) -> VMNLResult<()>`.
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

Fixed-size CPU state; no heap allocation or GPU work.

## Platform, Vulkan, and display constraints

Keyboard/mouse state depends on enabled polling, platform focus, and processed events.
Joystick sampling is independent of those polling flags. Only GLFW slot 1 is
tracked; sticks require a gamepad mapping. Presence does not require a mapping.
Other gamepad buttons, triggers, and additional device slots are not exposed.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::{Input, Key};

let input = Input::new();
assert!(!input.keyboard().is_down(Key::Escape));
```

Related: [`KeyboardState`](keyboard_state.md), [`MouseState`](mouse_state.md), and [`Window::input`](../events/event_processing_and_timers.md).
