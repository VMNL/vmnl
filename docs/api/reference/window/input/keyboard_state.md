# `KeyboardState`

## Public path and maturity

Import path: `vmnl::KeyboardState`. Status: experimental, operational snapshot.

## Purpose and use cases

Queries current key state and transitions between the previous and current processed event batch.

## Public API

Single-key methods: `is_down`, `is_pressed`, `is_released`. Slice methods: `is_any_down`, `is_any_pressed`, `is_any_released`, `is_any_used`. Whole-keyboard methods: `is_one_down`, `is_one_pressed`, `is_one_released`, `is_one_used`. State methods: `reset`, `new`; `Default` delegates to `new`.

`used` means down, newly pressed, or newly released. `any` tests the provided slice; `one` tests all supported keys.

## Construction, defaults, and validation

New/default/reset state contains no active key. Queries accept all `Key` variants and do not return errors.

## Units, coordinates, and valid ranges

Not applicable.

## Ownership, lifecycle, and threading

The window owns and updates its state. Pressed/released are transition snapshots; call event processing before querying a new batch. `reset` clears both current and previous arrays.

## Errors, panics, and failure conditions

Public queries are infallible. State can appear stale when events are not processed or key polling is disabled.

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

