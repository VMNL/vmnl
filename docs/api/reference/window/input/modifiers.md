# `Modifiers`

## Public path and maturity

Import path: `vmnl::Modifiers`. Status: experimental, operational flags.

## Purpose and use cases

Represents modifier keys captured with a mouse-button event. Derives `Debug`, `Copy`, `Clone`, `Eq`, `PartialEq`, `Hash`, and `Default`.

## Public API

Constants: `NONE`, `SHIFT`, `CONTROL`, `ALT`, `SUPER`, `CAPS_LOCK`, and `NUM_LOCK`. Methods: `is_empty`, `contains`, and `bits`. Bitwise `|`, `|=`, `&`, and `&=` combine or intersect flags.

## Construction, defaults, and validation

`Default` is `NONE`. Public values are built from constants and bitwise operations; VMNL does not expose construction from arbitrary bits.

## Units, coordinates, and valid ranges

`bits()` returns the internal `u8` mask. Its numeric assignments are VMNL values, not a promised GLFW ABI conversion surface.

## Ownership, lifecycle, and threading

Values are copied into `EventKind::MouseButtonPressed` and `MouseButtonReleased` payloads.

## Errors, panics, and failure conditions

Operations are infallible.

## Allocation, transfers, synchronization, and GPU cost

No allocation, synchronization, or GPU work.

## Platform, Vulkan, and display constraints

Which modifiers GLFW reports can depend on platform focus, lock-key support, and keyboard state.
Caps Lock and Num Lock bits require `Window::set_lock_key_modifier_reporting(true)`.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::Modifiers;

let modifiers = Modifiers::SHIFT | Modifiers::CONTROL;
assert!(modifiers.contains(Modifiers::SHIFT));
assert!(!modifiers.contains(Modifiers::ALT));
```

Related: [`EventKind`](../events/event_kind.md), [`MouseButton`](mouse_button.md), and
[window input modes](modes.md).
