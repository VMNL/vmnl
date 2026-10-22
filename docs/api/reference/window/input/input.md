# `Input`

## Public path and maturity

Import path: `vmnl::Input`. Status: experimental, operational.

## Purpose and use cases

Groups the per-window keyboard, mouse-button, and joystick snapshots updated by window event
processing. Joystick slots remain `NotTracked` until their owning window explicitly enables
joystick tracking.

## Public API

`new()`, `keyboard() -> &KeyboardState`, `mouse() -> &MouseState`, and
`joystick(id) -> &JoystickState`. `Default` delegates to `new`.

## Construction, defaults, and validation

New/default state has every key and button up, with no pressed/released transitions. Each joystick
slot starts as `JoystickStatus::NotTracked`.

## Units, coordinates, and valid ranges

Not applicable; cursor positions and scroll deltas are represented by `Event`, not stored here.

## Ownership, lifecycle, and threading

Owned by `Window`; `Window::input()` returns a shared borrow. One batch is one
`Window::poll_events` call. Keyboard and mouse state tracking remains active independently of
public event delivery. Joystick state tracking is explicitly opt-in and independent of joystick
connection-event delivery. Each window samples and owns its own joystick snapshots; standalone
`Input::new` creates detached states that do not connect to GLFW.

## Errors, panics, and failure conditions

Construction/accessors are infallible.

## Allocation, transfers, synchronization, and GPU cost

Fixed-size CPU transition state; present joystick samples own copied vectors and strings. No GPU
work is performed by input tracking.

## Platform, Vulkan, and display constraints

Observed keyboard/mouse state depends on platform focus and processed events. `wait_events` alone
does not update it; the next `poll_events` call does. Joystick state changes only during that
window's `poll_events` call when tracking is enabled.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::{Input, JoystickId, JoystickStatus, Key};

let input = Input::new();
assert!(!input.keyboard().is_down(Key::Escape));
assert_eq!(
    input.joystick(JoystickId::Joystick1).status(),
    JoystickStatus::NotTracked
);
```

Related: [`KeyboardState`](keyboard_state.md), [`MouseState`](mouse_state.md), and [`Window::input`](../events/event_processing_and_timers.md).
