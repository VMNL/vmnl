# `Key`

## Public path and maturity

Import path: `vmnl::Key`. Status: experimental.

## Purpose and use cases

Identifies keys supported by VMNL input snapshots and key events.

## Public API

Variants: `Unknown`; letters `A` through `Z`; digits `Num0` through `Num9`; `Escape`, `Enter`, `Tab`, `Backspace`; arrows `Left`, `Right`, `Up`, `Down`; and functions `F1` through `F12`. The enum is `#[repr(usize)]` and derives `Copy`, `Clone`, `Eq`, `PartialEq`, `Hash`, and `Debug`.

## Construction, defaults, and validation

No `Default`. Values are created directly or translated from GLFW. Unsupported native keys are omitted by the current translation; `Unknown` remains directly constructible.

## Units, coordinates, and valid ranges

The numeric representation is an implementation detail used for state indexing; do not persist it as a stable protocol.

## Ownership, lifecycle, and threading

Plain copied value.

## Errors, panics, and failure conditions

Constructing/matching a variant is infallible.

## Allocation, transfers, synchronization, and GPU cost

None.

## Platform, Vulkan, and display constraints

Physical layout, key labels, modifiers, scancodes, and unsupported keys are platform dependent. This enum does not expose modifiers or scancodes.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::Key;

let quit = Key::Escape;
assert_eq!(quit, Key::Escape);
```

Related: [`KeyboardState`](keyboard_state.md) and [`Event`](../events/event.md).
