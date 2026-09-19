# Joystick input

## Public path and maturity

`vmnl::JoystickId`, `vmnl::Joystick`, `vmnl::JoystickState`, `vmnl::StickSettings`, and `vmnl::StickState` are experimental.
VMNL samples all 16 GLFW slots. It tracks device presence and the two sticks of
a mapped gamepad, including their click buttons (L3/R3). All raw axes, buttons, and
hats are available even without a mapping. All 15 mapped buttons and all 6 axes,
including triggers, are exposed through `gamepad()`. Original mapped stick
axes and their unfiltered magnitude are available.

## Purpose and use cases

Read current stick directions through `window.input().joystick(id)` after each
`window.poll_events()`, or consume the joystick variants of [`Event`](../events/event.md).

## GLFW coverage

| GLFW capability | VMNL API |
| --- | --- |
| 16 slots and presence | `JoystickId::ALL`, `Input::joystick`, `is_connected` |
| Raw axes, buttons, hats | `JoystickState::raw`, including unmapped devices |
| Name, GUID, mapping name/status | `JoystickState::info`; GUID is the mapping key, not a serial number |
| 15 mapped buttons and 6 axes | `JoystickState::gamepad`, `GamepadButton`, `GamepadAxis` |
| Identified connection notifications | Per-window callback queues, drained by `poll_events` |
| Runtime mappings | `Context::update_gamepad_mappings` |
| Per-device application data | Typed `set_user_data`/`user_data`/`user_data_mut`/`clear_user_data`, per window |
| `JOYSTICK_HAT_BUTTONS` | `Context::with_joystick_options`, inspect with `joystick_options` |

This is API coverage, not hardware/platform qualification. VMNL uses owned Rust
data instead of exposing GLFW user pointers. GLFW has no separate mapping-ID getter.

## Public API

- `RawJoystickState`: `default`, `axes`, `buttons`, and `hats`; accessed through
  `input.joystick(id).raw()` after polling.
- `GamepadButton`: A/B/X/Y, bumpers, Back/Start/Guide, thumb clicks, and D-pad directions; `ALL` lists all 15.
- `GamepadAxis`: LeftX/LeftY/RightX/RightY/LeftTrigger/RightTrigger; `ALL` lists all 6.
- `GamepadState`: `buttons`, `axes`, `is_down(button)`, and `axis(axis)`.
- `JoystickInfo`: `name`, `guid`, `gamepad_name`, and `is_gamepad`.
- `JoystickState`: `gamepad`, `previous_gamepad`, `info`,
  `is_gamepad_pressed`, `is_gamepad_released`, `set_user_data`, `user_data`,
  `user_data_mut`, and `clear_user_data`.
- `HatState`: `Centered`, four cardinal directions, and four diagonals.
  A missing hat is an absent slice element; a centered hat is an existing element.

- `JoystickId`: `Slot1` through `Slot16`; `ALL` lists every slot in order.
  IDs identify reusable slots, not permanent devices. Each slot has independent
  snapshots and per-stick settings; `joystick(id)` borrows that slot without polling.

- `Joystick`: `JoystickLeft { degrees }`, `JoystickRight { degrees }`,
  `JoystickLeftButton`, and `JoystickRightButton`.
- `JoystickState`: `new`, `default`, `reset`, `left`, `right`, `is_down`,
  `is_connected`,
  `is_pressed`, `is_released`, `is_any_down`, `is_any_pressed`, `is_any_released`,
  `is_any_used`, `is_one_down`, `is_one_pressed`, `is_one_released`, and `is_one_used`.
- `JoystickState` also exposes `settings` and `set_settings` per stick.
- `StickSettings`: `default`, and public `dead_zone`, `zero_degrees`, `clockwise` fields.
- `StickState`: `default`, `with_axes`, `axes`, `magnitude`, `degrees`, and `is_clicked`.

