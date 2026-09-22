# `Key`

## Public path and maturity

Import path: `vmnl::Key`. Status: experimental.

## Purpose and use cases

Identifies keys supported by VMNL input snapshots and key events.

## Public API

Variants: `Unknown`; letters `A` through `Z`; digits `Num0` through `Num9`; `Space` and
punctuation; `World1` and `World2`; editing, navigation, lock, system, and arrow keys; functions
`F1` through `F25`; the complete keypad; distinct left/right Shift, Control, Alt, and Super keys;
and `Menu`. These are all 120 named GLFW 3.4 keys. The enum is `#[repr(usize)]` and derives
`Copy`, `Clone`, `Eq`, `PartialEq`, `Hash`, and `Debug`.

## Construction, defaults, and validation

No `Default`. Values are created directly or translated from GLFW. Every named GLFW 3.4 key is
translated. Native unknown-key events are preserved as `Unknown` together with their `Scancode`;
`Unknown` remains directly constructible and is not tracked by `KeyboardState`.

## Units, coordinates, and valid ranges

The numeric representation is an implementation detail used for state indexing; do not persist it as a stable protocol.

## Ownership, lifecycle, and threading

Plain copied value.

## Errors, panics, and failure conditions

Constructing/matching a variant is infallible.

`Context::get_key_name` and `Context::get_key_scancode` return `None` for `Unknown`.

## Allocation, transfers, synchronization, and GPU cost

None.

## Platform, Vulkan, and display constraints

Physical layout, key labels, modifiers, scancodes, and the availability of individual keys are
platform dependent. Named values identify physical key tokens, not produced Unicode text. This
enum does not embed modifiers or scancodes; keyboard event variants expose them separately.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::Key;

let quit = Key::Escape;
assert_eq!(quit, Key::Escape);
```

Related: [`Context`](../../context.md), [`KeyboardState`](keyboard_state.md),
[`Scancode`](scancode.md), and
[`Event`](../events/event.md).
