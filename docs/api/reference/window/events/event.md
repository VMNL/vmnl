# `Event`

## Public path and maturity

Import path: `vmnl::Event`. Status: experimental, operational translated window event.

## Purpose and use cases

Represents the subset of GLFW events VMNL translates for client event loops. Derives `Debug`, `Clone`, and `PartialEq`.

## Public API

Variants: `Closed`, `FocusGained`, `FocusLost`, `Resized { width, height }`, `FramebufferResized { width, height }`, `KeyPressed { key, repeat }`, `KeyReleased { key }`, `MouseMoved { x, y }`, `MouseEntered`, `MouseLeft`, `MouseButtonPressed { button }`, `MouseButtonReleased { button }`, `MouseScrolled { dx, dy }`, and `Text(char)`. All named variant fields are public through pattern matching.

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
