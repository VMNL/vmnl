# `EventKind`

## Public path and maturity

Import path: `vmnl::EventKind`. Status: experimental, operational event payload.

## Purpose and use cases

Represents the translated payload carried by an [`Event`](event.md). Derives `Debug`, `Clone`, and `PartialEq`.

## Public API

Variants: `Closed`, `FocusGained`, `FocusLost`, `Resized { width, height }`, `FramebufferResized { width, height }`, `KeyPressed { key, repeat }`, `KeyReleased { key }`, `MouseMoved { x, y }`, `MouseEntered`, `MouseLeft`, `MouseButtonPressed { button, modifiers }`, `MouseButtonReleased { button, modifiers }`, `MouseScrolled { dx, dy }`, and `Text(char)`.

## Construction, defaults, and validation

There is no default. Direct construction is valid. Native negative size events and unsupported keys are omitted when they cannot be translated.

## Units, coordinates, and valid ranges

Window/framebuffer sizes are pixels; cursor positions are `f64` window coordinates; scroll values are backend offsets. `repeat` distinguishes native repeated key notifications. Mouse-button modifier flags describe modifier state at event generation.

## Ownership, lifecycle, and threading

Payloads own/copy their data and do not remain synchronized with later window state.

## Errors, panics, and failure conditions

Translation is not fallible through the public API; unrepresentable or unsupported native events may be omitted.

## Allocation, transfers, synchronization, and GPU cost

Payloads are allocation-free and perform no GPU work.

## Platform, Vulkan, and display constraints

Key mapping, cursor coordinates, repeat behavior, modifier reporting, and available events depend on GLFW/platform.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::{EventKind, Modifiers, MouseButton};

let kind = EventKind::MouseButtonPressed {
    button: MouseButton::Left,
    modifiers: Modifiers::SHIFT,
};
assert!(matches!(kind, EventKind::MouseButtonPressed { .. }));
```

Related: [`Event`](event.md), [`Modifiers`](../input/modifiers.md), [`Key`](../input/key.md), and [`MouseButton`](../input/mouse_button.md).
