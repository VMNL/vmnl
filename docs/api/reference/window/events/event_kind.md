# `EventKind`

## Public path and maturity

Import path: `vmnl::EventKind`. Status: experimental, operational event payload.

## Purpose and use cases

Represents the translated payload carried by an [`Event`](event.md). Derives `Debug`, `Clone`, and `PartialEq`.

## Public API

Variants: `Closed`, `FocusGained`, `FocusLost`, `Resized { width, height }`,
`FramebufferResized { width, height }`,
`KeyPressed { key, scancode, modifiers, repeat }`,
`KeyReleased { key, scancode, modifiers }`, `MouseMoved { x, y }`, `MouseEntered`, `MouseLeft`,
`MouseButtonPressed { button, modifiers }`, `MouseButtonReleased { button, modifiers }`,
`MouseScrolled { dx, dy }`, `Text(char)`, and
`TextWithModifiers { character, modifiers }`.

`TextWithModifiers` preserves GLFW's deprecated modified-character callback for compatibility.
New code should combine `Text` with key events instead of relying on this legacy source.

## Construction, defaults, and validation

There is no default. Direct construction is valid. Unknown physical keys use `Key::Unknown` and
retain their scancode and modifiers. Native negative size events are omitted.

## Units, coordinates, and valid ranges

Window/framebuffer sizes are pixels; cursor positions are `f64` window coordinates; scroll values
are backend offsets. `repeat` distinguishes native repeated key notifications. Keyboard and
mouse-button modifier flags describe modifier state at event generation. Scancodes are raw
platform-specific values. Text payloads contain Unicode scalar values; they are distinct from
physical `Key` values.

## Ownership, lifecycle, and threading

Payloads own/copy their data and do not remain synchronized with later window state.

## Errors, panics, and failure conditions

Translation is not fallible through the public API; unrepresentable or unsupported native events may be omitted.

## Allocation, transfers, synchronization, and GPU cost

Payloads are allocation-free and perform no GPU work.

## Platform, Vulkan, and display constraints

Key mapping, cursor coordinates, repeat behavior, modifier reporting, text composition, and
available events depend on GLFW/platform. The legacy modified-character callback may behave
differently across platform input methods.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::{EventKind, Key, Modifiers, Scancode};

let kind = EventKind::KeyPressed {
    key: Key::Unknown,
    scancode: Scancode::from_raw(42),
    modifiers: Modifiers::SHIFT,
    repeat: false,
};
assert!(matches!(kind, EventKind::KeyPressed { .. }));
```

Related: [`Event`](event.md), [`Modifiers`](../input/modifiers.md), [`Key`](../input/key.md),
[`Scancode`](../input/scancode.md), and [`MouseButton`](../input/mouse_button.md).
