# `Input`

## Public path and maturity

Import path: `vmnl::Input`. Status: experimental, operational.

## Purpose and use cases

Groups keyboard and mouse state updated by window event processing.

## Public API

`new()`, `keyboard() -> &KeyboardState`, and `mouse() -> &MouseState`. `Default` delegates to `new`.

## Construction, defaults, and validation

New/default state has every key and button up, with no pressed/released transitions.

## Units, coordinates, and valid ranges

Not applicable; cursor positions and scroll deltas are represented by `Event`, not stored here.

## Ownership, lifecycle, and threading

Owned by `Window`; `Window::input()` returns a shared borrow. Manual `Input::new` creates an independent snapshot not connected to GLFW.

## Errors, panics, and failure conditions

Construction/accessors are infallible.

## Allocation, transfers, synchronization, and GPU cost

Fixed-size CPU state; no heap allocation or GPU work.

## Platform, Vulkan, and display constraints

Observed state depends on enabled polling, platform focus, and processed events.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::{Input, Key};

let input = Input::new();
assert!(!input.keyboard().is_down(Key::Escape));
```

Related: [`KeyboardState`](keyboard_state.md), [`MouseState`](mouse_state.md), and [`Window::input`](../events/event_processing_and_timers.md).

