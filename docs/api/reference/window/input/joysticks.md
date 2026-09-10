# Joystick input

## Public path and maturity

`vmnl::Joystick`, `vmnl::JoystickState`, and `vmnl::StickState` are experimental.
VMNL samples GLFW slot 1 only. It tracks device presence and the two sticks of
a mapped gamepad, including their click buttons (L3/R3). Other buttons, triggers,
tilt magnitude, raw joystick axes, and additional devices are not exposed.

## Purpose and use cases

Read current stick directions through `window.input().joystick()` after each
`window.poll_events()`, or consume the joystick variants of [`Event`](../events/event.md).

## Public API

- `Joystick`: `JoystickLeft { degrees }`, `JoystickRight { degrees }`,
  `JoystickLeftButton`, and `JoystickRightButton`.
- `JoystickState`: `new`, `default`, `reset`, `left`, `right`, `is_down`,
  `is_pressed`, `is_released`, `is_any_down`, `is_any_pressed`, `is_any_released`,
  `is_any_used`, `is_one_down`, `is_one_pressed`, `is_one_released`, and `is_one_used`.
- `StickState`: `default`, `degrees`, and `is_clicked`.

Query selectors identify a control, not an exact angle: their `degrees` field is
ignored by the boolean queries. For a direction, `is_down` means tilted outside
the dead zone, `is_pressed` means leaving it, and `is_released` means returning to
it. Rotation while tilted changes the angle and emits `JoystickMoved`, but does
not count as another press. Click queries are independent of stick tilt.

## Construction, defaults, and validation

New/default states are centered and released, with no known device presence.
`reset` clears current and previous snapshots and presence history; a device
present at the next update produces a connection event again. Manually constructed
snapshots do not poll GLFW. An empty slice passed to an `is_any_*` query returns false.

## Units, coordinates, and valid ranges

Angles are `Option<f32>` in degrees: right is 0, up 90, left 180, down 270.
`None` means centered within the fixed radial dead zone of 0.15 in normalized
GLFW axis units. Computed angles are in `[0, 360)`; directly constructed enum
payloads are not validated. Non-finite axis samples are treated as centered.
Movement events compare computed angles exactly; no angular smoothing is applied.

## Ownership, lifecycle, and threading

`Input` owns the state and exposes shared borrows. Poll once per input update;
transition queries compare consecutive samples and can miss changes between polls.
Joystick polling runs through the window on GLFW's main thread, independently
of keyboard/mouse callback flags. Blocking window waits do not update these snapshots.
Use regular polling or a timed wait followed by polling for controller-driven loops.

## Errors, panics, and failure conditions

Queries are infallible. Presence is sampled independently of the gamepad mapping.
A present but unmapped controller can emit connection events but has centered,
released stick state. Unavailable gamepad samples release held clicks and center
sticks once. Backend errors use the existing GLFW error callback path.

## Allocation, transfers, synchronization, and GPU cost

Snapshots use fixed-size CPU storage. Queries and state conversion do not allocate
or perform GPU work. Appending events may grow the `Vec<Event>` returned by polling.

## Platform, Vulkan, and display constraints

Native controller behavior remains hardware/backend dependent and has not been
qualified by physical controller testing. Pure state and transition tests are
headless. Normal `Window` creation still requires the VMNL Vulkan/display context.

If a connected controller has no gamepad mapping, a custom SDL-format file can be
loaded at context creation with `VMNL_GAMEPAD_MAPPINGS`. See
[mapping troubleshooting](../../../../troubleshooting.md#joystick-connected-but-sticks-do-not-respond).

## Example and related types

```rust
# extern crate vmnl;
use vmnl::{Input, Joystick, JoystickState, StickState};

let input = Input::new();
let joystick: &JoystickState = input.joystick();
let left: &StickState = joystick.left();
assert_eq!(left.degrees(), None);
assert!(!joystick.is_down(Joystick::JoystickLeftButton));
```

For a manual controller check, run `just run window_events_input` from the repository
root. Move both sticks through the cardinal directions and back to center, hold and
release each click, then unplug and reconnect the controller. Expect one transition
event per observed state change, with a presence event before stick transitions.
Also verify that keyboard/mouse controls and window closing still work. This is a
procedure, not a record of completed hardware validation.

Related: [`Input`](input.md), [`Event`](../events/event.md), and
[event processing](../events/event_processing_and_timers.md).
