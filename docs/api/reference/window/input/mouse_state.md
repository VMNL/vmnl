# `MouseState`

## Public path and maturity

Import path: `vmnl::MouseState`. Status: experimental, operational snapshot.

## Purpose and use cases

Queries final held state and every mouse-button transition observed in the most recently processed event batch.

## Public API

Single-button: `is_down`, `is_pressed`, `is_released`. Slice: `is_any_down`, `is_any_pressed`, `is_any_released`, `is_any_used`. All buttons: `is_one_down`, `is_one_pressed`, `is_one_released`, `is_one_used`. State method: `new`; `Default` delegates to `new`.

## Construction, defaults, and validation

New/default state has every button up and no transitions.

## Units, coordinates, and valid ranges

Not applicable; positions and scrolling are `Event` payloads.

## Ownership, lifecycle, and threading

Window-owned snapshots update during `Window::poll_events`. `is_down` reports the final state; `is_pressed` and `is_released` independently retain every transition observed in that batch, so both can be true after a short click. Queries do not consume transitions. `Window::clear_input_transitions` clears only press/release flags.

`Window::set_sticky_mouse_buttons` configures GLFW's consuming native button reads. VMNL snapshots
do not perform those reads, so enabling the mode does not change `MouseState` semantics.

## Errors, panics, and failure conditions

Queries are infallible. State remains unchanged until pending events are processed by `Window::poll_events`.

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

Related: [`MouseButton`](mouse_button.md), [`Input`](input.md), [window input modes](modes.md), and
[`Event`](../events/event.md).