Configure window-owned input with `Window::set_stick_settings`; independently owned
input also has `Input::set_stick_settings`. Inspect resolved settings with
`input.joystick(id).settings(selector)`. Invalid settings return `InvalidState` without
mutation. Settings changes reinterpret both snapshots, preserving their original axes
and clicks, without polling or enqueueing events. Previously returned events remain unchanged.

Query selectors identify a control, not an exact angle: their `degrees` field is
ignored by the boolean queries. For a direction, `is_down` means tilted outside
the dead zone, `is_pressed` means leaving it, and `is_released` means returning to
it. Rotation while tilted changes the angle and emits `JoystickMoved`, but does
not count as another press. Click queries are independent of stick tilt.

## Construction, defaults, and validation

`input.joystick(id).is_connected()` reports device presence at the last input
update, independently of gamepad mapping. It does not poll hardware. New and reset
snapshots report `false`; a present but unmapped device reports `true` after polling.

New/default states are centered and released, with no known device presence.
`reset` preserves settings while clearing current and previous snapshots and presence history; a device
present at the next update produces a connection event again. Manually constructed
snapshots do not poll GLFW. An empty slice passed to an `is_any_*` query returns false.

## Units, coordinates, and valid ranges

Raw axes are normally in `[-1, 1]` and are preserved without VMNL filtering or
clamping. Their order and orientation depend on the device/backend. Raw buttons
are booleans (`true` means pressed); `JoystickOptions::hat_buttons` controls
synthesized hat buttons at context initialization (default true).
Raw hats retain cardinal and diagonal directions; unexpected backend masks map to
`Centered`. No raw-control change events or raw button transition queries are added:
callers can compare samples themselves. Raw getters do not poll hardware.

Raw samples remain available during mapping loss. Disconnection and reset clear
them to empty vectors. Empty vectors may also mean zero controls or unavailable
backend data. The raw and mapped reads within a poll are not an atomic hardware snapshot.

Axes preserve mapped samples before VMNL filtering or clamping: X points right, Y
points down, and each normally lies in `[-1, 1]`. `magnitude()` is their Euclidean
length, which can exceed 1 at diagonals. Non-finite axes are retained; magnitude
follows floating-point `hypot` semantics and direction is `None`.

Default angles are `Option<f32>` in degrees: right is 0, up 90, left 180, down 270.
`None` means radius less than or equal to `dead_zone` (default 0.15). The threshold
must be finite and nonnegative; zero disables the dead zone except at exact center.
Values above 1 are allowed. `zero_degrees` is any finite counterclockwise angle from
right, interpreted modulo 360; `clockwise` reverses rotation from that zero.
Computed angles are in `[0, 360)`; directly constructed enum payloads are not validated.

Movement events carry the original axes and compare their bits, including magnitude
changes, dead-zone motion, signed zero, and NaN payload changes. Identical samples
do not repeat events. Callers may filter events or apply arbitrary calibration,
dead-zone shapes, and response curves to original axes; built-in direction and
active-state queries use the configured radial threshold and angle convention.

## Ownership, lifecycle, and threading

`Input` owns the state. `Input::joystick_mut` and `Window::joystick_mut` provide
mutable access for settings, reset, and typed application data. Data is local to
each window's slot, allocated in a box, and dropped on replacement, reset, clear,
or an observed disconnect (including disconnect/reconnect in one poll).
A typed getter returns `None` on a type mismatch. These snapshots are not Send/Sync. Poll once per input update;
transition queries compare consecutive samples and can miss changes between polls.
Joystick polling runs through the window on GLFW's main thread, independently
of keyboard/mouse callback flags. Blocking window waits do not update these snapshots.
Use regular polling or a timed wait followed by polling for controller-driven loops.

## Errors, panics, and failure conditions

Queries are infallible. `with_axes` and setting methods reject invalid settings with
`InvalidState`. Presence is sampled independently of the gamepad mapping.
A present but unmapped controller can emit connection events but has centered,
released stick state. Unavailable gamepad samples release held clicks and center
sticks once. Backend errors use the existing GLFW error callback path.

## Allocation, transfers, synchronization, and GPU cost

