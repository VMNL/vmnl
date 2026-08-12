# `MouseState`

## Public path and maturity

Import path: `vmnl::MouseState`. Status: experimental, operational snapshot.

## Purpose and use cases

Queries current mouse-button state and transitions between processed event batches.

## Public API

Single-button: `is_down`, `is_pressed`, `is_released`. Slice: `is_any_down`, `is_any_pressed`, `is_any_released`, `is_any_used`. All buttons: `is_one_down`, `is_one_pressed`, `is_one_released`, `is_one_used`. State: `reset`, `new`; `Default` delegates to `new`.

## Construction, defaults, and validation

New/default/reset state has every button up and no transitions.

## Units, coordinates, and valid ranges

Not applicable; positions and scrolling are `Event` payloads.

## Ownership, lifecycle, and threading

Window-owned snapshots update during event processing. Pressed/released compare previous and current arrays.

## Errors, panics, and failure conditions

Queries are infallible. State can be stale when events are not processed or mouse polling is disabled.

## Allocation, transfers, synchronization, and GPU cost

Fixed-size CPU arrays; no allocation or GPU work.

## Platform, Vulkan, and display constraints

Focus, capture, auxiliary buttons, and event delivery depend on the platform.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::{MouseButton, MouseState};

let state = MouseState::new();
assert!(!state.is_down(MouseButton::Left));
```

Related: [`MouseButton`](mouse_button.md), [`Input`](input.md), and [`Event`](../events/event.md).

