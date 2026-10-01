# Joystick and gamepad samples

## Public path and maturity

Import paths: `vmnl::{JoystickId, JoystickSample, JoystickButtonState, JoystickHatState,
GamepadButton, GamepadAxis, GamepadState, JoystickState, JoystickStatus, StickConfig,
StickAngleConvention, StickState}`. Status: experimental input API.

## Purpose and use cases

Use [`InputRuntime::sample_joystick`](input_runtime.md) to read one GLFW slot without creating a
Vulkan context or window. A sample owns its raw values, identity strings, and optional mapped
gamepad state; it does not carry transition history. For transition-aware input, opt a window into
tracking and read [`Input::joystick`](window/input/input.md) after that window's
`poll_events()` call.

## Public API

| Type | Contract |
|---|---|
| `JoystickId` | One of GLFW's sixteen reusable slots. `from_index` accepts only zero-based indices `0..16`; `index` returns that index. |
| `JoystickSample` | Owns the slot ID, optional raw name and GUID, raw axes, raw buttons, hats, and optional mapped state. |
| `JoystickButtonState` | Preserves GLFW release/press values and any other raw byte. |
| `JoystickHatState` | Preserves the raw direction bit field; direction methods test cardinal bits, including diagonals. |
| `GamepadButton` | Selects one of GLFW's fifteen standard mapped buttons. |
| `GamepadAxis` | Selects one of GLFW's six standard mapped axes. |
| `GamepadState` | Owns the mapped name, all fifteen button states, and all six mapped axis values. |
| `JoystickStatus` | Reports `NotTracked`, `Absent`, or `Present` for one window slot. |
| `JoystickState` | Per-window owned sample, mapped gamepad state, processed sticks, and mapped-button transitions. |
| `StickConfig` | Pure radial dead-zone and angle configuration for one mapped stick. |
| `StickAngleConvention` | Chooses the `Math2D`, `Screen2D`, or local-planar `Heading` angle formula. |
| `StickState` | Processed X/Y vector and magnitude; calculates its configured angle on request. |

## Construction, defaults, and validation

`JoystickId::from_index` returns `None` outside `0..JoystickId::COUNT`. A joystick slot is a
reusable index, not a physical-device handle. GLFW's GUID identifies a model and can be shared by
identical units; raw and gamepad names are informational.

`InputRuntime::sample_joystick` distinguishes an absent slot (`Ok(None)`), a present joystick with
or without a mapping (`Ok(Some(sample))`), and a backend failure (`Err`). `sample.gamepad()` is
`None` if GLFW has no mapping. Raw joystick data remains available when a mapping exists.

`WindowBuilder::joystick_tracking(true)` or `Window::set_joystick_tracking(true)` opts that window
into sampling all sixteen slots on each of its `poll_events()` calls. Tracking is disabled by
default and is independent of connection-event delivery. Its first successful poll establishes a
button baseline without synthetic presses/releases. Disabling tracking clears that window's
snapshots and transitions; `Input::new()` remains detached and reports `NotTracked` for every slot.

`StickConfig::default()` uses a zero dead zone and `Math2D`. Configure left and right sticks
independently through `WindowBuilder::left_stick_config` and `right_stick_config`, or replace both
at runtime with `Window::set_stick_configs`. A dead zone must be finite and in `[0, 1)`. Magnitude
`<= dead_zone` yields `(0, 0)` and no angle; outside it, the original axes and magnitude are kept
without rescaling. `StickConfig::process(x, y)` applies the same pure math to axes from a standalone
sample. No raw joystick axis pair is inferred as a stick.

## Units, coordinates, and valid ranges

Raw axes are copied unchanged in GLFW order and its documented `[-1, 1]` range. Raw button values
are in GLFW order; the configured `GLFW_JOYSTICK_HAT_BUTTONS` hint may append hat directions to
that array. Separate hats preserve GLFW's four direction bits and diagonal combinations.

Mapped button selectors follow GLFW order: A, B, X, Y, left/right bumper, back, start, guide,
left/right thumb, then D-pad up/right/down/left. Mapped axis selectors follow left X/Y, right X/Y,
then left/right trigger. `GamepadState` axes remain unchanged from GLFW; processing affects only the
`JoystickState::left_stick` and `right_stick` views.

Angles are in radians normalized to `[0, 2π)` and computed only when `StickState::angle()` is
called. `Math2D` uses `atan2(-y, x)`, with zero right and counter-clockwise rotation in Y-up
coordinates. `Screen2D` uses `atan2(y, x)`, with zero right and clockwise rotation in Y-down screen
coordinates. `Heading` uses `atan2(x, -y)`, with zero up and positive rotation toward the right on a
local movement plane; it is not world or camera yaw.

`JoystickState::is_down` reflects the latest mapped sample. `is_pressed` and `is_released` are
non-consuming transitions accumulated within that window's current poll batch. Disconnect or
mapping loss releases previously held mapped buttons for that batch. A press and release wholly
between samples cannot be observed.

## Ownership, lifecycle, and threading

Samples own copied data; no pointer into GLFW's temporary arrays or strings escapes. Sampling may
allocate for arrays and strings, with no allocation-count guarantee. GLFW joystick queries must run
on its main platform thread. `InputRuntime` is neither `Send` nor `Sync`. Each `Window` owns an
independent snapshot, updated only by its own `poll_events()`; polling a standalone runtime does not
advance window snapshots. Disabling joystick event delivery does not disable state tracking.

`InputRuntime::joystick_user_pointer` and `set_joystick_user_pointer` expose GLFW's process-wide,
per-slot raw pointer as `unsafe` operations. GLFW does not own or synchronize the pointed-to
allocation and clears the pointer when the device disconnects. The caller controls its validity,
alignment, initialization, aliasing, synchronization, and destruction. Deferred events and samples
do not extend its lifetime. Follow both methods' Rustdoc `# Safety` sections before dereferencing or
replacing a pointer.

## Errors, panics, and failure conditions

Sampling errors use `VMNLErrorKind::GlfwInputOperationFailed` with the operation and backend detail.
Mapping updates are documented on [`InputRuntime`](input_runtime.md): GLFW callback parse errors are
reported even where the C function returns success.

## Allocation, transfers, synchronization, and GPU cost

Raw arrays and identity strings are copied to produce owned samples. No GPU work or Vulkan resource
is created by these input operations.

## Platform, Vulkan, and display constraints

Joystick functions follow GLFW 3.4's platform and thread constraints. Runtime initialization needs a
usable GLFW backend and its system services even though joystick sampling creates no window or
Vulkan object. Native controller availability and names vary by machine.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{GamepadAxis, InputRuntime, JoystickId};

# fn main() -> vmnl::VMNLResult<()> {
let runtime = InputRuntime::acquire()?;
if let Some(sample) = runtime.sample_joystick(JoystickId::Joystick1)? {
    for (index, value) in sample.axes().iter().enumerate() {
        println!("raw axis {index}: {value}");
    }
    if let Some(gamepad) = sample.gamepad() {
        println!("left stick X: {}", gamepad.axis(GamepadAxis::LeftX));
    }
}
# Ok(())
# }
```

Related: [`InputRuntime`](input_runtime.md), [`Input`](window/input/input.md), and [joystick/gamepad
capability coverage](../maintenance/joystick_capability_matrix.md).
