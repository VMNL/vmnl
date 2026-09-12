# `Event`

## Public path and maturity

Import path: `vmnl::Event`. Status: experimental, operational translated window event.

## Purpose and use cases

Represents the subset of GLFW events VMNL translates for client event loops. Derives `Debug`, `Clone`, and `PartialEq`.

## Public API

Variants: `Closed`, `FocusGained`, `FocusLost`, `Resized { width, height }`, `FramebufferResized { width, height }`, `KeyPressed { key, repeat }`, `KeyReleased { key }`, `MouseMoved { x, y }`, `MouseEntered`, `MouseLeft`, `MouseButtonPressed { button }`, `MouseButtonReleased { button }`, `MouseScrolled { dx, dy }`, and `Text(char)`. All named variant fields are public through pattern matching.

Joystick integration also declares `JoystickConnected`, `JoystickDisconnected`,
`JoystickButtonPressed { joystick }`, `JoystickButtonReleased { joystick }`, and
`JoystickMoved { joystick, axes }`.
`Window::poll_events` emits click and movement transitions for the mapped gamepad
in GLFW slot 1 after native window events. Left stick transitions precede right
stick transitions, with a click transition before a movement transition per stick.
Presence transitions for GLFW slot 1 are sampled independently of gamepad mapping
and precede stick transitions. A device already present on the first poll emits
`JoystickConnected`. Repeated presence states do not repeat events. Loss of mapping
alone does not emit `JoystickDisconnected`. Disconnect/reconnect cycles entirely
between polls can be missed.
Button payloads designate `Joystick::JoystickLeftButton` or `Joystick::JoystickRightButton`;
the type also permits direction variants, so construction does not enforce this restriction.
The connection variants do not yet carry a device identifier.
Movement payloads use `Joystick::JoystickLeft { degrees }` or
`Joystick::JoystickRight { degrees }`: `Some(angle)` is in `[0, 360)` using the
configured zero direction and rotation sense; `None` means inside the configured
dead zone or non-finite axes. Defaults are counterclockwise from right with a 0.15
radial threshold. `axes` preserves original mapped X/Y samples, right/down positive,
normally in `[-1, 1]` each, even inside the dead zone. Construction does not validate
the payload. Movement events compare consecutive axis bits, including magnitude-only
changes, signed zero, and NaN payload changes. Identical samples do not repeat events.
An unavailable or unmapped gamepad releases held clicks and
centers tilted sticks once. Presence events distinguish reported device absence
from mapping loss. Changes between samples can be missed.

## Construction, defaults, and validation

There is no default. Clients normally receive values from `Window::poll_events`; direct construction is valid. Native negative size events and unsupported keys are omitted when they cannot be translated.

## Units, coordinates, and valid ranges

Window/framebuffer sizes are pixels; cursor positions are `f64` window coordinates; scroll values are backend offsets; `repeat` distinguishes repeated press notifications.

## Ownership, lifecycle, and threading

Events own/copy all payloads and do not borrow the window. They are snapshots and do not remain synchronized with later window state.

## Errors, panics, and failure conditions

Translation is not fallible through the public API; unrepresentable/unsupported native events may be omitted.

## Allocation, transfers, synchronization, and GPU cost

No GPU work. Collecting events allocates the returned `Vec`; individual variants are allocation-free.

## Platform, Vulkan, and display constraints

Delivery, key mapping, cursor coordinates, repeat behavior, and available events depend on GLFW/platform. Polling must be enabled for the corresponding source.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::{Event, Key};

let event = Event::KeyPressed { key: Key::A, repeat: false };
assert!(matches!(event, Event::KeyPressed { key: Key::A, repeat: false }));
```

Related: [`Key`](../input/key.md), [`MouseButton`](../input/mouse_button.md), and [event processing](event_processing_and_timers.md).
