# `Scancode`

## Public path and maturity

Import path: `vmnl::Scancode`. Status: experimental, operational keyboard-event metadata.

## Purpose and use cases

Preserves the platform-specific physical-key identifier supplied with a keyboard event. Derives
`Debug`, `Copy`, `Clone`, `Eq`, `PartialEq`, and `Hash`.

## Public API

`from_raw(i32) -> Scancode` constructs a value without validation. `as_raw() -> i32` returns the
unchanged platform value.

## Construction, defaults, and validation

There is no default. Every `i32`, including negative values, is preserved because VMNL does not
reinterpret the platform encoding.

## Units, coordinates, and valid ranges

The value is an opaque platform scancode, not a Unicode code point, key enum discriminant, or
portable identifier.

## Ownership, lifecycle, and threading

Plain copied value. It remains associated only with the environment that generated or queried it.

## Errors, panics, and failure conditions

Construction and raw-value access are infallible. `Context::get_scancode_name` returns `None` when
the value is invalid, unmapped, or maps to a non-printable key.

## Allocation, transfers, synchronization, and GPU cost

No allocation, synchronization, transfer, or GPU work.

## Platform, Vulkan, and display constraints

The same physical key may use different values across operating systems or window-system
backends. Do not persist a scancode as a portable key binding without recording its platform
context.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::Scancode;

let scancode = Scancode::from_raw(42);
assert_eq!(scancode.as_raw(), 42);
```

Related: [`Context`](../../context.md), [`Key`](key.md), [`KeyboardState`](keyboard_state.md), and
[`EventKind`](../events/event_kind.md).
