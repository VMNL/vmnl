# `MouseButton`

## Public path and maturity

Import path: `vmnl::MouseButton`. Status: experimental.

## Purpose and use cases

Identifies mouse buttons supported by VMNL events and snapshots.

## Public API

Variants: `Left`, `Right`, `Middle`, `Button4`, `Button5`, `Button6`, `Button7`, and `Button8`. The enum is `#[repr(usize)]` and derives `Copy`, `Clone`, `Eq`, `PartialEq`, `Hash`, and `Debug`.

## Construction, defaults, and validation

No default. Every listed variant is valid.

## Units, coordinates, and valid ranges

Numeric representation is an internal index, not a stable serialized value.

## Ownership, lifecycle, and threading

Plain copied value.

## Errors, panics, and failure conditions

Infallible.

## Allocation, transfers, synchronization, and GPU cost

None.

## Platform, Vulkan, and display constraints

Availability of auxiliary buttons depends on device/platform. Cursor positions and scroll axes are separate events.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::MouseButton;

assert_ne!(MouseButton::Left, MouseButton::Right);
```

Related: [`MouseState`](mouse_state.md) and [`Event`](../events/event.md).
