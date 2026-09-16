# `StandardCursor`

## Public path and maturity

Import path: `vmnl::StandardCursor`. Status: experimental, operational when the active system
cursor theme provides the requested shape.

## Purpose and use cases

Selects one of the ten GLFW 3.4 system cursor shapes without exposing GLFW constants.

## Public API

| Variant | Contract |
|---|---|
| `Arrow` | Regular arrow. |
| `IBeam` | Text input. |
| `Crosshair` | Crosshair targeting. |
| `PointingHand` | Link or pointing action. |
| `ResizeEastWest` | Horizontal resize. |
| `ResizeNorthSouth` | Vertical resize. |
| `ResizeNorthwestSoutheast` | Northwest/southeast diagonal resize. |
| `ResizeNortheastSouthwest` | Northeast/southwest diagonal resize. |
| `ResizeAll` | Omnidirectional resize. |
| `NotAllowed` | Action unavailable. |

Derives `Clone`, `Copy`, `Debug`, `Eq`, `Hash`, and `PartialEq`.

## Construction, defaults, and validation

There is no default variant. Pass a variant to `Cursor::standard`, then call
`StandardCursorBuilder::build(&context)`; exact image and size come from the current system cursor
theme.

## Units, coordinates, and valid ranges

The enum has no units or numeric public representation.

## Ownership, lifecycle, and threading

The value is copied into cursor creation. The resulting [`Cursor`](cursor.md) owns the native
resource and remains confined to the GLFW thread.

## Errors, panics, and failure conditions

`StandardCursorBuilder::build` returns `GlfwUnsupportedPlatform` if the requested system shape is
unavailable. Configuration and building do not panic for any variant.

## Allocation, transfers, synchronization, and GPU cost

The enum and builder allocate nothing. Building the corresponding `Cursor` performs one native
allocation and no GPU work.

## Platform, Vulkan, and display constraints

GLFW guarantees the common shapes on supported native platforms. Diagonal resize and `NotAllowed`
may be unavailable with some X11 or Wayland cursor themes. Cocoa diagonal shapes currently use a
private system API. Exact appearance is platform/theme-defined.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, Cursor, StandardCursor};

# fn main() -> vmnl::VMNLResult<()> {
let context = Context::new()?;
let text_cursor = Cursor::standard(StandardCursor::IBeam).build(&context)?;
# let _ = text_cursor;
# Ok(())
# }
```

Related: [`Cursor`](cursor.md), [`StandardCursorBuilder`](standard_cursor_builder.md), and
[cursor controls](../cursor.md).