Complete mapped snapshots and derived sticks use fixed-size CPU storage.
Metadata polling obtains owned strings and may allocate. Connection queues are
owned by each window, grow to retain every backend notification, and drain on that
window's next poll. Dropping a window releases its queue; weak backend registrations
do not retain dropped windows. Raw polling obtains owned vectors from
GLFW and allocates converted button/hat vectors for nonempty data. Axes are moved
into the snapshot; old raw vectors are dropped when replaced. Cloning raw state may
allocate; getters borrow slices without allocating. No GPU work is performed by
input conversion. Appending events may grow the `Vec<Event>` returned by polling.

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
use vmnl::{Input, Joystick, JoystickId, JoystickState, StickSettings, StickState};

# fn main() -> vmnl::VMNLResult<()> {
let mut input = Input::new();
let id = JoystickId::Slot3;
input.set_stick_settings(id, Joystick::JoystickLeftButton, StickSettings {
    dead_zone: 0.05,
    zero_degrees: 90.0,
    clockwise: true,
})?;
let joystick: &JoystickState = input.joystick(id);
let left: &StickState = joystick.left();
assert_eq!(left.degrees(), None);
assert!(!joystick.is_down(Joystick::JoystickLeftButton));
assert_eq!(left.axes(), [0.0, 0.0]);
# Ok(())
# }
```

For a manual controller check, run `just run window_events_input` from the repository
root. Move both sticks through the cardinal directions and back to center, hold and
release each click, then unplug and reconnect the controller. Expect one transition
event per observed state change, with a presence event before stick transitions.
Also verify that keyboard/mouse controls and window closing still work. This is a
procedure, not a record of completed hardware validation.

For the axis-preservation regression, vary tilt along one direction and verify that
the printed axes and magnitude change. Move within the default dead zone and verify
that axes still change while direction stays `None`. Configure a different dead zone
and zero direction through `window.set_stick_settings`, then verify both sticks
retain independent settings and click behavior.

Mapped triggers normally range from -1 (released) to 1 (fully pressed).
`gamepad()` is `None` when absent, unmapped, or unavailable, while raw input can remain
available. `previous_gamepad()` preserves the previous poll for transition queries;
mapping loss and disconnect report mapped button releases once. Stick convenience
views use the same unfiltered mapped axes.

Metadata is cleared on disconnect/reset. GUID is the SDL mapping key, not a unique
physical serial number; GLFW has no separate mapping-ID getter.
`Context::update_gamepad_mappings` adds/replaces mappings at runtime, affecting all
contexts sharing GLFW. Inspect refreshed metadata and samples after the next poll.

Related: [`Input`](input.md), [`Event`](../events/event.md), and
[event processing](../events/event_processing_and_timers.md).

For multi-device validation, connect two controllers and verify that events carry
separate IDs, settings affect only the selected device, and disconnecting one leaves
the other's input active. Device events follow ascending slot order, with presence,
then left and right stick transitions within each slot when there are no queued callbacks.
Queued connection notifications precede these sampled transitions in backend order.
Each window receives its own notifications, even when another window or a blocking
wait processes GLFW events. Initial presence and final-sample discrepancies have
fallback events; callbacks are not duplicated by matching snapshot transitions.
Only notifications actually reported by GLFW are guaranteed to be retained.

The example also prints each connected device's raw axes, buttons, and hats. Check
an unmapped device, cardinal/diagonal hat directions, and devices with different
control counts. Raw data should remain readable without mapped stick input. This
procedure has not yet been validated with physical controllers.

For the remaining controller APIs, check every named button and both triggers;
inspect the printed device and mapping names/GUID. Call
`context.update_gamepad_mappings` with a mapping for that device and check the next
poll's mapped state. Attach a value with `window.joystick_mut(id).set_user_data`,
unplug/reconnect, and verify the value was cleared. With two windows, poll one more
frequently and verify both receive the same reported connection notifications.
Finally restart with `JoystickOptions { hat_buttons: false }` and compare raw
button counts and hat directions against the default setting. None of these
hardware checks has been reported as performed.
