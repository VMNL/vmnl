# `KeyboardState`

## Public path and maturity

Import path: `vmnl::KeyboardState`. Status: experimental, operational snapshot.

## Purpose and use cases

Queries final held state and every key transition observed in the most recently processed event batch.

## Public API

Single-key methods: `is_down`, `is_pressed`, `is_released`. Slice methods: `is_any_down`, `is_any_pressed`, `is_any_released`, `is_any_used`. Whole-keyboard methods: `is_one_down`, `is_one_pressed`, `is_one_released`, `is_one_used`. State method: `new`; `Default` delegates to `new`.

`used` means down, newly pressed, or newly released. `any` tests the provided slice; `one` tests all supported keys.

## Construction, defaults, and validation

New/default state contains no active key. Queries accept all `Key` variants and do not return
errors. `Key::Unknown` is event-only: all snapshot queries return `false` for it, and whole-state
queries scan only named keys.

## Units, coordinates, and valid ranges

Not applicable.

## Ownership, lifecycle, and threading

The window owns and updates its state. One `Window::poll_events` call defines one batch. `is_down` reports the final state; `is_pressed` and `is_released` independently report whether each transition occurred at least once, so both can be true after a short press/release. Queries do not consume transitions. Key repeats keep a key down but do not count as a new press. `Window::clear_input_transitions` clears only press/release flags.

## Errors, panics, and failure conditions

Public queries are infallible. State remains unchanged until pending events are processed by `Window::poll_events`.

## Allocation, transfers, synchronization, and GPU cost

Fixed-size arrays; slice/whole-state scans are CPU-only. No performance bound beyond ordinary linear scans is specified.

## Platform, Vulkan, and display constraints

Focus, keyboard layout, native mapping, and repeat delivery are platform dependent.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::{Key, KeyboardState};

let state = KeyboardState::new();
assert!(!state.is_any_used(&[Key::A, Key::Escape]));
```

Related: [`Key`](key.md), [`Input`](input.md), and [`Event`](../events/event.md).
