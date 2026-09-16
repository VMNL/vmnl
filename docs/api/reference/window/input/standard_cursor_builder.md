# `StandardCursorBuilder`

## Public path and maturity

Import path: `vmnl::StandardCursorBuilder`. Status: experimental, operational when the active
system cursor theme provides the requested shape.

## Purpose and use cases

Defers native allocation of a [`StandardCursor`](standard_cursor.md) until a
[`Context`](../../context.md) is supplied to `build`.

## Public API

| Member | Contract |
|---|---|
| `Cursor::standard` | Create a builder containing the requested standard shape. |
| `build` | Allocate and return the corresponding native [`Cursor`](cursor.md). |

## Construction, defaults, and validation

There is no default shape. `Cursor::standard` requires one explicit `StandardCursor`; `build`
passes that validated enum value to the backend.

## Units, coordinates, and valid ranges

The builder has no numeric units or configurable coordinates.

## Ownership, lifecycle, and threading

The builder owns the copied enum value and is consumed by `build`. The resulting cursor owns the
native resource and remains confined to the GLFW platform thread.

## Errors, panics, and failure conditions

`build` returns `GlfwUnsupportedPlatform` when the active theme lacks the requested shape. Other
native failures retain their VMNL GLFW category. Building does not panic for a valid enum variant.

## Allocation, transfers, synchronization, and GPU cost

Configuration allocates nothing. `build` performs one native cursor allocation and no GPU work.

## Platform, Vulkan, and display constraints

Building requires initialized GLFW on its platform-compatible thread. Exact appearance and size
come from the platform theme. Vulkan and the GPU do not participate.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, Cursor, StandardCursor};

# fn main() -> vmnl::VMNLResult<()> {
let context = Context::new()?;
let cursor = Cursor::standard(StandardCursor::PointingHand).build(&context)?;
# let _ = cursor;
# Ok(())
# }
```

Related: [`Cursor`](cursor.md), [`CursorBuilder`](cursor_builder.md),
[`StandardCursor`](standard_cursor.md), and [cursor controls](../cursor.md).
